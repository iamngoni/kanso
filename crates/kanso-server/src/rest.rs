//! The REST endpoints of the local-server API.

use std::collections::HashMap;

use axum::Json;
use axum::Router;
use axum::body::Bytes;
use axum::extract::rejection::{BytesRejection, PathRejection, QueryRejection};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use kanso_core::DocType;
use kanso_core::id::with_prefix;
use serde_json::{Value, json};

use crate::AppState;
use crate::api::{self, NoteQuery, Since};
use crate::error::{ApiError, ApiResult};

type Params = Result<Query<HashMap<String, String>>, QueryRejection>;
type Body = Result<Bytes, BytesRejection>;
type JsonResult = ApiResult<Json<Value>>;

pub(crate) fn routes() -> Router<AppState> {
    let mut router = Router::new()
        .route("/", get(meta))
        .route("/_changes", get(changes))
        .route("/loops", get(list_loops))
        .route(
            "/notes",
            get(list_notes).post(|state: State<AppState>, body: Body| {
                post_collection(state, body, DocType::Note)
            }),
        );
    for doc_type in [DocType::Book, DocType::Tag, DocType::File] {
        router = router.route(
            &format!("/{}", doc_type.collection()),
            get(move |state: State<AppState>, params: Params| {
                list_collection(state, params, doc_type)
            })
            .post(move |state: State<AppState>, body: Body| post_collection(state, body, doc_type)),
        );
    }
    router
        .route("/{id}", get(get_doc).post(post_doc).delete(delete_doc))
        .route(
            "/{kind}/{id}",
            get(get_alias).post(post_alias).delete(delete_alias),
        )
}

pub(crate) async fn not_found() -> ApiError {
    ApiError::not_found()
}

pub(crate) async fn method_not_allowed() -> ApiError {
    ApiError::method_not_allowed()
}

/// Runs a store operation on the blocking pool.
pub(crate) async fn blocking<T: Send + 'static>(
    state: &AppState,
    f: impl FnOnce(&kanso_store::Store) -> ApiResult<T> + Send + 'static,
) -> ApiResult<T> {
    let store = state.store.clone();
    tokio::task::spawn_blocking(move || f(&store))
        .await
        .map_err(|err| ApiError::internal(format!("The request failed: {err}")))?
}

/// The request body as JSON, with CouchDB-style errors for bodies that are
/// too large or malformed.
pub(crate) fn json_body(body: Body) -> ApiResult<Value> {
    let bytes = body.map_err(|rejection| {
        if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE {
            ApiError::too_large()
        } else {
            ApiError::bad_request(rejection.body_text())
        }
    })?;
    serde_json::from_slice(&bytes)
        .map_err(|err| ApiError::bad_request(format!("The request body is not valid JSON: {err}")))
}

/// Query-string parameters; empty values count as absent.
struct QueryParams(HashMap<String, String>);

impl QueryParams {
    fn parse(params: Params) -> ApiResult<Self> {
        params
            .map(|Query(map)| QueryParams(map))
            .map_err(|rejection| ApiError::bad_request(rejection.body_text()))
    }

    fn get(&self, name: &str) -> Option<&str> {
        self.0
            .get(name)
            .map(String::as_str)
            .filter(|v| !v.is_empty())
    }

    fn count(&self, name: &str) -> ApiResult<Option<usize>> {
        self.get(name)
            .map(|raw| api::parse_count(name, raw))
            .transpose()
    }

    fn flag(&self, name: &str, default: bool) -> ApiResult<bool> {
        self.get(name)
            .map_or(Ok(default), |raw| api::parse_bool(name, raw))
    }
}

fn doc_id(path: Result<Path<String>, PathRejection>) -> ApiResult<String> {
    path.map(|Path(id)| id)
        .map_err(|rejection| ApiError::bad_request(rejection.body_text()))
}

/// `/note/<id>` is an alias of `/note:<id>`, likewise for books, tags and files.
fn alias_id(path: Result<Path<(String, String)>, PathRejection>) -> ApiResult<String> {
    let Path((kind, id)) =
        path.map_err(|rejection| ApiError::bad_request(rejection.body_text()))?;
    let doc_type = DocType::from_name(&kind).ok_or_else(ApiError::not_found)?;
    Ok(with_prefix(doc_type, &id))
}

/// What `GET /` says the API is compatible with. Clients may check it, so
/// it stays exactly as written.
pub const COMPATIBLE_WITH: &str = "inkdrop-local-server";

async fn meta() -> Json<Value> {
    Json(json!({
        "name": "Kanso",
        "version": env!("CARGO_PKG_VERSION"),
        "compatibleWith": COMPATIBLE_WITH,
    }))
}

async fn list_notes(State(state): State<AppState>, params: Params) -> JsonResult {
    let params = QueryParams::parse(params)?;
    let query = NoteQuery {
        keyword: params.get("keyword").unwrap_or_default().to_owned(),
        limit: Some(params.count("limit")?.unwrap_or(api::DEFAULT_NOTE_LIMIT)),
        skip: params.count("skip")?.unwrap_or(0),
        sort: params.get("sort").map(api::parse_sort).unwrap_or_default(),
        descending: params.flag("descending", true)?,
    };
    let notes = blocking(&state, move |store| api::list_notes(store, &query)).await?;
    Ok(Json(Value::Array(notes)))
}

/// `GET /loops?bookId=&kind=&limit=`: open loops (things said in notes that
/// were not checkboxes). Locked notes and the trash have none.
async fn list_loops(State(state): State<AppState>, params: Params) -> JsonResult {
    let params = QueryParams::parse(params)?;
    let limit = params.count("limit")?;
    let book_id = params.get("bookId").map(str::to_owned);
    let kind =
        match params.get("kind") {
            None => None,
            Some(raw) => Some(kanso_core::loops::LoopKind::parse(raw).ok_or_else(|| {
                ApiError::bad_request("'kind' must be mine, waiting or question.")
            })?),
        };
    let loops = blocking(&state, move |store| {
        api::list_loops(store, book_id, kind, limit)
    })
    .await?;
    Ok(Json(Value::Array(loops)))
}

async fn list_collection(
    State(state): State<AppState>,
    params: Params,
    doc_type: DocType,
) -> JsonResult {
    let params = QueryParams::parse(params)?;
    let limit = params.count("limit")?;
    let skip = params.count("skip")?.unwrap_or(0);
    let docs = blocking(&state, move |store| {
        api::list_docs(store, doc_type, limit, skip)
    })
    .await?;
    Ok(Json(Value::Array(docs)))
}

async fn post_collection(
    State(state): State<AppState>,
    body: Body,
    doc_type: DocType,
) -> JsonResult {
    let body = json_body(body)?;
    Ok(Json(
        blocking(&state, move |store| api::upsert(store, doc_type, body)).await?,
    ))
}

async fn get_doc(
    State(state): State<AppState>,
    path: Result<Path<String>, PathRejection>,
    params: Params,
) -> JsonResult {
    read(state, doc_id(path)?, params).await
}

async fn get_alias(
    State(state): State<AppState>,
    path: Result<Path<(String, String)>, PathRejection>,
    params: Params,
) -> JsonResult {
    read(state, alias_id(path)?, params).await
}

async fn read(state: AppState, id: String, params: Params) -> JsonResult {
    let params = QueryParams::parse(params)?;
    let rev = params.get("rev").map(str::to_owned);
    let attachments = params.flag("attachments", false)?;
    let doc = blocking(&state, move |store| {
        api::get_doc(store, &id, rev.as_deref(), attachments)
    })
    .await?;
    Ok(Json(doc))
}

async fn post_doc(
    State(state): State<AppState>,
    path: Result<Path<String>, PathRejection>,
    body: Body,
) -> JsonResult {
    merge(state, doc_id(path)?, body).await
}

async fn post_alias(
    State(state): State<AppState>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Body,
) -> JsonResult {
    merge(state, alias_id(path)?, body).await
}

async fn merge(state: AppState, id: String, body: Body) -> JsonResult {
    let body = json_body(body)?;
    Ok(Json(
        blocking(&state, move |store| api::merge(store, &id, body)).await?,
    ))
}

async fn delete_doc(
    State(state): State<AppState>,
    path: Result<Path<String>, PathRejection>,
) -> JsonResult {
    let id = doc_id(path)?;
    Ok(Json(
        blocking(&state, move |store| api::delete(store, &id)).await?,
    ))
}

async fn delete_alias(
    State(state): State<AppState>,
    path: Result<Path<(String, String)>, PathRejection>,
) -> JsonResult {
    let id = alias_id(path)?;
    Ok(Json(
        blocking(&state, move |store| api::delete(store, &id)).await?,
    ))
}

async fn changes(State(state): State<AppState>, params: Params) -> JsonResult {
    let params = QueryParams::parse(params)?;
    let since =
        match params.get("since") {
            None => Since::Seq(0),
            Some("now") => Since::Now,
            Some(raw) => Since::Seq(raw.trim().parse().map_err(|_| {
                ApiError::bad_request("'since' must be a sequence number or \"now\".")
            })?),
        };
    let limit = params.count("limit")?;
    let descending = params.flag("descending", false)?;
    let include_docs = params.flag("include_docs", false)?;
    Ok(Json(
        blocking(&state, move |store| {
            api::changes(store, since, limit, descending, include_docs)
        })
        .await?,
    ))
}

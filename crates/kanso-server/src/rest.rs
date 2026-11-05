//! The REST endpoints of the local-server API.

use std::collections::HashMap;

use axum::body::Bytes;
use axum::extract::rejection::{BytesRejection, PathRejection, QueryRejection};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Extension, Json, Router};
use kanso_core::DocType;
use kanso_core::id::with_prefix;
use serde_json::{Value, json};

use crate::AppState;
use crate::api::{self, NoteQuery, Since};
use crate::error::{ApiError, ApiResult};
use crate::keyed::{self, Key};
use crate::scope::Caller;

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
            get(list_notes).post(
                |state: State<AppState>, caller: Extension<Caller>, body: Body| {
                    post_collection(state, caller, body, DocType::Note)
                },
            ),
        );
    for doc_type in [DocType::Book, DocType::Tag, DocType::File] {
        router = router.route(
            &format!("/{}", doc_type.collection()),
            get(
                move |state: State<AppState>, caller: Extension<Caller>, params: Params| {
                    list_collection(state, caller, params, doc_type)
                },
            )
            .post(
                move |state: State<AppState>, caller: Extension<Caller>, body: Body| {
                    post_collection(state, caller, body, doc_type)
                },
            ),
        );
    }
    router
        .route("/{id}", get(get_doc).post(post_doc).delete(delete_doc))
        .route(
            "/{kind}/{id}",
            get(get_alias).post(post_alias).delete(delete_alias),
        )
}

/// Refuses a write by a key that may not write, before the route runs, so
/// a write route is never reached by a read or suggest key. `/mcp` is
/// POST for every key: its tools are gated one by one.
pub(crate) async fn write_gate(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::http::Method;
    use axum::response::IntoResponse as _;
    let reads = matches!(*request.method(), Method::GET | Method::HEAD);
    if !reads
        && request.uri().path() != crate::auth::MCP_PATH
        && let Some(Caller::Key { access, .. }) = request.extensions().get::<Caller>()
        && *access != kanso_store::KeyAccess::Write
    {
        return ApiError::forbidden(crate::keyed::cannot_write(*access)).into_response();
    }
    next.run(request).await
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

async fn list_notes(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    params: Params,
) -> JsonResult {
    let params = QueryParams::parse(params)?;
    let query = NoteQuery {
        keyword: params.get("keyword").unwrap_or_default().to_owned(),
        limit: Some(params.count("limit")?.unwrap_or(api::DEFAULT_NOTE_LIMIT)),
        skip: params.count("skip")?.unwrap_or(0),
        sort: params.get("sort").map(api::parse_sort).unwrap_or_default(),
        descending: params.flag("descending", true)?,
    };
    blocking_as(&state, caller, move |store, key| {
        keyed::list_notes(store, key, &query)
    })
    .await
    .map(|notes| Json(Value::Array(notes)))
}

/// `GET /loops?bookId=&kind=&limit=`: open loops (things said in notes that
/// were not checkboxes). Locked notes and the trash have none.
async fn list_loops(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    params: Params,
) -> JsonResult {
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
    blocking_as(&state, caller, move |store, key| {
        keyed::list_loops(store, key, book_id, kind, limit)
    })
    .await
    .map(|loops| Json(Value::Array(loops)))
}

async fn list_collection(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    params: Params,
    doc_type: DocType,
) -> JsonResult {
    let params = QueryParams::parse(params)?;
    let limit = params.count("limit")?;
    let skip = params.count("skip")?.unwrap_or(0);
    blocking_as(&state, caller, move |store, key| {
        keyed::list_docs(store, key, doc_type, limit, skip)
    })
    .await
    .map(|docs| Json(Value::Array(docs)))
}

async fn post_collection(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    body: Body,
    doc_type: DocType,
) -> JsonResult {
    Key::of(&caller)?.require_write()?;
    let body = json_body(body)?;
    blocking_as(&state, caller, move |store, key| {
        let vis = key.visibility(store)?;
        let fields = body.as_object().cloned().unwrap_or_default();
        let id = fields.get("_id").and_then(Value::as_str);
        keyed::check_write(store, key, &vis, doc_type, id, &fields)?;
        api::upsert(store, doc_type, body)
    })
    .await
    .map(Json)
}

async fn get_doc(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    path: Result<Path<String>, PathRejection>,
    params: Params,
) -> JsonResult {
    read(state, caller, doc_id(path)?, params).await
}

async fn get_alias(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    path: Result<Path<(String, String)>, PathRejection>,
    params: Params,
) -> JsonResult {
    read(state, caller, alias_id(path)?, params).await
}

async fn read(state: AppState, caller: Caller, id: String, params: Params) -> JsonResult {
    let params = QueryParams::parse(params)?;
    let rev = params.get("rev").map(str::to_owned);
    let attachments = params.flag("attachments", false)?;
    blocking_as(&state, caller, move |store, key| {
        keyed::get_doc(store, key, &id, rev.as_deref(), attachments)
    })
    .await
    .map(Json)
}

async fn post_doc(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    path: Result<Path<String>, PathRejection>,
    body: Body,
) -> JsonResult {
    merge(state, caller, doc_id(path)?, body).await
}

async fn post_alias(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Body,
) -> JsonResult {
    merge(state, caller, alias_id(path)?, body).await
}

async fn merge(state: AppState, caller: Caller, id: String, body: Body) -> JsonResult {
    Key::of(&caller)?.require_write()?;
    let body = json_body(body)?;
    blocking_as(&state, caller, move |store, key| {
        let doc_type = DocType::of_id(&id).ok_or_else(ApiError::not_found)?;
        let vis = key.visibility(store)?;
        let fields = body.as_object().cloned().unwrap_or_default();
        keyed::check_write(store, key, &vis, doc_type, Some(&id), &fields)?;
        api::merge(store, &id, body)
    })
    .await
    .map(Json)
}

async fn delete_doc(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    path: Result<Path<String>, PathRejection>,
) -> JsonResult {
    remove(state, caller, doc_id(path)?).await
}

async fn delete_alias(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    path: Result<Path<(String, String)>, PathRejection>,
) -> JsonResult {
    remove(state, caller, alias_id(path)?).await
}

async fn remove(state: AppState, caller: Caller, id: String) -> JsonResult {
    Key::of(&caller)?.require_write()?;
    blocking_as(&state, caller, move |store, key| {
        let doc_type = DocType::of_id(&id).ok_or_else(ApiError::not_found)?;
        let vis = key.visibility(store)?;
        keyed::check_write(store, key, &vis, doc_type, Some(&id), &Default::default())?;
        api::delete(store, &id)
    })
    .await
    .map(Json)
}

async fn changes(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    params: Params,
) -> JsonResult {
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
    blocking_as(&state, caller, move |store, key| {
        keyed::changes(store, key, since, limit, descending, include_docs)
    })
    .await
    .map(Json)
}

/// [`blocking`] for the key behind `caller`.
async fn blocking_as<T: Send + 'static>(
    state: &AppState,
    caller: Caller,
    f: impl FnOnce(&kanso_store::Store, &Key) -> ApiResult<T> + Send + 'static,
) -> ApiResult<T> {
    blocking(state, move |store| f(store, &Key::of(&caller)?)).await
}

//! `kanso capture "text"`: saves a note to the Inbox (or the notebook
//! `--book` names) from a terminal, a script or a desktop shortcut. With no
//! text (or `-`) it reads standard input, so `pbpaste | kanso capture`
//! works.
//!
//! While Kanso is open it holds the data folder, so the note goes through
//! its local API and shows at once; otherwise it is written to the store
//! directly. The title is the text's first line (`capture::split_capture`).

use std::io::Read as _;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::Path;
use std::time::Duration;

use anyhow::{Context as _, Result, bail};
use kanso_core::roles::{ROLE_FIELD, Role};
use kanso_store::{Origin, Store, lock_data_dir};
use serde_json::{Value, json};

use crate::capture::{save_capture, split_capture};
use crate::config::ServerSettings;

pub const USAGE: &str = "\
Usage: kanso capture [--book <name or id>] [text…]

Saves a note to the Inbox, or to the notebook --book names. The first line
is its title. With no text, or `-`, the text is read from standard input.";

/// The notebook `wanted` names among `books` (`(id, name)`, outside the
/// trash): by ID, else by name ignoring case. An error says why none
/// matches, or that several share the name. Pure.
pub fn pick_book<'a>(
    books: &'a [(String, String)],
    wanted: &str,
) -> Result<&'a (String, String), String> {
    let wanted = wanted.trim();
    if let Some(book) = books.iter().find(|(id, _)| id == wanted) {
        return Ok(book);
    }
    let named: Vec<_> = books
        .iter()
        .filter(|(_, name)| name.trim().to_lowercase() == wanted.to_lowercase())
        .collect();
    match named.as_slice() {
        [book] => Ok(book),
        [] => Err(format!("there's no notebook named “{wanted}”")),
        several => Err(format!(
            "{} notebooks are named “{wanted}”; use the ID of one: {}",
            several.len(),
            several
                .iter()
                .map(|(id, _)| id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

/// `--book <name or id>` taken out of the arguments.
fn book_arg(rest: &[String]) -> Result<(Option<String>, Vec<String>)> {
    let mut book = None;
    let mut words = Vec::new();
    let mut args = rest.iter();
    while let Some(arg) = args.next() {
        if arg == "--book" {
            book = Some(
                args.next()
                    .context("--book needs a notebook's name or ID")?
                    .clone(),
            );
        } else if let Some(value) = arg.strip_prefix("--book=") {
            book = Some(value.to_owned());
        } else {
            words.push(arg.clone());
        }
    }
    Ok((book, words))
}

pub fn run_cli(data_dir: &Path, rest: &[String]) -> Result<()> {
    if rest.iter().any(|arg| arg == "help" || arg == "--help") {
        println!("{USAGE}");
        return Ok(());
    }
    let (book, rest) = book_arg(rest)?;
    let text = if rest.is_empty() || rest == ["-"] {
        let mut text = String::new();
        std::io::stdin()
            .read_to_string(&mut text)
            .context("reading standard input")?;
        text
    } else {
        rest.join(" ")
    };
    let Some((title, _)) = split_capture(&text) else {
        bail!("nothing to capture: the text is empty\n\n{USAGE}");
    };
    std::fs::create_dir_all(data_dir)
        .with_context(|| format!("creating {}", data_dir.display()))?;
    let place = match lock_data_dir(data_dir)? {
        Some(_lock) => {
            let store = Store::open(crate::paths::db_path(data_dir))?;
            let (book, place) = match &book {
                Some(wanted) => {
                    let books: Vec<(String, String)> =
                        store.books()?.into_iter().map(|b| (b.id, b.name)).collect();
                    let (id, name) = pick_book(&books, wanted).map_err(anyhow::Error::msg)?;
                    (Some(id.clone()), name.clone())
                }
                None => (None, "the Inbox".to_owned()),
            };
            save_capture(&store, &text, book.as_deref(), Origin::Api)?;
            place
        }
        None => {
            let config = crate::load_config(data_dir)?;
            if !config.server.enabled {
                bail!(
                    "Kanso is open and its local API is off. Turn it on in Settings › Local API, or quit Kanso, to capture from here."
                );
            }
            Api::new(&config.server)?.capture(&text, book.as_deref())?.1
        }
    };
    println!("Saved “{title}” to {place}.");
    Ok(())
}

/// The variable that holds the key `kanso capture` signs in with while
/// Kanso is open.
const KEY_VAR: &str = "KANSO_API_KEY";

/// A running Kanso's local API.
pub struct Api {
    base: String,
    auth: String,
    agent: ureq::Agent,
}

impl Api {
    pub fn new(settings: &ServerSettings) -> Result<Api> {
        let ip: IpAddr = settings
            .bind_address
            .trim()
            .parse()
            .with_context(|| format!("“{}” isn't an IP address", settings.bind_address))?;
        // Listening everywhere answers on this machine's own address too.
        let ip = match ip {
            IpAddr::V4(ip) if ip.is_unspecified() => IpAddr::V4(Ipv4Addr::LOCALHOST),
            IpAddr::V6(ip) if ip.is_unspecified() => IpAddr::V6(Ipv6Addr::LOCALHOST),
            ip => ip,
        };
        let host = match ip {
            IpAddr::V4(ip) => ip.to_string(),
            IpAddr::V6(ip) => format!("[{ip}]"),
        };
        // The local link (issue #243, step 6) replaces this; until then a
        // capture while Kanso is open signs in with a key that can write.
        let key = std::env::var(KEY_VAR)
            .ok()
            .filter(|key| !key.trim().is_empty());
        let Some(key) = key else {
            bail!(
                "Kanso is open, and capturing into it needs an API key. Set {KEY_VAR} to a key that can read and write (Kanso › Settings › Local API › API keys), or quit Kanso to capture from here."
            );
        };
        Ok(Api::at(
            &format!("http://{host}:{}", settings.port),
            key.trim(),
        ))
    }

    pub fn at(base: &str, key: &str) -> Api {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .max_redirects(0)
            .timeout_global(Some(Duration::from_secs(20)))
            .build()
            .into();
        Api {
            base: base.trim_end_matches('/').to_owned(),
            auth: format!("Bearer {key}"),
            agent,
        }
    }

    fn call(&self, method: &str, path: &str, body: Option<&Value>) -> Result<(u16, Value)> {
        let url = format!("{}{path}", self.base);
        let response = match body {
            None => self
                .agent
                .get(&url)
                .header("authorization", &self.auth)
                .call(),
            Some(body) => {
                debug_assert_eq!(method, "POST");
                self.agent
                    .post(&url)
                    .header("authorization", &self.auth)
                    .header("content-type", "application/json")
                    .send(serde_json::to_vec(body)?.as_slice())
            }
        };
        let mut response = response
            .with_context(|| format!("couldn't reach Kanso's local API at {}", self.base))?;
        let status = response.status().as_u16();
        let text = response.body_mut().read_to_string().unwrap_or_default();
        if status == 401 {
            bail!(
                "Kanso's local API didn't accept the key in {KEY_VAR}. It may have been revoked. Create a new one in Kanso › Settings › Local API › API keys."
            );
        }
        Ok((status, serde_json::from_str(&text).unwrap_or(Value::Null)))
    }

    fn books(&self) -> Result<Value> {
        let (status, books) = self.call("GET", "/books", None)?;
        if status != 200 {
            bail!("Kanso's local API answered {status} for the notebooks");
        }
        Ok(books)
    }

    /// The Inbox's ID, made through the API when there is none.
    fn inbox(&self) -> Result<String> {
        let books = self.books()?;
        let role = Role::Inbox;
        let found = books.as_array().and_then(|books| {
            let id = |book: &Value| book["_id"].as_str().map(str::to_owned);
            books
                .iter()
                .find(|b| b["_id"] == role.book_id())
                .or_else(|| books.iter().find(|b| b[ROLE_FIELD] == role.as_str()))
                .and_then(id)
        });
        if let Some(id) = found {
            return Ok(id);
        }
        let book = json!({
            "_id": role.book_id(),
            "name": role.default_name(),
            ROLE_FIELD: role.as_str(),
        });
        let (status, made) = self.call("POST", "/books", Some(&book))?;
        match made["id"].as_str() {
            Some(id) if status == 200 => Ok(id.to_owned()),
            _ => bail!("Kanso's local API couldn't make the Inbox ({status})"),
        }
    }

    /// Saves `text` as a note in the Inbox, or the notebook `book` names;
    /// returns its ID and where it went.
    pub fn capture(&self, text: &str, book: Option<&str>) -> Result<(String, String)> {
        let Some((title, body)) = split_capture(text) else {
            bail!("nothing to capture");
        };
        let (book, place) = match book {
            Some(wanted) => {
                let books: Vec<(String, String)> = self
                    .books()?
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|b| {
                        Some((
                            b["_id"].as_str()?.to_owned(),
                            b["name"].as_str()?.to_owned(),
                        ))
                    })
                    .collect();
                let (id, name) = pick_book(&books, wanted).map_err(anyhow::Error::msg)?;
                (id.clone(), name.clone())
            }
            None => (self.inbox()?, "the Inbox".to_owned()),
        };
        let note = json!({"title": title, "body": body, "bookId": book});
        let (status, made) = self.call("POST", "/notes", Some(&note))?;
        match made["id"].as_str() {
            Some(id) if status == 200 => Ok((id.to_owned(), place)),
            _ => bail!("Kanso's local API couldn't save the note ({status}): {made}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::Api;
    use kanso_store::Store;

    /// Through a running API on an ephemeral port: the Inbox is made once,
    /// and both notes land in it.
    #[test]
    fn captures_through_the_local_api() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
        let keys = kanso_server::KeyStore::open(store.clone()).unwrap();
        let key = keys
            .mint(kanso_server::NewKey {
                name: "Capture".into(),
                client: "other".into(),
                access: kanso_server::KeyAccess::Write,
                books: None,
            })
            .unwrap()
            .key;
        let handle = kanso_server::spawn(
            store.clone(),
            kanso_server::ServerConfig {
                bind: "127.0.0.1:0".parse().unwrap(),
            },
            keys,
        );
        let addr = handle.bind_result().unwrap();
        let api = Api::at(&format!("http://{addr}"), &key);
        let first = api.capture("Buy milk\nand eggs", None).unwrap().0;
        let second = api.capture("Call Sam", None).unwrap().0;
        let note = store.note(&first).unwrap().unwrap();
        assert_eq!(
            (note.title.as_str(), note.body.as_str()),
            ("Buy milk", "and eggs")
        );
        assert_eq!(note.book_id, "book:inbox");
        assert_eq!(store.note(&second).unwrap().unwrap().book_id, "book:inbox");
        assert_eq!(store.books().unwrap().len(), 1);
        let wrong = Api::at(&format!("http://{addr}"), "kcl_nope");
        assert!(
            wrong
                .capture("x", None)
                .unwrap_err()
                .to_string()
                .contains("didn't accept")
        );
        // By a notebook's name, any case.
        let ideas = store
            .put(
                kanso_core::DocType::Book,
                serde_json::json!({"name": "Ideas"}),
                kanso_store::Put::Upsert,
                kanso_store::Origin::Ui,
            )
            .unwrap()
            .id;
        let (id, place) = api.capture("Kites", Some("ideas")).unwrap();
        assert_eq!(place, "Ideas");
        assert_eq!(store.note(&id).unwrap().unwrap().book_id, ideas);
        assert!(api.capture("Kites", Some("Nope")).is_err());
    }

    /// Without the app: straight into the store, the Inbox made on the way.
    #[test]
    fn captures_into_the_store_without_the_app() {
        let dir = tempfile::tempdir().unwrap();
        super::run_cli(dir.path(), &["Idea:".into(), "kites".into()]).unwrap();
        let store = Store::open(crate::paths::db_path(dir.path())).unwrap();
        let books = store.books().unwrap();
        assert_eq!(books.len(), 1);
        assert_eq!(books[0].id, "book:inbox");
        assert!(super::run_cli(dir.path(), &[" ".into()]).is_err());
        // --book by ID or name; a name nothing has is an error, and
        // nothing is saved.
        let store = Store::open(crate::paths::db_path(dir.path())).unwrap();
        let ideas = store
            .put(
                kanso_core::DocType::Book,
                serde_json::json!({"name": "Ideas"}),
                kanso_store::Put::Upsert,
                kanso_store::Origin::Ui,
            )
            .unwrap()
            .id;
        drop(store);
        super::run_cli(
            dir.path(),
            &["--book".into(), "IDEAS".into(), "Kites".into()],
        )
        .unwrap();
        super::run_cli(dir.path(), &[format!("--book={ideas}"), "Gliders".into()]).unwrap();
        assert!(super::run_cli(dir.path(), &["--book".into(), "Nope".into(), "x".into()]).is_err());
        assert!(super::run_cli(dir.path(), &["x".into(), "--book".into()]).is_err());
        let store = Store::open(crate::paths::db_path(dir.path())).unwrap();
        let titles: Vec<String> = store
            .search_summaries(&Default::default())
            .unwrap()
            .into_iter()
            .filter(|n| n.book_id == ideas)
            .map(|n| n.title)
            .collect();
        assert_eq!(titles.len(), 2, "{titles:?}");
    }

    #[test]
    fn picks_a_book_by_id_or_name() {
        let books = vec![
            ("book:a".to_owned(), "Ideas".to_owned()),
            ("book:b".to_owned(), "Work".to_owned()),
            ("book:c".to_owned(), "work".to_owned()),
        ];
        assert_eq!(super::pick_book(&books, "book:b").unwrap().0, "book:b");
        assert_eq!(super::pick_book(&books, " ideas ").unwrap().0, "book:a");
        assert!(
            super::pick_book(&books, "work")
                .unwrap_err()
                .contains("book:b, book:c")
        );
        assert!(
            super::pick_book(&books, "nope")
                .unwrap_err()
                .contains("no notebook")
        );
    }
}

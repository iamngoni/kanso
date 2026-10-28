//! The theme gallery: the signed index, the checks before anything is
//! installed, installed themes and their tokens, and the cache.
//!
//! The index and the files are served from memory by a transport the tests
//! script, signed with a key made for the test; the real crypto, the real
//! parser and the real installer run.

mod common;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use common::*;
use kanso_engine::theme::Tokens;
use kanso_engine::theme::builtin::BUILTIN;
use kanso_engine::theme::file::ThemeFile;
use kanso_gallery::{Appearance, FileRef, HeadlineColors, Index, Kind, Revoked, Theme, sha256_hex};
use kanso_mobile::{Gallery, GalleryItem, GalleryKind, KansoError, Renderer};
use kanso_update::fetch::Response;
use kanso_update::{FetchError, Transport};
use proptest::prelude::*;

/// A server that answers from a table, can go offline and answers a
/// conditional request for an unchanged body with a `304`.
#[derive(Default)]
struct Net {
    files: Mutex<HashMap<String, Vec<u8>>>,
    etags: Mutex<HashMap<String, String>>,
    offline: AtomicBool,
    requests: Mutex<Vec<String>>,
}

impl Net {
    fn put(&self, url: &str, bytes: Vec<u8>) {
        self.files.lock().unwrap().insert(url.to_owned(), bytes);
    }

    fn put_file(&self, path: &str, bytes: Vec<u8>) {
        self.put(&kanso_gallery::file_url(path), bytes);
    }

    /// Serves `index` signed by `key`.
    fn publish(&self, index: &Index, key: &ed25519_dalek::SigningKey) {
        let (bytes, signature) = sign(index, key);
        self.publish_raw(bytes, signature);
    }

    fn publish_raw(&self, bytes: Vec<u8>, signature: Vec<u8>) {
        self.put(kanso_gallery::INDEX_URL, bytes);
        self.put(kanso_gallery::SIGNATURE_URL, signature);
    }

    fn requests_for(&self, url: &str) -> usize {
        self.requests
            .lock()
            .unwrap()
            .iter()
            .filter(|asked| asked.as_str() == url)
            .count()
    }

    fn answer(&self, url: &str, etag: Option<&str>) -> Result<Response, FetchError> {
        self.requests.lock().unwrap().push(url.to_owned());
        if self.offline.load(Ordering::SeqCst) {
            return Err(FetchError::Offline);
        }
        let files = self.files.lock().unwrap();
        let Some(body) = files.get(url) else {
            return Ok(response(404, None, Vec::new()));
        };
        let current = sha256_hex(body);
        self.etags
            .lock()
            .unwrap()
            .insert(url.to_owned(), current.clone());
        if etag == Some(current.as_str()) {
            return Ok(response(304, Some(current), Vec::new()));
        }
        Ok(response(200, Some(current), body.clone()))
    }
}

fn response(status: u16, etag: Option<String>, body: Vec<u8>) -> Response {
    Response {
        status,
        location: None,
        rate_limit_remaining: None,
        etag,
        body: Box::new(std::io::Cursor::new(body)),
    }
}

impl Transport for Net {
    fn get(&self, url: &str) -> Result<Response, FetchError> {
        self.answer(url, None)
    }

    fn get_if_none_match(&self, url: &str, etag: Option<&str>) -> Result<Response, FetchError> {
        self.answer(url, etag)
    }
}

fn sign(index: &Index, key: &ed25519_dalek::SigningKey) -> (Vec<u8>, Vec<u8>) {
    let bytes = serde_json::to_vec(index).unwrap();
    let signature = kanso_update::sign_feed(&bytes, key).into_bytes();
    (bytes, signature)
}

/// A phone whose gallery reads `net` and trusts `key`.
struct Setup {
    phone: Phone,
    gallery: Gallery,
    net: Arc<Net>,
    key: ed25519_dalek::SigningKey,
}

fn setup() -> Setup {
    let (key, _) = kanso_update::feed::generate_signing_key();
    setup_with(key)
}

fn setup_with(key: ed25519_dalek::SigningKey) -> Setup {
    let phone = lone_phone();
    let net = Arc::new(Net::default());
    let gallery = phone.library.gallery();
    gallery.use_source_for_test(net.clone(), vec![key.verifying_key()]);
    Setup {
        phone,
        gallery,
        net,
        key,
    }
}

impl Setup {
    fn dir(&self) -> PathBuf {
        self.phone.dir.path().join("gallery")
    }

    fn publish(&self, index: &Index) {
        self.net.publish(index, &self.key);
    }
}

/// A Kanso color theme file under another name.
fn color_file(name: &str) -> Vec<u8> {
    let (_, text) = BUILTIN[0];
    let mut value: serde_json::Value = serde_json::from_str(text).unwrap();
    value["name"] = name.into();
    serde_json::to_vec(&value).unwrap()
}

fn color_theme(id: &str, name: &str, bytes: &[u8], version: u64) -> Theme {
    Theme {
        id: id.into(),
        kind: Kind::Color,
        family: None,
        version,
        name: name.into(),
        appearance: Some(Appearance::Light),
        author: "Ada".into(),
        license: "MIT".into(),
        source: "https://github.com/iamngoni/kanso-themes".into(),
        description: "A theme for the tests.".into(),
        min_kanso: "0.1.0".into(),
        file: Some(FileRef {
            path: format!("{id}.json"),
            sha256: sha256_hex(bytes),
            size: bytes.len() as u64,
        }),
        bundle: None,
        preview: Vec::new(),
        colors: Some(HeadlineColors {
            editor: "#FFFFFF".into(),
            text: "#222".into(),
            accent: "#0A84FF".into(),
            sidebar: "#F5F5F5".into(),
            list: "#FAFAFAFF".into(),
        }),
    }
}

fn index_of(serial: u64, themes: Vec<Theme>, revoked: Vec<Revoked>) -> Index {
    Index {
        schema: 1,
        serial,
        generated_at: "2026-10-23T12:00:00Z".into(),
        source_commit: "0123456789abcdef0123456789abcdef01234567".into(),
        signing_key_id: "test".into(),
        themes,
        revoked,
    }
}

/// An index with one color theme, and the theme's file served.
fn dawn(setup: &Setup, serial: u64, version: u64) -> (Index, Vec<u8>) {
    let bytes = color_file("Gallery Dawn");
    let theme = color_theme("gallery-dawn", "Gallery Dawn", &bytes, version);
    setup.net.put_file("gallery-dawn.json", bytes.clone());
    (index_of(serial, vec![theme], Vec::new()), bytes)
}

fn icon_bundle(id: &str) -> Vec<u8> {
    let json = br#"{"schema_version": 1, "name": "Demo"}"#.to_vec();
    let svg = br#"<svg xmlns="http://www.w3.org/2000/svg"><path d="M0 0h4v4z"/></svg>"#.to_vec();
    let manifest = format!(
        r#"{{"files":[{{"path":"{id}/{id}.json","sha256":"{}","size":{}}},{{"path":"{id}/icons/a.svg","sha256":"{}","size":{}}}]}}"#,
        sha256_hex(&json),
        json.len(),
        sha256_hex(&svg),
        svg.len()
    );
    let mut tar = tar::Builder::new(flate2::write::GzEncoder::new(
        Vec::new(),
        flate2::Compression::default(),
    ));
    let mut add = |path: &str, bytes: &[u8]| {
        let mut header = tar::Header::new_gnu();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append_data(&mut header, path, bytes).unwrap();
    };
    add("manifest.json", manifest.as_bytes());
    add(&format!("{id}/{id}.json"), &json);
    add(&format!("{id}/icons/a.svg"), &svg);
    tar.into_inner().unwrap().finish().unwrap()
}

fn icon_theme(id: &str, bundle: &[u8]) -> Theme {
    Theme {
        id: id.into(),
        kind: Kind::Icons,
        family: None,
        version: 1,
        name: "Demo Icons".into(),
        appearance: None,
        author: "Ada".into(),
        license: "MIT".into(),
        source: "https://example.com/icons".into(),
        description: "Icons for the tests.".into(),
        min_kanso: "0.1.0".into(),
        file: None,
        bundle: Some(FileRef {
            path: format!("gallery/icon-themes/{id}.tar.gz"),
            sha256: sha256_hex(bundle),
            size: bundle.len() as u64,
        }),
        preview: Vec::new(),
        colors: None,
    }
}

fn find<'a>(items: &'a [GalleryItem], id: &str) -> &'a GalleryItem {
    items.iter().find(|item| item.id == id).unwrap()
}

fn refused(result: Result<Vec<GalleryItem>, KansoError>) -> String {
    match result {
        Err(err) => err.to_string(),
        Ok(items) => panic!("expected a refusal, got {} items", items.len()),
    }
}

/// Every file and folder under `dir`, relative.
fn tree(dir: &Path) -> Vec<PathBuf> {
    fn walk(base: &Path, at: &Path, out: &mut Vec<PathBuf>) {
        let Ok(read) = std::fs::read_dir(at) else {
            return;
        };
        for entry in read.flatten() {
            let path = entry.path();
            out.push(path.strip_prefix(base).unwrap().to_owned());
            if path.is_dir() {
                walk(base, &path, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

/// Nothing was written for a theme: only the phone's database and the
/// gallery's own bookkeeping are on disk.
fn assert_nothing_installed(setup: &Setup) {
    for path in tree(setup.phone.dir.path()) {
        let name = path.to_string_lossy().into_owned();
        let ok = name.starts_with("library.db")
            || name == "gallery"
            || name == "gallery/meta.json"
            || name == "gallery/index.json"
            || name == "gallery/index.json.sig";
        assert!(ok, "{name} should not exist");
    }
}

#[test]
fn a_signed_index_lists_its_themes() {
    let setup = setup();
    let (mut index, _) = dawn(&setup, 1, 3);
    let bundle = icon_bundle("demo");
    setup
        .net
        .put_file("gallery/icon-themes/demo.tar.gz", bundle.clone());
    index.themes.push(icon_theme("demo", &bundle));
    setup.publish(&index);

    let items = setup.gallery.index(true).unwrap();
    assert_eq!(items.len(), 2);
    let dawn = find(&items, "gallery-dawn");
    assert_eq!(dawn.name, "Gallery Dawn");
    assert_eq!(dawn.kind, GalleryKind::ColorTheme);
    assert_eq!(dawn.author, "Ada");
    assert_eq!(dawn.version, "3");
    assert!(dawn.official);
    assert!(!dawn.dark);
    assert!(dawn.runs_here);
    assert!(!dawn.installed && !dawn.update_available);
    assert_eq!(dawn.license, "MIT");
    assert_eq!(dawn.updated_ms, 1_792_756_800_000);
    let colors = dawn.headline.unwrap();
    assert_eq!(colors.editor, 0xFFFFFF);
    assert_eq!(colors.text, 0x222222);
    assert_eq!(colors.list, 0xFAFAFA);
    let icons = find(&items, "demo");
    assert_eq!(icons.kind, GalleryKind::IconTheme);
    assert!(!icons.official);
    assert_eq!(icons.source.as_deref(), Some("https://example.com/icons"));
}

#[test]
fn a_tampered_index_is_refused_and_nothing_is_kept() {
    let setup = setup();
    let (index, _) = dawn(&setup, 1, 1);
    let (mut bytes, signature) = sign(&index, &setup.key);
    let at = bytes.iter().position(|b| *b == b'1').unwrap();
    bytes[at] = b'2';
    setup.net.publish_raw(bytes, signature);
    let message = refused(setup.gallery.index(true));
    assert!(message.contains("refused"), "{message}");
    assert_nothing_installed(&setup);
    assert!(!setup.dir().join("index.json").exists());
}

#[test]
fn an_index_signed_by_another_key_is_refused() {
    let setup = setup();
    let (index, _) = dawn(&setup, 1, 1);
    let (other, _) = kanso_update::feed::generate_signing_key();
    setup.net.publish(&index, &other);
    refused(setup.gallery.index(true));
    // Nothing ever installs from it, either.
    assert!(matches!(
        setup.gallery.install("gallery-dawn".into()),
        Err(KansoError::Invalid { .. } | KansoError::Network { .. })
    ));
    assert_nothing_installed(&setup);
}

#[test]
fn a_gallery_with_no_trusted_key_is_not_set_up() {
    let setup = setup();
    let (index, _) = dawn(&setup, 1, 1);
    setup.publish(&index);
    setup
        .gallery
        .use_source_for_test(setup.net.clone(), Vec::new());
    let message = refused(setup.gallery.index(true));
    assert!(message.contains("isn't set up"), "{message}");
}

#[test]
fn a_payload_whose_hash_does_not_match_is_refused() {
    let setup = setup();
    let (index, bytes) = dawn(&setup, 1, 1);
    setup.publish(&index);
    setup.gallery.index(true).unwrap();
    // The same length, one byte different.
    let mut other = bytes.clone();
    let at = other.iter().position(|b| *b == b'#').unwrap() + 1;
    other[at] = if other[at] == b'0' { b'1' } else { b'0' };
    setup.net.put_file("gallery-dawn.json", other);
    let message = refused(setup.gallery.install("gallery-dawn".into()));
    assert!(message.contains("didn't match"), "{message}");
    assert!(setup.gallery.installed_themes().unwrap().is_empty());
    assert!(!setup.dir().join("themes/gallery-dawn.json").exists());
    // And one of another length.
    setup
        .net
        .put_file("gallery-dawn.json", bytes[..bytes.len() - 1].to_vec());
    refused(setup.gallery.install("gallery-dawn".into()));
    assert!(!setup.dir().join("themes/gallery-dawn.json").exists());
}

#[test]
fn a_revoked_theme_is_never_installed_and_is_removed_when_the_revocation_arrives() {
    let setup = setup();
    let (index, _) = dawn(&setup, 1, 1);
    setup.publish(&index);
    setup.gallery.index(true).unwrap();
    setup.gallery.install("gallery-dawn".into()).unwrap();
    assert_eq!(setup.gallery.installed_themes().unwrap().len(), 1);

    let revoked = Revoked {
        id: "gallery-dawn".into(),
        reason: "It was withdrawn.".into(),
        since: "2026-10-24T00:00:00Z".into(),
    };
    setup.publish(&index_of(2, Vec::new(), vec![revoked]));
    let items = setup.gallery.index(true).unwrap();
    assert!(items.is_empty());
    assert!(setup.gallery.installed_themes().unwrap().is_empty());
    assert!(!setup.dir().join("themes/gallery-dawn.json").exists());
    // It is no longer listed, so it can't be installed again.
    assert_eq!(
        setup.gallery.install("gallery-dawn".into()).unwrap_err(),
        KansoError::NotFound
    );
}

#[test]
fn a_revoked_theme_the_person_edited_is_kept_but_not_drawn() {
    let setup = setup();
    let (index, _) = dawn(&setup, 1, 1);
    setup.publish(&index);
    setup.gallery.index(true).unwrap();
    setup.gallery.install("gallery-dawn".into()).unwrap();
    let file = setup.dir().join("themes/gallery-dawn.json");
    let mut edited = std::fs::read(&file).unwrap();
    edited.push(b' ');
    std::fs::write(&file, &edited).unwrap();

    let revoked = Revoked {
        id: "gallery-dawn".into(),
        reason: "It was withdrawn.".into(),
        since: "2026-10-24T00:00:00Z".into(),
    };
    setup.publish(&index_of(2, Vec::new(), vec![revoked]));
    let items = setup.gallery.index(true).unwrap();
    assert_eq!(std::fs::read(&file).unwrap(), edited);
    let item = find(&items, "gallery-dawn");
    assert!(item.disabled && item.edited);
    assert!(setup.gallery.installed_themes().unwrap().is_empty());
    assert_eq!(
        setup
            .gallery
            .installed_theme_tokens("gallery-dawn".into())
            .unwrap_err(),
        KansoError::NotFound
    );
}

#[test]
fn an_installed_theme_is_listed_and_its_tokens_are_the_desktops() {
    let setup = setup();
    let (index, bytes) = dawn(&setup, 1, 1);
    setup.publish(&index);
    let items = setup.gallery.install("gallery-dawn".into()).unwrap();
    assert!(find(&items, "gallery-dawn").installed);

    // The file on disk is exactly what the index pinned.
    let file = setup.dir().join("themes/gallery-dawn.json");
    assert_eq!(std::fs::read(&file).unwrap(), bytes);

    let themes = setup.gallery.installed_themes().unwrap();
    assert_eq!(themes.len(), 1);
    let row = &themes[0];
    assert_eq!(row.id, "gallery-dawn");
    assert_eq!(row.name, "Gallery Dawn");
    assert!(!row.dark);

    // The desktop resolves the same file through the engine's theme code.
    let text = String::from_utf8(bytes).unwrap();
    let parsed = ThemeFile::parse(&text).unwrap();
    let resolved = parsed.resolve();
    let desktop = Tokens::fit(&resolved, &resolved, &resolved);
    let tokens = setup
        .gallery
        .installed_theme_tokens("gallery-dawn".into())
        .unwrap();
    assert_eq!(tokens, row.tokens);
    assert_eq!(tokens, Renderer::theme_tokens(text).unwrap());
    assert_eq!(tokens.dark, desktop.dark);
    assert_eq!(tokens.caret, desktop.caret.0);
    assert_eq!(tokens.selection, desktop.selection.rgb.0);
    let ui = |name: &str| tokens.ui.iter().find(|row| row.name == name).unwrap().rgb;
    assert_eq!(ui("editor"), desktop.ui.editor.0);
    assert_eq!(ui("text"), desktop.ui.text.0);
    assert_eq!(ui("accent"), desktop.ui.accent.0);
    assert_eq!(ui("sidebar"), desktop.ui.sidebar.0);
    assert_eq!(tokens.syntax.len(), desktop.syntax.len());
}

#[test]
fn uninstalling_removes_the_theme() {
    let setup = setup();
    let (index, _) = dawn(&setup, 1, 1);
    setup.publish(&index);
    setup.gallery.install("gallery-dawn".into()).unwrap();
    let items = setup.gallery.uninstall("gallery-dawn".into()).unwrap();
    assert!(!find(&items, "gallery-dawn").installed);
    assert!(setup.gallery.installed_themes().unwrap().is_empty());
    assert!(!setup.dir().join("themes/gallery-dawn.json").exists());
    assert_eq!(
        setup.gallery.uninstall("gallery-dawn".into()).unwrap_err(),
        KansoError::NotFound
    );
    assert_eq!(
        setup
            .gallery
            .installed_theme_tokens("gallery-dawn".into())
            .unwrap_err(),
        KansoError::NotFound
    );
}

#[test]
fn the_last_good_index_serves_when_the_fetch_fails() {
    let setup = setup();
    let (index, _) = dawn(&setup, 1, 1);
    setup.publish(&index);
    let first = setup.gallery.index(true).unwrap();

    setup.net.offline.store(true, Ordering::SeqCst);
    assert_eq!(setup.gallery.index(true).unwrap(), first);
    assert_eq!(setup.gallery.index(false).unwrap(), first);

    // A server that answers with an error is no different.
    setup.net.offline.store(false, Ordering::SeqCst);
    setup.net.files.lock().unwrap().clear();
    assert!(matches!(
        setup.gallery.index(true),
        Err(KansoError::Network { .. })
    ));
    assert_eq!(setup.gallery.index(false).unwrap(), first);

    // The cache survives a restart of the gallery handle.
    let again = setup.phone.library.gallery();
    again.use_source_for_test(setup.net.clone(), vec![setup.key.verifying_key()]);
    setup.net.offline.store(true, Ordering::SeqCst);
    assert_eq!(again.index(false).unwrap(), first);
}

#[test]
fn with_no_index_and_no_network_there_is_an_error() {
    let setup = setup();
    setup.net.offline.store(true, Ordering::SeqCst);
    assert!(matches!(
        setup.gallery.index(false),
        Err(KansoError::Network { .. })
    ));
    assert!(matches!(
        setup.gallery.install("gallery-dawn".into()),
        Err(KansoError::Network { .. })
    ));
}

#[test]
fn an_unchanged_index_is_not_downloaded_twice() {
    let setup = setup();
    let (index, _) = dawn(&setup, 1, 1);
    setup.publish(&index);
    let first = setup.gallery.index(true).unwrap();
    let second = setup.gallery.index(true).unwrap();
    assert_eq!(first, second);
    assert_eq!(setup.net.requests_for(kanso_gallery::INDEX_URL), 2);
    assert_eq!(
        setup.net.requests_for(kanso_gallery::SIGNATURE_URL),
        1,
        "a 304 verifies the cached signature instead of fetching it"
    );

    // A cache that no longer verifies is fetched afresh.
    std::fs::write(setup.dir().join("index.json.sig"), b"garbage").unwrap();
    assert_eq!(setup.gallery.index(true).unwrap(), first);
    assert_eq!(setup.net.requests_for(kanso_gallery::SIGNATURE_URL), 2);
}

#[test]
fn an_older_index_and_a_rewritten_one_are_refused() {
    let setup = setup();
    let (index, _) = dawn(&setup, 2, 1);
    setup.publish(&index);
    let good = setup.gallery.index(true).unwrap();

    // Older: a rollback.
    setup.publish(&index_of(1, Vec::new(), Vec::new()));
    let message = refused(setup.gallery.index(true));
    assert!(message.contains("refused"), "{message}");

    // The same serial, other bytes: a rewrite.
    let mut rewritten = index.clone();
    rewritten.generated_at = "2026-10-25T12:00:00Z".into();
    setup.publish(&rewritten);
    refused(setup.gallery.index(true));

    // Neither changed what the phone knows.
    assert_eq!(setup.gallery.index(false).unwrap(), good);
}

#[test]
fn a_catalog_from_a_newer_kanso_says_so() {
    let setup = setup();
    let mut index = index_of(1, Vec::new(), Vec::new());
    index.schema = 2;
    setup.publish(&index);
    let message = refused(setup.gallery.index(true));
    assert!(message.contains("newer Kanso"), "{message}");
}

#[test]
fn a_theme_that_needs_a_newer_kanso_is_listed_and_not_installed() {
    let setup = setup();
    let bytes = color_file("Gallery Dawn");
    let mut theme = color_theme("gallery-dawn", "Gallery Dawn", &bytes, 1);
    theme.min_kanso = "99.0.0".into();
    setup.net.put_file("gallery-dawn.json", bytes);
    setup.publish(&index_of(1, vec![theme], Vec::new()));
    let items = setup.gallery.index(true).unwrap();
    assert!(!find(&items, "gallery-dawn").runs_here);
    let message = refused(setup.gallery.install("gallery-dawn".into()));
    assert!(message.contains("newer Kanso"), "{message}");
    assert!(setup.gallery.installed_themes().unwrap().is_empty());
}

#[test]
fn an_update_is_offered_and_an_edited_copy_is_not_overwritten() {
    let setup = setup();
    let (index, _) = dawn(&setup, 1, 1);
    setup.publish(&index);
    setup.gallery.install("gallery-dawn".into()).unwrap();

    // Version 2 has the same name and a changed color.
    let mut value: serde_json::Value = serde_json::from_slice(&color_file("Gallery Dawn")).unwrap();
    value["ui"]["accent"] = "#0A5FCC".into();
    let newer = serde_json::to_vec(&value).unwrap();
    let theme = color_theme("gallery-dawn", "Gallery Dawn", &newer, 2);
    setup.net.put_file("gallery-dawn.json", newer.clone());
    setup.publish(&index_of(2, vec![theme], Vec::new()));
    let items = setup.gallery.index(true).unwrap();
    assert!(find(&items, "gallery-dawn").update_available);
    // Listing alone never changes an installed theme.
    assert_ne!(
        std::fs::read(setup.dir().join("themes/gallery-dawn.json")).unwrap(),
        newer
    );

    let items = setup.gallery.install("gallery-dawn".into()).unwrap();
    let item = find(&items, "gallery-dawn");
    assert!(!item.update_available && item.installed);
    assert_eq!(
        std::fs::read(setup.dir().join("themes/gallery-dawn.json")).unwrap(),
        newer
    );

    // Edit it; version 3 is offered but refused.
    let file = setup.dir().join("themes/gallery-dawn.json");
    std::fs::write(&file, b"{\"edited\":true}").unwrap();
    let theme = color_theme("gallery-dawn", "Gallery Dawn", &newer, 3);
    setup.publish(&index_of(3, vec![theme], Vec::new()));
    let items = setup.gallery.index(true).unwrap();
    assert!(find(&items, "gallery-dawn").edited);
    assert!(!find(&items, "gallery-dawn").update_available);
    let message = refused(setup.gallery.install("gallery-dawn".into()));
    assert!(message.contains("edited"), "{message}");
    assert_eq!(std::fs::read(&file).unwrap(), b"{\"edited\":true}");
}

#[test]
fn a_built_ins_name_and_a_theme_that_does_not_read_are_refused() {
    let setup = setup();
    // A copy of a built-in under its own name.
    let (_, text) = BUILTIN[1];
    let builtin = ThemeFile::parse(text).unwrap().name;
    let copy = text.as_bytes().to_vec();
    let shadow = color_theme("shadow", &builtin, &copy, 1);
    setup.net.put_file("shadow.json", copy);

    // Text the same color as the editor behind it.
    let mut value: serde_json::Value = serde_json::from_slice(&color_file("Murky")).unwrap();
    value["ui"]["text"] = value["ui"]["editor"].clone();
    let murky_bytes = serde_json::to_vec(&value).unwrap();
    let murky = color_theme("murky", "Murky", &murky_bytes, 1);
    setup.net.put_file("murky.json", murky_bytes);

    // A file that calls itself something else than the catalog does.
    let liar_bytes = color_file("Honest Name");
    let liar = color_theme("liar", "Another Name", &liar_bytes, 1);
    setup.net.put_file("liar.json", liar_bytes);

    setup.publish(&index_of(1, vec![shadow, murky, liar], Vec::new()));
    setup.gallery.index(true).unwrap();
    let message = refused(setup.gallery.install("shadow".into()));
    assert!(message.contains("built-in"), "{message}");
    let message = refused(setup.gallery.install("murky".into()));
    assert!(message.contains("doesn't read well"), "{message}");
    let message = refused(setup.gallery.install("liar".into()));
    assert!(message.contains("calls itself"), "{message}");
    assert!(setup.gallery.installed_themes().unwrap().is_empty());
    for id in ["shadow", "murky", "liar"] {
        assert!(!setup.dir().join(format!("themes/{id}.json")).exists());
    }
}

#[test]
fn an_icon_theme_installs_into_its_own_folder_sanitized() {
    let setup = setup();
    let bundle = icon_bundle("demo");
    setup
        .net
        .put_file("gallery/icon-themes/demo.tar.gz", bundle.clone());
    setup.publish(&index_of(1, vec![icon_theme("demo", &bundle)], Vec::new()));
    let items = setup.gallery.install("demo".into()).unwrap();
    assert!(find(&items, "demo").installed);
    assert!(setup.dir().join("icon-themes/demo/demo.json").is_file());
    let svg = std::fs::read_to_string(setup.dir().join("icon-themes/demo/icons/a.svg")).unwrap();
    assert!(svg.starts_with("<svg"));
    // Icon themes are not color themes.
    assert!(setup.gallery.installed_themes().unwrap().is_empty());

    let items = setup.gallery.uninstall("demo".into()).unwrap();
    assert!(!find(&items, "demo").installed);
    assert!(!setup.dir().join("icon-themes/demo").exists());
}

#[test]
fn a_bundle_that_does_not_match_the_catalog_is_refused() {
    let setup = setup();
    let bundle = icon_bundle("demo");
    let mut other = bundle.clone();
    let last = other.len() - 9;
    other[last] ^= 1;
    setup.net.put_file("gallery/icon-themes/demo.tar.gz", other);
    setup.publish(&index_of(1, vec![icon_theme("demo", &bundle)], Vec::new()));
    refused(setup.gallery.install("demo".into()));
    assert!(!setup.dir().join("icon-themes/demo").exists());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// Any one byte of the index, changed, is refused; and the index the
    /// phone already trusts is still the one it serves.
    #[test]
    fn prop_a_changed_byte_in_the_index_is_refused(at in any::<prop::sample::Index>(), flip in 1u8..=255) {
        let setup = setup();
        let (index, _) = dawn(&setup, 1, 1);
        setup.publish(&index);
        let good = setup.gallery.index(true).unwrap();

        let (mut bytes, signature) = sign(&index_of(2, index.themes.clone(), Vec::new()), &setup.key);
        let at = at.index(bytes.len());
        bytes[at] ^= flip;
        setup.net.publish_raw(bytes, signature);
        prop_assert!(setup.gallery.index(true).is_err());
        prop_assert_eq!(setup.gallery.index(false).unwrap(), good);
    }

    /// Any one character of the signature that carries the signature's bits,
    /// changed, is refused.
    #[test]
    fn prop_a_changed_byte_in_the_signature_is_refused(at in 0usize..84, flip in 1u8..=255) {
        let setup = setup();
        let (index, _) = dawn(&setup, 1, 1);
        let (bytes, mut signature) = sign(&index, &setup.key);
        signature[at] ^= flip;
        setup.net.publish_raw(bytes, signature);
        prop_assert!(setup.gallery.index(true).is_err());
        assert_nothing_installed(&setup);
    }

    /// Any one byte of a theme's file, changed, is refused at install and
    /// leaves nothing on disk.
    #[test]
    fn prop_a_changed_byte_in_the_payload_is_refused(at in any::<prop::sample::Index>(), flip in 1u8..=255) {
        let setup = setup();
        let (index, mut bytes) = dawn(&setup, 1, 1);
        setup.publish(&index);
        let at = at.index(bytes.len());
        bytes[at] ^= flip;
        setup.net.put_file("gallery-dawn.json", bytes);
        prop_assert!(setup.gallery.install("gallery-dawn".into()).is_err());
        prop_assert!(setup.gallery.installed_themes().unwrap().is_empty());
        prop_assert!(!setup.dir().join("themes").exists() || tree(&setup.dir().join("themes")).is_empty());
    }

    /// A name or a path the index gives that could leave the gallery's
    /// folders is refused by the checks, even when the index is signed, and
    /// nothing is written outside the gallery's folder.
    #[test]
    fn prop_hostile_names_and_paths_write_nothing_outside(
        id in prop_oneof![
            Just("../evil".to_owned()),
            Just("..".to_owned()),
            Just("a/b".to_owned()),
            Just("/abs".to_owned()),
            Just("x\\y".to_owned()),
            Just("%2e%2e".to_owned()),
            Just(String::new()),
            ".{0,30}",
        ],
        path in prop_oneof![
            Just("../evil.json".to_owned()),
            Just("a/../../b.json".to_owned()),
            Just("/etc/x.json".to_owned()),
            Just("a\\b.json".to_owned()),
            Just("%2e%2e/x.json".to_owned()),
            ".{0,30}",
        ],
    ) {
        let setup = setup();
        let bytes = color_file("Hostile");
        let mut by_id = color_theme("legit", "Hostile", &bytes, 1);
        if let Some(file) = by_id.file.as_mut() {
            file.path = format!("{id}.json");
        }
        by_id.id = id.clone();
        let mut by_path = color_theme("legit", "Hostile", &bytes, 1);
        if let Some(file) = by_path.file.as_mut() {
            file.path = path.clone();
        }
        for theme in [by_id, by_path] {
            let hostile = !kanso_gallery::is_valid_name(&theme.id)
                || theme.file.as_ref().is_some_and(|file| file.path != format!("{}.json", theme.id));
            setup.publish(&index_of(1, vec![theme], Vec::new()));
            setup.net.put_file("legit.json", bytes.clone());
            let listed = setup.gallery.index(true);
            if hostile {
                prop_assert!(listed.is_err(), "{listed:?}");
            }
            if let Ok(items) = listed {
                for item in items {
                    // Even an id the checks let through installs inside the folder.
                    let _ = setup.gallery.install(item.id);
                }
            }
            for path in tree(setup.phone.dir.path()) {
                let name = path.to_string_lossy().into_owned();
                prop_assert!(
                    name.starts_with("library.db") || name == "gallery" || name.starts_with("gallery/"),
                    "{name} is outside the gallery's folder"
                );
            }
            prop_assert!(!setup.phone.dir.path().join("evil.json").exists());
            prop_assert!(!setup.phone.dir.path().parent().unwrap().join("evil.json").exists());
        }
    }
}

//! In-app feedback (issue #232): the payload a report sends, and the rules
//! that keep notes out of it.
//!
//! **No note content, ever.** [`build_payload`] takes a [`Draft`] (what the
//! person typed) and an [`AppInfo`] (enums and short patterned tokens). It
//! has no `Store`, no note, no title and no file name to read, so the
//! separation is in the types. The only free text is the person's own, and
//! the only image is one they attached. The log lines in `AppInfo::errors`
//! come through [`scrub_log_line`], a whitelist: a line that is not exactly
//! the grammar is dropped, never cleaned.
//!
//! The rules here mirror the Worker's (`cloud/src/feedback.ts`); the Worker
//! is the second wall. The payload is shown to the person as built, and
//! those same bytes are what is sent.

use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;

pub use crate::feedback_model::{
    AppInfo, Chip, Draft, DraftError, Kind, Platform, ReportView, Screen, ScreenshotDecl, Status,
    SyncMode,
};

/// The shortest report text, in characters.
pub const MIN_TEXT: usize = 10;
/// The longest report text, in characters.
pub const MAX_TEXT: usize = 4000;
/// The counter shows from here.
pub const COUNTER_FROM: usize = 3500;
/// The largest screenshot, in bytes.
pub const MAX_SCREENSHOT: u64 = 5 * 1024 * 1024;
/// Most log lines in a report.
pub const MAX_ERRORS: usize = 20;
/// Longest credit name, in characters.
pub const MAX_NAME: usize = 39;
/// The Worker refuses a body over this.
pub const MAX_BODY: usize = 48 * 1024;
/// The payload's version.
pub const VERSION: u32 = 1;

const ID_ALPHABET: &[u8; 32] = b"0123456789abcdefghjkmnpqrstvwxyz";

/// A new report id: `fb_` and 26 base32 characters from 16 random bytes.
/// It is also the idempotency key, so a retry can't make a duplicate.
pub fn new_id(entropy: [u8; 16]) -> String {
    let mut value = u128::from_be_bytes(entropy);
    let mut chars = [b'0'; 26];
    for slot in chars.iter_mut().rev() {
        *slot = ID_ALPHABET[(value & 31) as usize];
        value >>= 5;
    }
    format!("fb_{}", String::from_utf8_lossy(&chars))
}

/// Whether `id` is `fb_` and 26 characters of `[0-9a-z]`.
pub fn is_id(id: &str) -> bool {
    id.strip_prefix("fb_").is_some_and(|rest| {
        rest.len() == 26
            && rest
                .bytes()
                .all(|b| b.is_ascii_digit() || b.is_ascii_lowercase())
    })
}

/// The report text as it will be sent: NFC, 10 to 4,000 characters, no
/// control characters but line feeds.
pub fn clean_text(input: &str) -> Result<String, DraftError> {
    let text: String = input.nfc().collect();
    let count = text.chars().count();
    if text.chars().any(|c| c != '\n' && c.is_control()) {
        return Err(DraftError::BadCharacter);
    }
    // A bare CR is a control character too; line feeds are the only breaks.
    if count < MIN_TEXT {
        return Err(DraftError::TooShort);
    }
    if count > MAX_TEXT {
        return Err(DraftError::TooLong);
    }
    Ok(text)
}

/// Whether the character counter shows for text of this length.
pub fn shows_counter(chars: usize) -> bool {
    chars >= COUNTER_FROM
}

/// A credit name: letters, numbers, space, `.`, `_` and `-`, 1 to 39
/// characters, not blank. Plain text; it is never turned into an @-mention.
pub fn clean_credit_name(input: &str) -> Result<String, DraftError> {
    let name: String = input.trim().nfc().collect();
    let count = name.chars().count();
    let ok = (1..=MAX_NAME).contains(&count)
        && name
            .chars()
            .all(|c| c.is_alphabetic() || c.is_numeric() || matches!(c, ' ' | '.' | '_' | '-'));
    if ok && !name.trim().is_empty() {
        Ok(name)
    } else {
        Err(DraftError::BadCredit)
    }
}

/// The image type of PNG or JPEG bytes, from their first bytes.
pub fn sniff_image(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]) {
        Some("image/png")
    } else if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        Some("image/jpeg")
    } else {
        None
    }
}

/// A screenshot's declaration from its bytes, or why it can't be sent.
pub fn declare_screenshot(bytes: &[u8]) -> Result<ScreenshotDecl, DraftError> {
    let mime = sniff_image(bytes).ok_or(DraftError::BadScreenshot)?;
    if bytes.is_empty() || bytes.len() as u64 > MAX_SCREENSHOT {
        return Err(DraftError::BadScreenshot);
    }
    Ok(ScreenshotDecl {
        mime,
        bytes: bytes.len() as u64,
        sha256: hex(&Sha256::digest(bytes)),
    })
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

// -------------------------------------------------------------- log lines

const LOG_SOURCES: [&str; 7] = ["update", "theme", "sync", "ai", "voice", "export", "import"];

/// Whether `line` is exactly `^(update|theme|sync|ai|voice|export|import): [a-z][a-z ,:_-]{0,180}$`
/// or `crash: <type name>`.
pub fn is_log_line(line: &str) -> bool {
    if let Some(name) = line.strip_prefix("crash: ") {
        let mut chars = name.chars();
        return chars
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '$')
            && name.len() <= 128
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '$' | '.'));
    }
    let Some((source, rest)) = line.split_once(": ") else {
        return false;
    };
    if !LOG_SOURCES.contains(&source) {
        return false;
    }
    let mut chars = rest.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && rest.len() <= 181
        && rest
            .chars()
            .all(|c| c.is_ascii_lowercase() || matches!(c, ' ' | ',' | ':' | '_' | '-'))
}

/// A log line, kept only if it is exactly the grammar. A line with a path, a
/// quote, a URL or digits is dropped, not cleaned, so nothing a note
/// contributed can ride along. Idempotent: a kept line scrubs to itself.
pub fn scrub_log_line(line: &str) -> Option<String> {
    let line = line.trim_end_matches(['\n', '\r']);
    is_log_line(line).then(|| line.to_owned())
}

/// The last [`MAX_ERRORS`] lines that survive [`scrub_log_line`], oldest first.
pub fn scrub_log_lines<'a>(lines: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let kept: Vec<String> = lines.into_iter().filter_map(scrub_log_line).collect();
    let skip = kept.len().saturating_sub(MAX_ERRORS);
    kept.into_iter().skip(skip).collect()
}

// --------------------------------------------------------------- app info

fn all_of(text: &str, max: usize, ok: impl Fn(char) -> bool) -> bool {
    !text.is_empty() && text.len() <= max && text.chars().all(ok)
}

/// `^[0-9]{1,4}(\.[0-9]{1,4}){2}$`.
pub fn is_version(text: &str) -> bool {
    let parts: Vec<&str> = text.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|p| (1..=4).contains(&p.len()) && p.bytes().all(|b| b.is_ascii_digit()))
}

fn is_os(text: &str) -> bool {
    all_of(text, 32, |c| {
        c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-')
    })
}

fn is_device(text: &str) -> bool {
    all_of(text, 64, |c| {
        c.is_ascii_alphanumeric() || matches!(c, ' ' | ',' | '.' | '_' | '(' | ')' | '-')
    })
}

/// `^[A-Za-z]{2,3}([-_][A-Za-z0-9]{2,8}){0,2}$`.
fn is_locale(text: &str) -> bool {
    let mut parts = text.split(['-', '_']);
    let Some(lang) = parts.next() else {
        return false;
    };
    if !(2..=3).contains(&lang.len()) || !lang.bytes().all(|b| b.is_ascii_alphabetic()) {
        return false;
    }
    let rest: Vec<&str> = parts.collect();
    rest.len() <= 2
        && rest
            .iter()
            .all(|p| (2..=8).contains(&p.len()) && p.bytes().all(|b| b.is_ascii_alphanumeric()))
}

fn sanitized(text: &str, max: usize, ok: impl Fn(char) -> bool, fallback: &str) -> String {
    let cleaned: String = text.chars().map(|c| if ok(c) { c } else { '_' }).collect();
    let cleaned: String = cleaned.chars().take(max).collect();
    if cleaned.trim_matches('_').is_empty() {
        fallback.to_owned()
    } else {
        cleaned
    }
}

/// The operating system's version, cut to the Worker's pattern.
pub fn clean_os(text: &str) -> String {
    sanitized(
        text,
        32,
        |c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'),
        "unknown",
    )
}

/// The device's model, cut to the Worker's pattern.
pub fn clean_device(text: &str) -> String {
    sanitized(
        text,
        64,
        |c| c.is_ascii_alphanumeric() || matches!(c, ' ' | ',' | '.' | '_' | '(' | ')' | '-'),
        "unknown",
    )
}

/// The locale tag (`en-GB`), or `en` if it doesn't have the shape.
pub fn clean_locale(text: &str) -> String {
    let text = text.split('.').next().unwrap_or("");
    if is_locale(text) {
        text.to_owned()
    } else {
        "en".to_owned()
    }
}

/// The build number's digits (at most 10), or `0`.
pub fn clean_build(text: &str) -> String {
    let digits: String = text.chars().filter(char::is_ascii_digit).take(10).collect();
    if digits.is_empty() {
        "0".to_owned()
    } else {
        digits
    }
}

/// Checks every field against the Worker's patterns.
pub fn validate_app(app: &AppInfo) -> Result<(), DraftError> {
    if !is_version(&app.version) {
        return Err(DraftError::BadApp("app.version"));
    }
    if !all_of(&app.build, 10, |c| c.is_ascii_digit()) {
        return Err(DraftError::BadApp("app.build"));
    }
    if app.patch.is_some_and(|p| p > 100_000) {
        return Err(DraftError::BadApp("app.patch"));
    }
    if !is_os(&app.os) {
        return Err(DraftError::BadApp("app.os"));
    }
    if !is_device(&app.device) {
        return Err(DraftError::BadApp("app.device"));
    }
    if !is_locale(&app.locale) {
        return Err(DraftError::BadApp("app.locale"));
    }
    if app.errors.len() > MAX_ERRORS || app.errors.iter().any(|l| l.len() > 200 || !is_log_line(l))
    {
        return Err(DraftError::BadApp("app.errors"));
    }
    Ok(())
}

// ---------------------------------------------------------------- payload

/// The exact JSON of a report, in the Worker's key order, pretty-printed so
/// "Show what's sent" can display it as is. These are the bytes that are
/// sent and the bytes the outbox keeps.
///
/// `app` is `None` when "Include app info" is off.
pub fn build_payload(id: &str, draft: &Draft, app: Option<&AppInfo>) -> Result<String, DraftError> {
    if !is_id(id) {
        return Err(DraftError::BadId);
    }
    let text = clean_text(&draft.text)?;
    let credit = match &draft.credit_name {
        Some(name) => json!({ "name": clean_credit_name(name)? }),
        None => Value::Null,
    };
    let app = match app {
        Some(app) => {
            validate_app(app)?;
            json!({
                "platform": app.platform.wire(),
                "version": app.version,
                "build": app.build,
                "patch": app.patch,
                "os": app.os,
                "device": app.device,
                "locale": app.locale,
                "screen": app.screen.wire(),
                "sync": app.sync.wire(),
                "errors": app.errors,
            })
        }
        None => Value::Null,
    };
    let screenshot = match &draft.screenshot {
        Some(shot) => {
            let sha_ok = shot.sha256.len() == 64
                && shot
                    .sha256
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
            if !(1..=MAX_SCREENSHOT).contains(&shot.bytes)
                || !matches!(shot.mime, "image/png" | "image/jpeg")
                || !sha_ok
            {
                return Err(DraftError::BadScreenshot);
            }
            json!({ "type": shot.mime, "bytes": shot.bytes, "sha256": shot.sha256 })
        }
        None => Value::Null,
    };
    let mut body = Map::new();
    body.insert("v".into(), json!(VERSION));
    body.insert("id".into(), json!(id));
    body.insert("kind".into(), json!(draft.kind.wire()));
    body.insert("text".into(), json!(text));
    body.insert("credit".into(), credit);
    body.insert("app".into(), app);
    body.insert("screenshot".into(), screenshot);
    let out = serde_json::to_string_pretty(&Value::Object(body)).expect("a JSON value serializes");
    if out.len() > MAX_BODY {
        return Err(DraftError::TooLong);
    }
    Ok(out)
}

// ----------------------------------------------------------------- status

/// What a report's chip says.
pub fn chip(report: &ReportView) -> Chip {
    if report.deleting {
        return Chip::Deleting;
    }
    match report.status {
        Status::Waiting => Chip::WaitingToSend,
        Status::Received => Chip::Received,
        Status::Noise => Chip::NotFiled,
        Status::Duplicate => report
            .duplicate_of
            .or(report.issue)
            .map_or(Chip::Received, Chip::SameAs),
        Status::Filed => report.issue.map_or(Chip::Received, Chip::FiledAs),
        Status::Building => Chip::Building,
        Status::PrOpen => Chip::FixProposed,
        Status::Merged => Chip::Merged,
        Status::Released => report
            .release
            .clone()
            .map_or(Chip::Merged, Chip::ReleasedIn),
        Status::Closed => Chip::Closed,
    }
}

/// A release tag without its `v`: `v0.9.2` is `0.9.2`.
pub fn release_name(tag: &str) -> &str {
    tag.strip_prefix('v').unwrap_or(tag)
}

/// The row link target: the issue, when the report was filed.
pub fn issue_url(repo: &str, report: &ReportView) -> Option<String> {
    let number = match report.status {
        Status::Duplicate => report.duplicate_of.or(report.issue),
        Status::Waiting | Status::Received | Status::Noise => None,
        _ => report.issue,
    }?;
    Some(format!("https://github.com/{repo}/issues/{number}"))
}

/// The repository the issues live in.
pub const REPO: &str = "iamngoni/kanso";

/// The newest report's chip, for the Settings row's value.
pub fn newest_chip(reports: &[ReportView]) -> Option<Chip> {
    reports.iter().max_by_key(|r| r.created_ms).map(chip)
}

/// How many of these reports `release` fixed (for "Fixes 2 of your reports").
pub fn fixed_in(reports: &[ReportView], release: &str) -> usize {
    let release = release_name(release);
    reports
        .iter()
        .filter(|r| {
            r.status == Status::Released && r.release.as_deref().map(release_name) == Some(release)
        })
        .count()
}

// ------------------------------------------------------------------ retry

const MINUTE_MS: i64 = 60_000;

/// How long to wait before the next send attempt after `attempts` failures:
/// 1, 5, then 30 minutes. After that: `None`, and the report waits for the
/// next launch.
pub fn retry_delay_ms(attempts: u32) -> Option<i64> {
    match attempts {
        0 => Some(0),
        1 => Some(MINUTE_MS),
        2 => Some(5 * MINUTE_MS),
        3 => Some(30 * MINUTE_MS),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::RngSeed;

    fn config() -> ProptestConfig {
        ProptestConfig {
            rng_seed: RngSeed::Fixed(0x006b_616e_736f),
            failure_persistence: None,
            ..ProptestConfig::with_cases(256)
        }
    }

    fn app() -> AppInfo {
        AppInfo {
            platform: Platform::Macos,
            version: "0.9.1".into(),
            build: "57".into(),
            patch: None,
            os: "15.1".into(),
            device: "Mac14,9".into(),
            locale: "en-GB".into(),
            screen: Screen::Editor,
            sync: SyncMode::Kanso,
            errors: vec!["update: check failed: timeout".into()],
        }
    }

    fn draft(text: &str) -> Draft {
        Draft {
            kind: Kind::Bug,
            text: text.into(),
            credit_name: None,
            screenshot: None,
        }
    }

    const ID: &str = "fb_01jbx9k2m4n6p8q0r2s4t6v8w0";

    #[test]
    fn the_payload_matches_the_contract() {
        let mut d = draft("Export to PDF cuts off wide tables.");
        d.credit_name = Some("tendai".into());
        let out = build_payload(ID, &d, Some(&app())).unwrap();
        let v: Value = serde_json::from_str(&out).unwrap();
        let keys: Vec<&str> = v.as_object().unwrap().keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            ["v", "id", "kind", "text", "credit", "app", "screenshot"]
        );
        assert_eq!(v["v"], 1);
        assert_eq!(v["credit"]["name"], "tendai");
        assert_eq!(v["app"]["screen"], "editor");
        assert_eq!(v["app"]["patch"], Value::Null);
        assert_eq!(v["screenshot"], Value::Null);
    }

    #[test]
    fn app_info_off_sends_null() {
        let v: Value =
            serde_json::from_str(&build_payload(ID, &draft("0123456789"), None).unwrap()).unwrap();
        assert_eq!(v["app"], Value::Null);
    }

    #[test]
    fn text_limits() {
        assert_eq!(clean_text("123456789"), Err(DraftError::TooShort));
        assert!(clean_text("1234567890").is_ok());
        assert!(clean_text(&"a".repeat(4000)).is_ok());
        assert_eq!(clean_text(&"a".repeat(4001)), Err(DraftError::TooLong));
        assert_eq!(
            clean_text("hello\u{7}world!!"),
            Err(DraftError::BadCharacter)
        );
        assert_eq!(clean_text("hello\rworld!!"), Err(DraftError::BadCharacter));
        assert!(clean_text("two\nlines here").is_ok());
        // Combining marks are composed before counting.
        assert_eq!(
            clean_text(
                "e\u{301}e\u{301}e\u{301}e\u{301}e\u{301}e\u{301}e\u{301}e\u{301}e\u{301}e\u{301}"
            )
            .unwrap()
            .chars()
            .count(),
            10
        );
    }

    #[test]
    fn credit_names() {
        assert_eq!(clean_credit_name("  tendai  ").unwrap(), "tendai");
        assert!(clean_credit_name("Zoë_1.2-x").is_ok());
        assert!(clean_credit_name("").is_err());
        assert!(clean_credit_name("   ").is_err());
        assert!(clean_credit_name("@tendai").is_err());
        assert!(clean_credit_name(&"a".repeat(40)).is_err());
        assert!(clean_credit_name(&"a".repeat(39)).is_ok());
    }

    #[test]
    fn screenshots() {
        let png = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 1, 2];
        let decl = declare_screenshot(&png).unwrap();
        assert_eq!(
            (decl.mime, decl.bytes, decl.sha256.len()),
            ("image/png", 10, 64)
        );
        assert!(declare_screenshot(&[0xff, 0xd8, 0xff, 0]).is_ok());
        assert_eq!(
            declare_screenshot(b"GIF89a"),
            Err(DraftError::BadScreenshot)
        );
        let mut big = vec![0xff, 0xd8, 0xff];
        big.resize(MAX_SCREENSHOT as usize + 1, 0);
        assert_eq!(declare_screenshot(&big), Err(DraftError::BadScreenshot));
    }

    #[test]
    fn every_app_field_has_a_pattern() {
        let mut a = app();
        assert!(validate_app(&a).is_ok());
        a.version = "0.9".into();
        assert_eq!(validate_app(&a), Err(DraftError::BadApp("app.version")));
        a = app();
        a.build = "5x".into();
        assert_eq!(validate_app(&a), Err(DraftError::BadApp("app.build")));
        a = app();
        a.os = "Mac OS".into();
        assert_eq!(validate_app(&a), Err(DraftError::BadApp("app.os")));
        a = app();
        a.device = "/Users/me/x".into();
        assert_eq!(validate_app(&a), Err(DraftError::BadApp("app.device")));
        a = app();
        a.locale = "english".into();
        assert_eq!(validate_app(&a), Err(DraftError::BadApp("app.locale")));
        a = app();
        a.errors = vec!["update: open /Users/me/notes".into()];
        assert_eq!(validate_app(&a), Err(DraftError::BadApp("app.errors")));
        a.errors = vec!["update: ok".into(); 21];
        assert_eq!(validate_app(&a), Err(DraftError::BadApp("app.errors")));
    }

    #[test]
    fn log_lines() {
        assert!(scrub_log_line("update: check failed: timeout").is_some());
        assert!(scrub_log_line("crash: FormatException").is_some());
        // Paths, quotes, URLs, digits: dropped, not cleaned.
        assert!(scrub_log_line("update: failed for /Users/me/Notes").is_none());
        assert!(scrub_log_line("theme: bad \"nord\"").is_none());
        assert!(scrub_log_line("update: GET https://x.y").is_none());
        assert!(scrub_log_line("update: status 503").is_none());
        assert!(scrub_log_line("note: my secret title").is_none());
        assert!(scrub_log_line("update: ").is_none());
        assert_eq!(
            scrub_log_line("sync: offline\n").as_deref(),
            Some("sync: offline")
        );
    }

    #[test]
    fn only_the_last_twenty_lines_stay() {
        let lines: Vec<String> = (0..30)
            .map(|i| format!("sync: {}", ["a", "b", "c"][i % 3]))
            .collect();
        let kept = scrub_log_lines(lines.iter().map(String::as_str));
        assert_eq!(kept.len(), 20);
        assert_eq!(kept.last(), lines.last());
    }

    #[test]
    fn ids() {
        assert!(is_id(ID));
        assert!(!is_id("fb_short"));
        assert!(!is_id("fb_01JBX9K2M4N6P8Q0R2S4T6V8W0"));
        assert!(is_id(&new_id([0; 16])));
        assert!(is_id(&new_id([255; 16])));
        assert_ne!(new_id([1; 16]), new_id([2; 16]));
    }

    #[test]
    fn retries_back_off() {
        assert_eq!(retry_delay_ms(1), Some(60_000));
        assert_eq!(retry_delay_ms(2), Some(300_000));
        assert_eq!(retry_delay_ms(3), Some(1_800_000));
        assert_eq!(retry_delay_ms(4), None);
    }

    fn view(status: Status) -> ReportView {
        ReportView {
            id: ID.into(),
            created_ms: 1,
            kind: Kind::Bug,
            preview: "p".into(),
            status,
            issue: Some(241),
            duplicate_of: Some(198),
            pr: Some(252),
            release: Some("v0.9.2".into()),
            deleting: false,
        }
    }

    #[test]
    fn chips_follow_the_status() {
        assert_eq!(chip(&view(Status::Filed)), Chip::FiledAs(241));
        assert_eq!(chip(&view(Status::Duplicate)), Chip::SameAs(198));
        assert_eq!(chip(&view(Status::PrOpen)), Chip::FixProposed);
        assert_eq!(
            chip(&view(Status::Released)),
            Chip::ReleasedIn("v0.9.2".into())
        );
        assert_eq!(chip(&view(Status::Noise)), Chip::NotFiled);
        let mut deleting = view(Status::Filed);
        deleting.deleting = true;
        assert_eq!(chip(&deleting), Chip::Deleting);
        assert_eq!(
            issue_url(REPO, &view(Status::Filed)).unwrap(),
            "https://github.com/iamngoni/kanso/issues/241"
        );
        assert_eq!(issue_url(REPO, &view(Status::Received)), None);
        assert_eq!(
            fixed_in(&[view(Status::Released), view(Status::Filed)], "0.9.2"),
            1
        );
        assert_eq!(fixed_in(&[view(Status::Released)], "0.9.3"), 0);
    }

    /// Text of lowercase words, the shape of a note's body.
    fn note_text() -> impl Strategy<Value = String> {
        "[a-z]{3,9}( [a-z]{3,9}){2,12}"
    }

    /// Every window of `needle` of 9 characters.
    fn windows(needle: &str) -> Vec<String> {
        let chars: Vec<char> = needle.chars().collect();
        if chars.len() < 9 {
            return Vec::new();
        }
        chars.windows(9).map(|w| w.iter().collect()).collect()
    }

    proptest! {
        #![proptest_config(config())]

        /// The payload never holds note content: for any library of notes,
        /// the report built from the person's text and the app info shares
        /// no run of 9 characters with any note title or body. (The builder
        /// has no way to read a note; this pins that it stays so.)
        #[test]
        fn prop_payload_holds_no_note_text(
            notes in proptest::collection::vec((note_text(), note_text()), 1..6),
            screen in proptest::sample::select(Screen::ALL.to_vec()),
            sync in proptest::sample::select(SyncMode::ALL.to_vec()),
            kind in proptest::sample::select(Kind::ALL.to_vec()),
        ) {
            let dir = tempfile::tempdir().unwrap();
            let store = kanso_store::Store::open(dir.path().join("k.db")).unwrap();
            let book = store
                .put(
                    kanso_core::DocType::Book,
                    json!({"name": "Notes"}),
                    kanso_store::Put::Upsert,
                    kanso_store::Origin::Ui,
                )
                .unwrap()
                .id;
            for (title, body) in &notes {
                store
                    .put(
                        kanso_core::DocType::Note,
                        json!({"title": title, "body": body, "bookId": book}),
                        kanso_store::Put::Upsert,
                        kanso_store::Origin::Ui,
                    )
                    .unwrap();
            }
            // What the app writes in its own logs; none of it comes from a note.
            let logs = ["update: check failed: timeout", "theme: install failed: signature", "sync: offline"];
            let mut a = app();
            a.screen = screen;
            a.sync = sync;
            a.errors = scrub_log_lines(logs);
            let d = Draft { kind, text: "The export button does nothing at all".into(), credit_name: Some("tendai".into()), screenshot: None };
            let payload = build_payload(ID, &d, Some(&a)).unwrap();
            for (title, body) in &notes {
                for w in windows(title).into_iter().chain(windows(body)) {
                    prop_assert!(!payload.contains(&w), "note text {w:?} is in the payload");
                }
            }
        }

        /// Whatever the builder accepts passes the Worker's patterns: the
        /// payload parses back, with exactly the contract's keys.
        #[test]
        fn prop_payload_is_always_valid(
            text in "[ -~\\n]{0,4100}",
            os in ".{0,80}",
            device in ".{0,100}",
            locale in ".{0,20}",
            build in ".{0,20}",
            name in ".{0,50}",
            lines in proptest::collection::vec(".{0,60}", 0..30),
        ) {
            let mut a = app();
            a.os = clean_os(&os);
            a.device = clean_device(&device);
            a.locale = clean_locale(&locale);
            a.build = clean_build(&build);
            a.errors = scrub_log_lines(lines.iter().map(String::as_str));
            let d = Draft { kind: Kind::Idea, text, credit_name: Some(name), screenshot: None };
            if let Ok(out) = build_payload(ID, &d, Some(&a)) {
                let v: Value = serde_json::from_str(&out).unwrap();
                prop_assert_eq!(v.as_object().unwrap().len(), 7);
                prop_assert!(validate_app(&a).is_ok());
                let n = v["text"].as_str().unwrap().chars().count();
                prop_assert!((MIN_TEXT..=MAX_TEXT).contains(&n));
                prop_assert!(out.len() <= MAX_BODY);
            }
        }

        /// A kept log line is in the grammar; scrubbing it again changes nothing.
        #[test]
        fn prop_scrub_is_in_the_grammar_and_idempotent(line in ".{0,120}") {
            if let Some(kept) = scrub_log_line(&line) {
                prop_assert!(is_log_line(&kept));
                prop_assert!(kept.len() <= 200);
                prop_assert_eq!(scrub_log_line(&kept), Some(kept.clone()));
            }
        }

        /// A line built from the grammar always survives.
        #[test]
        fn prop_grammar_lines_survive(
            source in proptest::sample::select(LOG_SOURCES.to_vec()),
            rest in "[a-z][a-z ,:_-]{0,180}",
        ) {
            let line = format!("{source}: {rest}");
            prop_assert_eq!(scrub_log_line(&line), Some(line.clone()));
        }

        /// Ids from any entropy are valid and distinct entropy gives distinct ids.
        #[test]
        fn prop_ids(a in any::<[u8; 16]>(), b in any::<[u8; 16]>()) {
            prop_assert!(is_id(&new_id(a)));
            prop_assert_eq!(new_id(a) == new_id(b), a == b);
        }

        /// The retry schedule only grows, then stops.
        #[test]
        fn prop_backoff_is_monotonic(n in 0u32..10) {
            match (retry_delay_ms(n), retry_delay_ms(n + 1)) {
                (Some(a), Some(b)) => prop_assert!(a <= b),
                (None, Some(_)) => prop_assert!(false),
                _ => {}
            }
        }
    }
}

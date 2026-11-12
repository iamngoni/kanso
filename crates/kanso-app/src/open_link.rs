//! `kanso://open/…` links: the ones that open something in Kanso.
//!
//! The phone reads the same links (`mobile/lib/core/router/deep_links.dart`),
//! so one link works on both:
//!
//! - `kanso://open/note/<id>` shows the note;
//! - `kanso://open/notebook/<id>` lists the notebook;
//! - `kanso://open/search?q=<text>` searches every note;
//! - `kanso://open/new` opens the capture window for a new note.
//!
//! A link only ever opens or navigates. It names no action that writes: the
//! new-note link opens the window where someone types, and nothing is saved
//! until they do. Anything else (another host, a path with more or fewer
//! parts, an empty or odd id, a sign-in callback) is not an open link and is
//! refused with [`LinkError`], which says why and never carries the URL.
//!
//! The link reaches the app three ways, like files do
//! ([`crate::open_documents`]): the operating system's open event (macOS),
//! the command line of a Kanso started by the link (cold start), and a
//! second `kanso <url>` that forwards it to the running one
//! ([`crate::url_callback`]).

use gpui_kit::App;

use crate::workspace::main_workspace;

/// The scheme of the app's own links.
pub const SCHEME: &str = "kanso";

/// The host of the links that open something.
pub const OPEN_HOST: &str = "open";

/// The longest note or notebook ID a link may carry, in bytes.
pub const MAX_ID: usize = 128;

/// The longest search text a link may carry, in bytes.
pub const MAX_QUERY: usize = 1024;

/// The longest link read at all, in bytes.
pub const MAX_LINK: usize = 4096;

/// Where an open link goes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OpenLink {
    /// A note, by ID.
    Note(String),
    /// A notebook, by ID.
    Notebook(String),
    /// A search over every note; `None` just opens the search box.
    Search(Option<String>),
    /// The window for a new note.
    New,
}

/// Why a URL isn't an open link.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkError {
    /// Not a `kanso://open/…` URL at all.
    NotAnOpenLink,
    /// A `kanso://open/…` URL that names nothing Kanso opens.
    UnknownTarget,
    /// A note or notebook ID that can't be one.
    BadId,
    /// Search text that is too long or has control characters.
    BadQuery,
}

impl std::fmt::Display for LinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            LinkError::NotAnOpenLink => "that isn't a kanso://open link, so Kanso did nothing",
            LinkError::UnknownTarget => {
                "Kanso opens notes, notebooks, searches and new notes from a link (kanso://open/note/<id>, /notebook/<id>, /search?q=..., /new), so it did nothing"
            }
            LinkError::BadId => "that link's note or notebook ID isn't valid, so Kanso did nothing",
            LinkError::BadQuery => "that link's search text isn't valid, so Kanso did nothing",
        })
    }
}

/// Whether `url` starts like a `kanso://open` link, whatever follows. Used
/// to route a link here rather than to the sign-in callbacks.
pub fn is_open_url(url: &str) -> bool {
    let head = format!("{SCHEME}://{OPEN_HOST}");
    url.get(..head.len())
        .is_some_and(|start| start.eq_ignore_ascii_case(&head))
        && url[head.len()..]
            .chars()
            .next()
            .is_none_or(|c| matches!(c, '/' | '?' | '#'))
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= MAX_ID
        && !id.chars().any(|c| c.is_control() || c.is_whitespace())
        && !id.contains(['/', '\\'])
}

/// Reads `text` as an open link. Pure; never panics.
pub fn parse(text: &str) -> Result<OpenLink, LinkError> {
    if text.len() > MAX_LINK || !is_open_url(text) {
        return Err(LinkError::NotAnOpenLink);
    }
    let url = url::Url::parse(text).map_err(|_| LinkError::NotAnOpenLink)?;
    if url.scheme() != SCHEME
        || !url
            .host_str()
            .is_some_and(|host| host.eq_ignore_ascii_case(OPEN_HOST))
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
    {
        return Err(LinkError::NotAnOpenLink);
    }
    let mut segments: Vec<String> = Vec::new();
    for part in url.path_segments().into_iter().flatten() {
        if part.is_empty() {
            continue;
        }
        let decoded = percent_encoding::percent_decode_str(part)
            .decode_utf8()
            .map_err(|_| LinkError::BadId)?;
        segments.push(decoded.into_owned());
    }
    let segments: Vec<&str> = segments.iter().map(String::as_str).collect();
    match segments.as_slice() {
        ["note", id] if valid_id(id) => Ok(OpenLink::Note((*id).to_owned())),
        ["notebook", id] if valid_id(id) => Ok(OpenLink::Notebook((*id).to_owned())),
        ["note" | "notebook", _] => Err(LinkError::BadId),
        ["search"] => {
            let query = url
                .query_pairs()
                .find(|(key, _)| key == "q")
                .map(|(_, value)| value.into_owned());
            match query {
                Some(q) if q.len() > MAX_QUERY || q.chars().any(char::is_control) => {
                    Err(LinkError::BadQuery)
                }
                Some(q) if !q.trim().is_empty() => Ok(OpenLink::Search(Some(q))),
                _ => Ok(OpenLink::Search(None)),
            }
        }
        ["new"] => Ok(OpenLink::New),
        _ => Err(LinkError::UnknownTarget),
    }
}

impl OpenLink {
    /// The link that [`parse`] reads back as `self`.
    pub fn to_url(&self) -> String {
        use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
        let encode = |text: &str| utf8_percent_encode(text, NON_ALPHANUMERIC).to_string();
        match self {
            OpenLink::Note(id) => format!("{SCHEME}://{OPEN_HOST}/note/{}", encode(id)),
            OpenLink::Notebook(id) => format!("{SCHEME}://{OPEN_HOST}/notebook/{}", encode(id)),
            OpenLink::Search(None) => format!("{SCHEME}://{OPEN_HOST}/search"),
            OpenLink::Search(Some(q)) => {
                format!("{SCHEME}://{OPEN_HOST}/search?q={}", encode(q))
            }
            OpenLink::New => format!("{SCHEME}://{OPEN_HOST}/new"),
        }
    }
}

/// Where links to open are sent from anywhere: the system's open event, the
/// command line, a forwarding process.
#[derive(Clone)]
pub struct LinkSender(async_channel::Sender<OpenLink>);

pub fn channel() -> (LinkSender, async_channel::Receiver<OpenLink>) {
    let (sender, receiver) = async_channel::unbounded();
    (LinkSender(sender), receiver)
}

impl LinkSender {
    /// Queues the open links among `urls`. Other URLs are ignored; an open
    /// link that can't be opened says so on stderr.
    pub fn urls(&self, urls: &[String]) {
        for url in urls.iter().filter(|url| is_open_url(url)) {
            if let Err(error) = self.url(url) {
                eprintln!("kanso: {error}");
            }
        }
    }

    /// Queues one link.
    pub fn url(&self, url: &str) -> Result<(), LinkError> {
        self.send(parse(url)?);
        Ok(())
    }

    pub fn send(&self, link: OpenLink) {
        self.0.try_send(link).ok();
    }
}

/// Opens what arrives on `receiver` in the main window, opening it first if
/// it was closed, and brings it forward. Call after the first window is
/// open; what arrived at launch waits in the channel.
pub fn serve(
    receiver: async_channel::Receiver<OpenLink>,
    open_main_window: fn(&mut App),
    cx: &mut App,
) {
    cx.spawn(async move |cx| {
        while let Ok(link) = receiver.recv().await {
            cx.update(|cx| show(link, open_main_window, cx));
        }
    })
    .detach();
}

fn show(link: OpenLink, open_main_window: fn(&mut App), cx: &mut App) {
    let mut found = main_workspace(cx);
    if found.is_none() {
        open_main_window(cx);
        found = main_workspace(cx);
    }
    let Some((window, workspace)) = found else {
        return;
    };
    window
        .update(cx, |_, window, cx| {
            window.activate_window();
            workspace.update(cx, |workspace, cx| workspace.open_link(&link, window, cx));
        })
        .ok();
    cx.activate(true);
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn the_phones_links_read_the_same_here() {
        assert_eq!(
            parse("kanso://open/note/abc123"),
            Ok(OpenLink::Note("abc123".into()))
        );
        assert_eq!(
            parse("kanso://open/notebook/nb-1"),
            Ok(OpenLink::Notebook("nb-1".into()))
        );
        assert_eq!(
            parse("kanso://open/search?q=pixel%20review"),
            Ok(OpenLink::Search(Some("pixel review".into())))
        );
        assert_eq!(parse("kanso://open/search"), Ok(OpenLink::Search(None)));
        assert_eq!(parse("kanso://open/new"), Ok(OpenLink::New));
        assert_eq!(
            parse("KANSO://OPEN/note/x/"),
            Ok(OpenLink::Note("x".into()))
        );
    }

    #[test]
    fn what_is_not_an_open_link_is_refused() {
        for url in [
            "",
            "kanso://",
            "kanso://open",
            "kanso://auth/apple?flow=x&s=y",
            "kanso://opened/note/x",
            "kanso://open.evil/note/x",
            "kanso://u:p@open/note/x",
            "kanso://open:80/note/x",
            "https://open/note/x",
            "kanso://open/note",
            "kanso://open/note/",
            "kanso://open/note/a/b",
            "kanso://open/note/a%2Fb",
            "kanso://open/note/%00",
            "kanso://open/note/%ff",
            "kanso://open/delete/x",
            "kanso://open/trash",
            "kanso://open/daily",
            "kanso://open/search/x",
            "kanso://open/new/x",
        ] {
            assert!(parse(url).is_err(), "{url}");
        }
        assert_eq!(
            parse(&format!("kanso://open/note/{}", "a".repeat(MAX_ID + 1))),
            Err(LinkError::BadId)
        );
    }

    #[test]
    fn a_refusal_says_so_without_the_link() {
        let said = parse("kanso://open/delete/secret-token")
            .unwrap_err()
            .to_string();
        assert!(said.contains("did nothing"));
        assert!(!said.contains("secret-token"));
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f),
            failure_persistence: None,
            ..ProptestConfig::with_cases(512)
        })]

        /// Every link the app can make reads back as itself.
        #[test]
        fn prop_a_link_round_trips(
            id in "[^\\s\\p{C}/\\\\]{1,40}",
            q in "[^\\p{C}]{1,60}",
            which in 0..5usize,
        ) {
            let link = match which {
                0 => OpenLink::Note(id),
                1 => OpenLink::Notebook(id),
                2 => OpenLink::Search(Some(q).filter(|q| !q.trim().is_empty())),
                3 => OpenLink::Search(None),
                _ => OpenLink::New,
            };
            prop_assert_eq!(parse(&link.to_url()), Ok(link));
        }

        /// No text panics the parser, and anything it accepts is an open link
        /// that goes where it says.
        #[test]
        fn prop_parse_never_panics_and_accepts_only_open_links(text in "\\PC{0,200}") {
            if let Ok(link) = parse(&text) {
                prop_assert!(is_open_url(&text));
                prop_assert_eq!(parse(&link.to_url()), Ok(link));
            }
        }

        /// Noise after a real prefix is handled too.
        #[test]
        fn prop_noise_after_the_prefix_never_panics(tail in "[ -~%\u{80}-\u{ff}]{0,80}") {
            let _ = parse(&format!("kanso://open/{tail}"));
            let _ = parse(&format!("kanso://open/note/{tail}"));
            let _ = parse(&format!("kanso://open/search?{tail}"));
        }

        /// Anything that isn't `open/note|notebook|search|new` is refused:
        /// other hosts, other first segments, extra segments.
        #[test]
        fn prop_other_targets_are_refused(
            host in "[a-z]{1,8}",
            head in "[a-z]{1,8}",
            tail in "[a-zA-Z0-9]{1,8}",
        ) {
            prop_assume!(host != OPEN_HOST);
            let other_host = format!("kanso://{host}/{head}/{tail}");
            prop_assert!(parse(&other_host).is_err());
            let extra = format!("kanso://open/note/{tail}/{tail}");
            prop_assert!(parse(&extra).is_err());
            let known = ["note", "notebook", "search", "new"];
            prop_assume!(!known.contains(&head.as_str()));
            let unknown = format!("kanso://open/{head}/{tail}");
            prop_assert!(parse(&unknown).is_err());
        }
    }
}

//! The quiet line that stands for a PDF's highlights in a page (issue #43,
//! step 6).
//!
//! A note keeps the highlights of each PDF it reads in a `kanso-pdf` fence
//! (see `kanso_pdf::annotation`). It is data for the reader, not text: a page
//! (the preview on a phone, an export, a published note) shows
//! "4 highlights · 1 comment · pencil on 2 pages" instead of the JSON. Only what the line needs is
//! read here, so this crate does not depend on the PDF crate; the app's tests
//! check the two agree.

use serde_json::Value;

use crate::escape;

/// The fence's language.
pub const FENCE: &str = "kanso-pdf";

/// What a block says: how many highlights and comments, and which
/// attachment it is for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notes {
    /// The attachment's document id (`file:…`).
    pub file: String,
    pub highlights: usize,
    pub comments: usize,
    /// Pages with pencil marks in the block itself.
    pub ink_pages: usize,
    /// The marks outgrew the block and live in an attachment it names.
    pub ink_attached: bool,
}

impl Notes {
    /// "4 highlights · 1 comment · pencil on 2 pages"; `None` when there is
    /// nothing to count.
    pub fn summary(&self) -> Option<String> {
        let (h, c, p) = (self.highlights, self.comments, self.ink_pages);
        let mut parts: Vec<String> = Vec::new();
        if h > 0 {
            parts.push(format!("{h} highlight{}", if h == 1 { "" } else { "s" }));
        }
        if h > 0 && c > 0 {
            parts.push(format!("{c} comment{}", if c == 1 { "" } else { "s" }));
        }
        if p > 0 {
            parts.push(format!(
                "pencil on {p} page{}",
                if p == 1 { "" } else { "s" }
            ));
        } else if self.ink_attached {
            parts.push("pencil marks".to_owned());
        }
        (!parts.is_empty()).then(|| parts.join(" · "))
    }

    /// The link that opens the PDF, `inkdrop://file:…`.
    pub fn url(&self) -> Option<String> {
        let id = self.file.strip_prefix("file:")?;
        let plain = !id.is_empty()
            && id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'));
        plain.then(|| format!("inkdrop://file:{id}"))
    }
}

/// Reads a block's JSON; `None` when it is not a block this version reads.
pub fn parse(json: &str) -> Option<Notes> {
    let value: Value = serde_json::from_str(json).ok()?;
    let file = value.get("file")?.as_str()?.to_owned();
    let highlights = value.get("highlights")?.as_array()?;
    let comments = highlights
        .iter()
        .filter(|h| h.get("comment").is_some_and(|c| !c.is_null()))
        .count();
    // Pages with a stroke that is drawn (a rubbed-out one stays as a
    // tombstone, `deleted`, so a sync cannot bring it back).
    let ink_pages = value
        .get("ink")
        .and_then(Value::as_array)
        .map_or(0, |pages| {
            pages
                .iter()
                .filter(|page| {
                    page.get("strokes")
                        .and_then(Value::as_array)
                        .is_some_and(|strokes| {
                            strokes
                                .iter()
                                .any(|s| s.get("deleted").and_then(Value::as_bool) != Some(true))
                        })
                })
                .count()
        });
    let ink_attached = value.get("inkFile").is_some_and(|f| f.is_string());
    Some(Notes {
        file,
        highlights: highlights.len(),
        comments,
        ink_pages,
        ink_attached,
    })
}

/// The words for a block that cannot be read.
pub const UNREADABLE: &str = "PDF highlights that can't be read";

/// The line for a page. `link` makes it a link to the PDF (the app's pages);
/// a block with no highlights and no pencil marks draws nothing.
pub fn html(json: &str, link: bool) -> String {
    let (words, url) = match parse(json) {
        Some(notes) => match notes.summary() {
            Some(words) => (words, notes.url()),
            None => return String::new(),
        },
        None => (UNREADABLE.to_owned(), None),
    };
    let text = escape(&words);
    match url.filter(|_| link) {
        Some(url) => format!(
            "<p class=\"kanso-pdf-notes\"><a href=\"{}\">{text}</a></p>",
            escape(&url)
        ),
        None => format!("<p class=\"kanso-pdf-notes\">{text}</p>"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn block(highlights: usize, comments: usize) -> String {
        let hs: Vec<String> = (0..highlights)
            .map(|i| {
                let comment = if i < comments {
                    "\"comment\":\"c\","
                } else {
                    ""
                };
                format!("{{\"id\":\"h{i}\",\"page\":1,{comment}\"text\":\"t\"}}")
            })
            .collect();
        format!(
            "{{\"file\":\"file:abc\",\"md5\":\"\",\"version\":1,\"highlights\":[{}],\"ink\":[]}}",
            hs.join(",")
        )
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 128, ..ProptestConfig::default() })]

        #[test]
        fn the_line_counts_what_the_block_holds(h in 0usize..30, c in 0usize..30) {
            let c = c.min(h);
            let notes = parse(&block(h, c)).unwrap();
            prop_assert_eq!((notes.highlights, notes.comments), (h, c));
            let line = html(&block(h, c), true);
            if h == 0 {
                prop_assert_eq!(line, "");
            } else {
                let counted = format!("{h} highlight");
                prop_assert!(line.contains(&counted));
                prop_assert!(line.contains("inkdrop://file:abc"));
                prop_assert_eq!(line.contains("comment"), c > 0);
                // Never the JSON, however it was written.
                let brace = '{';
                prop_assert!(!line.contains(brace));
            }
        }

        /// A published or exported page shows the count line and nothing of
        /// the block: no comment, no highlighted words, no JSON, wherever
        /// the fence sits (top level, in a list or a quote, with `~~~`).
        #[test]
        fn a_published_page_never_shows_a_comment(
            comment in "[a-z]{4,12}",
            words in "[a-z]{4,12}",
            place in 0usize..4,
        ) {
            prop_assume!(comment != words);
            let (secret, quoted) = (format!("zq{comment}"), format!("zw{words}"));
            let json = format!(
                "{{\"file\":\"file:abc\",\"md5\":\"\",\"version\":1,\"highlights\":[{{\"id\":\"h1\",\"page\":1,\"quads\":{{\"1\":[[0.1,0.1,0.2,0.2]]}},\"text\":\"{quoted}\",\"comment\":\"{secret}\"}}],\"ink\":[]}}"
            );
            let fence = match place {
                0 => format!("```kanso-pdf\n{json}\n```"),
                1 => format!("~~~kanso-pdf\n{json}\n~~~"),
                2 => format!("- item\n\n  ```kanso-pdf\n  {json}\n  ```"),
                _ => format!("> ```kanso-pdf\n> {json}\n> ```"),
            };
            let body = format!("# Notes\n\nSome text.\n\n{fence}\n\nAfter.\n");
            let none = |_: &str| None;
            let page = crate::html::page("Notes", &body, &crate::Palette::LIGHT, &none);
            prop_assert!(!page.contains(&secret), "the comment was published: {}", page);
            prop_assert!(!page.contains(&quoted), "the block's text was published");
            prop_assert!(!page.contains("\"highlights\""));
            prop_assert!(page.contains("1 highlight · 1 comment"));
        }

        /// Whatever is in the fence, the page shows no markup of it.
        #[test]
        fn nothing_in_the_fence_reaches_the_page_as_markup(junk in ".{0,200}") {
            let line = html(&junk, true);
            prop_assert!(!line.contains("<script") && !line.contains("<img"));
        }
    }

    fn inked(live: usize, erased: usize, attached: bool) -> String {
        let page = |n: usize, alive: bool| {
            let stroke = if alive {
                "{\"id\":\"s\",\"points\":[[1,2,0.5]]}"
            } else {
                "{\"id\":\"s\",\"deleted\":true}"
            };
            format!("{{\"page\":{},\"strokes\":[{stroke}]}}", n + 1)
        };
        let pages: Vec<String> = (0..live)
            .map(|n| page(n, true))
            .chain((live..live + erased).map(|n| page(n, false)))
            .collect();
        let file = if attached {
            ",\"inkFile\":\"inkdrop://file:ink1\""
        } else {
            ""
        };
        format!(
            "{{\"file\":\"file:abc\",\"md5\":\"\",\"version\":1,\"highlights\":[],\"ink\":[{}]{file}}}",
            pages.join(",")
        )
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 64, ..ProptestConfig::default() })]

        /// The line counts the pages with a stroke still drawn, never the
        /// rubbed-out ones, and shows nothing of the strokes.
        #[test]
        fn the_line_counts_pages_with_pencil_on_them(live in 0usize..6, erased in 0usize..6, attached in any::<bool>()) {
            let json = inked(live, erased, attached);
            let notes = parse(&json).unwrap();
            prop_assert_eq!(notes.ink_pages, live);
            let line = html(&json, true);
            match (live, attached) {
                (0, false) => prop_assert_eq!(&line, ""),
                (0, true) => prop_assert!(line.contains("pencil marks")),
                (n, _) => {
                    let counted = format!("pencil on {n} page");
                    prop_assert!(line.contains(&counted));
                }
            }
            prop_assert!(!line.contains("points") && !line.contains('['));
        }
    }

    #[test]
    fn highlights_and_pencil_share_a_line() {
        let json = "{\"file\":\"file:abc\",\"highlights\":[{\"id\":\"a\",\"comment\":\"c\"}],\"ink\":[{\"page\":2,\"strokes\":[{\"id\":\"s\"}]}]}";
        assert_eq!(
            parse(json).unwrap().summary().as_deref(),
            Some("1 highlight · 1 comment · pencil on 1 page")
        );
    }

    #[test]
    fn a_file_with_odd_characters_makes_no_link() {
        let json = "{\"file\":\"file:a\\\"><script>\",\"highlights\":[{\"id\":\"x\"}]}";
        let line = html(json, true);
        assert!(!line.contains("href"), "{line}");
        assert!(!line.contains("<script"));
    }

    #[test]
    fn an_unreadable_block_says_so_in_one_line() {
        assert_eq!(
            html("{oops", true),
            "<p class=\"kanso-pdf-notes\">PDF highlights that can&#39;t be read</p>"
        );
    }
}

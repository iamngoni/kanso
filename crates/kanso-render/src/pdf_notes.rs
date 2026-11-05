//! The quiet line that stands for a PDF's highlights in a page (issue #43,
//! step 6).
//!
//! A note keeps the highlights of each PDF it reads in a `kanso-pdf` fence
//! (see `kanso_pdf::annotation`). It is data for the reader, not text: a page
//! (the preview on a phone, an export, a published note) shows
//! "4 highlights · 1 comment" instead of the JSON. Only what the line needs is
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
}

impl Notes {
    /// "4 highlights · 1 comment"; `None` when there are no highlights.
    pub fn summary(&self) -> Option<String> {
        let (h, c) = (self.highlights, self.comments);
        if h == 0 {
            return None;
        }
        let mut s = format!("{h} highlight{}", if h == 1 { "" } else { "s" });
        if c > 0 {
            s.push_str(&format!(" · {c} comment{}", if c == 1 { "" } else { "s" }));
        }
        Some(s)
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
    Some(Notes {
        file,
        highlights: highlights.len(),
        comments,
    })
}

/// The words for a block that cannot be read.
pub const UNREADABLE: &str = "PDF highlights that can't be read";

/// The line for a page. `link` makes it a link to the PDF (the app's pages);
/// a block with no highlights draws nothing.
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

        /// Whatever is in the fence, the page shows no markup of it.
        #[test]
        fn nothing_in_the_fence_reaches_the_page_as_markup(junk in ".{0,200}") {
            let line = html(&junk, true);
            prop_assert!(!line.contains("<script") && !line.contains("<img"));
        }
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

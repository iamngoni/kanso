//! The `canvas` fence in an ordinary note, which shows a canvas (or one
//! group of it) in place:
//!
//! ````text
//! ```canvas
//! note:AbCd1234
//! group:g1
//! w:640
//! ```
//! ````
//!
//! A canvas note's own fence holds JSON, so its body starts with `{`; an
//! embed's starts with `note:`. The first character that is not a space
//! tells them apart, and no body is both.

use super::model::Embed;
use super::{FENCE, NOTE_PREFIX};

/// The narrowest and widest an embed may be asked to show, in points.
pub const WIDTHS: std::ops::RangeInclusive<u32> = 16..=4096;

/// Whether `text` could be a note ID's part after `note:`, or a group's ID:
/// something a line can hold and a person can type.
fn is_id(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= 128
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// The embed a `canvas` fence's body is, or `None` for a canvas document
/// (which starts with `{`) or a body with no usable `note:` line. Unknown
/// lines and unusable `group:` and `w:` values are ignored; a key given
/// twice keeps its first value. Pure.
pub fn parse(body: &str) -> Option<Embed> {
    if body.trim_start().starts_with('{') {
        return None;
    }
    let (mut note, mut group, mut width) = (None, None, None);
    for line in body.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim();
        match key.trim().to_ascii_lowercase().as_str() {
            "note" if note.is_none() => {
                let suffix = value.strip_prefix(NOTE_PREFIX).unwrap_or(value);
                note = Some(is_id(suffix).then(|| format!("{NOTE_PREFIX}{suffix}")));
            }
            "group" if group.is_none() => {
                group = Some(is_id(value).then(|| value.to_owned()));
            }
            "w" if width.is_none() => {
                width = Some(value.parse().ok().filter(|w| WIDTHS.contains(w)));
            }
            _ => {}
        }
    }
    Some(Embed {
        note: note??,
        group: group.flatten(),
        width: width.flatten(),
    })
}

/// The embeds written in a note's `body`, in order. Fences inside other
/// fences (a longer fence, or one in a code block) are not looked into
/// beyond the line scan this does: it reads ```` ```canvas ```` lines and
/// the closing line, which is how the editor writes them. Pure.
pub fn find(body: &str) -> Vec<Embed> {
    let mut out = Vec::new();
    let mut inner: Option<String> = None;
    for line in body.lines() {
        let line = line.trim();
        match inner.as_mut() {
            Some(text) if line == "```" => {
                out.extend(parse(text));
                inner = None;
            }
            Some(text) => {
                text.push_str(line);
                text.push('\n');
            }
            None if line
                .strip_prefix("```")
                .is_some_and(|i| i.trim().eq_ignore_ascii_case(FENCE)) =>
            {
                inner = Some(String::new());
            }
            None => {}
        }
    }
    out
}

impl Embed {
    /// An embed of the whole of canvas note `note`.
    pub fn of(note: impl Into<String>) -> Embed {
        Embed {
            note: note.into(),
            ..Embed::default()
        }
    }

    /// The same embed, showing group `group`.
    pub fn at_group(mut self, group: impl Into<String>) -> Embed {
        self.group = Some(group.into());
        self
    }

    /// What goes between the fence lines. Pure.
    pub fn body(&self) -> String {
        let suffix = self.note.strip_prefix(NOTE_PREFIX).unwrap_or(&self.note);
        let mut out = format!("{NOTE_PREFIX}{suffix}");
        if let Some(group) = &self.group {
            out.push_str("\ngroup:");
            out.push_str(group);
        }
        if let Some(width) = self.width {
            out.push_str(&format!("\nw:{width}"));
        }
        out
    }

    /// The whole fenced block, as the editor stores it. Pure.
    pub fn block(&self) -> String {
        format!("```{FENCE}\n{}\n```", self.body())
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn embeds_are_found_in_order_and_documents_skipped() {
        let body = "Intro\n```canvas\nnote:a\n```\ntext\n```canvas\n{\"nodes\":[]}\n```\n```canvas\nnote:b\ngroup:g\n```\n";
        assert_eq!(
            find(body),
            vec![Embed::of("note:a"), Embed::of("note:b").at_group("g")]
        );
    }

    #[test]
    fn the_documented_block_is_read() {
        let embed = parse("note:AbCd1234\ngroup:g1\nw:640").unwrap();
        assert_eq!(
            embed,
            Embed {
                note: "note:AbCd1234".into(),
                group: Some("g1".into()),
                width: Some(640)
            }
        );
    }

    #[test]
    fn a_canvas_document_is_not_an_embed() {
        assert_eq!(parse("{\"nodes\":[]}"), None);
        assert_eq!(parse("  \n {\"note:x\": 1}"), None);
        assert_eq!(parse("group:g1"), None);
        assert_eq!(parse(""), None);
        assert_eq!(parse("note:"), None);
        assert_eq!(parse("note:has space"), None);
    }

    #[test]
    fn bad_options_are_dropped_not_fatal() {
        let embed = parse("note:a\ngroup:\nw:huge\nflavour:mint").unwrap();
        assert_eq!(embed, Embed::of("note:a"));
        assert_eq!(parse("note:a\nw:5").unwrap().width, None);
        assert_eq!(parse("note:a\nw:99999").unwrap().width, None);
        // The first value of a repeated key wins, a leading `note:` is kept once.
        assert_eq!(parse("note:note:a\nnote:b").unwrap().note, "note:a");
        assert_eq!(parse("NOTE: a \r\nW: 300\r\n").unwrap().width, Some(300));
    }

    fn id() -> impl Strategy<Value = String> {
        "[A-Za-z0-9_-]{1,24}"
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Invariant: an embed written as a block reads back as itself.
        #[test]
        fn prop_the_fence_round_trips(
            note in id(),
            group in prop::option::of(id()),
            width in prop::option::of(16u32..=4096),
        ) {
            let embed = Embed { note: format!("note:{note}"), group, width };
            prop_assert_eq!(parse(&embed.body()), Some(embed.clone()));
            let block = embed.block();
            let inner = block
                .strip_prefix("```canvas\n")
                .and_then(|rest| rest.strip_suffix("\n```"))
                .unwrap();
            prop_assert_eq!(parse(inner), Some(embed));
        }

        /// Invariant: reading never panics, and what it finds writes back to
        /// an embed that reads the same.
        #[test]
        fn prop_any_text_reads_or_is_refused(text in "\\PC{0,120}") {
            if let Some(embed) = parse(&text) {
                prop_assert!(embed.note.starts_with(NOTE_PREFIX));
                prop_assert_eq!(parse(&embed.body()), Some(embed));
            }
        }

        /// Invariant: a body that starts like a document is never an embed.
        #[test]
        fn prop_documents_are_never_embeds(spaces in "[ \\t\\n]{0,4}", rest in "\\PC{0,60}") {
            prop_assert_eq!(parse(&format!("{spaces}{{{rest}")), None);
        }
    }
}

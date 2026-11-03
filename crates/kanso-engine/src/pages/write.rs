//! Writing one AI section: numbered passages and an instruction go to the
//! person's own model, and every line of the reply is checked against the
//! lines that were sent. A line whose receipts fail is dropped (a page is
//! read days later without the question's context, so an unverifiable line
//! is removed, not flagged as in Ask).

use std::collections::BTreeSet;

use kanso_ai::{AiError, ChatProvider, SentExcerpt};
use kanso_core::page_cache::{CachedItem, CachedSection, not_right_key};
use kanso_store::Store;

use super::{
    PAGE_BUDGET, PageSource, SectionKind, cached_source, fingerprint, inputs_of, not_right_words,
};
use crate::ask::{CITE_PREFIX, check, note_is_eligible};

/// What went wrong writing a section.
#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    #[error("{0}")]
    Ai(#[from] AiError),
    #[error("{0}")]
    Store(#[from] kanso_store::StoreError),
    #[error("The page has no such section.")]
    NoSuchSection,
}

/// What a write needs besides the notes.
pub struct WriteContext<'a> {
    pub provider: &'a dyn ChatProvider,
    /// The model's name, recorded with what it wrote.
    pub model: &'a str,
    pub now: i64,
    /// Characters the passages may take; [`PAGE_BUDGET`] by default.
    pub budget: usize,
}

impl<'a> WriteContext<'a> {
    pub fn new(provider: &'a dyn ChatProvider, model: &'a str, now: i64) -> Self {
        WriteContext {
            provider,
            model,
            now,
            budget: PAGE_BUDGET,
        }
    }
}

/// Writes AI section `key` of `source`. Only passages of notes that may be
/// sent to AI are read; the reply is checked line by line.
pub fn write_section(
    store: &Store,
    source: &dyn PageSource,
    key: &str,
    not_right: &BTreeSet<String>,
    context: &WriteContext,
) -> Result<CachedSection, WriteError> {
    let spec = source
        .sections()
        .iter()
        .find(|spec| spec.key == key && spec.kind == SectionKind::Ai)
        .ok_or(WriteError::NoSuchSection)?;
    let passages = source.passages_for(store, spec.key, context.budget)?;
    let inputs = inputs_of(store, &passages.excerpts)?;
    let based_on = passages.based_on;
    if passages.excerpts.is_empty() {
        return Ok(CachedSection {
            written_at: context.now,
            model: context.model.to_owned(),
            fingerprint: fingerprint(&inputs),
            inputs,
            none: true,
            based_on,
            ..CachedSection::default()
        });
    }
    let mut instruction = source.ai_instruction(spec.key);
    let wrong = not_right_words(not_right);
    if !wrong.is_empty() {
        instruction.push_str("\nThe user said these are wrong; do not repeat them: ");
        instruction.push_str(
            &wrong
                .iter()
                .map(|w| kanso_ai::prompt::escape_excerpt(w))
                .collect::<Vec<_>>()
                .join("; "),
        );
        instruction.push('.');
    }
    let numbered = kanso_ai::build_numbered(&instruction, &passages.excerpts, context.budget);
    let reply = crate::loops_ai::ask(context.provider, &numbered.prompt)?;
    let parsed = parse_reply(&reply, &numbered.sent, not_right);
    // A cited note must still be one that may be read.
    let items: Vec<CachedItem> = parsed
        .items
        .into_iter()
        .filter(|item| {
            item.sources
                .iter()
                .all(|s| note_is_eligible(store, &s.note))
        })
        .collect();
    Ok(CachedSection {
        none: items.is_empty(),
        items,
        written_at: context.now,
        model: context.model.to_owned(),
        fingerprint: fingerprint(&inputs),
        inputs,
        pinned: false,
        kept_at: 0,
        based_on,
    })
}

/// What a reply made of.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Parsed {
    pub items: Vec<CachedItem>,
    /// Lines dropped because their receipts failed or were missing.
    pub dropped: usize,
}

/// The lines of `reply` that are list items, with their citations checked
/// against `sent`. A line with no surviving receipt, or with a sentence
/// that fails its check, is dropped; so is one the person said was wrong.
/// A reply that is the sentinel (`NOT_IN_NOTES`) or `NONE` gives nothing.
pub fn parse_reply(reply: &str, sent: &[SentExcerpt], not_right: &BTreeSet<String>) -> Parsed {
    let trimmed = reply.trim();
    if trimmed.eq_ignore_ascii_case("none")
        || trimmed.trim_end_matches('.').eq_ignore_ascii_case("none")
        || check(trimmed, sent).not_in_notes
    {
        return Parsed::default();
    }
    let mut parsed = Parsed::default();
    for line in reply.lines() {
        let Some(text) = list_item(line) else {
            continue;
        };
        let (who, text) = split_who(text);
        let checked = check(text, sent);
        let valid = !checked.not_in_notes
            && !checked.receipts.is_empty()
            && checked.unsupported.is_empty()
            && checked.uncited.is_empty();
        if !valid {
            parsed.dropped += 1;
            continue;
        }
        let plain = strip_cites(&checked.markdown);
        if plain.is_empty() {
            parsed.dropped += 1;
            continue;
        }
        let first = &checked.receipts[0];
        if not_right.contains(&not_right_key(&first.note_id, first.start, &plain)) {
            parsed.dropped += 1;
            continue;
        }
        parsed.items.push(CachedItem {
            text: plain,
            who,
            sources: checked.receipts.iter().map(cached_source).collect(),
        });
    }
    parsed
}

/// The text of a `- item` / `* item` / `1. item` line.
fn list_item(line: &str) -> Option<&str> {
    let line = line.trim_start();
    if let Some(rest) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
        return Some(rest.trim());
    }
    let digits = line.bytes().take_while(u8::is_ascii_digit).count();
    if digits > 0
        && let Some(rest) = line[digits..].strip_prefix(". ")
    {
        return Some(rest.trim());
    }
    None
}

/// "YOU: …" and "THEM: …" name who made a promise.
fn split_who(text: &str) -> (Option<String>, &str) {
    for (tag, who) in [("you:", "you"), ("them:", "them")] {
        if text.len() >= tag.len()
            && text.is_char_boundary(tag.len())
            && text[..tag.len()].eq_ignore_ascii_case(tag)
        {
            return (Some(who.to_owned()), text[tag.len()..].trim());
        }
    }
    (None, text)
}

/// `markdown` without the `[n](kanso-cite:n)` links, and the space before
/// each.
pub fn strip_cites(markdown: &str) -> String {
    let mut out = String::with_capacity(markdown.len());
    let mut rest = markdown;
    while let Some(at) = rest.find("](") {
        let after = &rest[at + 2..];
        let open = rest[..at].rfind('[');
        match (after.strip_prefix(CITE_PREFIX), open) {
            (Some(tail), Some(open)) => {
                let end = tail.find(')').map_or(tail.len(), |e| e + 1);
                out.push_str(rest[..open].trim_end_matches(' '));
                rest = &tail[end..];
            }
            _ => {
                out.push_str(&rest[..at + 2]);
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Whether `text` could have been cited at all: for tests of the checker.
#[cfg(test)]
pub(crate) fn sent(number: usize, id: &str, lines: &[(u32, &str)]) -> SentExcerpt {
    SentExcerpt {
        number,
        id: id.to_owned(),
        title: format!("Title of {id}"),
        updated: Some("2026-10-28".to_owned()),
        lines: lines.iter().map(|(n, t)| (*n, (*t).to_owned())).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn shown() -> Vec<SentExcerpt> {
        vec![sent(
            1,
            "note:abc",
            &[
                (11, "Promised Sam the revised contract by Friday"),
                (12, "Sam will introduce me to the Lusaka team"),
            ],
        )]
    }

    #[test]
    fn a_cited_line_survives_with_its_receipt_and_who() {
        let reply = "- YOU: Promised Sam the revised contract by Friday [1:L12]\n- THEM: Sam will introduce me to the Lusaka team [1:L13]\n";
        let parsed = parse_reply(reply, &shown(), &BTreeSet::new());
        assert_eq!(parsed.items.len(), 2);
        assert_eq!(parsed.items[0].who.as_deref(), Some("you"));
        assert_eq!(
            parsed.items[0].text,
            "Promised Sam the revised contract by Friday"
        );
        assert_eq!(parsed.items[0].sources[0].start, 11);
        assert_eq!(parsed.items[1].sources[0].note, "note:abc");
        assert_eq!(parsed.dropped, 0);
    }

    #[test]
    fn invented_citations_and_unsupported_lines_are_dropped() {
        let reply = "- Bought a shiny ferrari yesterday [1:L12]\n- Promised Sam the revised contract by Friday [1:L99]\n- Promised Sam the revised contract by Friday [7:L12]\n- No citation at all about contracts\nplain prose line\n";
        let parsed = parse_reply(reply, &shown(), &BTreeSet::new());
        assert!(parsed.items.is_empty(), "{parsed:?}");
        assert_eq!(parsed.dropped, 4);
    }

    #[test]
    fn none_and_the_sentinel_give_nothing() {
        for reply in ["NONE", "none.", "NOT_IN_NOTES", " NOT_IN_NOTES \n"] {
            assert_eq!(
                parse_reply(reply, &shown(), &BTreeSet::new()),
                Parsed::default()
            );
        }
    }

    #[test]
    fn a_line_the_person_said_was_wrong_is_not_repeated() {
        let reply = "- Promised Sam the revised contract by Friday [1:L12]\n";
        let mut wrong = BTreeSet::new();
        wrong.insert(not_right_key(
            "note:abc",
            11,
            "Promised Sam the revised contract by Friday",
        ));
        let parsed = parse_reply(reply, &shown(), &wrong);
        assert!(parsed.items.is_empty());
        assert_eq!(parsed.dropped, 1);
    }

    #[test]
    fn cite_links_come_out_cleanly() {
        assert_eq!(
            strip_cites(
                "Sends the deck [1](kanso-cite:1) and a note [2](kanso-cite:2). See [x](http://a)."
            ),
            "Sends the deck and a note. See [x](http://a)."
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig { rng_seed: proptest::test_runner::RngSeed::Fixed(20261017), ..ProptestConfig::default() })]

        /// Whatever a model writes, no item survives without a receipt that
        /// points at lines it was shown, and every kept item's quote is one
        /// of the shown lines.
        #[test]
        fn prop_no_item_survives_without_a_valid_receipt(
            lines in proptest::collection::vec(
                (
                    "[A-Za-z ]{0,40}",
                    prop::option::of((0usize..4, 0u32..30)),
                ),
                0..8,
            ),
        ) {
            let sent = shown();
            let mut reply = String::new();
            for (text, cite) in &lines {
                reply.push_str("- ");
                reply.push_str(text);
                if let Some((n, line)) = cite {
                    reply.push_str(&format!(" [{n}:L{line}]"));
                }
                reply.push('\n');
            }
            let parsed = parse_reply(&reply, &sent, &BTreeSet::new());
            for item in &parsed.items {
                prop_assert!(!item.sources.is_empty());
                prop_assert!(!item.text.contains("kanso-cite"));
                for source in &item.sources {
                    prop_assert_eq!(&source.note, "note:abc");
                    let shown_text: Vec<&str> = sent[0].lines.iter().map(|(_, t)| t.as_str()).collect();
                    for quoted in source.quote.split('\n') {
                        prop_assert!(shown_text.contains(&quoted), "{quoted:?}");
                    }
                }
            }
        }
    }
}

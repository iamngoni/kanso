//! `#` autocomplete: typing `#` and a few letters offers the library's
//! tags, nested ones (`work/clients`) by any part of their path.
//!
//! What counts as a tag is `kanso_core::tags::inline_tags`: not in code, a
//! link or a heading's `#`, and only names a `#tag` can spell are offered.
//!
//! With related notes on, the tags the note's nearest notes have come
//! first ([`suggested_first`]).
//!
//! With "Tags from #words" on, saving gives the note the tags its text
//! names ([`apply_inline_tags`]).

use kanso_core::Note;
use kanso_core::tags::{SEPARATOR, inline_tags};
use kanso_store::{Origin, Store};
use serde_json::Value;

/// The most tags offered at once.
pub const LIMIT: usize = 12;
/// How far back from the caret a `#tag` is looked for.
pub const LOOK_BACK: usize = 200;

/// Whether `ch` can be part of a `#tag`.
fn is_tag_char(ch: char) -> bool {
    ch.is_alphanumeric() || matches!(ch, '_' | '-' | '/')
}

/// A `#tag` being typed at the end of `line_before` (the caret's line up
/// to the caret): the byte offset of its `#` and what follows it. `None`
/// when the caret isn't in one, and for a lone `#` that starts the line,
/// which is usually a heading on its way. Pure.
pub fn tag_query(line_before: &str) -> Option<(usize, &str)> {
    let start = line_before
        .char_indices()
        .rev()
        .take_while(|&(_, ch)| is_tag_char(ch))
        .last()
        .map_or(line_before.len(), |(ix, _)| ix);
    let hash = start.checked_sub(1)?;
    if line_before.as_bytes()[hash] != b'#' {
        return None;
    }
    let before = line_before[..hash].chars().next_back();
    if !before.is_none_or(|ch| ch.is_whitespace() || matches!(ch, '(' | '[' | '{')) {
        return None;
    }
    let query = &line_before[start..];
    if query.is_empty() && line_before[..hash].trim().is_empty() {
        return None;
    }
    Some((hash, query))
}

/// Whether a `#tag` would start right after `before_hash` (everything in
/// the note before the `#`): not in code, a link, a heading or front
/// matter, by the store's own rule. Pure.
pub fn tag_starts_at(before_hash: &str) -> bool {
    let mut probe = String::with_capacity(before_hash.len() + 2);
    probe.push_str(before_hash);
    probe.push_str("#x");
    inline_tags(&probe)
        .last()
        .is_some_and(|tag| tag.range.start == before_hash.len())
}

/// Tags matching `query`, best first: by the whole name, or by the part
/// after a `/`. Names a `#tag` can't spell (with spaces, say) are left
/// out. Pure.
pub fn candidates<'a>(names: &[&'a str], query: &str, limit: usize) -> Vec<&'a str> {
    let spellable = |name: &str| {
        !name.is_empty()
            && name.chars().all(is_tag_char)
            && inline_tags(&format!("#{name}"))
                .first()
                .is_some_and(|tag| tag.name == name)
    };
    let mut scored: Vec<(i64, &str)> = names
        .iter()
        .copied()
        .filter(|name| spellable(name))
        .filter_map(|name| {
            if query.is_empty() {
                return Some((0, name));
            }
            let whole = crate::fuzzy::score(query, name);
            let leaf = name
                .rsplit(SEPARATOR)
                .next()
                .and_then(|leaf| crate::fuzzy::score(query, leaf));
            whole.max(leaf).map(|score| (score, name))
        })
        .collect();
    scored.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| a.1.to_lowercase().cmp(&b.1.to_lowercase()))
    });
    scored
        .into_iter()
        .take(limit)
        .map(|(_, name)| name)
        .collect()
}

/// `found` (tags matching the query, best first) with the `suggested`
/// tags that match the query moved to the front, in their own order, at
/// most `limit` in all. Pure.
pub fn suggested_first<'a>(
    found: Vec<&'a str>,
    suggested: &[&'a str],
    query: &str,
    limit: usize,
) -> Vec<&'a str> {
    let fitting = candidates(suggested, query, usize::MAX);
    let mut out: Vec<&str> = Vec::new();
    for name in suggested.iter().copied() {
        if fitting.contains(&name) && !out.contains(&name) {
            out.push(name);
        }
    }
    for name in found {
        if !out.contains(&name) {
            out.push(name);
        }
    }
    out.truncate(limit);
    out
}

/// One tag `#` autocomplete offers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TagChoice<'a> {
    pub name: &'a str,
    /// Whether the note's nearest notes have it ("Suggested by close
    /// notes").
    pub suggested: bool,
}

/// The tags `#` autocomplete offers for `query` among `names`, the
/// `suggested` ones first, at most [`LIMIT`], leaving out the tag already
/// typed in full. Pure.
pub fn choices<'a>(names: &[&'a str], suggested: &[&'a str], query: &str) -> Vec<TagChoice<'a>> {
    let found = candidates(names, query, LIMIT);
    suggested_first(found, suggested, query, LIMIT)
        .into_iter()
        .filter(|name| *name != query)
        .map(|name| TagChoice {
            name,
            suggested: suggested.contains(&name),
        })
        .collect()
}

/// The note's field listing the tags its text gave it, so a `#tag` taken
/// out of the text takes its tag off, and a tag added by hand never is.
pub const INLINE_TAGS_FIELD: &str = "kansoInlineTags";

/// Gives `note` the tags its text names (made when missing), keeping every
/// tag added by hand; a `#tag` the caret is in (byte `caret` of the body),
/// still being typed, is left for later.
pub fn apply_inline_tags(store: &Store, note: &mut Note, caret: Option<usize>) {
    let mut seen = std::collections::HashSet::new();
    let named: Vec<String> = inline_tags(&note.body)
        .into_iter()
        .filter(|tag| caret.is_none_or(|at| !(tag.range.start < at && at <= tag.range.end)))
        .filter(|tag| seen.insert(tag.name.to_lowercase()))
        .filter_map(|tag| store.tag_named(&tag.name, Origin::Ui).ok())
        .collect();
    let before: Vec<String> = note
        .extra
        .get(INLINE_TAGS_FIELD)
        .and_then(Value::as_array)
        .map(|ids| {
            ids.iter()
                .filter_map(|id| id.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    let (tags, given) = kanso_core::tags::merge_inline(&note.tags, &before, &named);
    note.tags = tags;
    if given.is_empty() {
        note.extra.remove(INLINE_TAGS_FIELD);
    } else {
        note.extra.insert(INLINE_TAGS_FIELD.into(), given.into());
    }
}

/// The scheme of the links inline `#tags` become on the phone's page.
pub const TAG_SCHEME: &str = "kanso-tag:";

/// The tag name a `kanso-tag:` link holds. Pure.
pub fn tag_from_url(url: &str) -> Option<String> {
    let name = super::percent_decode(url.strip_prefix(TAG_SCHEME)?)?;
    (!name.is_empty()).then_some(name)
}

/// The note's Markdown with each inline `#tag` (what [`inline_tags`] finds:
/// not in code, a link or a heading) written as a `kanso-tag:` link, for the
/// phone's page to make tappable. Text without tags comes back as it is
/// (borrowed). Pure.
pub fn rewrite_for_app(text: &str) -> std::borrow::Cow<'_, str> {
    if !text.contains('#') {
        return std::borrow::Cow::Borrowed(text);
    }
    let tags = inline_tags(text);
    if tags.is_empty() {
        return std::borrow::Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len() + tags.len() * 24);
    let mut last = 0;
    for tag in tags {
        out.push_str(&text[last..tag.range.start]);
        out.push_str(&format!(
            "[#{}]({TAG_SCHEME}{})",
            tag.name,
            super::percent_encode(&tag.name)
        ));
        last = tag.range.end;
    }
    out.push_str(&text[last..]);
    std::borrow::Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use super::{
        INLINE_TAGS_FIELD, LIMIT, apply_inline_tags, candidates, choices, suggested_first,
        tag_query, tag_starts_at,
    };
    use kanso_core::tags::inline_tags;
    use kanso_core::{DocType, Note};
    use kanso_store::{Origin, Put, Store};
    use proptest::prelude::*;
    use serde_json::json;

    /// Tags from the text come and go with it; a tag added by hand stays,
    /// and the `#tag` being typed waits.
    #[test]
    fn inline_tags_follow_the_text() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        let book = store
            .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
            .unwrap()
            .id;
        let hand = store.tag_named("pinned-by-hand", Origin::Ui).unwrap();
        let mut note: Note = serde_json::from_value(json!({
            "_id": "note:x", "title": "T", "bookId": book, "tags": [hand],
            "body": "Notes on #work/acme and `#code` and #rea",
        }))
        .unwrap();
        let caret = note.body.len();
        apply_inline_tags(&store, &mut note, Some(caret));
        let names = |note: &Note| {
            let mut names: Vec<String> = note
                .tags
                .iter()
                .map(|id| {
                    store.get(id).unwrap().unwrap().fields["name"]
                        .as_str()
                        .unwrap()
                        .to_owned()
                })
                .collect();
            names.sort();
            names
        };
        assert_eq!(names(&note), ["pinned-by-hand", "work/acme"]);
        apply_inline_tags(&store, &mut note, None);
        assert_eq!(names(&note), ["pinned-by-hand", "rea", "work/acme"]);
        assert_eq!(note.extra[INLINE_TAGS_FIELD].as_array().unwrap().len(), 2);
        note.body = "Nothing now; #pinned-by-hand".into();
        apply_inline_tags(&store, &mut note, None);
        assert_eq!(names(&note), ["pinned-by-hand"]);
        note.body = String::new();
        apply_inline_tags(&store, &mut note, None);
        // Added by hand before the text named it: still there.
        assert_eq!(names(&note), ["pinned-by-hand"]);
        assert!(!note.extra.contains_key(INLINE_TAGS_FIELD));
    }

    #[test]
    fn queries() {
        assert_eq!(tag_query("see #wor"), Some((4, "wor")));
        assert_eq!(tag_query("see #"), Some((4, "")));
        assert_eq!(tag_query("(#work/cl"), Some((1, "work/cl")));
        assert_eq!(tag_query("#work"), Some((0, "work")));
        // A heading on its way.
        assert_eq!(tag_query("#"), None);
        assert_eq!(tag_query("  #"), None);
        assert_eq!(tag_query("## Head"), None);
        assert_eq!(tag_query("issue#4"), None);
        assert_eq!(tag_query("see #work done"), None);
        assert_eq!(tag_query("plain"), None);
    }

    #[test]
    fn a_tag_starts_outside_code_and_links() {
        assert!(tag_starts_at("see "));
        assert!(tag_starts_at(""));
        assert!(!tag_starts_at("```\nfenced\n"));
    }

    #[test]
    fn nested_tags_match_by_any_part() {
        let names = ["work", "work/clients", "home", "read later", "Writing"];
        assert_eq!(candidates(&names, "cli", 5), vec!["work/clients"]);
        assert_eq!(candidates(&names, "wor", 5)[0], "work");
        // Not spellable as a #tag.
        assert!(!candidates(&names, "", 10).contains(&"read later"));
        assert_eq!(candidates(&names, "", 2).len(), 2);
    }

    #[test]
    fn suggested_tags_come_first() {
        let found = vec!["work", "writing", "home"];
        assert_eq!(
            suggested_first(found.clone(), &["home", "garden"], "", 10),
            ["home", "garden", "work", "writing"]
        );
        // Only those that fit what is typed.
        assert_eq!(
            suggested_first(found, &["home", "writing"], "wr", 10),
            ["writing", "work", "home"]
        );
    }

    #[test]
    fn choices_mark_the_suggested_and_skip_what_is_typed() {
        let names = ["work", "writing", "home"];
        let got = choices(&names, &["home"], "");
        assert_eq!(got[0].name, "home");
        assert!(got[0].suggested);
        assert!(!got[1].suggested);
        let got = choices(&names, &[], "work");
        assert!(got.iter().all(|c| c.name != "work"));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// Suggestions that fit the query lead, nothing repeats, nothing
        /// found is lost while there is room, and nothing comes from
        /// nowhere.
        #[test]
        fn prop_suggestions_lead_without_losing_matches(
            names in prop::collection::btree_set("[a-z]{1,5}", 0..8),
            suggested in prop::collection::btree_set("[a-z]{1,5}", 0..4),
            query in "[a-z]{0,2}",
            limit in 0usize..16,
        ) {
            let names: Vec<&str> = names.iter().map(String::as_str).collect();
            let suggested: Vec<&str> = suggested.iter().map(String::as_str).collect();
            let found = candidates(&names, &query, usize::MAX);
            let out = suggested_first(found.clone(), &suggested, &query, limit);
            prop_assert!(out.len() <= limit);
            let mut seen = std::collections::HashSet::new();
            for name in &out {
                prop_assert!(seen.insert(*name));
                prop_assert!(found.contains(name) || suggested.contains(name));
            }
            let fitting = candidates(&suggested, &query, usize::MAX);
            let lead = out.iter().take_while(|n| fitting.contains(n)).count();
            prop_assert_eq!(lead, fitting.len().min(limit));
            if fitting.len() + found.len() <= limit {
                for name in &found {
                    prop_assert!(out.contains(name));
                }
            }
        }

        /// Invariants: a query found ends at the caret, starts after a `#`,
        /// and is made of tag characters only.
        #[test]
        fn prop_query_ends_at_the_caret(line in "[a-z #/(\\-]{0,30}") {
            if let Some((hash, query)) = tag_query(&line) {
                prop_assert_eq!(&line[hash..hash + 1], "#");
                prop_assert_eq!(&line[hash + 1..], query);
                prop_assert!(query.chars().all(|c| c.is_alphanumeric() || c == '/' || c == '-' || c == '_'));
            }
        }

        /// Every candidate matches the query and comes from the names.
        #[test]
        fn prop_candidates_come_from_names(
            names in prop::collection::vec("[a-z]{1,6}(/[a-z]{1,6})?", 0..12),
            query in "[a-z]{0,3}",
        ) {
            let names: Vec<&str> = names.iter().map(String::as_str).collect();
            let found = candidates(&names, &query, 50);
            for name in &found {
                prop_assert!(names.contains(name));
            }
            if query.is_empty() {
                let mut unique = names.clone();
                unique.sort_unstable();
                unique.dedup();
                prop_assert!(found.len() >= unique.len().min(1).min(names.len()));
            }
        }

        /// A tag starts after some text exactly when `inline_tags` finds a
        /// tag there once a real tag is written after it.
        #[test]
        fn prop_a_tag_starts_where_inline_tags_finds_one(
            before in prop::collection::vec(
                prop_oneof![
                    "[a-z ]{0,5}", Just("`".to_owned()), Just("```\n".to_owned()),
                    Just("# ".to_owned()), Just("[".to_owned()), Just("](x) ".to_owned()),
                    Just("\n".to_owned()), Just("---\n".to_owned()),
                ],
                0..8,
            ),
        ) {
            let before = before.concat();
            let found = inline_tags(&format!("{before}#work"))
                .iter()
                .any(|tag| tag.range.start == before.len() && tag.name == "work");
            prop_assert_eq!(tag_starts_at(&before), found, "{:?}", before);
        }

        /// The choices never offer the query itself, offer no tag twice and
        /// stay within the limit; the suggested flag says which came from the
        /// suggestions.
        #[test]
        fn prop_choices_are_clean(
            names in prop::collection::vec("[a-z]{1,5}(/[a-z]{1,5})?", 0..16),
            suggested in prop::collection::vec("[a-z]{1,5}", 0..6),
            query in "[a-z]{0,3}",
        ) {
            let names: Vec<&str> = names.iter().map(String::as_str).collect();
            let suggested: Vec<&str> = suggested.iter().map(String::as_str).collect();
            let got = choices(&names, &suggested, &query);
            prop_assert!(got.len() <= LIMIT);
            let mut seen = std::collections::HashSet::new();
            for choice in &got {
                prop_assert!(choice.name != query);
                prop_assert!(seen.insert(choice.name));
                prop_assert_eq!(choice.suggested, suggested.contains(&choice.name));
            }
        }
    }
}

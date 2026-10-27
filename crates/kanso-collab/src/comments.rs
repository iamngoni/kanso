//! Comments (docs/ACCOUNTS.md › Collaboration): signed events kept outside
//! the note's text, folded into comments in log order, and placed in the
//! text by their sticky anchors.

use std::collections::BTreeMap;

use crate::doc::{CollabDoc, Field};
use crate::model::{Comment, CommentEvent, CommentOp, CommentView, Placement, Role};

/// The longest comment body, in characters.
pub const MAX_BODY_CHARS: usize = 4_000;
/// The longest quote an anchor keeps, in characters.
pub const MAX_QUOTE_CHARS: usize = 280;

/// One comment event as the log delivered it, with what its envelope says
/// about its author.
#[derive(Clone, Debug)]
pub struct Logged {
    pub seq: u64,
    pub author: String,
    pub author_name: String,
    /// The author's role in the member list of the event's epoch.
    pub role: Role,
    pub event: CommentEvent,
}

/// Folds events into comments, in log order: `edit` and `delete` count from
/// the comment's author only; `resolve` and `reopen` from anyone who may
/// comment; nothing counts from a viewer, or for a comment never added (or
/// added twice).
pub fn fold(events: &[Logged]) -> BTreeMap<String, Comment> {
    let mut comments: BTreeMap<String, Comment> = BTreeMap::new();
    let mut deleted: std::collections::BTreeSet<String> = Default::default();
    // When each comment was last resolved or reopened: an older event the
    // log delivers again doesn't undo a newer one.
    let mut state_at: BTreeMap<String, i64> = BTreeMap::new();
    let mut ordered: Vec<&Logged> = events.iter().collect();
    ordered.sort_by_key(|e| e.seq);
    for logged in ordered {
        if !logged.role.may_comment() || deleted.contains(&logged.event.id) {
            continue;
        }
        let event = &logged.event;
        match event.op {
            CommentOp::Add => {
                let (Some(anchor), Some(body)) = (&event.anchor, &event.body) else {
                    continue;
                };
                if comments.contains_key(&event.id) || body.chars().count() > MAX_BODY_CHARS {
                    continue;
                }
                comments.insert(
                    event.id.clone(),
                    Comment {
                        id: event.id.clone(),
                        author: logged.author.clone(),
                        author_name: logged.author_name.clone(),
                        anchor: anchor.clone(),
                        body: body.clone(),
                        mentions: event.mentions.clone(),
                        resolved: false,
                        created_at: event.at,
                        edited_at: None,
                        seq: logged.seq,
                    },
                );
            }
            CommentOp::Edit => {
                if let (Some(comment), Some(body)) = (comments.get_mut(&event.id), &event.body)
                    && comment.author == logged.author
                    && body.chars().count() <= MAX_BODY_CHARS
                    && event.at >= comment.edited_at.unwrap_or(comment.created_at)
                {
                    comment.body = body.clone();
                    comment.edited_at = Some(event.at);
                    comment.seq = logged.seq;
                }
            }
            CommentOp::Resolve | CommentOp::Reopen => {
                let last = state_at.get(&event.id).copied().unwrap_or(i64::MIN);
                if event.at >= last
                    && let Some(comment) = comments.get_mut(&event.id)
                {
                    state_at.insert(event.id.clone(), event.at);
                    comment.resolved = event.op == CommentOp::Resolve;
                    comment.seq = logged.seq;
                }
            }
            CommentOp::Delete => {
                if comments
                    .get(&event.id)
                    .is_some_and(|c| c.author == logged.author)
                {
                    comments.remove(&event.id);
                    deleted.insert(event.id.clone());
                }
            }
        }
    }
    comments
}

/// Where a comment sits in the note's body now. Its anchor's two positions
/// span its words; when they meet and the quoted words are gone, the
/// comment is orphaned at that place. An anchor this document can't resolve
/// is orphaned at the start.
pub fn place(doc: &CollabDoc, comment: &Comment) -> Placement {
    let start = doc.resolve(&comment.anchor.start);
    let end = doc.resolve(&comment.anchor.end);
    match (start, end) {
        (Some(start), Some(end)) if start < end => Placement::Anchored(start..end),
        (Some(at), _) | (None, Some(at)) => {
            if comment.anchor.quote.is_empty() {
                Placement::Anchored(at..at)
            } else {
                Placement::Orphaned(at)
            }
        }
        (None, None) => Placement::Orphaned(0),
    }
}

/// An anchor for the byte range `range` of the body.
pub fn anchor(doc: &CollabDoc, range: std::ops::Range<usize>) -> crate::model::Anchor {
    let body = doc.text(Field::Body);
    let start = range.start.min(body.len());
    let end = range.end.clamp(start, body.len());
    let quote: String = body
        .get(start..end)
        .unwrap_or("")
        .chars()
        .take(MAX_QUOTE_CHARS)
        .collect();
    crate::model::Anchor {
        start: doc.sticky(Field::Body, start),
        end: doc.sticky(Field::Body, end),
        quote,
    }
}

/// The accounts a comment mentions: each `@name` that is a member's
/// username, as their account IDs, once each, the author left out.
pub fn mentions(body: &str, members: &[(String, String)], author: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (i, _) in body.match_indices('@') {
        let before_ok = body[..i]
            .chars()
            .next_back()
            .is_none_or(|c| !c.is_ascii_alphanumeric());
        if !before_ok {
            continue;
        }
        let name: String = body[i + 1..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
            .collect::<String>()
            .to_ascii_lowercase();
        let name = name.trim_end_matches(['-', '_']);
        if let Some((account, _)) = members.iter().find(|(_, username)| username == name)
            && account != author
            && !out.contains(account)
        {
            out.push(account.clone());
        }
    }
    out
}

/// The Comments tab's two groups, as indices into `comments`: those whose
/// words are still in the text, in text order, then those whose words were
/// deleted ("Orphaned"), by where their words were. Earlier comments first
/// where two start at the same place.
pub fn comment_groups(comments: &[CommentView]) -> (Vec<usize>, Vec<usize>) {
    let mut anchored = Vec::new();
    let mut orphaned = Vec::new();
    for (ix, view) in comments.iter().enumerate() {
        match &view.placement {
            Placement::Anchored(range) => anchored.push((range.start, view.comment.created_at, ix)),
            Placement::Orphaned(at) => orphaned.push((*at, view.comment.created_at, ix)),
        }
    }
    anchored.sort();
    orphaned.sort();
    (
        anchored.into_iter().map(|(_, _, ix)| ix).collect(),
        orphaned.into_iter().map(|(_, _, ix)| ix).collect(),
    )
}

/// A comment's words in runs, each marked when it is an @mention of one of
/// `members` (usernames): `@` at the start or after a non-letter, then the
/// name, as the engine finds mentions. The runs cover `body` in order.
pub fn mention_runs(body: &str, members: &[String]) -> Vec<(std::ops::Range<usize>, bool)> {
    let mut runs = Vec::new();
    let mut plain = 0;
    for (i, _) in body.match_indices('@') {
        if i < plain {
            continue;
        }
        let after_word = body[..i]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_ascii_alphanumeric());
        if after_word {
            continue;
        }
        let name: &str = {
            let rest = &body[i + 1..];
            let end = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '_' | '-')))
                .unwrap_or(rest.len());
            rest[..end].trim_end_matches(['-', '_'])
        };
        if name.is_empty()
            || !members
                .iter()
                .any(|member| member.eq_ignore_ascii_case(name))
        {
            continue;
        }
        if plain < i {
            runs.push((plain..i, false));
        }
        let end = i + 1 + name.len();
        runs.push((i..end, true));
        plain = end;
    }
    if plain < body.len() {
        runs.push((plain..body.len(), false));
    }
    runs
}

/// The Comments tab's summary: "2 open · 1 resolved"; empty with none.
pub fn comments_summary(comments: &[CommentView]) -> String {
    let resolved = comments.iter().filter(|v| v.comment.resolved).count();
    let open = comments.len() - resolved;
    let mut parts = Vec::new();
    if open > 0 {
        parts.push(format!("{open} open"));
    }
    if resolved > 0 {
        parts.push(format!("{resolved} resolved"));
    }
    parts.join(" · ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Anchor;
    use proptest::prelude::*;

    fn add(seq: u64, author: &str, role: Role, id: &str) -> Logged {
        Logged {
            seq,
            author: author.into(),
            author_name: author.into(),
            role,
            event: CommentEvent {
                id: id.into(),
                op: CommentOp::Add,
                anchor: Some(Anchor {
                    start: String::new(),
                    end: String::new(),
                    quote: "q".into(),
                }),
                body: Some(format!("by {author}")),
                mentions: vec![],
                at: seq as i64,
            },
        }
    }

    fn op(seq: u64, author: &str, role: Role, id: &str, op: CommentOp) -> Logged {
        Logged {
            seq,
            author: author.into(),
            author_name: author.into(),
            role,
            event: CommentEvent {
                id: id.into(),
                op,
                anchor: None,
                body: (op == CommentOp::Edit).then(|| format!("edited by {author}")),
                mentions: vec![],
                at: seq as i64,
            },
        }
    }

    #[test]
    fn only_the_author_edits_or_deletes_and_anyone_commenting_resolves() {
        let events = vec![
            add(1, "a", Role::Comment, "c1"),
            op(2, "b", Role::Edit, "c1", CommentOp::Edit),
            op(3, "b", Role::Edit, "c1", CommentOp::Delete),
            op(4, "b", Role::Comment, "c1", CommentOp::Resolve),
            op(5, "v", Role::View, "c1", CommentOp::Reopen),
            add(6, "v", Role::View, "c2"),
            op(7, "a", Role::Comment, "c1", CommentOp::Edit),
        ];
        let folded = fold(&events);
        assert_eq!(folded.len(), 1);
        let c1 = &folded["c1"];
        assert_eq!(c1.body, "edited by a");
        assert!(c1.resolved);
    }

    #[test]
    fn mentions_pick_members_only() {
        let members = vec![
            ("a_1".to_owned(), "ana".to_owned()),
            ("a_2".to_owned(), "bo-b".to_owned()),
        ];
        assert_eq!(
            mentions(
                "hi @ana and @bo-b, not me@ana or @carl. @Ana again",
                &members,
                "a_9"
            ),
            vec!["a_1".to_owned(), "a_2".to_owned()]
        );
        assert!(mentions("@ana", &members, "a_1").is_empty());
    }

    #[test]
    fn a_comment_whose_words_are_deleted_is_orphaned() {
        let doc = CollabDoc::from_text("", "one two three");
        let comment = Comment {
            id: "c".into(),
            author: "a".into(),
            author_name: "a".into(),
            anchor: anchor(&doc, 4..7),
            body: "b".into(),
            mentions: vec![],
            resolved: false,
            created_at: 0,
            edited_at: None,
            seq: 1,
        };
        assert_eq!(comment.anchor.quote, "two");
        assert_eq!(place(&doc, &comment), Placement::Anchored(4..7));
        doc.set_text(Field::Body, "zero one two three");
        assert_eq!(place(&doc, &comment), Placement::Anchored(9..12));
        doc.set_text(Field::Body, "zero one  three");
        assert!(matches!(place(&doc, &comment), Placement::Orphaned(9)));
    }

    // no-prop-test: the words at the very end of the text, one fixed case.
    #[test]
    fn a_comment_on_the_last_words_stays_on_them() {
        let doc = CollabDoc::from_text("", "one two three");
        let comment = Comment {
            id: "c".into(),
            author: "a".into(),
            author_name: "a".into(),
            anchor: anchor(&doc, 8..13),
            body: "b".into(),
            mentions: vec![],
            resolved: false,
            created_at: 0,
            edited_at: None,
            seq: 1,
        };
        assert_eq!(comment.anchor.quote, "three");
        assert_eq!(place(&doc, &comment), Placement::Anchored(8..13));
        doc.set_text(Field::Body, "zero one two three");
        assert_eq!(place(&doc, &comment), Placement::Anchored(13..18));
    }

    // no-prop-test: the end of texts with wide characters, fixed cases; the
    // mobile comments test holds the property over random texts.
    #[test]
    fn the_end_of_a_text_with_wide_characters_stays_the_end() {
        for text in ["é end", "😀 end", "end é", "aé", "é"] {
            let doc = CollabDoc::from_text("", text);
            let end = doc.sticky(Field::Body, text.len());
            assert_eq!(doc.resolve(&end), Some(text.len()), "{text:?}");
            doc.set_text(Field::Body, &format!("xx{text}"));
            assert_eq!(doc.resolve(&end), Some(text.len() + 2), "{text:?} after");
        }
    }

    // no-prop-test: the last words after the document is saved and loaded.
    #[test]
    fn a_comment_on_the_last_words_survives_a_reload() {
        let doc = CollabDoc::from_text("", "one two three");
        let sticky_end = doc.sticky(Field::Body, 13);
        let loaded = CollabDoc::load(&doc.encode_state()).unwrap();
        assert_eq!(loaded.resolve(&sticky_end), Some(13), "end after reload");
        doc.set_text(Field::Body, "zero one two three");
        assert_eq!(doc.resolve(&sticky_end), Some(18), "end after an edit");
        let loaded = CollabDoc::load(&doc.encode_state()).unwrap();
        assert_eq!(loaded.resolve(&sticky_end), Some(18), "end after both");
    }

    proptest! {
        #![proptest_config(crate::tests::config(128))]

        /// Folding is a function of the log, not of delivery order, and a
        /// viewer's events never change it.
        #[test]
        fn prop_fold_ignores_order_and_viewers(ops in prop::collection::vec((0usize..3, 0usize..3, 0usize..6, 0usize..3), 0..30)) {
            let authors = ["a", "b", "v"];
            let ids = ["x", "y", "z"];
            let kinds = [CommentOp::Add, CommentOp::Edit, CommentOp::Resolve, CommentOp::Reopen, CommentOp::Delete, CommentOp::Add];
            let mut events = Vec::new();
            for (seq, (a, i, k, _)) in ops.iter().enumerate() {
                let role = if *a == 2 { Role::View } else { Role::Comment };
                let seq = seq as u64 + 1;
                events.push(if kinds[*k] == CommentOp::Add { add(seq, authors[*a], role, ids[*i]) } else { op(seq, authors[*a], role, ids[*i], kinds[*k]) });
            }
            let folded = fold(&events);
            let mut reversed = events.clone();
            reversed.reverse();
            prop_assert_eq!(&fold(&reversed), &folded);
            let without_viewer: Vec<Logged> = events.iter().filter(|e| e.role != Role::View).cloned().collect();
            prop_assert_eq!(&fold(&without_viewer), &folded);
            for comment in folded.values() {
                prop_assert_ne!(comment.author.as_str(), "v");
            }
        }
    }

    fn view(id: &str, placement: Placement, resolved: bool, created_at: i64) -> CommentView {
        CommentView {
            comment: Comment {
                id: id.into(),
                author: "a".into(),
                author_name: "user1".into(),
                anchor: Anchor {
                    start: String::new(),
                    end: String::new(),
                    quote: "q".into(),
                },
                body: "b".into(),
                mentions: Vec::new(),
                resolved,
                created_at,
                edited_at: None,
                seq: 0,
            },
            placement,
            pending: false,
            mine: false,
        }
    }

    #[test]
    fn mentions_of_members_stand_out() {
        let members = vec!["user2".to_owned(), "ann-b".to_owned()];
        let runs = |body: &str| -> Vec<(String, bool)> {
            mention_runs(body, &members)
                .into_iter()
                .map(|(r, m)| (body[r].to_owned(), m))
                .collect()
        };
        assert_eq!(
            runs("Ask @User2, and @ann-b_."),
            [
                ("Ask ".into(), false),
                ("@User2".into(), true),
                (", and ".into(), false),
                ("@ann-b".into(), true),
                ("_.".into(), false),
            ]
        );
        // Not a member, or inside a word: plain.
        assert_eq!(
            runs("mail me@user2 or @nobody"),
            [("mail me@user2 or @nobody".into(), false)]
        );
        assert!(mention_runs("", &members).is_empty());
    }

    proptest! {
        /// Every comment is in exactly one group; the anchored ones in text
        /// order, the orphaned ones in the order their words were.
        #[test]
        fn prop_comment_groups_cover_every_comment(specs in prop::collection::vec((0usize..60, 0usize..10, any::<bool>(), 0i64..5), 0..12)) {
            let comments: Vec<CommentView> = specs
                .iter()
                .enumerate()
                .map(|(i, (start, len, orphaned, at))| {
                    let placement = if *orphaned {
                        Placement::Orphaned(*start)
                    } else {
                        Placement::Anchored(*start..start + len)
                    };
                    view(&format!("c{i}"), placement, false, *at)
                })
                .collect();
            let (anchored, orphaned) = comment_groups(&comments);
            let mut all: Vec<usize> = anchored.iter().chain(&orphaned).copied().collect();
            all.sort_unstable();
            prop_assert_eq!(all, (0..comments.len()).collect::<Vec<_>>());
            let start = |ix: usize| match &comments[ix].placement {
                Placement::Anchored(r) => r.start,
                Placement::Orphaned(at) => *at,
            };
            for pair in anchored.windows(2) {
                prop_assert!(matches!(comments[pair[0]].placement, Placement::Anchored(_)));
                prop_assert!(start(pair[0]) <= start(pair[1]));
            }
            for pair in orphaned.windows(2) {
                prop_assert!(start(pair[0]) <= start(pair[1]));
            }
            for &ix in &orphaned {
                prop_assert!(matches!(comments[ix].placement, Placement::Orphaned(_)));
            }
        }

        /// The runs cover the words exactly, in order, and every marked run
        /// is `@` and a member's name.
        #[test]
        fn prop_mention_runs_cover_the_words(words in prop::collection::vec(prop::sample::select(vec!["@user2", "@USER2", "@ann", "x@user2", "@", "@nobody", " ", "é", "-", "_", "hi"]), 0..12)) {
            let body: String = words.concat();
            let members = vec!["user2".to_owned(), "ann".to_owned()];
            let runs = mention_runs(&body, &members);
            let mut at = 0;
            for (range, mention) in &runs {
                prop_assert_eq!(range.start, at);
                prop_assert!(range.start < range.end);
                at = range.end;
                if *mention {
                    let name = &body[range.start + 1..range.end];
                    prop_assert!(body[range.clone()].starts_with('@'));
                    prop_assert!(members.iter().any(|m| m.eq_ignore_ascii_case(name)));
                }
            }
            prop_assert_eq!(at, body.len());
        }
    }
}

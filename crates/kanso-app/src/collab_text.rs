//! The pure side of shared notes in the editor: where carets and
//! selections go when someone else's edits are merged in, where other
//! people's cursors and comments sit in the text, and the words the
//! editor and the Share sheet use.
//!
//! Every offset is a UTF-8 byte offset, as the engine's splices and the
//! text editor's selections both count.

use std::ops::Range;

pub use kanso_collab::comments::{comment_groups, comments_summary, mention_runs};
use kanso_collab::{CommentView, Placement, Problem, RemoteCursor, Role, SizeUse, Splice, remap};

use crate::collab_model::RemoteMark;

/// The share a note's `kansoShare` names: its ID, this account's role and
/// the owner's username. `None` for a note that isn't shared.
pub fn share_mark(
    extra: &serde_json::Map<String, serde_json::Value>,
) -> Option<(String, Role, String)> {
    let mark = extra.get(kanso_core::model::SHARE_FIELD)?.as_object()?;
    let share_id = mark.get("shareId")?.as_str()?.to_owned();
    let role = mark
        .get("role")
        .and_then(|r| r.as_str())
        .and_then(Role::parse)
        .unwrap_or(Role::View);
    let owner = mark
        .get("ownerName")
        .and_then(|o| o.as_str())
        .unwrap_or_default()
        .to_owned();
    Some((share_id, role, owner))
}

/// The largest char boundary of `text` at or before `at`.
pub fn floor_boundary(text: &str, at: usize) -> usize {
    let mut at = at.min(text.len());
    while !text.is_char_boundary(at) {
        at -= 1;
    }
    at
}

/// Where a selection goes through `splices` (applied in order): each end
/// follows its own text, so a caret before the edits stays put and one
/// after them moves with what it is next to. `anchor` and `head` keep their
/// order, so a selection made backwards stays backwards.
pub fn map_selection(anchor: usize, head: usize, splices: &[Splice]) -> (usize, usize) {
    (remap(anchor, splices), remap(head, splices))
}

/// Vim's cursor and shown Visual selection after a merge: they follow
/// their text as the editor's selection does, and in Normal mode the
/// cursor stays on a character, never past the end of its line.
pub fn map_vim(
    head: usize,
    shown: Option<Range<usize>>,
    normal: bool,
    splices: &[Splice],
    text: &str,
) -> (usize, Option<Range<usize>>) {
    let head = floor_boundary(text, remap(head, splices));
    let head = if normal {
        crate::vim::clamp_normal(text, head)
    } else {
        head
    };
    let shown = shown.map(|range| {
        let start = floor_boundary(text, remap(range.start, splices));
        let end = floor_boundary(text, remap(range.end, splices));
        start.min(end)..start.max(end)
    });
    (head, shown)
}

/// Other people's cursors as the editor draws them: in the text (clamped
/// to it, on character boundaries), the selection in order.
pub fn remote_marks(cursors: &[RemoteCursor], text: &str) -> Vec<RemoteMark> {
    cursors
        .iter()
        .map(|cursor| {
            let anchor = floor_boundary(text, cursor.anchor);
            let head = floor_boundary(text, cursor.head);
            RemoteMark {
                name: cursor.name.clone(),
                caret: head,
                selection: anchor.min(head)..anchor.max(head),
                colour: cursor_colour(cursor.colour),
            }
        })
        .collect()
}

/// Which of the theme's cursor colours an index from the engine picks:
/// the same one on every device, whatever the index.
pub fn cursor_colour(index: usize) -> usize {
    index % crate::theme::CURSOR_COLOURS.len()
}

/// The text the open, anchored comments are on, to tint in the editor:
/// clamped to `text` and on character boundaries; empty ranges left out.
pub fn comment_ranges(comments: &[CommentView], text: &str) -> Vec<Range<usize>> {
    comments
        .iter()
        .filter(|view| !view.comment.resolved)
        .filter_map(|view| match &view.placement {
            Placement::Anchored(range) => {
                let start = floor_boundary(text, range.start);
                let end = floor_boundary(text, range.end.max(range.start));
                (start < end).then_some(start..end)
            }
            Placement::Orphaned(_) => None,
        })
        .collect()
}

/// What the editor's footer says about a shared note: that it is shared,
/// the role, whether it can be changed here, whether it is live, and when
/// it nears its size limit.
pub fn share_footer(role: Role, connected: bool, ended: bool, size: Option<SizeUse>) -> String {
    if ended {
        return "Private copy · No longer shared".into();
    }
    let mut parts = vec!["Shared", role.label()];
    if !role.may_edit() {
        parts.push("Read only");
    }
    parts.push(if connected { "Live" } else { "Offline" });
    if size.is_some_and(|size| size.is_near()) {
        parts.push(NEAR_LIMIT);
    }
    parts.join(" · ")
}

/// What the footer adds once a shared note nears its size limit.
pub const NEAR_LIMIT: &str = "Near this shared note's size limit";

/// What the banner above a shared note says of its problems: the newest
/// one's words, and how many more there are. `None` with none.
pub fn problem_banner(problems: &[Problem]) -> Option<String> {
    let newest = problems.iter().max_by_key(|p| p.at)?;
    let message = newest.message.trim();
    Some(match problems.len() - 1 {
        0 => message.to_owned(),
        1 => format!("{message} (1 more)"),
        more => format!("{message} ({more} more)"),
    })
}

/// A fingerprint's groups of digits, to set apart on screen.
pub fn fingerprint_groups(fingerprint: &str) -> Vec<String> {
    fingerprint.split_whitespace().map(str::to_owned).collect()
}

/// "Can edit, until 28 May" and the like, for an invite link.
pub fn link_label(role: Role, days_left: i64) -> String {
    match days_left {
        ..=0 => format!("{}, expired", role.label()),
        1 => format!("{}, for 1 more day", role.label()),
        n => format!("{}, for {n} more days", role.label()),
    }
}

/// Whole days from `now_ms` until `at_ms`, rounded up.
pub fn days_until(at_ms: i64, now_ms: i64) -> i64 {
    let left = at_ms - now_ms;
    if left <= 0 {
        0
    } else {
        (left + 86_399_999) / 86_400_000
    }
}

/// What clicking a comment selects in the editor: its words, or a caret
/// where an orphaned comment's words were; inside `text`, on character
/// boundaries.
pub fn comment_target(placement: &Placement, text: &str) -> Range<usize> {
    match placement {
        Placement::Anchored(range) => {
            let start = floor_boundary(text, range.start);
            start..floor_boundary(text, range.end.max(start))
        }
        Placement::Orphaned(at) => {
            let at = floor_boundary(text, *at);
            at..at
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_collab::{Anchor, Comment, apply_splices, splices};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config(cases: u32) -> ProptestConfig {
        ProptestConfig {
            cases,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
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

    /// Text with multi-byte characters, so offsets fall inside them.
    fn text() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop::sample::select(vec!['a', 'b', ' ', '\n', 'é', '😀', '中']),
            0..40,
        )
        .prop_map(|chars| chars.into_iter().collect())
    }

    #[test]
    fn a_caret_keeps_its_place_through_someone_elses_edit() {
        // "Hello world" → "Oh, Hello brave world": the caret after "world"
        // ends after "world" still; one inside "Hello" stays in it.
        let before = "Hello world";
        let after = "Oh, Hello brave world";
        let edits = splices(before, after);
        assert_eq!(map_selection(11, 11, &edits), (21, 21));
        assert_eq!(map_selection(2, 2, &edits).0, 6);
        // A backwards selection stays backwards.
        let (anchor, head) = map_selection(11, 6, &edits);
        assert!(anchor > head);
    }

    #[test]
    fn vim_stays_on_a_character_in_normal_mode() {
        // The line's last character was deleted under the cursor.
        let before = "abc\ndef";
        let after = "ab\ndef";
        let edits = splices(before, after);
        let (head, _) = map_vim(2, None, true, &edits, after);
        assert_eq!(head, 1, "on the `b`, not on the line's end");
        let (head, shown) = map_vim(2, Some(0..3), false, &edits, after);
        assert_eq!((head, shown), (2, Some(0..2)));
    }

    #[test]
    fn open_anchored_comments_are_tinted() {
        let comments = vec![
            view("late", Placement::Anchored(10..12), false, 1),
            view("gone", Placement::Orphaned(3), false, 5),
            view("done", Placement::Anchored(0..1), true, 3),
            view("past", Placement::Anchored(14..40), false, 3),
        ];
        assert_eq!(
            comment_ranges(&comments, "0123456789abcdef"),
            [10..12, 14..16]
        );
    }

    #[test]
    fn a_share_mark_names_the_share() {
        let extra = serde_json::json!({"kansoShare": {"shareId": "s1", "role": "comment", "owner": "a", "ownerName": "user2"}});
        assert_eq!(
            share_mark(extra.as_object().unwrap()),
            Some(("s1".into(), Role::Comment, "user2".into()))
        );
        let odd = serde_json::json!({"kansoShare": {"shareId": "s1", "role": "boss"}});
        assert_eq!(share_mark(odd.as_object().unwrap()).unwrap().1, Role::View);
        assert_eq!(share_mark(&serde_json::Map::new()), None);
    }

    #[test]
    fn footer_and_link_words() {
        assert_eq!(
            share_footer(Role::Edit, true, false, None),
            "Shared · Can edit · Live"
        );
        assert_eq!(
            share_footer(Role::View, false, false, None),
            "Shared · Can view · Read only · Offline"
        );
        assert_eq!(
            share_footer(Role::Owner, true, true, None),
            "Private copy · No longer shared"
        );
        assert_eq!(link_label(Role::Comment, 1), "Can comment, for 1 more day");
        assert_eq!(days_until(86_400_001, 0), 2);
        assert_eq!(days_until(5, 9), 0);
        assert_eq!(
            fingerprint_groups("01234 56789 00000 11111 22222 33333").len(),
            6
        );
    }

    #[test]
    fn comments_group_in_text_order_then_orphaned() {
        let comments = vec![
            view("late", Placement::Anchored(10..12), false, 1),
            view("gone", Placement::Orphaned(3), false, 5),
            view("first", Placement::Anchored(0..1), true, 3),
            view("same-place-older", Placement::Anchored(10..11), false, 0),
        ];
        assert_eq!(comment_groups(&comments), (vec![2, 3, 0], vec![1]));
        let (anchored, _) = comment_groups(&comments);
        let ids: Vec<&str> = anchored
            .iter()
            .map(|&i| comments[i].comment.id.as_str())
            .collect();
        assert_eq!(ids, ["first", "same-place-older", "late"]);
        assert_eq!(comments_summary(&comments), "3 open · 1 resolved");
        assert_eq!(comments_summary(&[]), "");
        assert_eq!(comment_target(&Placement::Orphaned(40), "abc"), 3..3);
        assert_eq!(comment_target(&Placement::Anchored(1..2), "é"), 0..2);
    }

    #[test]
    fn size_and_problems_are_told() {
        let near = SizeUse {
            used: 85,
            limit: 100,
        };
        assert_eq!(
            share_footer(Role::Edit, true, false, Some(near)),
            "Shared · Can edit · Live · Near this shared note's size limit"
        );
        assert_eq!(
            share_footer(
                Role::Edit,
                true,
                false,
                Some(SizeUse {
                    used: 10,
                    limit: 100
                })
            ),
            "Shared · Can edit · Live"
        );
        let problem = |message: &str, at| Problem {
            share_id: "s".into(),
            message: message.into(),
            at,
        };
        assert_eq!(problem_banner(&[]), None);
        assert_eq!(
            problem_banner(&[problem("Old.", 1), problem("New.", 5), problem("Mid.", 3)])
                .as_deref(),
            Some("New. (2 more)")
        );
    }

    proptest! {
        #![proptest_config(config(256))]

        /// The footer warns exactly when the note is at four fifths of its
        /// limit or more, and never for a private copy.
        #[test]
        fn prop_the_footer_warns_near_the_limit(used in 0usize..2_000, limit in 1usize..1_000, ended: bool) {
            let size = SizeUse { used, limit };
            let footer = share_footer(Role::Edit, true, ended, Some(size));
            prop_assert_eq!(footer.contains(NEAR_LIMIT), !ended && used * 5 >= limit * 4);
        }

        /// The banner shows the newest problem's words and counts the rest.
        #[test]
        fn prop_the_banner_shows_the_newest_problem(ats in prop::collection::vec(0i64..1_000, 0..6)) {
            let problems: Vec<Problem> = ats
                .iter()
                .enumerate()
                .map(|(i, at)| Problem { share_id: "s".into(), message: format!("P{i}."), at: *at })
                .collect();
            match problem_banner(&problems) {
                None => prop_assert!(problems.is_empty()),
                Some(text) => {
                    let newest = ats.iter().max().unwrap();
                    let shown = problems.iter().find(|p| text.starts_with(&p.message)).unwrap();
                    prop_assert_eq!(shown.at, *newest);
                    prop_assert_eq!(text.contains("more)"), problems.len() > 1);
                }
            }
        }

        /// What a click selects is inside the text, on character boundaries,
        /// and a caret for an orphaned comment.
        #[test]
        fn prop_a_comment_click_selects_inside_the_text(body in text(), start in 0usize..200, len in 0usize..20, orphaned in any::<bool>()) {
            let placement = if orphaned { Placement::Orphaned(start) } else { Placement::Anchored(start..start + len) };
            let range = comment_target(&placement, &body);
            prop_assert!(range.start <= range.end && range.end <= body.len());
            prop_assert!(body.is_char_boundary(range.start) && body.is_char_boundary(range.end));
            if orphaned {
                prop_assert!(range.is_empty());
            }
        }

        /// The splices take the editor's text to the merged text, and the
        /// caret ends on a character boundary of it, after the same text it
        /// was after when that text survived.
        #[test]
        fn prop_a_caret_follows_its_text(before in text(), after in text(), at in 0usize..200) {
            let at = floor_boundary(&before, at);
            let edits = splices(&before, &after);
            let mut applied = before.clone();
            apply_splices(&mut applied, &edits);
            prop_assert_eq!(&applied, &after);
            let (anchor, head) = map_selection(at, at, &edits);
            prop_assert_eq!(anchor, head);
            prop_assert!(head <= after.len());
            prop_assert!(after.is_char_boundary(head));
            // Untouched text before every edit keeps the caret where it was.
            let first = edits.iter().map(|s| s.at).min().unwrap_or(usize::MAX);
            if at <= first && edits.iter().all(|s| s.at >= at && !(s.at == at && s.delete == 0)) {
                prop_assert_eq!(head, at);
            }
        }

        /// A selection's ends never cross: mapped, the start stays at or
        /// before the end.
        #[test]
        fn prop_selections_keep_their_order(before in text(), after in text(), a in 0usize..200, b in 0usize..200) {
            let (a, b) = (floor_boundary(&before, a.min(b)), floor_boundary(&before, a.max(b)));
            let edits = splices(&before, &after);
            let (start, end) = map_selection(a, b, &edits);
            prop_assert!(start <= end);
        }

        /// Vim's Normal cursor lands on a character of its line (or the
        /// start of an empty one), and its Visual selection in the text.
        #[test]
        fn prop_vim_positions_stay_in_the_text(before in text(), after in text(), head in 0usize..200, a in 0usize..200, b in 0usize..200, normal in any::<bool>()) {
            let edits = splices(&before, &after);
            let head = floor_boundary(&before, head);
            let shown = Some(floor_boundary(&before, a.min(b))..floor_boundary(&before, a.max(b)));
            let (head, shown) = map_vim(head, shown, normal, &edits, &after);
            prop_assert!(after.is_char_boundary(head));
            if normal {
                prop_assert_eq!(head, crate::vim::clamp_normal(&after, head));
            }
            let shown = shown.unwrap();
            prop_assert!(shown.start <= shown.end && shown.end <= after.len());
        }

        /// Every remote cursor lands in the text, its caret at one end of
        /// its selection, its colour one of the theme's.
        #[test]
        fn prop_remote_marks_fit_the_text(body in text(), cursors in prop::collection::vec((0usize..200, 0usize..200, 0usize..64), 0..6)) {
            let cursors: Vec<RemoteCursor> = cursors
                .into_iter()
                .enumerate()
                .map(|(i, (anchor, head, colour))| RemoteCursor {
                    account: format!("a{i}"),
                    device: "d".into(),
                    name: format!("user{i}"),
                    anchor,
                    head,
                    colour,
                    seen_at: 0,
                })
                .collect();
            for (mark, cursor) in remote_marks(&cursors, &body).iter().zip(&cursors) {
                prop_assert!(mark.selection.end <= body.len());
                prop_assert!(body.is_char_boundary(mark.caret));
                prop_assert!(mark.caret == mark.selection.start || mark.caret == mark.selection.end);
                prop_assert!(mark.colour < crate::theme::CURSOR_COLOURS.len());
                prop_assert_eq!(mark.colour, cursor.colour % crate::theme::CURSOR_COLOURS.len());
            }
        }

        /// Tints are exactly the open, anchored comments with words left,
        /// each inside the text on character boundaries.
        #[test]
        fn prop_comment_tints_fit_the_text(body in text(), specs in prop::collection::vec((0usize..60, 0usize..10, any::<bool>(), any::<bool>()), 0..10)) {
            let comments: Vec<CommentView> = specs
                .iter()
                .enumerate()
                .map(|(i, (start, len, orphaned, resolved))| {
                    let placement = if *orphaned {
                        Placement::Orphaned(*start)
                    } else {
                        Placement::Anchored(*start..start + len)
                    };
                    view(&format!("c{i}"), placement, *resolved, 0)
                })
                .collect();
            let tints = comment_ranges(&comments, &body);
            for range in &tints {
                prop_assert!(range.start < range.end && range.end <= body.len());
                prop_assert!(body.is_char_boundary(range.start) && body.is_char_boundary(range.end));
            }
            let open = comments.iter().filter(|v| !v.comment.resolved && matches!(v.placement, Placement::Anchored(_))).count();
            prop_assert!(tints.len() <= open);
        }
    }
}

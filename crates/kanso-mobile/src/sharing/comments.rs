//! Comments on shared notes (docs/ACCOUNTS.md › Collaboration), the
//! desktop's Comments tab over the collab engine: the list with where each
//! comment sits, and adding, editing, deleting, resolving and reopening.
//!
//! Every call takes the note by its library ID, refuses a locked note first,
//! and answers in the desktop's words. Offsets cross the edge in UTF-16 code
//! units of the note's body, the unit Flutter counts in; the engine counts
//! UTF-8 bytes of its copy of the body, which the library's body is once the
//! room's pump (or `sync_note` here) has handed it over.

use std::ops::Range;

use kanso_collab::comments::{comment_groups, comments_summary, mention_runs};
use kanso_collab::{CommentView, Engine, Placement, Role};
use kanso_engine::text::{byte_to_utf16, utf16_to_byte};

use super::{SharedNotes, clip, collab_error};
use crate::boundary::guard;
use crate::comments_model::{CommentPlacement, CommentRow, MentionSpan};
use crate::errors::KansoError;

/// What the desktop says to someone who may only read the note.
const VIEW_ONLY: &str = "You can view this note, but not comment on it.";

impl SharedNotes {
    /// The note's comments as the Comments tab lists them: those on words
    /// still in the text in text order, then the ones whose words were
    /// deleted. Brought up to date with the note's text first, so each
    /// sits where its words are now. A note that isn't shared is refused.
    pub fn comments(&self, note_id: String) -> Result<Vec<CommentRow>, KansoError> {
        guard(|| {
            let (engine, share_id) = self.shared(&note_id)?;
            let body = self.body_for_offsets(&engine, &share_id, &note_id, false)?;
            let views = engine.comments(&share_id).map_err(|e| collab_error(&e))?;
            let role = engine.role(&share_id).unwrap_or(Role::View);
            let members = member_names(&engine, &share_id, &views);
            let (anchored, orphaned) = comment_groups(&views);
            Ok(anchored
                .into_iter()
                .chain(orphaned)
                .map(|ix| row_of(&views[ix], role, &members, &body))
                .collect())
        })
    }

    /// How many of the note's comments are open: the count on the tab. Zero
    /// for a note that isn't shared.
    pub fn open_comments(&self, note_id: String) -> Result<u32, KansoError> {
        guard(|| {
            self.refuse_locked(&note_id)?;
            let engine = self.engine()?;
            let Some(share_id) = engine.share_of(&note_id) else {
                return Ok(0);
            };
            let views = engine.comments(&share_id).map_err(|e| collab_error(&e))?;
            Ok(clip(views.iter().filter(|v| !v.comment.resolved).count()))
        })
    }

    /// The band under the Comments tab: "2 open · 1 resolved", empty with
    /// none, and empty for a note that isn't shared.
    pub fn comments_band(&self, note_id: String) -> Result<String, KansoError> {
        guard(|| {
            self.refuse_locked(&note_id)?;
            let engine = self.engine()?;
            let Some(share_id) = engine.share_of(&note_id) else {
                return Ok(String::new());
            };
            let views = engine.comments(&share_id).map_err(|e| collab_error(&e))?;
            Ok(comments_summary(&views))
        })
    }

    /// Comments on the words `[start, end)` (UTF-16 units of the note's
    /// body) and answers with the new comment's ID. Needs the role to
    /// comment. People named with `@` who are in the note are told when the
    /// comment reaches the server.
    pub fn add_comment(
        &self,
        note_id: String,
        start: u32,
        end: u32,
        body: String,
    ) -> Result<String, KansoError> {
        guard(|| {
            let (engine, share_id) = self.shared(&note_id)?;
            require_comment_role(&engine, &share_id)?;
            if body.trim().is_empty() {
                return Err(KansoError::invalid("A comment needs some words."));
            }
            // The engine's text first, so the selection is in it.
            let text = self.body_for_offsets(&engine, &share_id, &note_id, true)?;
            let range = bytes_of(&text, start, end)
                .ok_or_else(|| KansoError::invalid("Select the words to comment on first."))?;
            engine
                .add_comment(&share_id, range, &body)
                .map_err(|e| collab_error(&e))
        })
    }

    /// Changes the words of one of this account's comments.
    pub fn edit_comment(
        &self,
        note_id: String,
        comment_id: String,
        body: String,
    ) -> Result<(), KansoError> {
        guard(|| {
            if body.trim().is_empty() {
                return Err(KansoError::invalid("A comment needs some words."));
            }
            let (engine, share_id) = self.own_comment(&note_id, &comment_id)?;
            engine
                .edit_comment(&share_id, &comment_id, &body)
                .map_err(|e| collab_error(&e))
        })
    }

    /// Deletes one of this account's comments, for everyone in the note.
    pub fn delete_comment(&self, note_id: String, comment_id: String) -> Result<(), KansoError> {
        guard(|| {
            let (engine, share_id) = self.own_comment(&note_id, &comment_id)?;
            engine
                .delete_comment(&share_id, &comment_id)
                .map_err(|e| collab_error(&e))
        })
    }

    /// Resolves a comment (`resolved`), or opens it again. Anyone who may
    /// comment on the note may.
    pub fn resolve_comment(
        &self,
        note_id: String,
        comment_id: String,
        resolved: bool,
    ) -> Result<(), KansoError> {
        guard(|| {
            let (engine, share_id) = self.shared(&note_id)?;
            require_comment_role(&engine, &share_id)?;
            engine
                .resolve_comment(&share_id, &comment_id, resolved)
                .map_err(|e| collab_error(&e))
        })
    }

    /// The engine and share of a note, for changing a comment of this
    /// account's: the role must comment, and the comment must be this
    /// account's (the desktop's buttons show only then).
    fn own_comment(
        &self,
        note_id: &str,
        comment_id: &str,
    ) -> Result<(std::sync::Arc<Engine>, String), KansoError> {
        let (engine, share_id) = self.shared(note_id)?;
        require_comment_role(&engine, &share_id)?;
        let views = engine.comments(&share_id).map_err(|e| collab_error(&e))?;
        match views.iter().find(|v| v.comment.id == comment_id) {
            None => Err(KansoError::NotFound),
            Some(view) if !view.mine => Err(KansoError::invalid(
                "You can only change your own comments.",
            )),
            Some(_) => Ok((engine, share_id)),
        }
    }

    /// The text the offsets of this note count in: the library's body. With
    /// `catch_up`, the note's edits are first taken into the room and what
    /// others wrote is merged in (best effort, offline it is skipped), so a
    /// selection is in the engine's text. The editor-merge call is not used
    /// here: it keeps a base of its own that only one caller may feed.
    fn body_for_offsets(
        &self,
        engine: &Engine,
        share_id: &str,
        note_id: &str,
        catch_up: bool,
    ) -> Result<String, KansoError> {
        if catch_up {
            let _ = engine.sync_note(share_id);
        }
        let note = self
            .inner
            .store
            .note(note_id)?
            .ok_or(KansoError::NotFound)?;
        Ok(note.body)
    }
}

/// Refuses a role that may only read, in the desktop's words.
fn require_comment_role(engine: &Engine, share_id: &str) -> Result<(), KansoError> {
    if engine.role(share_id).is_some_and(Role::may_comment) {
        Ok(())
    } else {
        Err(KansoError::invalid(VIEW_ONLY))
    }
}

/// The usernames in a shared note, to set @mentions apart: its members as
/// last refreshed, and whoever wrote its comments (the desktop's list).
fn member_names(engine: &Engine, share_id: &str, comments: &[CommentView]) -> Vec<String> {
    let mut names: Vec<String> = engine
        .share_view(share_id)
        .map(|view| {
            std::iter::once(view.owner.username)
                .chain(view.members.into_iter().map(|m| m.person.username))
                .collect()
        })
        .unwrap_or_default();
    names.extend(comments.iter().map(|v| v.comment.author_name.clone()));
    names.push(engine.caller().username.clone());
    names.retain(|n| !n.is_empty());
    names.sort();
    names.dedup();
    names
}

/// A comment as the shell lists it. `body` is the note's text that the
/// placement's byte offsets count in.
fn row_of(view: &CommentView, role: Role, members: &[String], body: &str) -> CommentRow {
    let comment = &view.comment;
    let may = role.may_comment();
    let units = |byte: usize| clip(byte_to_utf16(body, byte));
    let placement = match &view.placement {
        Placement::Anchored(range) => CommentPlacement::Anchored {
            start: units(range.start),
            end: units(range.end),
        },
        Placement::Orphaned(at) => CommentPlacement::Orphaned { at: units(*at) },
    };
    CommentRow {
        id: comment.id.clone(),
        author: comment.author.clone(),
        author_name: comment.author_name.clone(),
        body: comment.body.clone(),
        created_ms: comment.created_at,
        edited_ms: comment.edited_at,
        resolved: comment.resolved,
        mine: view.mine,
        pending: view.pending,
        can_edit: may && view.mine,
        can_delete: may && view.mine,
        can_resolve: may,
        quote: comment.anchor.quote.trim().to_owned(),
        placement,
        mentions: mention_spans(&comment.body, members),
    }
}

/// The `@name`s of `body` that are members, in UTF-16 units of `body`.
fn mention_spans(body: &str, members: &[String]) -> Vec<MentionSpan> {
    mention_runs(body, members)
        .into_iter()
        .filter(|(_, mention)| *mention)
        .map(|(range, _)| MentionSpan {
            start: clip(byte_to_utf16(body, range.start)),
            end: clip(byte_to_utf16(body, range.end)),
        })
        .collect()
}

/// The byte range of `text` for UTF-16 units `[start, end)`: `None` when it
/// selects nothing or reaches past the text. An end inside a surrogate pair
/// is the end of that character, a start inside one its start.
pub(crate) fn bytes_of(text: &str, start: u32, end: u32) -> Option<Range<usize>> {
    let (start, end) = (start as usize, end as usize);
    if start >= end || end > text.encode_utf16().count() {
        return None;
    }
    let from = utf16_to_byte(text, start);
    let mut to = utf16_to_byte(text, end);
    // `utf16_to_byte` rounds down to a character's start; an end inside a
    // character takes the whole character.
    if byte_to_utf16(text, to) < end {
        to += text[to..].chars().next().map_or(0, char::len_utf8);
    }
    (from < to).then_some(from..to)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    /// Text with astral characters, combining marks and CRLF, so offsets
    /// fall between the halves of pairs and inside clusters.
    fn text() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop::sample::select(vec![
                "a",
                "b",
                " ",
                "\n",
                "\r\n",
                "é",
                "e\u{301}",
                "😀",
                "👩\u{200d}💻",
                "中",
                "\u{1f1e9}\u{1f1ea}",
            ]),
            0..30,
        )
        .prop_map(|parts| parts.concat())
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// Round-trip: a range of UTF-16 units on character boundaries
        /// becomes bytes that read back as the same units, and holds the
        /// same characters.
        #[test]
        fn prop_a_range_round_trips_through_bytes(
            text in text(),
            a in 0usize..64,
            b in 0usize..64,
        ) {
            let boundaries: Vec<usize> = std::iter::once(0)
                .chain(text.char_indices().map(|(at, c)| at + c.len_utf8()))
                .map(|byte| byte_to_utf16(&text, byte))
                .collect();
            let (x, y) = (boundaries[a % boundaries.len()], boundaries[b % boundaries.len()]);
            let (start, end) = (x.min(y), x.max(y));
            let units: Vec<u16> = text.encode_utf16().collect();
            match bytes_of(&text, start as u32, end as u32) {
                None => prop_assert!(start == end),
                Some(range) => {
                    prop_assert!(start < end);
                    prop_assert!(text.is_char_boundary(range.start) && text.is_char_boundary(range.end));
                    prop_assert_eq!(byte_to_utf16(&text, range.start), start);
                    prop_assert_eq!(byte_to_utf16(&text, range.end), end);
                    prop_assert_eq!(
                        String::from_utf16(&units[start..end]).unwrap(),
                        text[range].to_owned()
                    );
                }
            }
        }

        /// Whatever the offsets, a range that comes back is inside the text
        /// on character boundaries, never empty, and takes whole characters
        /// where an offset fell inside a pair; one past the text is refused.
        #[test]
        fn prop_any_offsets_give_whole_characters_or_nothing(
            text in text(),
            start in 0u32..80,
            end in 0u32..80,
        ) {
            let length = text.encode_utf16().count() as u32;
            match bytes_of(&text, start, end) {
                Some(range) => {
                    prop_assert!(start < end && end <= length);
                    prop_assert!(range.start < range.end && range.end <= text.len());
                    prop_assert!(text.is_char_boundary(range.start) && text.is_char_boundary(range.end));
                    // It covers at least the units asked for.
                    prop_assert!(byte_to_utf16(&text, range.start) <= start as usize);
                    prop_assert!(byte_to_utf16(&text, range.end) >= end as usize);
                }
                None => prop_assert!(start >= end || end > length || utf16_to_byte(&text, start as usize) >= utf16_to_byte(&text, end as usize)),
            }
        }

        /// Mention spans lie inside the comment's words in UTF-16 units and
        /// each reads as `@` and a name from the list.
        #[test]
        fn prop_mention_spans_read_as_members(
            words in prop::collection::vec(
                prop::sample::select(vec!["@user2", "@USER2", "@ann", "x@user2", "@", "@nobody", " ", "é", "😀", "-", "hi"]),
                0..12,
            ),
        ) {
            let body: String = words.concat();
            let members = vec!["user2".to_owned(), "ann".to_owned()];
            let units: Vec<u16> = body.encode_utf16().collect();
            let mut last = 0;
            for span in mention_spans(&body, &members) {
                prop_assert!(last <= span.start as usize && span.start < span.end);
                prop_assert!(span.end as usize <= units.len());
                last = span.end as usize;
                let said = String::from_utf16(&units[span.start as usize..span.end as usize]).unwrap();
                prop_assert!(said.starts_with('@'));
                prop_assert!(members.iter().any(|m| m.eq_ignore_ascii_case(&said[1..])), "{said}");
            }
        }
    }

    // no-prop-test: three fixed cases of the edge: a pair split, a start
    // past the end, an empty selection.
    #[test]
    fn offsets_inside_a_pair_take_the_whole_character() {
        let text = "a😀b";
        assert_eq!(bytes_of(text, 1, 3), Some(1..5));
        assert_eq!(bytes_of(text, 2, 4), Some(1..6));
        assert_eq!(bytes_of(text, 0, 5), None);
        assert_eq!(bytes_of(text, 2, 2), None);
        assert_eq!(bytes_of(text, 3, 1), None);
    }
}

//! Two editors of one note: the main window's and a note's own window
//! (`note_window.rs`), or two note windows. What is typed in one shows in
//! the others at once, as splices that leave their carets where they were.
//!
//! Only the editor typed in saves. The others know the text came from a
//! peer and don't save it again; when the save reaches them through the
//! store, its text is already theirs, and they take its revision quietly
//! rather than ask which version to keep. Should two of them save the same
//! text anyway (both had typing waiting), the second finds the first's
//! version already says it and takes that.

use gpui_kit::*;
use kanso_core::Note;

use super::NoteEditor;

/// Every editor open in any window, with its window, to find the ones on
/// the same note.
#[derive(Default)]
struct OpenEditors(Vec<(WeakEntity<NoteEditor>, AnyWindowHandle)>);

impl Global for OpenEditors {}

/// Keeps `editor` in the list of open editors until it goes.
pub(super) fn register(editor: WeakEntity<NoteEditor>, window: AnyWindowHandle, cx: &mut App) {
    let open = cx.default_global::<OpenEditors>();
    open.0.retain(|(editor, _)| editor.upgrade().is_some());
    open.0.push((editor, window));
}

/// The keys held for locked notes changed: every editor, in whichever
/// window, opens or closes its note to match, once the update that changed
/// them is over.
pub fn lock_keys_changed_everywhere(cx: &mut App) {
    cx.defer(|cx| {
        let open: Vec<_> = cx
            .try_global::<OpenEditors>()
            .map(|open| open.0.clone())
            .unwrap_or_default();
        for (editor, window) in open {
            let Some(editor) = editor.upgrade() else {
                continue;
            };
            window
                .update(cx, |_, window, cx| {
                    editor.update(cx, |editor, cx| editor.lock_keys_changed(window, cx));
                })
                .ok();
        }
    });
}

/// Saves the typing waiting in every open editor, in whichever window,
/// before a change to many notes at once. Not from inside an editor's own
/// update.
pub fn flush_everywhere(cx: &mut App) {
    let open: Vec<_> = cx
        .try_global::<OpenEditors>()
        .map(|open| open.0.iter().filter_map(|(e, _)| e.upgrade()).collect())
        .unwrap_or_default();
    for editor in open {
        editor.update(cx, |editor, cx| editor.flush(cx));
    }
}

/// The other open editors on note `id`, with their windows.
fn peers(id: &str, me: EntityId, cx: &App) -> Vec<(Entity<NoteEditor>, AnyWindowHandle)> {
    let Some(open) = cx.try_global::<OpenEditors>() else {
        return Vec::new();
    };
    open.0
        .iter()
        .filter_map(|(editor, window)| Some((editor.upgrade()?, *window)))
        .filter(|(editor, _)| editor.entity_id() != me && editor.read(cx).note_id() == Some(id))
        .collect()
}

// The rules for a save that lost a race live in the engine, where the
// phone's editing session follows them too.
pub(crate) use kanso_engine::saving::{has_text, text_caught_up, writes_the_same};

impl NoteEditor {
    /// Whether a change made elsewhere waits for an answer: which version
    /// to keep, or whether to keep a note deleted elsewhere.
    pub fn asks_about_a_change(&self) -> bool {
        self.incoming.is_some()
    }

    /// The title or body changed. Says whether it was a peer's text coming
    /// in, which this editor doesn't save; otherwise the change goes to the
    /// peers.
    pub(super) fn peer_change(&mut self, cx: &mut Context<Self>) -> bool {
        if let Some((title, body)) = &self.peer_echo {
            if self.title.read(cx).value() == title.as_str()
                && crate::text_stats::pieces_eq(self.body.read(cx).text().chunks(), body)
            {
                return true;
            }
            self.peer_echo = None;
        }
        self.from_peer = false;
        self.send_to_peers(cx);
        false
    }

    /// Hands the title and body to the other editors of the open note,
    /// once this update is over: they can't be reached from inside it.
    fn send_to_peers(&self, cx: &mut Context<Self>) {
        let Some(id) = self.note_id().map(str::to_owned) else {
            return;
        };
        let peers = peers(&id, cx.entity_id(), cx);
        if peers.is_empty() {
            return;
        }
        let text = self.current_text(cx);
        cx.defer(move |cx| {
            for (peer, window) in peers {
                let text = text.clone();
                let id = id.clone();
                window
                    .update(cx, |_, window, cx| {
                        peer.update(cx, |peer, cx| peer.take_peer_text(&id, text, window, cx));
                    })
                    .ok();
            }
        });
    }

    /// Puts a peer's text for note `id` in this editor, as splices that
    /// leave the caret and the scroll position with the text they were at.
    // no-prop-test: editor glue over `splices`, which kanso-collab tests;
    // `note_windows_edit_live_together` types through it.
    fn take_peer_text(
        &mut self,
        id: &str,
        (title, body): (String, String),
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // A note shown read only, or locked shut, takes the saved text
        // from the store instead.
        if self.note_id() != Some(id) || self.text_read_only || self.lock.is_closed() {
            return;
        }
        let (now_title, now_body) = self.current_text(cx);
        if now_title == title && now_body == body {
            return;
        }
        self.peer_echo = Some((title.clone(), body.clone()));
        self.from_peer = true;
        let title_edits = kanso_collab::splices(&now_title, &title);
        if !title_edits.is_empty() {
            self.title.update(cx, |state, cx| {
                let range = state.selected_range();
                for edit in &title_edits {
                    state.set_selected_range(edit.at..edit.at + edit.delete, cx);
                    state.replace(edit.insert.clone(), window, cx);
                }
                let start = kanso_collab::remap(range.start, &title_edits);
                let end = kanso_collab::remap(range.end, &title_edits);
                state.set_selected_range(start..end, cx);
            });
        }
        let body_edits = kanso_collab::splices(&now_body, &body);
        if !body_edits.is_empty() {
            self.splice_body(&body_edits, window, cx);
        }
        cx.notify();
    }

    /// After another note opens here: if another editor has it open with
    /// typing not saved yet, this one starts from that text.
    pub(super) fn catch_up_with_peers(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.note_id().map(str::to_owned) else {
            return;
        };
        let ahead = peers(&id, cx.entity_id(), cx)
            .into_iter()
            .find(|(peer, _)| peer.read(cx).has_unsaved_edits(cx))
            .map(|(peer, _)| peer.read(cx).current_text(cx));
        if let Some(text) = ahead {
            self.take_peer_text(&id, text, window, cx);
        }
    }

    /// The store has `latest` for the open note, with the text this editor
    /// shows already: it becomes the note as last saved, with nothing to
    /// ask about. Says whether it did.
    pub(super) fn adopt_if_same_text(&mut self, latest: &Note, cx: &mut Context<Self>) -> bool {
        let Some(note) = &self.note else {
            return false;
        };
        let (title, body) = self.current_text(cx);
        if note.is_locked() != latest.is_locked() || !has_text(latest, &title, &body) {
            return false;
        }
        self.note = Some(latest.clone());
        self.from_peer = false;
        self.incoming = None;
        cx.notify();
        true
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    use super::{has_text, text_caught_up, writes_the_same};
    use kanso_core::Note;

    fn note(title: &str, body: &str) -> Note {
        serde_json::from_value(serde_json::json!({
            "_id": "note:a",
            "_rev": "1-a",
            "title": title,
            "body": body,
            "bookId": "book:a",
            "updatedAt": 10,
        }))
        .unwrap()
    }

    #[test]
    fn caught_up_examples() {
        let base = note("T", "one");
        let mut latest = note("T", "one two");
        latest.rev = Some("2-b".into());
        latest.updated_at = 20;
        latest.tags = vec!["tag:x".into()];
        let writing = note("T", "one two");
        assert!(text_caught_up(&base, &latest, &writing));
        // Another text saved first: a real conflict.
        assert!(!text_caught_up(&base, &latest, &note("T", "one three")));
        // Pinned elsewhere as well: not only the text moved.
        latest.pinned = true;
        assert!(!text_caught_up(&base, &latest, &writing));
        assert!(writes_the_same(&latest, &latest.clone()));
        assert!(!writes_the_same(&writing, &latest));
    }

    fn gen_note() -> impl Strategy<Value = Note> {
        (
            "[a-c ]{0,6}",
            "[a-c\n ]{0,12}",
            any::<bool>(),
            proptest::collection::vec("tag:[ab]", 0..2),
            0i64..100,
        )
            .prop_map(|(title, body, pinned, tags, updated_at)| {
                let mut n = note(&title, &body);
                n.pinned = pinned;
                n.tags = tags;
                n.updated_at = updated_at;
                n
            })
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// A save by a peer of the very text being written, changing
        /// nothing else, is always caught up with; and what a note writes
        /// over itself, with a new revision and time, is the same write.
        #[test]
        fn a_peer_save_of_the_same_text_is_caught_up(
            base in gen_note(),
            title in "[a-c ]{0,6}",
            body in "[a-c\n ]{0,12}",
            later in 0i64..100,
        ) {
            let mut writing = base.clone();
            writing.title = title.clone();
            writing.body = body.clone();
            let mut latest = writing.clone();
            latest.rev = Some("9-z".into());
            latest.updated_at = later;
            prop_assert!(text_caught_up(&base, &latest, &writing));
            prop_assert!(writes_the_same(&writing, &latest));
        }

        /// Caught up means the text is the one being written, and nothing
        /// but the text, its tags and the time moved.
        #[test]
        fn caught_up_only_when_only_the_text_moved(
            base in gen_note(),
            latest in gen_note(),
            writing in gen_note(),
        ) {
            let caught = text_caught_up(&base, &latest, &writing);
            let expected = latest.title == writing.title
                && latest.body == writing.body
                && latest.pinned == base.pinned;
            prop_assert_eq!(caught, expected);
            // The same write never differs in text or flags.
            if writes_the_same(&writing, &latest) {
                prop_assert!(has_text(&latest, &writing.title, &writing.body));
                prop_assert_eq!(&latest.tags, &writing.tags);
                prop_assert_eq!(latest.pinned, writing.pinned);
            }
        }
    }
}

//! When an editor writes the note it shows, and when another writer's
//! version of it is not a conflict. Without a window, and without a clock
//! of its own: the caller passes the time.
//!
//! [`Autosave`] is the rule typing is saved by: [`AUTOSAVE_DELAY`] after the
//! last change, and not at all while a question about a change made
//! elsewhere is open (the caller doesn't ask). The rest decide, from the
//! versions of a note an editor knows, whether the version someone else saved
//! first makes its write a conflict:
//!
//! - [`same_content`]: only the revision moved (a sync won a race, a repair);
//! - [`writes_the_same`]: the other writer saved the very text being written;
//! - [`text_caught_up`]: the other writer saved the text being written along
//!   with what the text decides (its `#tags`).
//!
//! In each case the write goes on top of the new revision and nothing is
//! asked. The desktop editor and the phone's editing session both follow
//! these rules.

use std::time::{Duration, Instant};

use kanso_core::Note;

/// How long after the last change typing is written.
pub const AUTOSAVE_DELAY: Duration = Duration::from_millis(600);

/// The autosave rule for one open note: a deadline that every change pushes
/// out to [`AUTOSAVE_DELAY`] after it, and that a save clears.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Autosave {
    changed_at: Option<Instant>,
}

impl Autosave {
    /// The text changed at `now`: a write is due [`AUTOSAVE_DELAY`] from
    /// here, whatever was waiting before.
    pub fn changed(&mut self, now: Instant) {
        self.changed_at = Some(now);
    }

    /// When the write is due, if one waits.
    pub fn deadline(&self) -> Option<Instant> {
        self.changed_at.map(|at| at + AUTOSAVE_DELAY)
    }

    /// Whether a write waits and its time has come at `now`.
    pub fn due(&self, now: Instant) -> bool {
        self.deadline().is_some_and(|deadline| now >= deadline)
    }

    /// Whether a write waits, due or not.
    pub fn pending(&self) -> bool {
        self.changed_at.is_some()
    }

    /// Nothing waits any more: the text was written, or the question about
    /// a change made elsewhere took the write's place.
    pub fn saved(&mut self) {
        self.changed_at = None;
    }
}

/// Whether two versions of a note say the same thing: equal apart from the
/// revision and the task counts the store derives from the body. A sync that
/// only moved the revision (a local win, a stale rebase, a repair) leaves the
/// two versions the same. Pure.
pub fn same_content(a: &Note, b: &Note) -> bool {
    let plain = |note: &Note| Note {
        rev: None,
        num_of_tasks: 0,
        num_of_checked_tasks: 0,
        ..note.clone()
    };
    plain(a) == plain(b)
}

/// Whether `note` has `title` and `body` for its text. Pure.
pub fn has_text(note: &Note, title: &str, body: &str) -> bool {
    note.title == title && note.body == body
}

/// `note` without what a save moves by itself: the revision, the edit time
/// and the task counts the store derives from the body.
fn settled(note: &Note) -> Note {
    Note {
        rev: None,
        updated_at: 0,
        num_of_tasks: 0,
        num_of_checked_tasks: 0,
        ..note.clone()
    }
}

/// Whether writing `writing` would change nothing `latest` doesn't already
/// say: they differ at most in revision and edit time. Pure.
pub fn writes_the_same(writing: &Note, latest: &Note) -> bool {
    settled(writing) == settled(latest)
}

/// Whether `latest`, saved over `base` by someone else, only brought the
/// text this editor is writing in `writing`: it has `writing`'s title and
/// body, and otherwise differs from `base` only in what the text decides
/// (the `#tags` it names) and the edit time. Then this editor's write can go
/// on top of it. Pure.
pub fn text_caught_up(base: &Note, latest: &Note, writing: &Note) -> bool {
    if !has_text(latest, &writing.title, &writing.body) {
        return false;
    }
    let plain = |note: &Note| Note {
        title: String::new(),
        body: String::new(),
        tags: Vec::new(),
        ..settled(note)
    };
    plain(base) == plain(latest)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    use super::*;

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
    fn autosave_examples() {
        let start = Instant::now();
        let mut autosave = Autosave::default();
        assert!(!autosave.due(start + Duration::from_secs(60)));
        autosave.changed(start);
        assert!(!autosave.due(start + AUTOSAVE_DELAY - Duration::from_millis(1)));
        assert!(autosave.due(start + AUTOSAVE_DELAY));
        // Another change pushes the write out again.
        autosave.changed(start + Duration::from_millis(500));
        assert!(!autosave.due(start + AUTOSAVE_DELAY));
        assert!(autosave.due(start + Duration::from_millis(1100)));
        autosave.saved();
        assert!(!autosave.pending() && autosave.deadline().is_none());
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

    #[test]
    fn same_content_examples() {
        let a = note("t", "- [ ] one");
        assert!(same_content(&a, &a));
        let mut moved = a.clone();
        moved.rev = Some("2-b".into());
        moved.num_of_tasks = 1;
        assert!(same_content(&a, &moved));
        moved.book_id = "book:b".into();
        assert!(!same_content(&a, &moved));
        assert!(!same_content(&a, &note("T", "- [ ] one")));
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

        /// The write is due exactly [`AUTOSAVE_DELAY`] after the last
        /// change, whatever came before it; a save clears it; and a write
        /// never comes due before a change.
        #[test]
        fn prop_autosave_is_due_a_delay_after_the_last_change(
            gaps in proptest::collection::vec(0u64..2_000, 1..8),
            probe in 0u64..3_000,
        ) {
            let start = Instant::now();
            let mut autosave = Autosave::default();
            let mut at = Duration::ZERO;
            prop_assert!(!autosave.due(start + Duration::from_secs(1_000)));
            for gap in &gaps {
                at += Duration::from_millis(*gap);
                autosave.changed(start + at);
            }
            let last = start + at;
            prop_assert_eq!(autosave.deadline(), Some(last + AUTOSAVE_DELAY));
            let now = last + Duration::from_millis(probe);
            prop_assert_eq!(autosave.due(now), probe >= AUTOSAVE_DELAY.as_millis() as u64);
            autosave.saved();
            prop_assert!(!autosave.due(now) && !autosave.pending());
        }

        /// Equal apart from the revision and the derived counts is the
        /// relation of those two things, in both directions.
        #[test]
        fn prop_same_content_ignores_only_the_revision_and_counts(
            n in gen_note(),
            rev in "[0-9]-[a-z]",
            tasks in 0u32..9,
            checked in 0u32..9,
        ) {
            let mut bumped = n.clone();
            bumped.rev = Some(rev);
            bumped.num_of_tasks = tasks as _;
            bumped.num_of_checked_tasks = checked as _;
            prop_assert!(same_content(&n, &bumped) && same_content(&bumped, &n));
            let mut edited = n.clone();
            edited.body.push('a');
            prop_assert!(!same_content(&n, &edited));
        }

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

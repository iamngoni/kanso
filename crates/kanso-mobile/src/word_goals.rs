//! Word-goal settings for mobile editors, using the desktop engine's rules.
//!
//! Library defaults live in the store's local meta table. Per-note settings
//! travel in `Note.extra` and are saved by the editor when one is open.

use kanso_engine::word_goal::{self, GoalSetting, MAX_GOAL, PRESETS};

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::library::Library;
use crate::model::SaveState;

/// Store key for the library's default word goal. Empty means no default.
const DEFAULT_GOAL: &str = "mobile.word_goal";

impl Library {
    /// The note's own goal setting. An open editor's unsaved choice is
    /// returned; otherwise the note is read through the library's lock
    /// session, as it is for editing.
    pub fn note_word_goal(&self, id: &str) -> Result<GoalSetting, KansoError> {
        guard(|| {
            if let Some(setting) = self.inner.editors.goal_if_open(id)? {
                return Ok(setting);
            }
            let mut note = self.stored(id)?;
            if self.reveal(&mut note).is_closed() {
                return Err(KansoError::Locked);
            }
            Ok(word_goal::note_goal(&note))
        })
    }

    /// The library's default note goal, if one is set.
    pub fn default_word_goal(&self) -> Result<Option<u32>, KansoError> {
        guard(|| read_default(&self.inner.store))
    }

    /// Sets the default note goal. Only desktop presets (or `None`) are
    /// supported.
    pub fn set_default_word_goal(&self, goal: Option<u32>) -> Result<Option<u32>, KansoError> {
        guard(|| {
            validate_default_goal(goal)?;
            self.inner.store.set_meta(
                DEFAULT_GOAL,
                &goal.map_or_else(String::new, |n| n.to_string()),
            )?;
            Ok(goal)
        })
    }

    /// Desktop's offered word-goal presets, in menu order.
    pub fn word_goal_presets() -> Vec<u32> {
        PRESETS.to_vec()
    }

    /// Sets a note goal for a palette action. An open editor receives the
    /// change; otherwise a temporary editor saves it through the normal path.
    pub fn set_note_word_goal(&self, id: &str, setting: GoalSetting) -> Result<(), KansoError> {
        guard(|| {
            validate_note_goal(setting)?;
            if self.inner.editors.set_goal_if_open(id, setting)? {
                return Ok(());
            }
            let editor = self.edit(id)?;
            editor.set_word_goal(setting)?;
            match editor.close()? {
                SaveState::Saved { .. } => Ok(()),
                SaveState::Failed { message } => Err(KansoError::storage(message)),
                SaveState::Conflict { .. } => Err(KansoError::Conflict {
                    current_rev: String::new(),
                }),
                SaveState::Pending => Err(KansoError::storage(
                    "The word goal is still waiting to be saved.",
                )),
            }
        })
    }
}

/// Reads and validates the library's stored default.
pub(crate) fn read_default(store: &kanso_store::Store) -> Result<Option<u32>, KansoError> {
    let Some(value) = store.meta(DEFAULT_GOAL)? else {
        return Ok(None);
    };
    if value.is_empty() {
        return Ok(None);
    }
    let goal = Some(value.parse::<u32>().map_err(|_| {
        KansoError::invalid("The library's default word goal is not a valid number.")
    })?);
    validate_default_goal(goal)?;
    Ok(goal)
}

/// Rejects note-specific values outside the desktop engine's supported
/// positive range. `Off` is represented separately from `Words(0)`.
pub(crate) fn validate_note_goal(setting: GoalSetting) -> Result<(), KansoError> {
    match setting {
        GoalSetting::Default | GoalSetting::Off => Ok(()),
        GoalSetting::Words(goal) if (1..=MAX_GOAL).contains(&goal) => Ok(()),
        GoalSetting::Words(_) => Err(KansoError::invalid(format!(
            "A note goal must be between 1 and {MAX_GOAL} words, or turned off."
        ))),
    }
}

fn validate_default_goal(goal: Option<u32>) -> Result<(), KansoError> {
    if goal.is_none_or(|goal| PRESETS.contains(&goal)) {
        Ok(())
    } else {
        Err(KansoError::invalid(
            "Choose the library's default word goal from the offered presets, or turn it off.",
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use kanso_sync::{MemorySecrets, MemoryServer};

    use super::*;
    use crate::model::NewNote;

    fn library(path: &std::path::Path) -> Library {
        Library::open_for_test(
            path,
            Arc::new(MemorySecrets::new()),
            Arc::new(MemoryServer::new()),
        )
        .unwrap()
    }

    #[test]
    fn library_default_word_goal_is_preset_validated_and_persistent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("library.db");
        let opened = library(&path);

        assert_eq!(opened.default_word_goal().unwrap(), None);
        assert_eq!(opened.set_default_word_goal(Some(500)).unwrap(), Some(500));
        drop(opened);
        let library = library(&path);
        assert_eq!(library.default_word_goal().unwrap(), Some(500));
        assert_eq!(
            library.set_default_word_goal(Some(333)),
            Err(KansoError::invalid(
                "Choose the library's default word goal from the offered presets, or turn it off."
            ))
        );
        assert_eq!(library.set_default_word_goal(None).unwrap(), None);
        assert_eq!(library.default_word_goal().unwrap(), None);
        assert_eq!(Library::word_goal_presets(), PRESETS);

        library
            .inner
            .store
            .set_meta(DEFAULT_GOAL, "not-a-number")
            .unwrap();
        assert!(matches!(
            library.default_word_goal(),
            Err(KansoError::Invalid { .. })
        ));
    }

    #[test]
    fn palette_note_goal_preserves_and_persists_note_content() {
        let dir = tempfile::tempdir().unwrap();
        let library = library(&dir.path().join("library.db"));
        library.set_default_word_goal(Some(500)).unwrap();
        let saved = library
            .create(NewNote {
                title: "Keep this title".into(),
                body: "Keep this body".into(),
                ..NewNote::default()
            })
            .unwrap();

        library
            .set_note_word_goal(&saved.id, GoalSetting::Words(750))
            .unwrap();
        assert_eq!(
            library.note_word_goal(&saved.id).unwrap(),
            GoalSetting::Words(750)
        );
        let stored = library.inner.store.note(&saved.id).unwrap().unwrap();
        assert_eq!(stored.title, "Keep this title");
        assert_eq!(stored.body, "Keep this body");
        assert_eq!(word_goal::note_goal(&stored), GoalSetting::Words(750));

        let editor = library.edit(&saved.id).unwrap();
        assert_eq!(
            editor.snapshot().unwrap().word_goal,
            GoalSetting::Words(750)
        );
        assert_eq!(editor.effective_word_goal().unwrap(), Some(750));
        assert_eq!(editor.default_word_goal().unwrap(), Some(500));
        editor.set_word_goal(GoalSetting::Off).unwrap();
        assert_eq!(library.note_word_goal(&saved.id).unwrap(), GoalSetting::Off);
        editor.set_word_goal(GoalSetting::Default).unwrap();
        assert_eq!(editor.effective_word_goal().unwrap(), Some(500));
    }

    #[test]
    fn editor_goal_uses_normal_save_and_invalid_values_are_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let library = library(&dir.path().join("library.db"));
        let saved = library.create(NewNote::default()).unwrap();
        let editor = library.edit(&saved.id).unwrap();

        assert!(matches!(
            editor.set_word_goal(GoalSetting::Words(0)),
            Err(KansoError::Invalid { .. })
        ));
        editor.set_word_goal(GoalSetting::Words(1200)).unwrap();
        assert_eq!(
            editor.snapshot().unwrap().word_goal,
            GoalSetting::Words(1200)
        );
        editor.close().unwrap();
        let stored = library.inner.store.note(&saved.id).unwrap().unwrap();
        assert_eq!(word_goal::note_goal(&stored), GoalSetting::Words(1200));

        assert!(matches!(
            library.set_note_word_goal(&saved.id, GoalSetting::Words(MAX_GOAL + 1)),
            Err(KansoError::Invalid { .. })
        ));
    }
}

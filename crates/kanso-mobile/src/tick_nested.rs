//! "Tick nested tasks together" on the phone (#509): the setting, kept in
//! the library's local meta table, and the question the app asks before a
//! tick that the link would change.

use kanso_core::tasks::task_items;

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::library::Library;
use crate::model::TickNested;

/// Store key for the setting.
const KEY: &str = "mobile.tick_nested";

impl Library {
    /// Whether ticking a task also ticks the tasks nested under it:
    /// `Ask` until the person says otherwise.
    pub fn tick_nested(&self) -> Result<TickNested, KansoError> {
        guard(|| {
            Ok(self
                .inner
                .store
                .meta(KEY)?
                .and_then(|word| TickNested::from_key(&word))
                .unwrap_or_default())
        })
    }

    /// Sets [`Library::tick_nested`].
    pub fn set_tick_nested(&self, setting: TickNested) -> Result<TickNested, KansoError> {
        guard(|| {
            self.inner.store.set_meta(KEY, setting.key())?;
            Ok(setting)
        })
    }

    /// Whether ticking (or clearing) the task with `text` on `line` of note
    /// `note_id` with the link would write more than the plain tick does:
    /// what the app asks about the first time.
    pub fn task_would_cascade(
        &self,
        note_id: &str,
        line: u32,
        text: &str,
        checked: bool,
    ) -> Result<bool, KansoError> {
        guard(|| {
            let Some((_, body)) = self.readable_text(note_id)? else {
                return Ok(false);
            };
            let items = task_items(&body);
            let Some(index) = items
                .iter()
                .position(|item| item.line == line && item.text == text)
            else {
                return Ok(false);
            };
            if items[index].checked == checked {
                return Ok(false);
            }
            let today = kanso_store::local_today(kanso_core::now_ms());
            Ok(kanso_core::tick::would_cascade(&body, index, today))
        })
    }
}

#[cfg(test)]
#[path = "tick_nested_tests.rs"]
mod tests;

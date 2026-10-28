//! What related notes' embeddings say about the open note, for the views
//! around it: the tags its nearest notes suggest, for the tag list, `#`
//! autocomplete and the inspector, and a note that says nearly the same
//! as it does, for the editor to mention (`DUPLICATE_SIMILARITY`).
//!
//! Read from the stored vectors in the background whenever another note
//! opens or new embeddings are kept, so nothing waits on it; nothing is
//! there while related notes are off or the note isn't embedded yet.

use std::sync::Arc;

use gpui_kit::*;
use kanso_engine::discovery::meaning::{self, Meaning};
use kanso_store::{RelatedNote, Store, TagSuggestion};

use crate::related::Related;

pub struct NoteMeaning {
    store: Arc<Store>,
    /// What is read for the open note, and the embeddings (by
    /// `Related::generation`) it was read from.
    meaning: Meaning,
    read_for: Option<(String, u64)>,
    task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl NoteMeaning {
    pub fn new(store: Arc<Store>, cx: &mut Context<Self>) -> Self {
        // New embeddings, or related notes turned on or off: read again.
        let subscriptions = vec![cx.observe_global::<Related>(|this, cx| this.reload(cx))];
        NoteMeaning {
            store,
            meaning: Meaning::default(),
            read_for: None,
            task: None,
            _subscriptions: subscriptions,
        }
    }

    /// The note open now (`None`: none).
    pub fn follow(&mut self, note: Option<&str>, cx: &mut Context<Self>) {
        if !self.meaning.follow(note) {
            return;
        }
        self.reload(cx);
        cx.notify();
    }

    fn reload(&mut self, cx: &mut Context<Self>) {
        let generation = cx.try_global::<Related>().map_or(0, |r| r.generation);
        let Some(id) = self.meaning.note().map(str::to_owned) else {
            self.read_for = None;
            self.task = None;
            return;
        };
        let wanted = Some((id.clone(), generation));
        if self.read_for == wanted {
            return;
        }
        self.read_for = wanted;
        let store = self.store.clone();
        let read = cx
            .background_executor()
            .spawn(async move { meaning::read(&store, &id) });
        self.task = Some(cx.spawn(async move |this, cx| {
            let (suggested, twin) = read.await;
            this.update(cx, |this, cx| {
                this.task = None;
                if this.meaning.set(suggested, twin) {
                    cx.notify();
                }
            })
            .ok();
        }));
    }

    /// Tags suggested for the open note that `own` (its tags now) hasn't
    /// got, heaviest first.
    pub fn suggested_tags(&self, own: &[String]) -> Vec<TagSuggestion> {
        self.meaning.suggested_tags(own)
    }

    /// The note that says nearly the same as the open one, unless that was
    /// dismissed.
    pub fn duplicate(&self) -> Option<&RelatedNote> {
        self.meaning.twin()
    }

    /// The open note and its near twin are different notes after all: not
    /// mentioned again while Kanso runs, from either side.
    pub fn dismiss_duplicate(&mut self, cx: &mut Context<Self>) {
        if self.meaning.dismiss_twin() {
            cx.notify();
        }
    }

    /// The twin as the snapshots draw it.
    #[cfg(test)]
    pub fn set_duplicate_for_test(&mut self, duplicate: Option<RelatedNote>) {
        let suggested = self.meaning.suggested_tags(&[]);
        self.meaning.set(suggested, duplicate);
    }

    /// Suggestions as the snapshots draw them.
    #[cfg(test)]
    pub fn set_suggested_for_test(&mut self, suggested: Vec<TagSuggestion>) {
        let twin = self.meaning.twin().cloned();
        self.meaning.set(suggested, twin);
    }
}

//! `#` autocomplete in the editor: typing `#` and a few letters offers the
//! library's tags, nested ones (`work/clients`) by any part of their path.
//! The text editor has one completion provider, so [`NoteCompletion`] puts
//! this beside `[[` autocomplete ([`WikiCompletion`]): a `#tag` being typed
//! goes to the tags, anything else to the wiki-links.
//!
//! What counts as a tag, and which tags are offered, is decided in
//! `kanso_engine::links::tags`.
//!
//! With related notes on, the tags the note's nearest notes have come
//! first, from `note_meaning.rs`.
//!
//! With Settings › Editor › "Tags from #words" on, saving gives the note
//! the tags its text names ([`apply_inline_tags`]).

use std::cell::Cell;

use anyhow::Result;
use gpui_kit::base::input::{CompletionProvider, Rope, RopeExt as _};
use gpui_kit::*;
use kanso_engine::links::tags::{LOOK_BACK, choices, tag_query, tag_starts_at};
use lsp_types::{
    CompletionContext, CompletionItem, CompletionItemKind, CompletionResponse, CompletionTextEdit,
    TextEdit,
};

use crate::library::Library;
use crate::note_meaning::NoteMeaning;
use crate::wikilinks::WikiCompletion;

pub use kanso_engine::links::tags::apply_inline_tags;

/// `#` autocomplete beside `[[` autocomplete.
pub struct NoteCompletion {
    wiki: WikiCompletion,
    library: Entity<Library>,
    /// The tags suggested for the open note.
    meaning: Entity<NoteMeaning>,
    /// Whether the caret was in a `#tag` at the last keystroke, so the next
    /// one refines the list.
    in_tag: Cell<bool>,
    /// Whether the caret was after a task's `@` at the last keystroke.
    in_date: Cell<bool>,
    /// Whether the caret was after an `@name` at the last keystroke.
    in_mention: Cell<bool>,
}

impl NoteCompletion {
    pub fn new(
        wiki: WikiCompletion,
        library: Entity<Library>,
        meaning: Entity<NoteMeaning>,
    ) -> Self {
        NoteCompletion {
            wiki,
            library,
            meaning,
            in_tag: Cell::new(false),
            in_date: Cell::new(false),
            in_mention: Cell::new(false),
        }
    }

    /// `@name` autocomplete: the one-word names of the library's people
    /// and project pages (`people_data::mention_choices`).
    fn mentions(&self, text: &Rope, offset: usize, cx: &App) -> Option<Vec<CompletionItem>> {
        let offset = offset.min(text.len());
        let position = text.offset_to_position(offset);
        let line_start = text.line_start_offset(position.line as usize);
        let before = text.slice(line_start..offset).to_string();
        let (at, query) = crate::people_data::at_query(&before)?;
        let store = self.library.read(cx).store.clone();
        let pages: Vec<(String, Vec<String>)> = store
            .entities()
            .ok()?
            .into_iter()
            .map(|page| (page.title, page.names))
            .collect();
        let range = lsp_types::Range {
            start: text.offset_to_position(line_start + at),
            end: text.offset_to_position(offset),
        };
        let items: Vec<CompletionItem> = crate::people_data::mention_choices(&pages, query)
            .into_iter()
            .map(|choice| CompletionItem {
                label: format!("@{}", choice.handle),
                detail: Some(choice.title),
                kind: Some(CompletionItemKind::REFERENCE),
                filter_text: Some(String::new()),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range,
                    new_text: format!("@{}", choice.handle),
                })),
                ..Default::default()
            })
            .collect();
        (!items.is_empty()).then_some(items)
    }

    /// The `#tag` at `offset` of `text`: where its `#` is and the query.
    fn tag_at(&self, text: &Rope, offset: usize) -> Option<(usize, String)> {
        let offset = offset.min(text.len());
        let position = text.offset_to_position(offset);
        let line_start = text.line_start_offset(position.line as usize);
        let from = line_start.max(offset.saturating_sub(LOOK_BACK));
        let from = (from..=offset)
            .find(|&at| text.is_char_boundary(at))
            .unwrap_or(offset);
        let before = text.slice(from..offset).to_string();
        let (hash, query) = tag_query(&before)?;
        let hash = from + hash;
        // Whether a tag would start there, given everything before it
        // (code, links, headings, front matter): the store's own rule.
        tag_starts_at(&text.slice(0..hash).to_string()).then(|| (hash, query.to_owned()))
    }
}

// no-prop-test: delegates to kanso_engine::links::tags::choices, tested there.
impl CompletionProvider for NoteCompletion {
    fn completions(
        &self,
        text: &Rope,
        offset: usize,
        trigger: CompletionContext,
        window: &mut Window,
        cx: &mut App,
    ) -> Task<Result<CompletionResponse>> {
        // A task's due date after `@` (see `crate::due`).
        let dates = crate::due::completions(text, offset, cx);
        self.in_date.set(dates.is_some());
        if let Some(items) = dates {
            self.in_tag.set(false);
            return Task::ready(Ok(CompletionResponse::Array(items)));
        }
        let mentions = self.mentions(text, offset, cx);
        self.in_mention.set(mentions.is_some());
        if let Some(items) = mentions {
            self.in_tag.set(false);
            return Task::ready(Ok(CompletionResponse::Array(items)));
        }
        let Some((hash, query)) = self.tag_at(text, offset) else {
            self.in_tag.set(false);
            return self.wiki.completions(text, offset, trigger, window, cx);
        };
        self.in_tag.set(true);
        let offset = offset.min(text.len());
        let range = lsp_types::Range {
            start: text.offset_to_position(hash),
            end: text.offset_to_position(offset),
        };
        let library = self.library.read(cx);
        let names: Vec<&str> = library.tags.iter().map(|tag| tag.name.as_str()).collect();
        let suggestions = self.meaning.read(cx).suggested_tags(&[]);
        let suggested: Vec<&str> = suggestions.iter().map(|s| s.name.as_str()).collect();
        let items = choices(&names, &suggested, &query)
            .into_iter()
            .map(|choice| CompletionItem {
                label: format!("#{}", choice.name),
                detail: choice
                    .suggested
                    .then(|| "Suggested by close notes".to_owned()),
                kind: Some(CompletionItemKind::KEYWORD),
                filter_text: Some(String::new()),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range,
                    new_text: format!("#{}", choice.name),
                })),
                ..Default::default()
            })
            .collect();
        Task::ready(Ok(CompletionResponse::Array(items)))
    }

    fn is_completion_trigger(&self, offset: usize, new_text: &str, cx: &mut App) -> bool {
        if new_text.contains('\n') {
            self.in_tag.set(false);
            self.in_date.set(false);
            self.in_mention.set(false);
        }
        (!new_text.contains('\n')
            && (new_text.contains(['#', '@'])
                || self.in_tag.get()
                || self.in_date.get()
                || self.in_mention.get()))
            || self.wiki.is_completion_trigger(offset, new_text, cx)
    }
}

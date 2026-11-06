//! Which notebooks an agent may search and read through Kanso's read-only
//! notes server: every unlocked notebook, or the ones the person picks (with
//! the notebooks inside them). Kept as notebook IDs, so a rename neither
//! widens nor breaks it. A locked notebook is never offered and never in
//! scope, whatever the settings say; the server decides that again.

use gpui_component::scroll::ScrollableElement as _;
use std::collections::{BTreeSet, HashMap};

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::{Disableable as _, Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::TRASH_BOOK_ID;
use kanso_store::Store;

use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

/// One notebook the agent may be given, in the sidebar's order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeBook {
    pub id: String,
    pub name: String,
    /// How deep it is nested.
    pub depth: usize,
    /// Its parent, if it has one.
    pub parent: Option<String>,
}

/// The notebooks that can be in scope: live, not the trash, not locked (nor
/// inside a locked one), each after its parent. Nothing when the store
/// can't say which are locked.
pub fn scope_books(store: &Store) -> Vec<ScopeBook> {
    let (Ok(books), Ok(locked)) = (store.books(), store.locked_books()) else {
        return Vec::new();
    };
    let books: Vec<_> = books
        .into_iter()
        .filter(|book| book.id != TRASH_BOOK_ID && !locked.contains(&book.id))
        .collect();
    let known: BTreeSet<&str> = books.iter().map(|book| book.id.as_str()).collect();
    let mut children: HashMap<Option<&str>, Vec<&kanso_core::Book>> = HashMap::new();
    for book in &books {
        let parent = book
            .parent_book_id
            .as_deref()
            .filter(|parent| known.contains(parent));
        children.entry(parent).or_default().push(book);
    }
    for list in children.values_mut() {
        list.sort_by_key(|book| book.name.to_lowercase());
    }
    let mut out = Vec::new();
    let mut stack: Vec<(&kanso_core::Book, usize)> = children
        .get(&None)
        .into_iter()
        .flatten()
        .rev()
        .map(|book| (*book, 0))
        .collect();
    while let Some((book, depth)) = stack.pop() {
        if out.iter().any(|seen: &ScopeBook| seen.id == book.id) {
            continue;
        }
        out.push(ScopeBook {
            id: book.id.clone(),
            name: book.name.clone(),
            depth,
            parent: book
                .parent_book_id
                .clone()
                .filter(|parent| known.contains(parent.as_str())),
        });
        stack.extend(
            children
                .get(&Some(book.id.as_str()))
                .into_iter()
                .flatten()
                .rev()
                .map(|child| (*child, depth + 1)),
        );
    }
    out
}

/// Whether `id` is in scope because it or a notebook it is inside is chosen.
pub fn covered(id: &str, chosen: &BTreeSet<String>, books: &[ScopeBook]) -> bool {
    let mut at = Some(id.to_owned());
    let mut steps = 0;
    while let Some(here) = at {
        if chosen.contains(&here) {
            return true;
        }
        steps += 1;
        if steps > books.len() {
            return false;
        }
        at = books
            .iter()
            .find(|book| book.id == here)
            .and_then(|book| book.parent.clone());
    }
    false
}

/// What Settings says the agent may read, from the chosen IDs.
pub fn scope_summary(chosen: &[String], books: &[ScopeBook]) -> String {
    if chosen.is_empty() {
        return "Every unlocked notebook.".to_owned();
    }
    let names: Vec<&str> = books
        .iter()
        .filter(|book| chosen.contains(&book.id))
        .map(|book| book.name.as_str())
        .collect();
    match names.as_slice() {
        [] => "None: the notebooks chosen are gone or locked. Choose again.".to_owned(),
        [one] => format!("{one}, and the notebooks inside it."),
        many => format!("{}, and the notebooks inside them.", many.join(", ")),
    }
}

/// The scope sheet's choice.
pub struct ScopeSheet {
    books: Vec<ScopeBook>,
    /// Every unlocked notebook, rather than the chosen ones.
    all: bool,
    chosen: BTreeSet<String>,
}

impl ScopeSheet {
    pub fn new(books: Vec<ScopeBook>, saved: &[String]) -> ScopeSheet {
        let chosen: BTreeSet<String> = saved
            .iter()
            .filter(|id| books.iter().any(|book| &book.id == *id))
            .cloned()
            .collect();
        ScopeSheet {
            all: saved.is_empty(),
            books,
            chosen,
        }
    }

    fn toggle(&mut self, id: &str, cx: &mut Context<Self>) {
        if !self.chosen.remove(id) {
            self.chosen.insert(id.to_owned());
        }
        cx.notify();
    }

    /// What Done saves: no IDs for every notebook, else the chosen ones that
    /// aren't inside another chosen one. `None` while nothing is chosen.
    pub fn result(&self) -> Option<Vec<String>> {
        if self.all {
            return Some(Vec::new());
        }
        let roots: Vec<String> = self
            .books
            .iter()
            .filter(|book| self.chosen.contains(&book.id))
            .filter(|book| {
                book.parent
                    .as_deref()
                    .is_none_or(|parent| !covered(parent, &self.chosen, &self.books))
            })
            .map(|book| book.id.clone())
            .collect();
        (!roots.is_empty()).then_some(roots)
    }

    /// Saves what Done chose (see [`Self::result`]) and closes the sheet.
    fn save(chosen: Option<Vec<String>>, window: &mut Window, cx: &mut App) {
        let Some(notebooks) = chosen else {
            return;
        };
        AppState::update_config(cx, |config| config.ai.agent.notebooks = notebooks);
        // Tokens already given keep the scope they were minted with: they
        // end now, so the next question gets the new one.
        crate::services::end_agent_access(cx);
        window.close_dialog(cx);
        cx.refresh_windows();
    }
}

impl Render for ScopeSheet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let all = self.all;
        let rows = self.books.iter().enumerate().map(|(ix, book)| {
            let inside = book
                .parent
                .as_deref()
                .is_some_and(|parent| covered(parent, &self.chosen, &self.books));
            let id = book.id.clone();
            h_flex()
                .pl(px(8. + 16. * book.depth as f32))
                .py(px(3.))
                .gap(px(8.))
                .child(
                    Checkbox::new(ElementId::NamedInteger(
                        "agent-scope-book".into(),
                        ix as u64,
                    ))
                    .checked(all || inside || self.chosen.contains(&book.id))
                    .disabled(all || inside)
                    .label(book.name.clone())
                    .on_click(cx.listener(move |this, _: &bool, _, cx| this.toggle(&id, cx))),
                )
        });
        v_flex()
            .gap(px(6.))
            .child(
                div()
                    .text_size(px(12.5))
                    .line_height(px(18.))
                    .text_color(colors.text_secondary)
                    .child("The agent may search and read notes in these notebooks, and in the notebooks inside them. Locked notebooks aren't listed and never reach it, nor do the trash or notes tagged #ai-exclude."),
            )
            .child(
                div().debug_selector(|| "agent-scope-all".into()).child(
                    Checkbox::new("agent-scope-all")
                        .checked(all)
                        .label("Every unlocked notebook")
                        .on_click(cx.listener(|this, checked: &bool, _, cx| {
                            this.all = *checked;
                            cx.notify();
                        })),
                ),
            )
            .child(
                v_flex()
                    .id("agent-scope-books")
                    .max_h(px(280.))
                    .overflow_y_scrollbar()
                    .when(all, |el| el.opacity(0.55))
                    .children(rows),
            )
            .when(self.books.is_empty(), |el| {
                el.child(
                    div()
                        .text_size(px(12.5))
                        .text_color(colors.text_muted)
                        .child("There are no unlocked notebooks."),
                )
            })
    }
}

/// The width of the scope sheet.
pub const SHEET_WIDTH: f32 = 440.;

/// Opens the sheet that picks the agent's notebooks.
pub fn open_sheet(window: &mut Window, cx: &mut App) -> Entity<ScopeSheet> {
    let state = cx.global::<AppState>();
    let books = scope_books(&state.store);
    let saved = state.config.ai.agent.notebooks.clone();
    let sheet = cx.new(|_| ScopeSheet::new(books, &saved));
    let view = sheet.clone();
    window.open_alert_dialog(cx, move |alert, _, cx| {
        let done = view.clone();
        let ready = view.read(cx).result().is_some();
        alert
            .title("Notebooks the agent may read")
            .width(px(SHEET_WIDTH))
            .child(view.clone())
            .footer(
                h_flex()
                    .w_full()
                    .gap_2()
                    .justify_end()
                    .child(
                        Button::new("agent-scope-cancel")
                            .small()
                            .label("Cancel")
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                    )
                    .child(
                        Button::new("agent-scope-done")
                            .debug_selector(|| "agent-scope-done".into())
                            .small()
                            .primary()
                            .label("Done")
                            .disabled(!ready)
                            .on_click(move |_, window, cx| {
                                let chosen = done.read(cx).result();
                                ScopeSheet::save(chosen, window, cx);
                            }),
                    ),
            )
    });
    sheet
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{ScopeBook, ScopeSheet, covered, scope_books, scope_summary};
    use kanso_core::DocType;
    use kanso_store::Store;
    use kanso_store::{Origin, Put};
    use proptest::prelude::*;
    use serde_json::json;

    fn book(store: &Store, name: &str, parent: Option<&str>, locked: bool) -> String {
        let mut fields = json!({ "name": name });
        if let Some(parent) = parent {
            fields["parentBookId"] = json!(parent);
        }
        if locked {
            fields["kansoLock"] = json!({ "salt": "abc" });
        }
        store
            .put(DocType::Book, fields, Put::Upsert, Origin::Ui)
            .unwrap()
            .id
    }

    #[test]
    fn locked_notebooks_are_never_offered() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        let work = book(&store, "Work", None, false);
        let clients = book(&store, "Clients", Some(&work), false);
        let vault = book(&store, "Vault", None, true);
        let _inside = book(&store, "Inside vault", Some(&vault), false);
        let books = scope_books(&store);
        let names: Vec<&str> = books.iter().map(|book| book.name.as_str()).collect();
        assert_eq!(names, ["Work", "Clients"]);
        assert_eq!(books[1].depth, 1);
        assert_eq!(books[1].parent.as_deref(), Some(work.as_str()));
        assert_eq!(scope_summary(&[], &books), "Every unlocked notebook.");
        assert_eq!(
            scope_summary(std::slice::from_ref(&work), &books),
            "Work, and the notebooks inside it."
        );
        assert!(scope_summary(&[vault], &books).starts_with("None"));
        let chosen: BTreeSet<String> = [work.clone()].into();
        assert!(covered(&clients, &chosen, &books));
        let sheet = ScopeSheet::new(books, std::slice::from_ref(&clients));
        assert!(!sheet.all);
        assert_eq!(sheet.result(), Some(vec![clients]));
    }

    fn gen_books() -> impl Strategy<Value = Vec<ScopeBook>> {
        prop::collection::vec(any::<prop::sample::Index>(), 1..10).prop_map(|parents| {
            parents
                .iter()
                .enumerate()
                .map(|(ix, parent)| ScopeBook {
                    id: format!("book:{ix}"),
                    name: format!("Book {ix}"),
                    depth: 0,
                    parent: (ix > 0 && parent.index(3) > 0)
                        .then(|| format!("book:{}", parent.index(ix))),
                })
                .collect()
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// What Done saves covers exactly the notebooks the sheet shows as
        /// chosen: nothing chosen is lost, nothing unchosen is added, and no
        /// saved notebook is inside another saved one.
        #[test]
        fn prop_the_saved_scope_is_what_the_sheet_shows(
            books in gen_books(),
            picks in prop::collection::vec(any::<prop::sample::Index>(), 1..5),
        ) {
            let chosen: Vec<String> = picks.iter().map(|pick| books[pick.index(books.len())].id.clone()).collect();
            let sheet = ScopeSheet::new(books.clone(), &chosen);
            let saved: BTreeSet<String> = sheet.result().unwrap().into_iter().collect();
            let chosen: BTreeSet<String> = chosen.into_iter().collect();
            for book in &books {
                prop_assert_eq!(covered(&book.id, &saved, &books), covered(&book.id, &chosen, &books), "{}", book.id);
            }
            for id in &saved {
                let book = books.iter().find(|book| &book.id == id).unwrap();
                prop_assert!(book.parent.as_deref().is_none_or(|parent| !covered(parent, &saved, &books)));
            }
        }
    }
}

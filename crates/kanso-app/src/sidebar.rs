//! The left column: the app's name and a compose button, the library
//! sections, notebooks, tags, statuses and trash, and a footer with who is
//! using it, sync, the local API and settings.
//!
//! Collapsed (⌘\, or the panel button in its top band), it is a rail of
//! icons: a button to open it again, compose, the library places, the
//! notebooks and the tags (lists in the palette), trash, sync and settings.

use std::collections::HashSet;
use std::time::Duration;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::menu::{ContextMenuExt as _, PopupMenuItem};
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, StyledExt as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::NoteStatus;
use kanso_core::tags::{TagNode, is_within};

use crate::clock::Clock;
use crate::editor::rgb_color;
use crate::icon_themes::{Slot, library_icon};
use crate::layout::{BOTTOM_BAND_HEIGHT, TOP_BAND_HEIGHT};
use crate::library::{Filter, Library};
use crate::note_batch::Change as NoteChange;
use crate::note_list::DraggedNotes;
use crate::services::{AccountState, ApiState, Services, SyncState};
use crate::settings::OpenAccountSettings;
use crate::theme::{Colors, Rgb, muted_tag_color, section_label, status_glyph};
use crate::workspace::{AppState, NewNote, OpenSettings, ToggleSidebar};
use crate::zoom::px;

/// Things only the workspace can do (it owns dialogs and the editor).
pub enum SidebarEvent {
    NewNote,
    NewBook {
        parent: Option<String>,
    },
    RenameBook(String),
    /// "Change Icon…" in a notebook's menu: the workspace opens the picker.
    ChangeBookIcon(String),
    /// "Change Color…" in a notebook's menu: the workspace opens the dialog.
    ChangeBookColor(String),
    DeleteBook(String),
    LockBook(String),
    RemoveBookLock(String),
    PublishBook(String),
    RenameTag(String),
    RecolorTag(String, kanso_core::TagColor),
    DeleteTag(String),
    NewTag,
    EmptyTrash,
    /// The rail's notebooks button: the palette lists them to go to one.
    GoToNotebook,
    /// The rail's tags button: the palette lists them to go to one.
    GoToTag,
    /// Notes dragged from the list and let go on a notebook, a tag or the
    /// trash: the change to make to all of them.
    DropNotes(Vec<String>, NoteChange),
}

const ROW_HEIGHT: f32 = 28.;

/// What a sidebar row shows.
struct Entry {
    icon: Option<AnyElement>,
    label: SharedString,
    count: u32,
    /// How deep in the notebook tree.
    depth: usize,
    /// The color a notebook's row is washed in (`0xRRGGBB`), if it has one.
    color: Option<u32>,
}
/// Tags shown before "…" when there are many: the most used ones.
const TAG_LIMIT: usize = 10;
/// How often relative times ("Synced 3 min ago") are brought up to date.
const TICK: Duration = Duration::from_secs(30);

pub struct Sidebar {
    library: Entity<Library>,
    scroll: ScrollHandle,
    /// Whether the user opened or closed the Status group; until they do,
    /// it is open exactly while a status is the filter.
    status_choice: Option<bool>,
    /// Whether a status was the filter when the library last changed.
    status_filter: bool,
    /// Whether every tag shows, rather than the most used.
    all_tags: bool,
    /// The levels of the tag tree opened, by their paths in lowercase.
    open_tags: HashSet<String>,
    /// The notebooks opened to show the ones inside them, and those folded
    /// around the one showing.
    folds: BookFolds,
    /// Collapsed to the icon rail.
    collapsed: bool,
    /// Settings fill the window beside the sidebar: the gear is the place
    /// shown, not the filter.
    settings_open: bool,
    user: SharedString,
    /// "DEV" in Kanso Dev, beside the name, so it is never taken for the
    /// Kanso installed from a release.
    pub(crate) badge: Option<&'static str>,
    /// Shows the rail button with this tooltip hovered, its tooltip open,
    /// as a pointer resting on it would.
    #[cfg(test)]
    pub(crate) reveal_tooltip: Option<&'static str>,
    _tick: Task<()>,
}

impl EventEmitter<SidebarEvent> for Sidebar {}

impl Sidebar {
    pub fn new(library: Entity<Library>, cx: &mut Context<Self>) -> Self {
        cx.observe(&library, |this, library, cx| {
            let is_status = matches!(library.read(cx).filter, Filter::Status(_));
            let choice = status_choice_after(this.status_choice, this.status_filter, is_status);
            this.set_status_choice(choice, cx);
            this.status_filter = is_status;
            cx.notify();
        })
        .detach();
        // Mentions read or new: the Shared with me badge follows.
        cx.observe_global::<crate::collab_model::Collab>(|_, cx| cx.notify())
            .detach();
        let tick = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(TICK).await;
                if this.update(cx, |_, cx| cx.notify()).is_err() {
                    break;
                }
            }
        });
        let user = std::env::var("USER")
            .ok()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| "This device".into());
        // The Status group as the user last left it.
        let status_choice = cx
            .try_global::<AppState>()
            .and_then(|state| state.config.layout.status_open);
        // The notebooks as the user last left them.
        let open_books = cx
            .try_global::<AppState>()
            .map(|state| state.config.layout.open_books.iter().cloned().collect())
            .unwrap_or_default();
        Sidebar {
            library,
            scroll: ScrollHandle::new(),
            status_choice,
            status_filter: false,
            all_tags: false,
            open_tags: HashSet::new(),
            folds: BookFolds {
                open: open_books,
                ..BookFolds::default()
            },
            collapsed: false,
            settings_open: false,
            user: user.into(),
            badge: crate::flavour::CURRENT.badge,
            #[cfg(test)]
            reveal_tooltip: None,
            _tick: tick,
        }
    }

    /// Collapses the sidebar to its icon rail, or opens it again.
    pub fn set_collapsed(&mut self, collapsed: bool, cx: &mut Context<Self>) {
        if self.collapsed != collapsed {
            self.collapsed = collapsed;
            cx.notify();
        }
    }

    pub fn set_settings_open(&mut self, open: bool, cx: &mut Context<Self>) {
        if self.settings_open != open {
            self.settings_open = open;
            cx.notify();
        }
    }

    /// Whether the Status group shows its rows now.
    fn status_open(&self, cx: &App) -> bool {
        let filter = &self.library.read(cx).filter;
        status_group_open(self.status_choice, matches!(filter, Filter::Status(_)))
    }

    /// A click on the Status header: it opens or closes, for good.
    // no-prop-test: `status_group_open` has the properties and
    // `the_status_group_remembers` checks it is kept.
    pub fn toggle_status_group(&mut self, cx: &mut Context<Self>) {
        let open = self.status_open(cx);
        self.set_status_choice(Some(!open), cx);
        cx.notify();
    }

    /// Opens or closes the Status group, and keeps the choice for next time.
    fn set_status_choice(&mut self, choice: Option<bool>, cx: &mut Context<Self>) {
        if self.status_choice == choice {
            return;
        }
        self.status_choice = choice;
        if cx.has_global::<AppState>() {
            AppState::update_config(cx, |config| config.layout.status_open = choice);
        }
    }

    /// A click on a notebook's arrow: it opens to show the notebooks inside
    /// it, or folds them away, as it shows now, and stays that way next
    /// time. One the notebook showing is inside folds too, and stays folded
    /// while that one shows.
    pub fn toggle_book(&mut self, id: &str, cx: &mut Context<Self>) {
        let (showing, around) = self.showing_around(cx);
        self.folds.toggle(id, showing.as_deref(), &around);
        if cx.has_global::<AppState>() {
            let mut open: Vec<String> = self.folds.open.iter().cloned().collect();
            open.sort();
            AppState::update_config_soon(cx, |config| config.layout.open_books = open);
        }
        cx.notify();
    }

    /// The notebook showing, if one is, and the notebooks it is inside.
    fn showing_around(&self, cx: &App) -> (Option<String>, HashSet<String>) {
        let library = self.library.read(cx);
        match &library.filter {
            Filter::Book(id) => (
                Some(id.clone()),
                library
                    .book_path(id)
                    .iter()
                    .filter(|b| b.id != *id)
                    .map(|b| b.id.clone())
                    .collect(),
            ),
            _ => (None, HashSet::new()),
        }
    }

    /// The notebook rows that show, top to bottom, as drawn.
    fn book_rows(&self, cx: &App) -> Vec<BookRow> {
        let library = self.library.read(cx);
        let tree = library.book_tree();
        let depths: Vec<usize> = tree.iter().map(|(depth, _)| *depth).collect();
        // The notebooks the one showing is inside stay open around it.
        let (showing, around) = self.showing_around(cx);
        let open = |ix: usize| {
            self.folds
                .is_open(&tree[ix].1.id, showing.as_deref(), &around)
        };
        visible_book_rows(&depths, &open)
            .into_iter()
            .map(|(ix, parent)| {
                let (depth, book) = tree[ix];
                BookRow {
                    depth,
                    id: book.id.clone(),
                    name: book.name.clone(),
                    count: book.count,
                    icon: crate::book_icons::book_icon(book, cx),
                    color: crate::book_colors::book_color(book),
                    parent,
                    open: parent && open(ix),
                    // Locked by its own lock, or a notebook it is inside.
                    locked: library.is_book_locked(&book.id),
                    own_lock: book.is_locked(),
                }
            })
            .collect()
    }

    /// The notebooks showing, each with the icon its row draws and whether
    /// it shows open (tests).
    #[cfg(test)]
    pub fn book_rows_for_test(
        &self,
        cx: &App,
    ) -> Vec<(String, crate::book_icons::NotebookIcon, bool)> {
        self.book_rows(cx)
            .into_iter()
            .map(|row| {
                (
                    row.name,
                    crate::book_icons::shown(
                        crate::book_icons::opened(row.icon, row.open),
                        row.locked,
                        crate::locked::LockKeys::is_unlocked(cx),
                    ),
                    row.open,
                )
            })
            .collect()
    }

    /// The arrow before a notebook or tag level that has others inside it,
    /// which opens or folds them; an empty space of its size before one that
    /// doesn't, so the icons line up.
    fn fold_arrow(
        &self,
        id: SharedString,
        parent: bool,
        open: bool,
        on_click: impl Fn(&mut Self, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if !parent {
            return div().flex_none().size(px(16.)).into_any_element();
        }
        let colors = Colors::get(cx);
        div()
            .id(id)
            .flex_none()
            .size(px(16.))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(4.))
            .hover(|el| el.bg(colors.sidebar_hover))
            .child(
                library_icon(if open {
                    Slot::FoldOpen
                } else {
                    Slot::FoldClosed
                })
                .size(px(13.))
                .text_color(colors.text_muted),
            )
            .on_click(cx.listener(move |this, _, _, cx| {
                cx.stop_propagation();
                on_click(this, cx);
            }))
            .into_any_element()
    }

    /// Whether the gear shows as the place on screen.
    #[cfg(test)]
    pub fn settings_open_for_test(&self) -> bool {
        self.settings_open
    }

    fn row(
        &self,
        id: impl Into<ElementId>,
        filter: Filter,
        entry: Entry,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let Entry {
            icon,
            label,
            count,
            depth,
            color,
        } = entry;
        let selected = !self.settings_open && self.library.read(cx).filter == filter;
        let colors = Colors::get(cx);
        let library = self.library.clone();
        div()
            .id(id)
            .h(px(ROW_HEIGHT))
            .flex()
            .flex_none()
            .items_center()
            .gap(px(10.))
            .mx_2()
            .pr(px(10.))
            .pl(px(10. + depth as f32 * 14.))
            .rounded(px(6.))
            .text_size(px(13.5))
            .cursor_pointer()
            .map(|el| {
                use crate::book_colors::{WASH_HOVER, WASH_REST, WASH_SELECTED, wash};
                match (selected, color) {
                    (true, None) => el.bg(colors.sidebar_active),
                    (false, None) => el.hover(|el| el.bg(colors.sidebar_hover)),
                    (true, Some(rgb)) => el.bg(wash(rgb, WASH_SELECTED)),
                    (false, Some(rgb)) => el
                        .bg(wash(rgb, WASH_REST))
                        .hover(|el| el.bg(wash(rgb, WASH_HOVER))),
                }
            })
            .children(icon)
            .child(div().flex_1().min_w_0().truncate().child(label))
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(colors.quiet(selected))
                    .child(count.to_string()),
            )
            .on_click(move |_, _, cx| {
                library.update(cx, |library, cx| library.set_filter(filter.clone(), cx));
            })
    }

    /// Makes `el` take notes dragged from the list: it lights up while they
    /// are over it, and letting go makes `change` to them.
    fn drop_notes(
        &self,
        el: Stateful<Div>,
        change: NoteChange,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let accent = Hsla::from(Colors::get(cx).accent);
        el.drag_over::<DraggedNotes>(move |style, _, _, _| {
            style
                .bg(accent.opacity(0.26))
                .border_color(accent.opacity(0.7))
        })
        .on_drop(cx.listener(move |_, notes: &DraggedNotes, _, cx| {
            cx.emit(SidebarEvent::DropNotes(notes.ids.to_vec(), change.clone()));
        }))
    }

    /// Tasks, with the overdue ones counted in a warm red pill (nothing
    /// when none are).
    // no-prop-test: drawing; the Tasks snapshots show it.
    fn tasks_row(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let overdue = self.library.read(cx).overdue;
        let selected = !self.settings_open && self.library.read(cx).filter == Filter::Tasks;
        let colors = Colors::get(cx);
        let icon = self.icon(library_icon(Slot::Tasks), cx);
        let library = self.library.clone();
        div()
            .id("tasks")
            .h(px(ROW_HEIGHT))
            .flex()
            .flex_none()
            .items_center()
            .gap(px(10.))
            .mx_2()
            .pr(px(8.))
            .pl(px(10.))
            .rounded(px(6.))
            .text_size(px(13.5))
            .cursor_pointer()
            .when(selected, |el| el.bg(colors.sidebar_active))
            .when(!selected, |el| el.hover(|el| el.bg(colors.sidebar_hover)))
            .child(icon)
            .child(div().flex_1().min_w_0().truncate().child("Tasks"))
            .when(overdue > 0, |el| {
                el.child(
                    div()
                        .id("tasks-overdue")
                        .px(px(6.))
                        .h(px(17.))
                        .flex()
                        .items_center()
                        .rounded_full()
                        .bg(Hsla::from(colors.danger).opacity(0.16))
                        .text_size(px(11.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors.danger)
                        .child(overdue.to_string())
                        .tooltip(move |window, cx| {
                            Tooltip::new(format!(
                                "{overdue} overdue task{}",
                                if overdue == 1 { "" } else { "s" }
                            ))
                            .build(window, cx)
                        }),
                )
            })
            .on_click(move |_, _, cx| {
                library.update(cx, |library, cx| library.set_filter(Filter::Tasks, cx));
            })
    }

    /// Shared with me, with its notes counted and, when there are any,
    /// the unread mentions in an accent pill; `None` while nothing is
    /// shared and the account is signed out.
    // no-prop-test: drawing; `shows_shared` and `mention_badge` have the
    // tests, and the sidebar snapshots show the row.
    fn shared_row(&self, cx: &mut Context<Self>) -> Option<Stateful<Div>> {
        let (shown, unread) = shared_state(self.library.read(cx).shared, cx);
        if !shown {
            return None;
        }
        let count = self.library.read(cx).shared;
        let selected = !self.settings_open && self.library.read(cx).filter == Filter::Shared;
        let colors = Colors::get(cx);
        let icon = self.icon(library_icon(Slot::Shared), cx);
        let library = self.library.clone();
        let badge = mention_badge(unread);
        Some(
            div()
                .id("shared")
                .h(px(ROW_HEIGHT))
                .flex()
                .flex_none()
                .items_center()
                .gap(px(10.))
                .mx_2()
                .pr(px(10.))
                .pl(px(10.))
                .rounded(px(6.))
                .text_size(px(13.5))
                .cursor_pointer()
                .when(selected, |el| el.bg(colors.sidebar_active))
                .when(!selected, |el| el.hover(|el| el.bg(colors.sidebar_hover)))
                .child(icon)
                .child(div().flex_1().min_w_0().truncate().child("Shared with me"))
                .when_some(badge, |el, badge| {
                    el.child(
                        div()
                            .id("shared-mentions")
                            .debug_selector(|| "shared-mentions".into())
                            .px(px(6.))
                            .h(px(17.))
                            .flex()
                            .items_center()
                            .rounded_full()
                            .bg(Hsla::from(colors.accent).opacity(0.16))
                            .text_size(px(11.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors.accent)
                            .child(badge)
                            .tooltip(move |window, cx| {
                                Tooltip::new(format!(
                                    "{unread} unread mention{}",
                                    if unread == 1 { "" } else { "s" }
                                ))
                                .build(window, cx)
                            }),
                    )
                })
                .when(unread == 0, |el| {
                    el.child(
                        div()
                            .text_size(px(12.))
                            .text_color(colors.quiet(selected))
                            .child(count.to_string()),
                    )
                })
                .on_click(move |_, _, cx| {
                    library.update(cx, |library, cx| library.set_filter(Filter::Shared, cx));
                }),
        )
    }

    /// A section title, with an optional control on the right.
    fn heading(
        &self,
        label: &'static str,
        action: Option<AnyElement>,
        cx: &mut Context<Self>,
    ) -> Div {
        h_flex()
            .flex_none()
            .mt(px(14.))
            .mb_0p5()
            .mx_2()
            .pl(px(10.))
            .pr_1()
            .h(px(26.))
            .map(|el| section_label(el, Colors::get(cx)))
            .child(div().flex_1().child(label))
            .children(action)
    }

    fn icon(&self, name: impl Into<Icon>, cx: &mut Context<Self>) -> AnyElement {
        self.book_icon(name, None, cx)
    }

    /// A row's icon, tinted for a notebook's color (`0xRRGGBB`) if given.
    fn book_icon(
        &self,
        name: impl Into<Icon>,
        color: Option<u32>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let tint = match color {
            Some(rgb) => crate::book_colors::icon_tint(rgb, cx.theme().is_dark()),
            None => Colors::get(cx).text_secondary,
        };
        Icon::new(name)
            .size(px(15.))
            .text_color(tint)
            .into_any_element()
    }

    /// A small quiet icon button for a section heading.
    fn add_button(
        &self,
        id: &'static str,
        tooltip: &'static str,
        event: fn() -> SidebarEvent,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        Button::new(id)
            .ghost()
            .xsmall()
            .icon(Icon::new(IconName::Plus).text_color(Colors::get(cx).text_secondary))
            .tooltip(tooltip)
            .on_click(cx.listener(move |_, _, _, cx| cx.emit(event())))
            .into_any_element()
    }

    /// The top band: the traffic lights' row, with the button that folds
    /// the sidebar to its rail at the far end. The rest stays empty so the
    /// window drags from it.
    // no-prop-test: drawing; the sidebar snapshots show it.
    fn render_top_band(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let revealed = self.revealed_tooltip(TOGGLE_LABEL, Some(&ToggleSidebar), cx);
        h_flex()
            .flex_none()
            .h(px(TOP_BAND_HEIGHT))
            .justify_end()
            // In line with the compose button below, and centred on the
            // traffic lights rather than on the band.
            .pr(px(12.))
            .pb(px(2.))
            .child(
                div()
                    .relative()
                    .child(
                        Button::new("sidebar-toggle")
                            .ghost()
                            .small()
                            .icon(Icon::new(Lucide::PanelLeft).text_color(colors.text_secondary))
                            .tooltip_with_action(TOGGLE_LABEL, &ToggleSidebar, None)
                            .debug_selector(|| "sidebar-toggle".into())
                            .on_click(|_, window, cx| {
                                window.dispatch_action(Box::new(ToggleSidebar), cx)
                            }),
                    )
                    .children(revealed),
            )
    }

    fn render_brand(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        h_flex()
            .flex_none()
            .h(px(32.))
            .mx_2()
            .pl(px(10.))
            .pr_1()
            .gap(px(10.))
            .child(
                Icon::new(Lucide::NotebookText)
                    .size(px(16.))
                    .text_color(colors.text_secondary),
            )
            .child(
                h_flex()
                    .flex_1()
                    .gap(px(7.))
                    .child(div().text_size(px(15.)).font_semibold().child("Kanso"))
                    .children(self.badge.map(|badge| {
                        div()
                            .id("dev-badge")
                            .debug_selector(|| "dev-badge".into())
                            .px(px(5.))
                            .py(px(1.))
                            .rounded(px(4.))
                            .border_1()
                            .border_color(colors.warning)
                            .text_size(px(10.))
                            .line_height(px(13.))
                            .font_semibold()
                            .text_color(colors.warning)
                            .child(badge)
                    })),
            )
            .child(
                Button::new("compose")
                    .ghost()
                    .small()
                    .icon(Icon::new(Lucide::SquarePen).text_color(colors.text_secondary))
                    .tooltip_with_action("New note", &NewNote, None)
                    .on_click(cx.listener(|_, _, _, cx| cx.emit(SidebarEvent::NewNote))),
            )
    }

    fn render_tags(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let library = self.library.read(cx);
        // Tags nested with `/` show as a tree below the others' chips.
        let flat = flat_tag_ids(&library.tag_tree);
        let flat_tags: Vec<_> = library
            .tags
            .iter()
            .filter(|t| flat.contains(t.id.as_str()))
            .collect();
        let selected_tag = match &library.filter {
            Filter::Tag(id) => flat_tags.iter().position(|t| &t.id == id),
            _ => None,
        };
        let counts: Vec<u32> = flat_tags.iter().map(|t| t.count).collect();
        let shown = visible_tags(&counts, TAG_LIMIT, selected_tag, self.all_tags);
        let more = counts.len() > TAG_LIMIT;
        let tags: Vec<_> = shown
            .iter()
            .map(|&ix| {
                let t = flat_tags[ix];
                (t.id.clone(), t.name.clone(), t.count, t.color)
            })
            .collect();
        let colors = Colors::get(cx);
        let dark = cx.theme().is_dark();
        let chips = tags.into_iter().map(|(id, name, count, color)| {
            let view = cx.entity();
            let library = self.library.clone();
            let filter = Filter::Tag(id.clone());
            let selected = !self.settings_open && self.library.read(cx).filter == filter;
            let tag_id = id.clone();
            let chip = div()
                .id(SharedString::from(format!("tag-{id}")))
                .flex()
                .items_center()
                .h(px(24.))
                .px(px(7.))
                .rounded(px(6.))
                .border_1()
                .text_size(px(12.))
                .cursor_pointer()
                .map(|el| {
                    if selected {
                        el.bg(colors.sidebar_active)
                            .border_color(colors.sidebar_active)
                    } else {
                        el.border_color(colors.divider)
                            .hover(|el| el.bg(colors.sidebar_hover))
                    }
                })
                .child(
                    div()
                        .text_color(rgb_color(muted_tag_color(color.rgb(), dark)))
                        .child("#"),
                )
                .child(
                    div()
                        .text_color(if selected {
                            colors.text
                        } else {
                            colors.text_secondary
                        })
                        .child(name),
                )
                .child(
                    div()
                        .ml(px(5.))
                        .text_color(colors.quiet(selected))
                        .child(count.to_string()),
                )
                .on_click(move |_, _, cx| {
                    library.update(cx, |library, cx| library.set_filter(filter.clone(), cx));
                });
            let selector = format!("tag-{id}");
            self.drop_notes(chip, NoteChange::AddTag(id.clone()), cx)
                .debug_selector(move || selector)
                .context_menu(move |menu, window, cx| tag_menu(menu, &view, &tag_id, window, cx))
        });
        let chips: Vec<_> = chips.collect();
        let toggle = more.then(|| {
            div()
                .id("more-tags")
                .flex()
                .items_center()
                .h(px(24.))
                .px(px(7.))
                .rounded(px(6.))
                .text_size(px(12.))
                .text_color(colors.text_muted)
                .cursor_pointer()
                .hover(|el| el.bg(colors.sidebar_hover))
                .child(if self.all_tags { "Fewer" } else { "…" })
                .tooltip({
                    let all = self.all_tags;
                    move |window, cx| {
                        Tooltip::new(if all {
                            "Show the most used tags"
                        } else {
                            "Show all tags"
                        })
                        .build(window, cx)
                    }
                })
                .on_click(cx.listener(|this, _, _, cx| {
                    this.all_tags = !this.all_tags;
                    cx.notify();
                }))
        });
        let chips = h_flex()
            .flex_wrap()
            .flex_none()
            .gap(px(6.))
            .mx_2()
            .px(px(8.))
            .pt_1()
            .children(chips)
            .children(toggle);
        v_flex()
            .flex_none()
            .child(chips)
            .child(div().h(px(4.)))
            .children(self.render_tag_tree(cx))
    }

    /// The tags nested with `/`, one row per level, folded by default. A
    /// level shows the notes tagged with it or anything inside it.
    fn render_tag_tree(&self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let library = self.library.read(cx);
        let showing = match &library.filter {
            Filter::TagPath(path) => Some(path.clone()),
            Filter::Tag(id) => library.tag(id).map(|t| t.name.clone()),
            _ => None,
        };
        let nested: Vec<TagNode> = library
            .tag_tree
            .iter()
            .filter(|node| !node.children.is_empty())
            .cloned()
            .collect();
        let open = |path: &str| {
            self.open_tags.contains(&path.to_lowercase())
                || showing.as_deref().is_some_and(|shown| {
                    is_within(shown, path) && !shown.eq_ignore_ascii_case(path)
                })
        };
        let colors = Colors::get(cx);
        let mut rows = Vec::new();
        for (depth, node) in visible_tag_rows(&nested, &open) {
            let filter = if node.children.is_empty()
                && let Some(id) = &node.tag_id
            {
                Filter::Tag(id.clone())
            } else {
                Filter::TagPath(node.path.clone())
            };
            let parent = !node.children.is_empty();
            let is_open = parent && open(&node.path);
            let path = node.path.to_lowercase();
            let chevron = self.fold_arrow(
                SharedString::from(format!("tag-fold-{path}")),
                parent,
                is_open,
                move |this, cx| {
                    if !this.open_tags.remove(&path) {
                        this.open_tags.insert(path.clone());
                    }
                    cx.notify();
                },
                cx,
            );
            let icon = h_flex()
                .gap(px(2.))
                .child(chevron)
                .child(
                    div()
                        .text_size(px(13.))
                        .text_color(colors.text_muted)
                        .child("#"),
                )
                .into_any_element();
            let row = self.row(
                SharedString::from(format!("tag-tree-{}", node.path.to_lowercase())),
                filter,
                Entry {
                    icon: Some(icon),
                    label: node.label.clone().into(),
                    count: node.count,
                    depth,
                    color: None,
                },
                cx,
            );
            let row = match node.tag_id.clone() {
                Some(tag_id) => {
                    let view = cx.entity();
                    let row = self.drop_notes(row, NoteChange::AddTag(tag_id.clone()), cx);
                    row.context_menu(move |menu, window, cx| {
                        tag_menu(menu, &view, &tag_id, window, cx)
                    })
                    .into_any_element()
                }
                None => row.into_any_element(),
            };
            rows.push(row);
        }
        rows
    }

    fn render_status(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let open = self.status_open(cx);
        let counts = self.library.read(cx).counts.by_status.clone();
        let colors = Colors::get(cx);
        let header = h_flex()
            .id("status-group")
            .flex_none()
            .mt(px(14.))
            .mb_0p5()
            .mx_2()
            .pl(px(10.))
            .pr(px(8.))
            .h(px(26.))
            .rounded(px(6.))
            .map(|el| section_label(el, colors))
            .cursor_pointer()
            .hover(|el| el.bg(colors.sidebar_hover))
            .child(div().flex_1().child("Status"))
            .child(
                Icon::new(if open {
                    IconName::ChevronDown
                } else {
                    IconName::ChevronRight
                })
                .size(px(14.))
                .text_color(colors.text_muted),
            )
            .on_click(cx.listener(|this, _, _, cx| this.toggle_status_group(cx)));
        let rows = open.then(|| {
            [
                NoteStatus::Active,
                NoteStatus::OnHold,
                NoteStatus::Completed,
                NoteStatus::Dropped,
            ]
            .into_iter()
            .map(|status| {
                let count = counts.get(&status).copied().unwrap_or(0);
                // Colored as in the note list; Dropped as quiet as the
                // other rows' icons.
                let colors = Colors::get(cx);
                let icon =
                    status_glyph(status, colors, colors.text_secondary).map(|(icon, tint)| {
                        Icon::new(icon)
                            .size(px(15.))
                            .text_color(tint)
                            .into_any_element()
                    });
                self.row(
                    SharedString::from(format!("status-{}", status.as_str())),
                    Filter::Status(status),
                    Entry {
                        icon,
                        label: status.label().into(),
                        count,
                        depth: 0,
                        color: None,
                    },
                    cx,
                )
            })
            .collect::<Vec<_>>()
        });
        v_flex()
            .flex_none()
            .child(header)
            .children(rows.into_iter().flatten())
    }

    /// The local API's and sync's states, with what their tooltips say.
    fn service_states(cx: &App) -> ServiceStates {
        match cx.try_global::<Services>() {
            Some(services) => ServiceStates {
                api: services.api.clone(),
                api_tip: services.api_status().into(),
                sync: services.sync.clone(),
                account: services.account_state(),
                sync_tip: services.sync_status(cx).into(),
            },
            None => ServiceStates {
                api: ApiState::Off,
                api_tip: "API off".into(),
                sync: SyncState::Off,
                account: AccountState::SignedOut,
                sync_tip: SIGN_IN_TO_SYNC.into(),
            },
        }
    }

    fn render_footer(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let ServiceStates {
            api,
            api_tip,
            sync,
            account,
            sync_tip,
        } = Self::service_states(cx);
        let (tone, sync_label) = sync_line(&sync, &account, |at| Clock::edited_label(cx, at));
        // Signed out, the line is the way in: a click opens the Account page.
        let opens_account = !matches!(
            account,
            AccountState::SignedIn { .. } | AccountState::Busy(_)
        );
        let name = SharedString::from(footer_name(&account, &self.user).to_owned());
        let dot = tone_color(tone, colors);
        // The shape says whether the API is listening, off, or failing to
        // listen; the color only repeats it.
        let api_color = match api {
            ApiState::Running(_) => colors.text_secondary,
            ApiState::Off => colors.text_muted,
            ApiState::PortInUse(_) | ApiState::Failed(_) => colors.warning,
        };
        let api_icon = api_icon(&api);
        h_flex()
            .flex_none()
            .h(px(BOTTOM_BAND_HEIGHT))
            .pl(px(12.))
            .pr(px(6.))
            .gap(px(6.))
            .border_t_1()
            .border_color(colors.divider)
            .child(
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .size(px(24.))
                    .rounded_full()
                    .bg(colors.fill)
                    .text_size(px(10.5))
                    .font_semibold()
                    .text_color(colors.text_secondary)
                    .child(initials(&name)),
            )
            .child(
                v_flex()
                    .id("sync-status")
                    .debug_selector(|| "sync-status".into())
                    .flex_1()
                    .min_w_0()
                    .when(opens_account, |el| {
                        el.cursor_pointer().on_click(|_, window, cx| {
                            window.dispatch_action(Box::new(OpenAccountSettings), cx)
                        })
                    })
                    .child(
                        div()
                            .text_size(px(12.5))
                            .line_height(px(15.))
                            .font_medium()
                            .truncate()
                            .child(name),
                    )
                    .child(
                        h_flex()
                            .gap(px(5.))
                            .text_size(px(11.5))
                            .line_height(px(14.))
                            .text_color(colors.text_secondary)
                            .child(div().flex_none().size(px(6.)).rounded_full().bg(dot))
                            .child(div().min_w_0().truncate().child(sync_label)),
                    )
                    .tooltip(move |window, cx| Tooltip::new(sync_tip.clone()).build(window, cx)),
            )
            .child(
                div()
                    .id("api-status")
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .size(px(20.))
                    .rounded(px(6.))
                    .child(Icon::new(api_icon).size(px(15.)).text_color(api_color))
                    .tooltip(move |window, cx| Tooltip::new(api_tip.clone()).build(window, cx)),
            )
            .child(
                div()
                    .id("settings")
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .size(px(24.))
                    .rounded(px(6.))
                    .cursor_pointer()
                    .map(|el| {
                        if self.settings_open {
                            el.bg(colors.sidebar_active)
                        } else {
                            el.hover(|el| el.bg(colors.sidebar_hover))
                        }
                    })
                    .child(Icon::new(IconName::Settings).size(px(15.)).text_color(
                        if self.settings_open {
                            colors.text
                        } else {
                            colors.text_secondary
                        },
                    ))
                    .tooltip(|window, cx| {
                        Tooltip::new("Settings")
                            .action(&OpenSettings, None)
                            .build(window, cx)
                    })
                    .on_click(|_, window, cx| window.dispatch_action(Box::new(OpenSettings), cx)),
            )
    }
}

/// What the footer and the rail show about the services.
struct ServiceStates {
    api: ApiState,
    api_tip: SharedString,
    sync: SyncState,
    account: AccountState,
    sync_tip: SharedString,
}

/// The rail's buttons: one square for each icon.
const RAIL_BUTTON: f32 = 34.;
/// What the buttons that fold and open the sidebar are called, as the
/// palette and the View menu call the action.
const TOGGLE_LABEL: &str = "Toggle sidebar";

impl Sidebar {
    /// One of the rail's icon buttons, named by its tooltip (with the
    /// shortcut of `action`, when it has one), filled while `active`.
    // no-prop-test: drawing; the rail snapshots show it.
    fn rail_button(
        &self,
        id: &'static str,
        icon: impl Into<Icon>,
        label: &'static str,
        action: Option<Box<dyn Action>>,
        active: bool,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let colors = Colors::get(cx);
        let revealed = self.revealed_tooltip(label, action.as_deref(), cx);
        div()
            .id(id)
            .debug_selector(move || id.into())
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(px(RAIL_BUTTON))
            .rounded(px(6.))
            .cursor_pointer()
            .when(active, |el| el.bg(colors.sidebar_active))
            .when(!active, |el| el.hover(|el| el.bg(colors.sidebar_hover)))
            .when(!active && revealed.is_some(), |el| {
                el.bg(colors.sidebar_hover)
            })
            .child(Icon::new(icon).size(px(16.)).text_color(if active {
                colors.text
            } else {
                colors.text_secondary
            }))
            .children(revealed)
            .tooltip(move |window, cx| {
                Tooltip::new(label)
                    .when_some(action.as_ref(), |tooltip, action| {
                        tooltip.action(action.as_ref(), None)
                    })
                    .build(window, cx)
            })
    }

    /// The tooltip of the button named `label` (on the rail, or the one
    /// that folds the sidebar), open beside it, when a picture of the
    /// pointer resting there asks for it.
    #[cfg(test)]
    fn revealed_tooltip(
        &self,
        label: &'static str,
        action: Option<&dyn Action>,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        (self.reveal_tooltip == Some(label)).then(|| {
            let tooltip = cx.new(|_| {
                Tooltip::new(label)
                    .when_some(action, |tooltip, action| tooltip.action(action, None))
            });
            // Over the panes beside the rail, as the real one floats.
            deferred(
                div()
                    .absolute()
                    .top(px(-6.))
                    .left(px(RAIL_BUTTON - 4.))
                    .child(AnyView::from(tooltip)),
            )
            .with_priority(1)
            .into_any_element()
        })
    }

    #[cfg(not(test))]
    fn revealed_tooltip(
        &self,
        _: &'static str,
        _: Option<&dyn Action>,
        _: &mut Context<Self>,
    ) -> Option<AnyElement> {
        None
    }

    /// A rail button that shows a place in the note list.
    // no-prop-test: drawing; the rail snapshots show it.
    fn rail_place(
        &self,
        id: &'static str,
        icon: impl Into<Icon>,
        label: &'static str,
        filter: Filter,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let active = !self.settings_open && self.library.read(cx).filter == filter;
        let library = self.library.clone();
        self.rail_button(id, icon, label, None, active, cx)
            .on_click(move |_, _, cx| {
                library.update(cx, |library, cx| library.set_filter(filter.clone(), cx));
            })
    }

    /// The sidebar collapsed to icons. The top band keeps the traffic
    /// lights' room; the bottom band holds settings, with sync just above.
    // no-prop-test: drawing; the rail snapshots show it in both themes.
    fn render_rail(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let filter = self.library.read(cx).filter.clone();
        let ServiceStates {
            api,
            api_tip,
            sync,
            account,
            sync_tip,
        } = Self::service_states(cx);
        let (tone, _) = sync_line(&sync, &account, |at| Clock::edited_label(cx, at));
        let opens_account = !matches!(
            account,
            AccountState::SignedIn { .. } | AccountState::Busy(_)
        );
        let sync_color = tone_color(tone, colors);
        let api_warning = matches!(api, ApiState::PortInUse(_) | ApiState::Failed(_));
        let api_glyph = api_icon(&api);
        let trash_view = cx.entity();
        let (shared_shown, shared_unread) = shared_state(self.library.read(cx).shared, cx);

        // Under the traffic lights, near where the full sidebar's own
        // button was: the way back to the full sidebar.
        let expand = self
            .rail_button(
                "rail-expand",
                Lucide::PanelLeft,
                TOGGLE_LABEL,
                Some(Box::new(ToggleSidebar)),
                false,
                cx,
            )
            .on_click(|_, window, cx| window.dispatch_action(Box::new(ToggleSidebar), cx));
        let compose = self
            .rail_button(
                "rail-compose",
                Lucide::SquarePen,
                "New note",
                Some(Box::new(NewNote)),
                false,
                cx,
            )
            .on_click(cx.listener(|_, _, _, cx| cx.emit(SidebarEvent::NewNote)));
        let notebooks = self
            .rail_button(
                "rail-notebooks",
                library_icon(Slot::Notebooks),
                "Notebooks",
                None,
                !self.settings_open && matches!(filter, Filter::Book(_)),
                cx,
            )
            .on_click(cx.listener(|_, _, _, cx| cx.emit(SidebarEvent::GoToNotebook)));
        let tags = self
            .rail_button(
                "rail-tags",
                library_icon(Slot::Tags),
                "Tags",
                None,
                !self.settings_open && matches!(filter, Filter::Tag(_)),
                cx,
            )
            .on_click(cx.listener(|_, _, _, cx| cx.emit(SidebarEvent::GoToTag)));
        let trash = self
            .rail_place(
                "rail-trash",
                library_icon(Slot::Trash),
                "Trash",
                Filter::Trash,
                cx,
            )
            .context_menu(move |menu, _, _| {
                let view = trash_view.clone();
                menu.item(
                    PopupMenuItem::new("Empty Trash…").on_click(move |_, _, cx| {
                        view.update(cx, |_, cx| cx.emit(SidebarEvent::EmptyTrash));
                    }),
                )
            });
        // The shape says how sync is, not only the color: no state is told
        // apart by hue alone. The tooltip starts with the sync line's words.
        let sync_icon = div()
            .id("rail-sync")
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(px(RAIL_BUTTON))
            .when(opens_account, |el| {
                el.cursor_pointer().on_click(|_, window, cx| {
                    window.dispatch_action(Box::new(OpenAccountSettings), cx)
                })
            })
            .child(
                Icon::new(tone_icon(tone))
                    .size(px(15.))
                    .text_color(sync_color),
            )
            .tooltip(move |window, cx| Tooltip::new(sync_tip.clone()).build(window, cx));
        let api_icon = api_warning.then(|| {
            div()
                .id("rail-api")
                .flex()
                .flex_none()
                .items_center()
                .justify_center()
                .size(px(RAIL_BUTTON))
                .child(
                    Icon::new(api_glyph)
                        .size(px(15.))
                        .text_color(colors.warning),
                )
                .tooltip(move |window, cx| Tooltip::new(api_tip.clone()).build(window, cx))
        });
        let settings = self
            .rail_button(
                "rail-settings",
                IconName::Settings,
                "Settings",
                Some(Box::new(OpenSettings)),
                self.settings_open,
                cx,
            )
            .on_click(|_, window, cx| window.dispatch_action(Box::new(OpenSettings), cx));

        v_flex()
            .size_full()
            .items_center()
            .bg(colors.sidebar)
            .text_color(colors.text)
            // The traffic lights' row, empty so the window drags from it.
            .child(div().flex_none().h(px(TOP_BAND_HEIGHT)))
            .child(expand)
            .child(compose)
            .child(
                v_flex()
                    .id("rail")
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .items_center()
                    .gap(px(2.))
                    .pt(px(12.))
                    .overflow_y_scroll()
                    .child(self.rail_place(
                        "rail-all",
                        library_icon(Slot::AllNotes),
                        "All Notes",
                        Filter::All,
                        cx,
                    ))
                    .child(self.rail_place(
                        "rail-pinned",
                        library_icon(Slot::Pinned),
                        "Pinned",
                        Filter::Pinned,
                        cx,
                    ))
                    .child(self.rail_place(
                        "rail-recent",
                        library_icon(Slot::Recent),
                        "Recent",
                        Filter::Recent,
                        cx,
                    ))
                    .child(self.rail_place(
                        "rail-tasks",
                        library_icon(Slot::Tasks),
                        "Tasks",
                        Filter::Tasks,
                        cx,
                    ))
                    .when(shared_shown, |el| {
                        el.child(
                            self.rail_place(
                                "rail-shared",
                                library_icon(Slot::Shared),
                                "Shared with me",
                                Filter::Shared,
                                cx,
                            )
                            .relative()
                            .when(shared_unread > 0, |el| {
                                // A dot for unread mentions.
                                el.child(
                                    div()
                                        .absolute()
                                        .top(px(5.))
                                        .right(px(5.))
                                        .size(px(7.))
                                        .rounded_full()
                                        .bg(colors.accent),
                                )
                            }),
                        )
                    })
                    .child(
                        div()
                            .flex_none()
                            .w(px(20.))
                            .h(px(1.))
                            .my(px(8.))
                            .bg(colors.divider),
                    )
                    .child(notebooks)
                    .child(tags)
                    .child(trash),
            )
            .children(crate::updates::render_rail_button(cx))
            .children(api_icon)
            .child(sync_icon)
            .child(
                h_flex()
                    .flex_none()
                    .w_full()
                    .h(px(BOTTOM_BAND_HEIGHT))
                    .justify_center()
                    .border_t_1()
                    .border_color(colors.divider)
                    .child(settings),
            )
    }
}

#[cfg(test)]
impl Sidebar {
    pub fn is_collapsed(&self) -> bool {
        self.collapsed
    }

    pub fn is_status_open(&self, cx: &App) -> bool {
        self.status_open(cx)
    }

    /// Every tag, as after clicking "…", scrolled into view below a long
    /// notebook tree.
    pub fn show_all_tags(&mut self, cx: &mut Context<Self>) {
        self.all_tags = true;
        self.scroll.scroll_to_bottom();
        cx.notify();
    }

    /// Opens a level of the tag tree, as clicking its arrow does.
    #[cfg(test)]
    pub(crate) fn open_tag_level(&mut self, path: &str, cx: &mut Context<Self>) {
        self.open_tags.insert(path.to_lowercase());
        cx.notify();
    }
}

impl Render for Sidebar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.collapsed {
            return self.render_rail(cx).into_any_element();
        }
        let books = self.book_rows(cx);
        let library = self.library.read(cx);
        let counts = library.counts.clone();
        let recent = library.recent;

        let mut list = v_flex().pb_4();
        let icon = self.icon(library_icon(Slot::AllNotes), cx);
        list = list.child(self.row(
            "all",
            Filter::All,
            Entry {
                icon: Some(icon),
                label: "All Notes".into(),
                count: counts.all,
                depth: 0,
                color: None,
            },
            cx,
        ));
        let icon = self.icon(library_icon(Slot::Pinned), cx);
        list = list.child(self.row(
            "pinned",
            Filter::Pinned,
            Entry {
                icon: Some(icon),
                label: "Pinned".into(),
                count: counts.pinned,
                depth: 0,
                color: None,
            },
            cx,
        ));
        let icon = self.icon(library_icon(Slot::Recent), cx);
        list = list.child(self.row(
            "recent",
            Filter::Recent,
            Entry {
                icon: Some(icon),
                label: "Recent".into(),
                count: recent,
                depth: 0,
                color: None,
            },
            cx,
        ));
        list = list.child(self.tasks_row(cx));
        list = list.children(self.shared_row(cx));

        let add_book = self.add_button(
            "add-book",
            "New notebook",
            || SidebarEvent::NewBook { parent: None },
            cx,
        );
        list = list.child(self.heading("Notebooks", Some(add_book), cx));
        for BookRow {
            depth,
            id,
            name,
            count,
            icon,
            color,
            parent,
            open,
            locked,
            own_lock,
        } in books
        {
            let view = cx.entity();
            let book_id = id.clone();
            let fold_id = id.clone();
            let chevron = self.fold_arrow(
                SharedString::from(format!("book-fold-{id}")),
                parent,
                open,
                move |this, cx| this.toggle_book(&fold_id, cx),
                cx,
            );
            let icon = h_flex()
                .gap(px(4.))
                .child(chevron)
                // A locked notebook shows the lock in place of its icon, in
                // the notebook's color still.
                .child(self.book_icon(
                    crate::book_icons::shown(
                        crate::book_icons::opened(icon, open),
                        locked,
                        crate::locked::LockKeys::is_unlocked(cx),
                    ),
                    color,
                    cx,
                ))
                .into_any_element();
            let row = self.row(
                SharedString::from(format!("book-{id}")),
                Filter::Book(id.clone()),
                Entry {
                    icon: Some(icon),
                    label: name.into(),
                    count,
                    depth,
                    color,
                },
                cx,
            );
            let selector = format!("book-{id}");
            let row = self
                .drop_notes(row, NoteChange::Move(id.clone()), cx)
                .debug_selector(move || selector)
                .context_menu(move |menu, _, _| {
                    let (new, rename, delete) = (view.clone(), view.clone(), view.clone());
                    let (a, b, c) = (book_id.clone(), book_id.clone(), book_id.clone());
                    let (reicon, f) = (view.clone(), book_id.clone());
                    let (recolor, g) = (view.clone(), book_id.clone());
                    let (lock, d) = (view.clone(), book_id.clone());
                    // A notebook inside a locked one is locked with it, and
                    // has no lock of its own to take off.
                    let lock_item = if own_lock {
                        Some(PopupMenuItem::new("Remove Notebook Lock…").on_click(
                            move |_, _, cx| {
                                lock.update(cx, |_, cx| {
                                    cx.emit(SidebarEvent::RemoveBookLock(d.clone()))
                                });
                            },
                        ))
                    } else if !locked {
                        Some(
                            PopupMenuItem::new("Lock Notebook…").on_click(move |_, _, cx| {
                                lock.update(cx, |_, cx| cx.emit(SidebarEvent::LockBook(d.clone())));
                            }),
                        )
                    } else {
                        None
                    };
                    let menu = match lock_item {
                        Some(item) => menu.item(item).separator(),
                        None => menu,
                    };
                    let (publish, e) = (view.clone(), book_id.clone());
                    menu.item(
                        PopupMenuItem::new("New Sub-notebook").on_click(move |_, _, cx| {
                            new.update(cx, |_, cx| {
                                cx.emit(SidebarEvent::NewBook {
                                    parent: Some(a.clone()),
                                })
                            });
                        }),
                    )
                    .item(PopupMenuItem::new("Rename…").on_click(move |_, _, cx| {
                        rename.update(cx, |_, cx| cx.emit(SidebarEvent::RenameBook(b.clone())));
                    }))
                    .item(
                        PopupMenuItem::new("Change Icon…").on_click(move |_, _, cx| {
                            reicon.update(cx, |_, cx| {
                                cx.emit(SidebarEvent::ChangeBookIcon(f.clone()))
                            });
                        }),
                    )
                    .item(
                        PopupMenuItem::new("Change Color…").on_click(move |_, _, cx| {
                            recolor.update(cx, |_, cx| {
                                cx.emit(SidebarEvent::ChangeBookColor(g.clone()))
                            });
                        }),
                    )
                    .item(
                        PopupMenuItem::new("Publish Notebook…").on_click(move |_, _, cx| {
                            publish
                                .update(cx, |_, cx| cx.emit(SidebarEvent::PublishBook(e.clone())));
                        }),
                    )
                    .separator()
                    .item(
                        PopupMenuItem::new("Delete Notebook…").on_click(move |_, _, cx| {
                            delete.update(cx, |_, cx| cx.emit(SidebarEvent::DeleteBook(c.clone())));
                        }),
                    )
                });
            list = list.child(row);
        }

        let add_tag = self.add_button("add-tag", "New tag", || SidebarEvent::NewTag, cx);
        list = list
            .child(self.heading("Tags", Some(add_tag), cx))
            .child(self.render_tags(cx))
            .child(self.render_status(cx))
            .child(div().h(px(10.)));

        let trash_view = cx.entity();
        let trash = self.row(
            "trash",
            Filter::Trash,
            Entry {
                icon: None,
                label: "Trash".into(),
                count: counts.trash,
                depth: 0,
                color: None,
            },
            cx,
        );
        let trash_change = NoteChange::Move(kanso_core::model::TRASH_BOOK_ID.into());
        list = list.child(self.drop_notes(trash, trash_change, cx).context_menu(
            move |menu, _, _| {
                let view = trash_view.clone();
                menu.item(
                    PopupMenuItem::new("Empty Trash…").on_click(move |_, _, cx| {
                        view.update(cx, |_, cx| cx.emit(SidebarEvent::EmptyTrash));
                    }),
                )
            },
        ));

        let colors = Colors::get(cx);
        v_flex()
            .size_full()
            .bg(colors.sidebar)
            .text_color(colors.text)
            .child(self.render_top_band(cx))
            .child(self.render_brand(cx))
            .child(div().flex_none().h(px(6.)))
            .child(
                div()
                    .id("sidebar")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .child(list),
            )
            // An update to restart into, or one downloading: by the account
            // and sync line, calm until it needs a click.
            .children(crate::updates::render_pill(cx))
            .child(self.render_footer(cx))
            .into_any_element()
    }
}

/// Whether the Status group is open: as the user left it, or, until they
/// choose, while a status is the filter.
fn status_group_open(choice: Option<bool>, status_filter: bool) -> bool {
    choice.unwrap_or(status_filter)
}

/// The user's choice about the Status group once the filter changes:
/// picking a status from elsewhere opens a group they had closed; other
/// changes keep what they chose.
fn status_choice_after(choice: Option<bool>, was_status: bool, is_status: bool) -> Option<bool> {
    if is_status && !was_status && choice == Some(false) {
        None
    } else {
        choice
    }
}

pub use kanso_engine::sync::Tone;

/// The rail's sync icon for each tone: its shape alone says which.
// no-prop-test: a finite domain; `tone_icons_differ` checks every pair.
pub(crate) fn tone_icon(tone: Tone) -> Lucide {
    match tone {
        Tone::Good => Lucide::CloudCheck,
        Tone::Busy => Lucide::RefreshCcw,
        Tone::Quiet => Lucide::CloudOff,
        Tone::Warning => Lucide::TriangleAlert,
    }
}

/// The local API's icon, in the footer and the rail: its shape alone says
/// whether the API is listening, off, or failing to listen.
// no-prop-test: a finite domain; `api_icons_differ` checks every pair.
pub(crate) fn api_icon(api: &ApiState) -> Lucide {
    match api {
        ApiState::Running(_) => Lucide::Plug,
        ApiState::Off => Lucide::Unplug,
        ApiState::PortInUse(_) | ApiState::Failed(_) => Lucide::PlugZap,
    }
}

pub(crate) fn tone_color(tone: Tone, colors: &Colors) -> Rgb {
    match tone {
        Tone::Good => colors.success,
        Tone::Busy => colors.accent,
        Tone::Quiet => colors.text_muted,
        Tone::Warning => colors.warning,
    }
}

pub use kanso_engine::sync::words::{SIGN_IN_TO_SYNC, sync_line};

/// Who the footer names: the account while signed in (or asked to sign in
/// again), else this Mac's user.
pub fn footer_name<'a>(account: &'a AccountState, user: &'a str) -> &'a str {
    match account {
        AccountState::SignedIn { username }
        | AccountState::SignedInLocked { username }
        | AccountState::SessionEnded { username, .. } => username,
        AccountState::SignedOut | AccountState::Busy(_) => user,
    }
}

/// Up to two capitals for an avatar: the first letter of each of the first
/// two words, so "ngoni mangudya" is "NM" and "modestnerd" is "M".
pub fn initials(name: &str) -> String {
    name.split(|c: char| !c.is_alphanumeric())
        .filter_map(|word| word.chars().next())
        .take(2)
        .filter_map(|c| c.to_uppercase().next())
        .collect()
}

/// Which tags show, as indices in the library's (name) order: all of them
/// when asked, or else the `limit` most used plus the selected one. Ties go
/// to the earlier name.
/// A tag's menu: rename, color, delete.
fn tag_menu(
    menu: gpui_kit::component::menu::PopupMenu,
    view: &Entity<Sidebar>,
    tag_id: &str,
    window: &mut Window,
    cx: &mut Context<gpui_kit::component::menu::PopupMenu>,
) -> gpui_kit::component::menu::PopupMenu {
    let (rename, delete, recolor_view) = (view.clone(), view.clone(), view.clone());
    let (a, c, recolor_id) = (tag_id.to_owned(), tag_id.to_owned(), tag_id.to_owned());
    menu.item(PopupMenuItem::new("Rename…").on_click(move |_, _, cx| {
        rename.update(cx, |_, cx| cx.emit(SidebarEvent::RenameTag(a.clone())));
    }))
    .submenu("Color", window, cx, move |mut menu, _, _| {
        for option in kanso_core::TagColor::ALL {
            let view = recolor_view.clone();
            let id = recolor_id.clone();
            menu = menu.item(PopupMenuItem::new(format!("{option:?}")).on_click(
                move |_, _, cx| {
                    view.update(cx, |_, cx| {
                        cx.emit(SidebarEvent::RecolorTag(id.clone(), option))
                    });
                },
            ));
        }
        menu
    })
    .separator()
    .item(PopupMenuItem::new("Delete Tag…").on_click(move |_, _, cx| {
        delete.update(cx, |_, cx| cx.emit(SidebarEvent::DeleteTag(c.clone())));
    }))
}

/// The IDs of the tags that are neither nested nor have tags nested in
/// them: they show as chips, the rest in the tree. Pure.
pub fn flat_tag_ids(tree: &[TagNode]) -> HashSet<&str> {
    tree.iter()
        .filter(|node| node.children.is_empty())
        .filter_map(|node| node.tag_id.as_deref())
        .collect()
}

/// Which notebooks the sidebar shows open: those opened with their arrows,
/// and those around the notebook showing, unless folded with their arrows
/// while it shows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BookFolds {
    /// Opened with their arrows, by ID; kept in the config.
    pub open: HashSet<String>,
    /// Notebooks around the one showing folded with their arrows, with the
    /// one showing then; they open around another one again.
    folded_around: Option<(String, HashSet<String>)>,
}

impl BookFolds {
    /// Whether notebook `id` shows open while notebook `showing` (if one
    /// is) shows inside the notebooks `around`. Pure.
    pub fn is_open(&self, id: &str, showing: Option<&str>, around: &HashSet<String>) -> bool {
        self.open.contains(id) || (around.contains(id) && !self.folded(id, showing))
    }

    fn folded(&self, id: &str, showing: Option<&str>) -> bool {
        match (&self.folded_around, showing) {
            (Some((at, folded)), Some(showing)) => at == showing && folded.contains(id),
            _ => false,
        }
    }

    /// A click on notebook `id`'s arrow: folds it if it shows open, else
    /// opens it. Pure.
    pub fn toggle(&mut self, id: &str, showing: Option<&str>, around: &HashSet<String>) {
        if !self.is_open(id, showing, around) {
            self.open.insert(id.to_owned());
            if let Some((_, folded)) = &mut self.folded_around {
                folded.remove(id);
            }
            return;
        }
        self.open.remove(id);
        if let Some(showing) = showing
            && around.contains(id)
        {
            match &mut self.folded_around {
                Some((at, folded)) if at == showing => {
                    folded.insert(id.to_owned());
                }
                _ => {
                    self.folded_around = Some((showing.to_owned(), HashSet::from([id.to_owned()])))
                }
            }
        }
    }
}

/// A notebook row as the sidebar draws it.
struct BookRow {
    depth: usize,
    id: String,
    name: String,
    count: u32,
    icon: crate::book_icons::NotebookIcon,
    /// The color its row is washed in (`0xRRGGBB`), if one was chosen.
    color: Option<u32>,
    /// Has notebooks inside it, so it has an arrow.
    parent: bool,
    open: bool,
    /// Locked, by its own lock or a notebook it is inside.
    locked: bool,
    /// Locked by a lock of its own, which can be taken off.
    own_lock: bool,
}

/// The notebook tree's rows that show, top to bottom, from the depth of
/// each notebook in the tree's order: every notebook whose parents are all
/// open, with whether it has notebooks inside it. `open` is asked by index.
/// Pure.
pub fn visible_book_rows(depths: &[usize], open: &dyn Fn(usize) -> bool) -> Vec<(usize, bool)> {
    let mut rows = Vec::new();
    // Rows deeper than this are inside a folded notebook.
    let mut folded_at: Option<usize> = None;
    for (ix, &depth) in depths.iter().enumerate() {
        if folded_at.is_some_and(|folded| depth > folded) {
            continue;
        }
        folded_at = None;
        let parent = depths.get(ix + 1).is_some_and(|&next| next > depth);
        rows.push((ix, parent));
        if parent && !open(ix) {
            folded_at = Some(depth);
        }
    }
    rows
}

/// The tree's rows top to bottom with their depth: every level whose
/// parents are all open. Pure.
pub fn visible_tag_rows<'a>(
    tree: &'a [TagNode],
    open: &dyn Fn(&str) -> bool,
) -> Vec<(usize, &'a TagNode)> {
    fn walk<'a>(
        nodes: &'a [TagNode],
        depth: usize,
        open: &dyn Fn(&str) -> bool,
        rows: &mut Vec<(usize, &'a TagNode)>,
    ) {
        for node in nodes {
            rows.push((depth, node));
            if open(&node.path) {
                walk(&node.children, depth + 1, open, rows);
            }
        }
    }
    let mut rows = Vec::new();
    walk(tree, 0, open, &mut rows);
    rows
}

pub fn visible_tags(
    counts: &[u32],
    limit: usize,
    selected: Option<usize>,
    all: bool,
) -> Vec<usize> {
    if all || counts.len() <= limit {
        return (0..counts.len()).collect();
    }
    let mut by_use: Vec<usize> = (0..counts.len()).collect();
    by_use.sort_by_key(|&ix| std::cmp::Reverse(counts[ix]));
    let mut shown: Vec<usize> = by_use.into_iter().take(limit).collect();
    if let Some(selected) = selected.filter(|&ix| ix < counts.len() && !shown.contains(&ix)) {
        shown.push(selected);
    }
    shown.sort_unstable();
    shown
}

/// Whether the sidebar lists Shared with me, and how many mentions are
/// unread: the row shows once a note is shared with this account, while
/// signed in, and whenever invitations or mentions wait.
fn shared_state(shared: u32, cx: &App) -> (bool, usize) {
    let signed_in = cx.try_global::<Services>().is_some_and(Services::signed_in);
    let (unread, invitations) = cx
        .try_global::<crate::collab_model::Collab>()
        .map_or((0, 0), |c| (c.unread(), c.invitations.len()));
    (
        shows_shared(shared, signed_in, unread + invitations),
        unread,
    )
}

/// Whether Shared with me is listed: something is shared with this
/// account, it is signed in, or something waits there. Pure.
pub(crate) fn shows_shared(shared: u32, signed_in: bool, waiting: usize) -> bool {
    shared > 0 || signed_in || waiting > 0
}

/// The unread-mentions badge: nothing for none, "99+" past 99. Pure.
pub(crate) fn mention_badge(unread: usize) -> Option<String> {
    match unread {
        0 => None,
        1..=99 => Some(unread.to_string()),
        _ => Some("99+".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        SIGN_IN_TO_SYNC, SyncState, Tone, api_icon, footer_name, initials, status_choice_after,
        status_group_open, sync_line, tone_icon, visible_tags,
    };
    use kanso_engine::sync::words::{WAITING_FOR_APPROVAL, problem_label};
    use std::collections::HashSet;

    use super::{BookFolds, flat_tag_ids, visible_book_rows, visible_tag_rows};
    use super::{mention_badge, shows_shared};
    use crate::account_words::step_label;
    use crate::grouping::utc_label;
    use crate::services::{AccountState, strategies};
    use kanso_core::tags::{TagInfo, tag_tree};
    use kanso_sync::{QuotaKind, SignOutReason, SyncProblem};

    /// Every label in the problems table, in its order.
    const SYNC_PROBLEMS: [&str; 13] = [
        "Signed out",
        "Account turned off",
        "Sync paused",
        "Server busy",
        "Syncing shortly",
        "Account full",
        "Server full",
        "Some items skipped",
        "Update Kanso",
        "Keys don't match",
        "Server needs an update",
        "Offline",
        "Couldn't sync",
    ];
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    /// The bug this replaces: while a status was the filter, a click on
    /// the header changed nothing.
    #[test]
    fn status_group_examples() {
        assert!(!status_group_open(None, false));
        assert!(status_group_open(None, true));
        // Closed by the user while Active is the filter.
        assert!(!status_group_open(Some(false), true));
        // Leaving the status keeps it closed; picking one opens it again.
        assert_eq!(status_choice_after(Some(false), true, false), Some(false));
        assert_eq!(status_choice_after(Some(false), false, true), None);
        // Opened by hand, it stays open after a status comes and goes.
        let picked = status_choice_after(Some(true), false, true);
        assert_eq!(status_choice_after(picked, true, false), Some(true));
    }

    #[test]
    fn shared_with_me_shows_with_its_mentions() {
        assert!(!shows_shared(0, false, 0));
        assert!(shows_shared(0, true, 0));
        assert!(shows_shared(2, false, 0));
        assert!(shows_shared(0, false, 1));
        assert_eq!(mention_badge(0), None);
        assert_eq!(mention_badge(3).as_deref(), Some("3"));
        assert_eq!(mention_badge(140).as_deref(), Some("99+"));
    }

    proptest! {
        #![proptest_config(config())]

        /// The badge shows exactly when a mention is unread, never wider
        /// than three characters, and reads the count while it fits.
        #[test]
        fn prop_the_mention_badge_counts_unread(unread in 0usize..100_000) {
            let badge = mention_badge(unread);
            prop_assert_eq!(badge.is_some(), unread > 0);
            if let Some(badge) = badge {
                prop_assert!(badge.len() <= 3);
                if unread <= 99 {
                    prop_assert_eq!(badge, unread.to_string());
                }
            }
        }

        /// Shared with me hides only when nothing is shared, nothing waits
        /// and the account is signed out.
        #[test]
        fn prop_shared_with_me_hides_only_when_empty(shared in 0u32..5, signed_in: bool, waiting in 0usize..5) {
            prop_assert_eq!(!shows_shared(shared, signed_in, waiting), shared == 0 && !signed_in && waiting == 0);
        }

        /// Invariants: clicking the Status header always shows or hides its
        /// rows, whatever the filter; a status picked from elsewhere opens
        /// it; and leaving a status never undoes what the user chose.
        #[test]
        fn prop_the_status_header_always_toggles(choice in prop::option::of(any::<bool>()), was: bool, is: bool) {
            let open = status_group_open(choice, is);
            prop_assert_eq!(status_group_open(Some(!open), is), !open);
            let after = status_choice_after(choice, was, is);
            if is && !was {
                prop_assert!(status_group_open(after, is));
            }
            if choice.is_some() && !is {
                prop_assert_eq!(after, choice);
            }
        }

        /// The problems table: every problem has its own label, a failed
        /// sync and a sync that finished with the problem read alike, with a
        /// warning, and signed out reads "Set up sync" whatever the last
        /// state was.
        #[test]
        fn prop_sync_problems_read_from_the_table(problem in strategies::problem(), conflicts in 0usize..3, at in 0i64..1_000_000) {
            let signed_in = AccountState::SignedIn { username: "ngoni".into() };
            let label = problem_label(&problem);
            prop_assert!(SYNC_PROBLEMS.contains(&label), "{}", label);
            let failed = sync_line(&SyncState::Failed(problem.clone()), &signed_in, utc_label(at));
            prop_assert_eq!(failed.clone(), (Tone::Warning, label.to_owned()));
            let finished = SyncState::Synced { at_ms: at, conflicts, conflicted: Vec::new(), problem: Some(problem.clone()) };
            prop_assert_eq!(sync_line(&finished, &signed_in, utc_label(at)), failed);
            for state in [SyncState::Failed(problem), finished] {
                prop_assert_eq!(
                    sync_line(&state, &AccountState::SignedOut, utc_label(at)),
                    (Tone::Quiet, SIGN_IN_TO_SYNC.to_owned())
                );
            }
        }

    }

    proptest! {
        #![proptest_config(config())]

        /// A clean sync always reads "Synced …" with a good dot, however
        /// long ago; one with conflicts warns.
        #[test]
        fn prop_synced_reads_synced(at in 0i64..1_000_000_000_000, ago in 0i64..100_000_000_000, conflicts in 0usize..3) {
            let synced = SyncState::Synced { at_ms: at, conflicts, conflicted: Vec::new(), problem: None };
            let signed_in = AccountState::SignedIn { username: "ngoni".into() };
            let (tone, label) = sync_line(&synced, &signed_in, utc_label(at + ago));
            if conflicts == 0 {
                prop_assert_eq!(tone, Tone::Good);
                prop_assert!(label.starts_with("Synced "), "{}", label);
            } else {
                prop_assert_eq!(tone, Tone::Warning);
            }
        }

        /// The footer names the account exactly while one is signed in or
        /// asked to sign in again, and this Mac's user otherwise.
        #[test]
        fn prop_the_footer_names_the_account(account in strategies::account(), user in "[a-z]{1,12}") {
            let name = footer_name(&account, &user);
            match &account {
                AccountState::SignedIn { username } | AccountState::SessionEnded { username, .. } => {
                    prop_assert_eq!(name, username.as_str())
                }
                _ => prop_assert_eq!(name, user.as_str()),
            }
        }

        /// While an account task runs, the line says its step, busy.
        #[test]
        fn prop_running_tasks_show_their_step(state in strategies::sync_state(), step in strategies::step()) {
            prop_assert_eq!(
                sync_line(&state, &AccountState::Busy(step), utc_label(0)),
                (Tone::Busy, step_label(step).to_owned())
            );
        }

    }

    proptest! {
        #![proptest_config(config())]

        /// Initials: at most two capitals, one per word from the first two
        /// words, and none only when there is no letter or digit at all.
        #[test]
        fn prop_initials_take_the_first_letters(name in "[a-zA-Zé0-9 ._-]{0,24}") {
            let got: Vec<char> = initials(&name).chars().collect();
            let words: Vec<&str> = name.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).collect();
            prop_assert_eq!(got.len(), words.len().min(2));
            for (c, word) in got.iter().zip(&words) {
                prop_assert_eq!(Some(*c), word.chars().next().and_then(|f| f.to_uppercase().next()));
            }
        }

        /// The visible tags: in name order without repeats, the selected
        /// one always among them, and no hidden tag used more than a shown
        /// one (other than the selected).
        #[test]
        fn prop_visible_tags_are_the_most_used(
            counts in prop::collection::vec(0u32..20, 0..30),
            limit in 1usize..12,
            selected in prop::option::of(0usize..30),
            all in any::<bool>(),
        ) {
            let shown = visible_tags(&counts, limit, selected, all);
            prop_assert!(shown.windows(2).all(|w| w[0] < w[1]));
            prop_assert!(shown.iter().all(|&ix| ix < counts.len()));
            if all || counts.len() <= limit {
                prop_assert_eq!(shown.len(), counts.len());
            } else {
                prop_assert!(shown.len() <= limit + 1);
                if let Some(selected) = selected.filter(|&s| s < counts.len()) {
                    prop_assert!(shown.contains(&selected));
                }
                let least_shown = shown.iter().filter(|&&ix| Some(ix) != selected).map(|&ix| counts[ix]).min();
                for ix in (0..counts.len()).filter(|ix| !shown.contains(ix)) {
                    prop_assert!(least_shown.is_none_or(|least| counts[ix] <= least));
                }
                // The most used fill the limit; the selected one may come on top.
                let extra = shown.len() - limit;
                prop_assert!(extra <= 1);
                if extra == 1 {
                    prop_assert!(selected.is_some_and(|s| shown.contains(&s)));
                }
            }
        }
    }

    #[test]
    fn sync_line_examples() {
        let minute = 60_000;
        let day = 24 * 60 * minute;
        let signed_in = AccountState::SignedIn {
            username: "ngoni".into(),
        };
        let line = |state: &SyncState, now: i64| sync_line(state, &signed_in, utc_label(now));
        let synced = |at_ms| SyncState::Synced {
            at_ms,
            conflicts: 0,
            conflicted: Vec::new(),
            problem: None,
        };
        assert_eq!(
            line(&synced(0), 10_000),
            (Tone::Good, "Synced just now".into())
        );
        assert_eq!(
            line(&synced(0), 3 * minute),
            (Tone::Good, "Synced 3 min ago".into())
        );
        // A day or more ago, the day as the note list names it, never
        // "1 d ago" or a date in another form.
        // 2026-05-28 09:45 UTC.
        let now = 1_779_961_500_000;
        assert_eq!(line(&synced(now - day), now).1, "Synced yesterday");
        assert_eq!(line(&synced(now - 4 * day), now).1, "Synced May 24");
        assert_eq!(line(&synced(now - 400 * day), now).1, "Synced Apr 23, 2025");
        assert_eq!(line(&SyncState::Off, 0), (Tone::Quiet, "Sync off".into()));
        assert_eq!(line(&SyncState::Syncing, 0).1, "Syncing…");
        let failed = |problem| line(&SyncState::Failed(problem), 0).1;
        assert_eq!(
            failed(SyncProblem::SignedOut(SignOutReason::PasswordChanged)),
            "Signed out"
        );
        assert_eq!(failed(SyncProblem::AccountDisabled), "Account turned off");
        assert_eq!(failed(SyncProblem::Paused { until: 1 }), "Sync paused");
        assert_eq!(failed(SyncProblem::ServerBusy { until: 1 }), "Server busy");
        assert_eq!(failed(SyncProblem::BackOff { until: 1 }), "Syncing shortly");
        assert_eq!(
            failed(SyncProblem::QuotaExceeded(QuotaKind::FileBytes)),
            "Account full"
        );
        assert_eq!(failed(SyncProblem::ServerFull), "Server full");
        assert_eq!(
            failed(SyncProblem::Skipped { count: 3 }),
            "Some items skipped"
        );
        assert_eq!(failed(SyncProblem::UpdateKanso), "Update Kanso");
        assert_eq!(failed(SyncProblem::WrongKey), "Keys don't match");
        assert_eq!(
            failed(SyncProblem::ServerOutdated),
            "Server needs an update"
        );
        assert_eq!(failed(SyncProblem::Offline), "Offline");
        assert_eq!(failed(SyncProblem::Other), "Couldn't sync");
        // A sync that finished with a problem says the problem, not when.
        let skipped = SyncState::Synced {
            at_ms: 0,
            conflicts: 0,
            conflicted: Vec::new(),
            problem: Some(SyncProblem::Skipped { count: 2 }),
        };
        assert_eq!(
            line(&skipped, 0),
            (Tone::Warning, "Some items skipped".into())
        );
        // Signed out, the line asks to sign in; an ended session says so.
        assert_eq!(
            sync_line(&synced(0), &AccountState::SignedOut, utc_label(0)),
            (Tone::Quiet, "Set up sync".into())
        );
        let ended = AccountState::SessionEnded {
            username: "ngoni".into(),
            reason: SignOutReason::Revoked,
        };
        assert_eq!(
            sync_line(&SyncState::Off, &ended, utc_label(0)),
            (Tone::Warning, "Signed out".into())
        );
        assert_eq!(
            sync_line(
                &SyncState::Off,
                &AccountState::Busy(kanso_sync::Step::DerivingKeys),
                utc_label(0)
            ),
            (Tone::Busy, "Securing your keys…".into())
        );
        let locked = AccountState::SignedInLocked {
            username: "ngoni".into(),
        };
        assert_eq!(
            sync_line(&SyncState::Off, &locked, utc_label(0)),
            (Tone::Quiet, WAITING_FOR_APPROVAL.into())
        );
        assert_eq!(footer_name(&locked, "modestnerd"), "ngoni");
    }

    /// No two sync states share an icon in the rail, where the icon is all
    /// there is besides its color.
    #[test]
    fn tone_icons_differ() {
        let tones = [Tone::Good, Tone::Busy, Tone::Quiet, Tone::Warning];
        let icons: std::collections::HashSet<_> = tones.into_iter().map(tone_icon).collect();
        assert_eq!(icons.len(), tones.len());
    }

    /// The local API's states don't share an icon either: off and failing
    /// once looked alike but for the color. Both ways of failing are one
    /// state to the reader, and the tooltip says which.
    #[test]
    fn api_icons_differ() {
        use crate::services::ApiState;
        let running = api_icon(&ApiState::Running("http://127.0.0.1:19840".into()));
        let off = api_icon(&ApiState::Off);
        let port = api_icon(&ApiState::PortInUse(19840));
        let failed = api_icon(&ApiState::Failed("permission denied".into()));
        assert_eq!(port, failed);
        let icons: std::collections::HashSet<_> = [running, off, failed].into_iter().collect();
        assert_eq!(icons.len(), 3);
        // Nor does failing look like a sync state beside it in the rail.
        for tone in [Tone::Good, Tone::Busy, Tone::Quiet, Tone::Warning] {
            assert_ne!(tone_icon(tone), failed);
        }
    }

    #[test]
    fn initials_examples() {
        let signed_in = AccountState::SignedIn {
            username: "ngoni".into(),
        };
        assert_eq!(footer_name(&signed_in, "modestnerd"), "ngoni");
        assert_eq!(
            footer_name(&AccountState::SignedOut, "modestnerd"),
            "modestnerd"
        );
        assert_eq!(initials("modestnerd"), "M");
        assert_eq!(initials("ngoni mangudya"), "NM");
        assert_eq!(initials("jane.q.public"), "JQ");
        assert_eq!(initials(" -- "), "");
    }

    #[test]
    fn tag_tree_rows() {
        let tags: Vec<TagInfo> = ["work", "work/acme", "work/acme/q3", "home", "read"]
            .iter()
            .enumerate()
            .map(|(ix, name)| TagInfo {
                id: format!("tag:{ix}"),
                name: (*name).into(),
            })
            .collect();
        let tree = tag_tree(&tags, &[]);
        let flat = flat_tag_ids(&tree);
        assert_eq!(flat.len(), 2);
        assert!(flat.contains("tag:3") && flat.contains("tag:4"));
        let nested: Vec<_> = tree
            .into_iter()
            .filter(|n| !n.children.is_empty())
            .collect();
        let labels = |open: &dyn Fn(&str) -> bool| {
            visible_tag_rows(&nested, open)
                .into_iter()
                .map(|(depth, node)| format!("{depth}{}", node.label))
                .collect::<Vec<_>>()
        };
        assert_eq!(labels(&|_| false), ["0work"]);
        assert_eq!(labels(&|p| p == "work"), ["0work", "1acme"]);
        assert_eq!(labels(&|_| true), ["0work", "1acme", "2q3"]);
    }

    proptest::proptest! {
        /// With every level open, each nested tag has exactly one row, and
        /// together with the chips every tag shows once.
        #[test]
        fn prop_every_tag_shows_once(names in proptest::collection::btree_set("[ab]{1,2}(/[ab]{1,2}){0,2}", 0..10)) {
            let tags: Vec<TagInfo> = names
                .iter()
                .enumerate()
                .map(|(ix, name)| TagInfo { id: format!("tag:{ix}"), name: name.clone() })
                .collect();
            let tree = tag_tree(&tags, &[]);
            let flat = flat_tag_ids(&tree);
            let nested: Vec<_> = tree.iter().filter(|n| !n.children.is_empty()).cloned().collect();
            let mut shown: Vec<String> = visible_tag_rows(&nested, &|_| true)
                .into_iter()
                .filter_map(|(_, node)| node.tag_id.clone())
                .collect();
            shown.extend(flat.iter().map(|id| (*id).to_owned()));
            shown.sort();
            let mut all: Vec<String> = tags.iter().map(|t| t.id.clone()).collect();
            all.sort();
            proptest::prop_assert_eq!(shown, all);
        }
    }

    #[test]
    fn visible_tags_examples() {
        assert_eq!(visible_tags(&[5, 1, 9, 3], 2, None, false), vec![0, 2]);
        assert_eq!(
            visible_tags(&[5, 1, 9, 3], 2, Some(1), false),
            vec![0, 1, 2]
        );
        assert_eq!(visible_tags(&[5, 1, 9, 3], 2, None, true), vec![0, 1, 2, 3]);
        assert_eq!(visible_tags(&[1, 1, 1], 2, None, false), vec![0, 1]);
    }

    /// The sidebar's notebooks as in the screenshot that asked for this:
    /// folded, only the top levels show; opening one shows its children,
    /// and a folded child keeps its own children hidden.
    #[test]
    fn notebooks_fold_like_an_accordion() {
        // My Vault › (Agents, Clients › TDi, Etapath › (Driver App, Reviews › Filters)), Notes
        let depths = [0, 1, 1, 2, 1, 2, 2, 3, 0];
        let shown = |open: &[usize]| -> Vec<(usize, bool)> {
            visible_book_rows(&depths, &|ix| open.contains(&ix))
        };
        assert_eq!(shown(&[]), vec![(0, true), (8, false)]);
        assert_eq!(
            shown(&[0]),
            vec![(0, true), (1, false), (2, true), (4, true), (8, false)]
        );
        assert_eq!(
            shown(&[0, 4]),
            vec![
                (0, true),
                (1, false),
                (2, true),
                (4, true),
                (5, false),
                (6, true),
                (8, false)
            ]
        );
        // A child opened under a folded parent stays out of sight.
        assert_eq!(shown(&[4]), shown(&[]));
        assert_eq!(visible_book_rows(&[], &|_| true), vec![]);
    }

    /// Showing Filters (inside Reviews, inside Etapath, inside My Vault):
    /// the notebooks around it open, and an arrow among them folds it, as
    /// it shows, until another notebook shows.
    #[test]
    fn a_notebook_around_the_one_showing_folds_with_its_arrow() {
        let around: HashSet<String> = ["vault", "etapath", "reviews"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        let showing = Some("filters");
        let mut folds = BookFolds::default();
        assert!(folds.is_open("etapath", showing, &around));
        folds.toggle("etapath", showing, &around);
        assert!(!folds.is_open("etapath", showing, &around));
        assert!(folds.is_open("vault", showing, &around));
        assert!(folds.open.is_empty(), "nothing opened is kept");
        // Opened again, it stays open, now by its own choice.
        folds.toggle("etapath", showing, &around);
        assert!(folds.is_open("etapath", showing, &around));
        assert!(folds.open.contains("etapath"));
        folds.toggle("etapath", showing, &around);
        assert!(!folds.is_open("etapath", showing, &around));
        // Another notebook showing inside it opens it around that one.
        assert!(folds.is_open("etapath", Some("driver-app"), &around));
        // Away from the notebooks, only those opened show open.
        assert!(!folds.is_open("etapath", None, &HashSet::new()));
    }

    /// Depths of a notebook tree in its order: each one at most one deeper
    /// than the one before it.
    fn tree_depths() -> impl Strategy<Value = Vec<usize>> {
        prop::collection::vec(any::<bool>(), 0..40).prop_map(|steps| {
            let mut depths: Vec<usize> = Vec::new();
            for (ix, deeper) in steps.into_iter().enumerate() {
                let last = depths.last().copied().unwrap_or(0);
                depths.push(match (ix, deeper) {
                    (0, _) => 0,
                    (_, true) => last + 1,
                    (_, false) => last.saturating_sub(ix % 3),
                });
            }
            depths
        })
    }

    proptest! {
        #![proptest_config(config())]

        /// Invariants: a notebook shows exactly when every notebook it is
        /// inside is open; rows keep the tree's order; a row has an arrow
        /// exactly when the next notebook is inside it; and with everything
        /// open, every notebook shows.
        #[test]
        fn prop_a_notebook_shows_when_its_parents_are_open(
            depths in tree_depths(),
            open in prop::collection::vec(any::<bool>(), 40),
        ) {
            let is_open = |ix: usize| open[ix];
            let rows = visible_book_rows(&depths, &is_open);
            let parents_open = |ix: usize| {
                // Walk back to each ancestor: the nearest shallower row.
                let mut depth = depths[ix];
                (0..ix).rev().all(|up| {
                    if depths[up] < depth {
                        depth = depths[up];
                        is_open(up)
                    } else {
                        true
                    }
                })
            };
            let expected: Vec<usize> = (0..depths.len()).filter(|&ix| parents_open(ix)).collect();
            let shown: Vec<usize> = rows.iter().map(|(ix, _)| *ix).collect();
            prop_assert_eq!(shown, expected);
            for (ix, parent) in &rows {
                prop_assert_eq!(*parent, depths.get(ix + 1).is_some_and(|&next| next > depths[*ix]));
            }
            prop_assert_eq!(visible_book_rows(&depths, &|_| true).len(), depths.len());
        }

        /// Invariants: whichever notebook shows and whatever was opened or
        /// folded before, a click on a row's arrow flips whether that row
        /// shows open, and so whether the next row, inside it, shows; no
        /// other notebook changes; and a second click puts it back.
        #[test]
        fn prop_an_arrow_flips_what_shows(
            depths in tree_depths(),
            opened in prop::collection::vec(any::<bool>(), 40),
            showing in any::<prop::sample::Index>(),
            clicks in prop::collection::vec(any::<prop::sample::Index>(), 1..6),
        ) {
            prop_assume!(!depths.is_empty());
            let id = |ix: usize| format!("book:{ix}");
            let showing_ix = showing.index(depths.len());
            // The notebooks the one showing is inside: each nearest
            // shallower one, walking back.
            let mut around = HashSet::new();
            let mut depth = depths[showing_ix];
            for up in (0..showing_ix).rev() {
                if depths[up] < depth {
                    depth = depths[up];
                    around.insert(id(up));
                }
            }
            let showing = id(showing_ix);
            let mut folds = BookFolds {
                open: (0..depths.len()).filter(|&ix| opened[ix]).map(id).collect(),
                ..BookFolds::default()
            };
            for click in clicks {
                let state = |folds: &BookFolds| -> Vec<bool> {
                    (0..depths.len())
                        .map(|ix| folds.is_open(&id(ix), Some(&showing), &around))
                        .collect()
                };
                let rows = visible_book_rows(&depths, &|ix| state(&folds)[ix]);
                let parents: Vec<usize> =
                    rows.iter().filter(|(_, parent)| *parent).map(|(ix, _)| *ix).collect();
                if parents.is_empty() {
                    break;
                }
                let target = parents[click.index(parents.len())];
                let before = state(&folds);
                let mut after_folds = folds.clone();
                after_folds.toggle(&id(target), Some(&showing), &around);
                let after = state(&after_folds);
                for ix in 0..depths.len() {
                    prop_assert_eq!(after[ix], before[ix] != (ix == target), "{}", ix);
                }
                let shown_after: Vec<usize> = visible_book_rows(&depths, &|ix| after[ix])
                    .into_iter()
                    .map(|(ix, _)| ix)
                    .collect();
                prop_assert_eq!(shown_after.contains(&(target + 1)), after[target]);
                let mut back = after_folds.clone();
                back.toggle(&id(target), Some(&showing), &around);
                prop_assert_eq!(state(&back), before);
                folds = after_folds;
            }
        }
    }
}

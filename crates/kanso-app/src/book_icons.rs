//! The icons a notebook can show: a set to choose from, kept in the notebook
//! by name ([`kanso_core::model::BOOK_ICON_FIELD`]) so it syncs, and the
//! picker that finds and chooses one. The icon theme in use draws them
//! all, and can add more of its own, kept as `<theme>/<icon>`.

use std::rc::Rc;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::{Disableable as _, Icon, Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::Book;
use kanso_core::roles::Role;
pub use kanso_engine::catalogue::book_icons::{CHOICES, chosen, matches, role, split_key, words};

use crate::icon_themes::{self, IconTheme, Slot};
use crate::theme::Colors;
use crate::zoom::px;

/// The first choice in the picker, which stands for the default: a
/// notebook keeps no icon for it.
pub const DEFAULT: Lucide = Lucide::Notebook;

/// The Lucide icon that draws each of [`CHOICES`], in the same order: the
/// choice at an index has the icon at that index. A test holds the two lists
/// together.
pub const ICONS: &[Lucide] = &[
    Lucide::Notebook,
    Lucide::NotebookPen,
    Lucide::NotebookText,
    Lucide::BookOpen,
    Lucide::BookMarked,
    Lucide::Library,
    Lucide::Folder,
    Lucide::FolderOpen,
    Lucide::FolderHeart,
    Lucide::FolderLock,
    Lucide::Archive,
    Lucide::Inbox,
    Lucide::Mail,
    Lucide::FileText,
    Lucide::Files,
    Lucide::ScrollText,
    Lucide::StickyNote,
    Lucide::Bookmark,
    Lucide::Briefcase,
    Lucide::Building2,
    Lucide::Landmark,
    Lucide::Users,
    Lucide::User,
    Lucide::Contact,
    Lucide::Handshake,
    Lucide::Bot,
    Lucide::Brain,
    Lucide::Cpu,
    Lucide::Code,
    Lucide::Terminal,
    Lucide::GitBranch,
    Lucide::Database,
    Lucide::Server,
    Lucide::Cloud,
    Lucide::Bug,
    Lucide::Wrench,
    Lucide::Hammer,
    Lucide::Rocket,
    Lucide::Lightbulb,
    Lucide::Target,
    Lucide::Flag,
    Lucide::Trophy,
    Lucide::ChartLine,
    Lucide::ChartPie,
    Lucide::Presentation,
    Lucide::ClipboardList,
    Lucide::ListChecks,
    Lucide::Calendar,
    Lucide::CalendarDays,
    Lucide::Clock,
    Lucide::AlarmClock,
    Lucide::Hourglass,
    Lucide::GraduationCap,
    Lucide::School,
    Lucide::FlaskConical,
    Lucide::Microscope,
    Lucide::Atom,
    Lucide::Calculator,
    Lucide::Languages,
    Lucide::Palette,
    Lucide::PenTool,
    Lucide::Brush,
    Lucide::Pencil,
    Lucide::Feather,
    Lucide::Image,
    Lucide::Camera,
    Lucide::Film,
    Lucide::Clapperboard,
    Lucide::Mic,
    Lucide::Music,
    Lucide::Headphones,
    Lucide::Guitar,
    Lucide::Gamepad2,
    Lucide::Dices,
    Lucide::Puzzle,
    Lucide::Heart,
    Lucide::Star,
    Lucide::Sparkles,
    Lucide::Gift,
    Lucide::PartyPopper,
    Lucide::House,
    Lucide::Bed,
    Lucide::Sofa,
    Lucide::Key,
    Lucide::Lock,
    Lucide::Shield,
    Lucide::Plane,
    Lucide::Map,
    Lucide::MapPin,
    Lucide::Compass,
    Lucide::Mountain,
    Lucide::Tent,
    Lucide::Car,
    Lucide::Bike,
    Lucide::Truck,
    Lucide::Bus,
    Lucide::TrainFront,
    Lucide::Ship,
    Lucide::Globe,
    Lucide::Wallet,
    Lucide::PiggyBank,
    Lucide::Coins,
    Lucide::CreditCard,
    Lucide::Receipt,
    Lucide::Banknote,
    Lucide::ShoppingCart,
    Lucide::ShoppingBag,
    Lucide::Store,
    Lucide::Tag,
    Lucide::Utensils,
    Lucide::ChefHat,
    Lucide::Coffee,
    Lucide::Cake,
    Lucide::Apple,
    Lucide::Dumbbell,
    Lucide::HeartPulse,
    Lucide::Stethoscope,
    Lucide::Pill,
    Lucide::Baby,
    Lucide::Dog,
    Lucide::Cat,
    Lucide::PawPrint,
    Lucide::Leaf,
    Lucide::Flower,
    Lucide::Trees,
    Lucide::Sun,
    Lucide::Moon,
    Lucide::CloudRain,
    Lucide::Snowflake,
    Lucide::Flame,
    Lucide::Zap,
    Lucide::Droplet,
    Lucide::Church,
    Lucide::HandHeart,
    Lucide::Newspaper,
    Lucide::Rss,
    Lucide::Link,
    Lucide::MessageCircle,
    Lucide::Phone,
    Lucide::Video,
    Lucide::Megaphone,
    Lucide::LockKeyhole,
    Lucide::Eye,
    Lucide::Infinity,
];

/// The Lucide icon that draws the choice named `name`, if it is one.
pub fn icon_of(name: &str) -> Option<Lucide> {
    CHOICES
        .iter()
        .position(|choice| choice.name == name)
        .map(|ix| ICONS[ix])
}

/// What a notebook shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotebookIcon {
    /// One of the built-in choices, which the icon theme may draw.
    Choice(Lucide),
    /// The library's own: the default notebook, open or locked, or a
    /// notebook with a job, such as the Inbox.
    Slot(Slot),
    /// An icon the theme in use adds, by the path it is drawn under.
    Theme(SharedString),
}

impl NotebookIcon {
    /// The path it is drawn under (before the icon theme draws it).
    pub fn path(&self) -> SharedString {
        match self {
            NotebookIcon::Choice(icon) => gpui_kit::component::IconNamed::path(*icon),
            NotebookIcon::Slot(slot) => icon_themes::slot_path(*slot),
            NotebookIcon::Theme(path) => path.clone(),
        }
    }
}

impl From<NotebookIcon> for Icon {
    fn from(icon: NotebookIcon) -> Icon {
        Icon::default().path(icon.path())
    }
}

/// The icon a kept name shows under `theme`: its built-in choice, the
/// theme's own icon when it is the theme's, else the library's icon for a
/// notebook with `role`, or for any notebook. Pure.
pub fn icon_for(name: Option<&str>, role: Option<Role>, theme: &IconTheme) -> NotebookIcon {
    if let Some(name) = name {
        if let Some(icon) = icon_of(name) {
            return NotebookIcon::Choice(icon);
        }
        if let Some((id, icon)) = split_key(name)
            && id == theme.id
            && theme.extra(icon).is_some()
        {
            return NotebookIcon::Theme(icon_themes::extra_path(theme, icon));
        }
    }
    NotebookIcon::Slot(match role {
        Some(Role::Inbox) => Slot::Inbox,
        Some(Role::Journal) => Slot::Journal,
        Some(Role::Templates) => Slot::Templates,
        None => Slot::Notebook,
    })
}

/// The icon `book` keeps, drawn by the theme in use.
pub fn book_icon(book: &Book, cx: &App) -> NotebookIcon {
    icon_for(
        chosen(book).as_deref(),
        role(book),
        &icon_themes::active(cx),
    )
}

/// What a notebook shows for `icon`: the lock in its place while it is
/// `locked`, by its own lock or a notebook it is inside, so its own icon
/// says nothing about it then; the lock drawn open while the notes are
/// `unlocked` this session (`locked::LockKeys::is_unlocked`). Pure.
pub fn shown(icon: NotebookIcon, locked: bool, unlocked: bool) -> NotebookIcon {
    match (locked, unlocked) {
        (true, false) => NotebookIcon::Slot(Slot::NotebookLocked),
        (true, true) => NotebookIcon::Choice(Lucide::LockOpen),
        (false, _) => icon,
    }
}

/// What a notebook opened in the sidebar shows: the open notebook in place
/// of the plain one; any other icon as it is. Pure.
pub fn opened(icon: NotebookIcon, open: bool) -> NotebookIcon {
    match icon {
        NotebookIcon::Slot(Slot::Notebook) if open => NotebookIcon::Slot(Slot::NotebookOpen),
        icon => icon,
    }
}

/// One icon in the picker.
#[derive(Clone)]
struct Entry {
    /// What the notebook keeps.
    key: SharedString,
    label: SharedString,
    keywords: SharedString,
    icon: NotebookIcon,
}

/// The picker's sections: Kanso's icons, then the icon theme's own, if it
/// adds any, under its name.
fn sections(theme: &IconTheme) -> Vec<(SharedString, Vec<Entry>)> {
    let builtin = CHOICES
        .iter()
        .zip(ICONS)
        .map(|(choice, icon)| Entry {
            key: choice.name.into(),
            label: choice.label.into(),
            keywords: choice.keywords.into(),
            icon: NotebookIcon::Choice(*icon),
        })
        .collect();
    let mut sections = vec![(SharedString::from("Kanso"), builtin)];
    if !theme.extras.is_empty() {
        let entries = theme
            .extras
            .iter()
            .map(|icon| Entry {
                key: format!("{}/{}", theme.id, icon.name).into(),
                label: icon.label.clone().into(),
                keywords: format!("{} {}", icon.keywords, theme.name).into(),
                icon: NotebookIcon::Theme(icon_themes::extra_path(theme, &icon.name)),
            })
            .collect();
        sections.push((theme.name.clone().into(), entries));
    }
    sections
}

/// Columns in the picker's grid, and the rows it shows before scrolling.
const COLUMNS: usize = 8;
/// Most icons a section draws; typing finds the rest.
const SECTION_LIMIT: usize = 240;
const VISIBLE_ROWS: usize = 6;
const CELL: f32 = 36.;
const GAP: f32 = 4.;
/// The picker's width: the grid and the dialog's padding.
pub const PICKER_WIDTH: f32 = COLUMNS as f32 * (CELL + GAP) + 44.;

type PickHandler = Rc<dyn Fn(Option<String>, &mut Window, &mut App)>;

/// The picker's body: a search field over the grid of icons it finds,
/// Kanso's and the icon theme's.
pub struct IconPicker {
    search: Entity<InputState>,
    current: Option<String>,
    sections: Vec<(SharedString, Vec<Entry>)>,
    /// What the search last found, for the query it found it for.
    found: Option<(String, Vec<Found>)>,
    on_pick: PickHandler,
    scroll: ScrollHandle,
    _subscription: Subscription,
}

/// A section's icons the search found: the first [`SECTION_LIMIT`] of
/// them, and how many there are in all.
#[derive(Clone)]
struct Found {
    title: SharedString,
    entries: Vec<Entry>,
    total: usize,
}

impl IconPicker {
    fn new(
        current: Option<String>,
        sections: Vec<(SharedString, Vec<Entry>)>,
        on_pick: PickHandler,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search icons"));
        let subscription = cx.subscribe_in(
            &search,
            window,
            |this: &mut IconPicker, _, event: &InputEvent, window, cx| match event {
                InputEvent::Change => {
                    this.scroll.set_offset(point(px(0.), px(0.)));
                    cx.notify();
                }
                // Enter picks the first icon found.
                InputEvent::PressEnter { .. } => {
                    let first = this
                        .found(cx)
                        .into_iter()
                        .flat_map(|found| found.entries)
                        .next();
                    if let Some(first) = first {
                        this.pick(Some(first.key.to_string()), window, cx);
                    }
                }
                _ => {}
            },
        );
        IconPicker {
            search,
            current,
            sections,
            found: None,
            on_pick,
            scroll: ScrollHandle::new(),
            _subscription: subscription,
        }
    }

    /// The sections with the icons the search finds, leaving out the empty,
    /// worked out again only when the search changes.
    fn found(&mut self, cx: &App) -> Vec<Found> {
        let query = self.search.read(cx).value().to_string();
        if let Some((kept, found)) = &self.found
            && *kept == query
        {
            return found.clone();
        }
        let words = words(&query);
        let found: Vec<Found> = self
            .sections
            .iter()
            .filter_map(|(title, entries)| {
                let mut total = 0;
                let mut shown = Vec::new();
                for entry in entries {
                    if matches(&words, &entry.label, &entry.key, &entry.keywords) {
                        total += 1;
                        if shown.len() < SECTION_LIMIT {
                            shown.push(entry.clone());
                        }
                    }
                }
                (total > 0).then(|| Found {
                    title: title.clone(),
                    entries: shown,
                    total,
                })
            })
            .collect();
        self.found = Some((query, found.clone()));
        found
    }

    /// Closes the picker and hands on `key`; Kanso's first icon is the
    /// default, so nothing is kept for it.
    fn pick(&self, key: Option<String>, window: &mut Window, cx: &mut App) {
        let key = key.filter(|key| key != CHOICES[0].name);
        window.close_dialog(cx);
        (self.on_pick)(key, window, cx);
    }

    /// Types `query` into the search, as a person would (tests, snapshots).
    #[cfg(test)]
    pub fn search_for(&mut self, query: &str, window: &mut Window, cx: &mut Context<Self>) {
        let query = query.to_owned();
        self.search
            .update(cx, |search, cx| search.set_value(query, window, cx));
        cx.notify();
    }

    /// The keys of the icons showing, in order.
    #[cfg(test)]
    pub fn shown(&mut self, cx: &App) -> Vec<String> {
        self.found(cx)
            .into_iter()
            .flat_map(|found| found.entries)
            .map(|entry| entry.key.to_string())
            .collect()
    }
}

// no-prop-test: drawing; `matches` has the properties and the snapshots
// show the picker.
impl Render for IconPicker {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let found = self.found(cx);
        let current: SharedString = self
            .current
            .clone()
            .unwrap_or_else(|| CHOICES[0].name.to_owned())
            .into();
        // With only Kanso's icons, no title is needed over them.
        let titled = self.sections.len() > 1;
        let mut blocks: Vec<AnyElement> = Vec::new();
        let mut ix: usize = 0;
        for Found {
            title,
            entries,
            total,
        } in &found
        {
            if titled {
                blocks.push(
                    div()
                        .pt(px(4.))
                        .text_size(px(12.))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors.text_muted)
                        .child(title.clone())
                        .into_any_element(),
                );
            }
            let mut cells: Vec<_> = entries
                .iter()
                .map(|entry| {
                    ix += 1;
                    let selected = entry.key == current;
                    let key = entry.key.clone();
                    let selector = key.clone();
                    let label = entry.label.clone();
                    div()
                        .id(("book-icon", ix))
                        .debug_selector(move || format!("book-icon-{selector}"))
                        .size(px(CELL))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(8.))
                        .cursor_pointer()
                        .when(selected, |el| {
                            el.bg(colors.sidebar_active)
                                .border_1()
                                .border_color(colors.accent)
                        })
                        .when(!selected, |el| el.hover(|el| el.bg(colors.sidebar_hover)))
                        .child(Icon::from(entry.icon.clone()).size(px(18.)).text_color(
                            if selected {
                                colors.text
                            } else {
                                colors.text_secondary
                            },
                        ))
                        .tooltip(move |window, cx| {
                            gpui_kit::component::tooltip::Tooltip::new(label.clone())
                                .build(window, cx)
                        })
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.pick(Some(key.to_string()), window, cx)
                        }))
                })
                .collect();
            while !cells.is_empty() {
                let rest = cells.split_off(cells.len().min(COLUMNS));
                blocks.push(
                    h_flex()
                        .gap(px(GAP))
                        .children(std::mem::replace(&mut cells, rest))
                        .into_any_element(),
                );
            }
            if *total > entries.len() {
                blocks.push(
                    div()
                        .py(px(4.))
                        .text_size(px(12.))
                        .text_color(colors.text_muted)
                        .child(format!(
                            "{} of {} icons. Type to find the others.",
                            entries.len(),
                            total
                        ))
                        .into_any_element(),
                );
            }
        }
        let height = VISIBLE_ROWS as f32 * (CELL + GAP);
        v_flex()
            .gap(px(10.))
            .child(Input::new(&self.search).small())
            .child(
                div()
                    .id("book-icon-grid")
                    .h(px(height))
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .child(v_flex().gap(px(GAP)).children(blocks))
                    .when(found.is_empty(), |el| {
                        el.child(
                            div()
                                .size_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_size(px(13.))
                                .text_color(colors.text_muted)
                                .child("No icons match. Try another word."),
                        )
                    }),
            )
    }
}

/// Opens the picker for notebook `name`, whose icon is `current` (as kept),
/// with the keyboard in its search. `on_pick` gets the chosen icon as the
/// notebook keeps it, or `None` for the default, and the picker closes.
/// Returns the picker.
pub fn open_picker(
    name: &str,
    current: Option<String>,
    window: &mut Window,
    cx: &mut App,
    on_pick: impl Fn(Option<String>, &mut Window, &mut App) + 'static,
) -> Entity<IconPicker> {
    let title = SharedString::from(format!("Icon for \u{201c}{name}\u{201d}"));
    let on_pick: PickHandler = Rc::new(on_pick);
    let sections = sections(&icon_themes::active(cx));
    let has_icon = current.is_some();
    let picker = cx.new(|cx| IconPicker::new(current, sections, on_pick.clone(), window, cx));
    let search = picker.read(cx).search.clone();
    let first_build = std::cell::Cell::new(true);
    let view = picker.clone();
    window.open_alert_dialog(cx, move |alert, window, cx| {
        if first_build.replace(false) {
            search.update(cx, |search, cx| search.focus(window, cx));
        }
        let reset = on_pick.clone();
        alert
            .title(title.clone())
            .width(px(PICKER_WIDTH))
            .child(view.clone())
            .footer(
                h_flex()
                    .w_full()
                    .gap_2()
                    .justify_end()
                    .child(
                        Button::new("book-icon-default")
                            .small()
                            .label("Use Default")
                            .disabled(!has_icon)
                            .on_click(move |_, window, cx| {
                                window.close_dialog(cx);
                                reset(None, window, cx);
                            }),
                    )
                    .child(
                        Button::new("book-icon-cancel")
                            .small()
                            .primary()
                            .label("Done")
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                    ),
            )
    });
    picker
}

#[cfg(test)]
mod tests {
    use gpui_kit::AssetSource as _;
    use kanso_core::Book;
    use serde_json::json;

    use kanso_engine::catalogue::book_icons::{DEFAULT_NAME, matching};

    use super::{
        CHOICES, DEFAULT, ICONS, NotebookIcon, chosen, icon_for, icon_of, opened, role, shown,
    };
    use crate::assets::KansoAssets;
    use crate::icon_themes::{self, Slot};

    fn book(icon: serde_json::Value) -> Book {
        serde_json::from_value(json!({"_id": "book:a", "name": "A", "kansoIcon": icon})).unwrap()
    }

    fn names(query: &str) -> Vec<&'static str> {
        matching(query)
            .into_iter()
            .map(|choice| choice.name)
            .collect()
    }

    /// The desktop draws the engine's choices with Lucide's icons, in the
    /// same order, so the two lists cannot drift apart.
    #[test]
    fn every_choice_has_its_icon_and_it_is_bundled() {
        assert_eq!(CHOICES.len(), ICONS.len());
        for (choice, icon) in CHOICES.iter().zip(ICONS) {
            let path = gpui_kit::component::IconNamed::path(*icon);
            assert!(
                matches!(KansoAssets.load(&path), Ok(Some(_))),
                "{}: {path}",
                choice.name
            );
            // The name kept is the icon's own file name.
            assert!(path.ends_with(&format!("/{}.svg", choice.name)), "{path}");
            assert_eq!(icon_of(choice.name), Some(*icon));
        }
        assert_eq!(icon_of("not-an-icon"), None);
        assert_eq!(CHOICES[0].name, DEFAULT_NAME);
        assert_eq!(ICONS[0], DEFAULT);
    }

    #[test]
    fn a_notebook_shows_its_chosen_icon_or_the_default() {
        let lucide = icon_themes::default_theme();
        let phosphor = icon_themes::builtin("phosphor");
        let shows = |book: &Book| icon_for(chosen(book).as_deref(), role(book), lucide);
        let choice = NotebookIcon::Choice;
        let notebook = NotebookIcon::Slot(Slot::Notebook);
        assert_eq!(
            shows(&book(json!("rocket"))),
            choice(gpui_kit::assets::IconName::Rocket)
        );
        assert_eq!(chosen(&book(json!("rocket"))).as_deref(), Some("rocket"));
        // Unknown names, other types and none all show the default.
        for value in [json!("not-an-icon"), json!(3), json!(null), json!("../x/y")] {
            assert_eq!(shows(&book(value.clone())), notebook, "{value}");
            assert_eq!(chosen(&book(value)), None);
        }
        let plain: Book = serde_json::from_value(json!({"_id": "book:a", "name": "A"})).unwrap();
        assert_eq!(shows(&plain), notebook);
        // Notebooks with a job have slots of their own.
        let inbox: Book =
            serde_json::from_value(json!({"_id": "book:inbox", "name": "Inbox"})).unwrap();
        assert_eq!(shows(&inbox), NotebookIcon::Slot(Slot::Inbox));
        let journal: Book = serde_json::from_value(
            json!({"_id": "book:x", "name": "Days", "kansoRole": "journal"}),
        )
        .unwrap();
        assert_eq!(shows(&journal), NotebookIcon::Slot(Slot::Journal));
        // Locked, any notebook shows the lock instead, drawn open while the
        // notes are unlocked; open, the plain one shows open and one with
        // its own icon keeps it.
        let rocket = choice(gpui_kit::assets::IconName::Rocket);
        assert_eq!(shown(rocket.clone(), false, false), rocket);
        assert_eq!(shown(rocket.clone(), false, true), rocket);
        assert_eq!(
            shown(rocket.clone(), true, false),
            NotebookIcon::Slot(Slot::NotebookLocked)
        );
        assert_eq!(
            shown(rocket.clone(), true, true),
            choice(gpui_kit::assets::IconName::LockOpen)
        );
        assert_eq!(
            opened(notebook.clone(), true),
            NotebookIcon::Slot(Slot::NotebookOpen)
        );
        assert_eq!(opened(notebook.clone(), false), notebook);
        assert_eq!(opened(rocket.clone(), true), rocket);
        // A theme's icon is kept whether or not the theme is in use, shows
        // under its theme, and the default under any other; so does an old
        // icon pack's.
        let planet = book(json!("phosphor/planet"));
        assert_eq!(chosen(&planet).as_deref(), Some("phosphor/planet"));
        assert_eq!(shows(&planet), notebook);
        assert_eq!(
            icon_for(chosen(&planet).as_deref(), None, phosphor),
            NotebookIcon::Theme(icon_themes::extra_path(phosphor, "planet"))
        );
        let gone = book(json!("phosphor/not-there"));
        assert_eq!(icon_for(chosen(&gone).as_deref(), None, phosphor), notebook);
        let old_pack = book(json!("sketch/wave"));
        assert_eq!(chosen(&old_pack).as_deref(), Some("sketch/wave"));
        assert_eq!(icon_for(Some("sketch/wave"), None, phosphor), notebook);
        // Every icon is drawn by a path the icon theme can draw over.
        assert_eq!(rocket.path().as_ref(), "icons/rocket.svg");
        assert_eq!(notebook.path().as_ref(), "icons/library/notebook.svg");
    }

    /// The theme in use adds its own icons to the picker, in a section of
    /// their own, found by their words and the theme's name.
    #[test]
    fn a_themes_icons_are_found() {
        let phosphor = icon_themes::builtin("phosphor");
        let sections = super::sections(phosphor);
        let titles: Vec<&str> = sections.iter().map(|(title, _)| title.as_ref()).collect();
        assert_eq!(titles, vec!["Kanso", "Phosphor"]);
        assert_eq!(super::sections(icon_themes::default_theme()).len(), 1);
        let found = |query: &str| -> Vec<String> {
            let query = super::words(query);
            sections
                .iter()
                .flat_map(|(_, entries)| entries)
                .filter(|e| super::matches(&query, &e.label, &e.key, &e.keywords))
                .map(|e| e.key.to_string())
                .collect()
        };
        assert_eq!(found("orbit"), vec!["phosphor/planet".to_owned()]);
        assert!(found("phosphor").contains(&"phosphor/planet".to_owned()));
    }

    #[test]
    fn search_finds_icons_by_label_name_and_keyword() {
        assert_eq!(matching("").len(), CHOICES.len());
        // A keyword: several icons say money.
        let money = names("money");
        for expected in ["wallet", "piggy-bank", "coins", "banknote"] {
            assert!(money.contains(&expected), "{money:?}");
        }
        // The start of a word, any case, every word typed.
        assert!(names("ROCK").contains(&"rocket"));
        assert_eq!(names("pets cat"), vec!["cat"]);
        // A Lucide name with its dash.
        assert!(names("piggy-bank").contains(&"piggy-bank"));
        assert!(names("zzzz").is_empty());
        // Words match at their starts, not in their middles.
        assert!(!names("ocket").contains(&"rocket"));
    }

    struct Blank;

    impl gpui_kit::Render for Blank {
        fn render(
            &mut self,
            _: &mut gpui_kit::Window,
            _: &mut gpui_kit::Context<Self>,
        ) -> impl gpui_kit::IntoElement {
            gpui_kit::div()
        }
    }

    /// In a window: typing narrows the grid, Enter picks the first icon
    /// found and closes the picker.
    #[gpui_kit::test]
    fn typing_narrows_the_grid_and_enter_picks_the_first(cx: &mut gpui_kit::TestAppContext) {
        use std::cell::RefCell;
        use std::rc::Rc;

        use gpui_kit::AppContext as _;
        use gpui_kit::component::WindowExt as _;

        let picked: Rc<RefCell<Vec<Option<String>>>> = Rc::default();
        let (window, _) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(Default::default(), cx, |_, cx| cx.new(|_| Blank)).unwrap()
        });
        cx.run_until_parked();
        let kept = picked.clone();
        let picker = window
            .update(cx, |_, window, cx| {
                super::open_picker("Finance", None, window, cx, move |name, _, _| {
                    kept.borrow_mut().push(name)
                })
            })
            .unwrap();
        cx.run_until_parked();
        assert_eq!(
            picker.update(cx, |picker, cx| picker.shown(cx)).len(),
            CHOICES.len()
        );
        window
            .update(cx, |_, window, cx| {
                picker.update(cx, |picker, cx| picker.search_for("piggy", window, cx))
            })
            .unwrap();
        cx.run_until_parked();
        assert_eq!(
            picker.update(cx, |picker, cx| picker.shown(cx)),
            vec!["piggy-bank".to_owned()]
        );
        let search = cx.read(|cx| picker.read(cx).search.clone());
        window
            .update(cx, |_, window, cx| {
                search.update(cx, |_, cx| {
                    cx.emit(gpui_kit::component::input::InputEvent::PressEnter {
                        secondary: false,
                        shift: false,
                    })
                });
                let _ = window;
            })
            .unwrap();
        cx.run_until_parked();
        assert_eq!(*picked.borrow(), vec![Some("piggy-bank".to_owned())]);
        let open = window
            .update(cx, |_, window, cx| window.has_active_dialog(cx))
            .unwrap();
        assert!(!open, "the picker closed");
    }

    /// In a window, under Phosphor: a notebook keeping one of its icons
    /// shows it in the sidebar and the palette's notebooks; the Inbox shows
    /// its slot; a locked notebook shows the lock in both. Back on Kanso's
    /// own icons, the theme's icon gives way to the default.
    #[gpui_kit::test]
    fn a_themes_icon_shows_in_the_sidebar_and_palette(cx: &mut gpui_kit::TestAppContext) {
        use std::sync::Arc;

        use gpui_kit::AppContext as _;
        use kanso_core::DocType;
        use kanso_store::{Origin, Put, Store};

        use crate::palette::Mode;

        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
        for book in [
            json!({"name": "Trips", "kansoIcon": "phosphor/planet"}),
            json!({"name": "Vault", "kansoIcon": "phosphor/planet", "kansoLock": {"v": 1}}),
            json!({"_id": "book:inbox", "name": "Inbox", "kansoRole": "inbox"}),
        ] {
            store
                .put(DocType::Book, book, Put::Upsert, Origin::Ui)
                .unwrap();
        }
        let data_dir = dir.path().to_path_buf();
        let mut config = crate::config::AppConfig::default();
        config.appearance.icon_theme = Some("phosphor".into());
        let (window, workspace) = cx.update(|cx| {
            gpui_kit::init(cx);
            cx.set_global(crate::workspace::AppState {
                store,
                data_dir,
                config,
            });
            cx.set_global(crate::services::Services::default());
            crate::workspace::bind_keys(cx);
            crate::icon_themes::init(cx);
            gpui_kit::open_window(Default::default(), cx, |window, cx| {
                cx.new(|cx| crate::workspace::Workspace::new(window, cx))
            })
            .unwrap()
        });
        cx.run_until_parked();

        let phosphor = icon_themes::builtin("phosphor").clone();
        let planet = NotebookIcon::Theme(icon_themes::extra_path(&phosphor, "planet"));
        let lock = NotebookIcon::Slot(Slot::NotebookLocked);
        let inbox = NotebookIcon::Slot(Slot::Inbox);
        let rows = |cx: &mut gpui_kit::TestAppContext| {
            cx.read(|cx| {
                workspace
                    .read(cx)
                    .sidebar
                    .read(cx)
                    .book_rows_for_test(cx)
                    .into_iter()
                    .map(|(name, icon, _)| (name, icon))
                    .collect::<Vec<_>>()
            })
        };
        assert_eq!(
            rows(cx),
            vec![
                ("Inbox".to_owned(), inbox.clone()),
                ("Trips".to_owned(), planet.clone()),
                ("Vault".to_owned(), lock.clone())
            ]
        );
        let palette = cx.read(|cx| workspace.read(cx).palette.clone());
        window
            .update(cx, |_, window, cx| {
                palette.update(cx, |palette, cx| {
                    palette.open(
                        Mode::Pick(crate::commands::Picker::GoToNotebook),
                        window,
                        cx,
                    )
                })
            })
            .unwrap();
        cx.run_until_parked();
        let listed = cx.read(|cx| palette.read(cx).item_icons());
        assert_eq!(
            listed,
            vec![
                ("Inbox".to_owned(), inbox.path()),
                ("Trips".to_owned(), planet.path()),
                ("Vault".to_owned(), lock.path())
            ]
        );
        cx.update(|cx| icon_themes::choose(None, cx));
        cx.run_until_parked();
        assert_eq!(
            rows(cx)[1],
            ("Trips".to_owned(), NotebookIcon::Slot(Slot::Notebook))
        );
    }
}

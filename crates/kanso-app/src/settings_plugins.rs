//! Settings › Plugins: the plugins installed and what each may do, updates
//! the directory offers, the directory itself, and developer mode.
//!
//! Nothing here reaches the network unasked: the directory loads when the
//! person presses Load the directory. A stored token is never shown.

use std::collections::{BTreeSet, HashMap};

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::Button;
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Disableable as _, Icon, Sizable as _, StyledExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_plugins::model::{Disabled, EditorPermission, Installed};
use kanso_plugins::registry_model::Entry;

use crate::agent_scope::{ScopeBook, scope_books};
use crate::clock::Clock;
use crate::plugins::Plugins;
use crate::plugins_model::{Directory, status_line};
use crate::settings::{Row, section, status_row};
use crate::settings_plugins_model::{
    egress_line, grant_lines, log_tail, recent_egress, status, with_books,
};
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

/// The width of the notebook picker and the remove question.
pub const SHEET_WIDTH: f32 = 440.;

/// The Plugins page.
pub struct PluginsPage {
    /// The plugin whose details show.
    expanded: Option<String>,
    /// The directory search box.
    search: Entity<InputState>,
    /// A token field for each plugin and token name, made when first shown.
    tokens: HashMap<String, Entity<InputState>>,
    _subscriptions: Vec<Subscription>,
}

impl PluginsPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search plugins"));
        let subscription = cx.subscribe_in(&search, window, |_, _, event: &InputEvent, _, cx| {
            if matches!(event, InputEvent::Change) {
                cx.notify();
            }
        });
        PluginsPage {
            expanded: None,
            search,
            tokens: HashMap::new(),
            _subscriptions: vec![subscription],
        }
    }

    /// Brings the page up to date each time it shows. The directory is not
    /// fetched here: that waits for the person to ask.
    pub fn shown(&mut self, cx: &mut Context<Self>) {
        cx.notify();
    }

    /// Opens a plugin's details, or closes them.
    #[cfg(test)]
    pub(crate) fn snapshot_expand(&mut self, id: &str) {
        self.expanded = Some(id.to_owned());
    }

    fn toggle(&mut self, id: &str, cx: &mut Context<Self>) {
        self.expanded = if self.expanded.as_deref() == Some(id) {
            None
        } else {
            Some(id.to_owned())
        };
        cx.notify();
    }

    /// The field for plugin `id`'s token `name`.
    fn token_field(
        &mut self,
        id: &str,
        name: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<InputState> {
        self.tokens
            .entry(format!("{id}\u{0}{name}"))
            .or_insert_with(|| {
                cx.new(|cx| {
                    InputState::new(window, cx)
                        .masked(true)
                        .placeholder("Paste the token")
                })
            })
            .clone()
    }
}

// MARK: Drawing helpers

/// A titled, bordered group of custom rows, drawn as `section` draws its
/// rows.
// no-prop-test: layout; the snapshots show the page.
fn group(title: &'static str, rows: Vec<AnyElement>, colors: &Colors) -> AnyElement {
    v_flex()
        .mt(px(26.))
        .child(
            div()
                .mb(px(10.))
                .text_size(px(14.))
                .line_height(px(20.))
                .font_medium()
                .text_color(colors.text_secondary)
                .child(title),
        )
        .child(
            v_flex()
                .border_1()
                .border_color(colors.divider)
                .rounded(px(8.))
                .children(rows.into_iter().enumerate().map(|(ix, row)| {
                    div()
                        .when(ix > 0, |el| el.border_t_1().border_color(colors.divider))
                        .child(row)
                })),
        )
        .into_any_element()
}

/// A small heading inside a plugin's details.
// no-prop-test: layout.
fn heading(text: &'static str, colors: &Colors) -> Div {
    div()
        .mt(px(14.))
        .mb(px(4.))
        .text_size(px(12.5))
        .line_height(px(17.))
        .font_medium()
        .text_color(colors.text)
        .child(text)
}

/// One plain line in a plugin's details.
// no-prop-test: layout.
fn line(text: impl Into<SharedString>, colors: &Colors) -> Div {
    div()
        .text_size(px(12.5))
        .line_height(px(18.))
        .text_color(colors.text_secondary)
        .child(text.into())
}

/// A quiet pill, such as "Developer".
// no-prop-test: layout.
fn pill(text: &'static str, colors: &Colors) -> Div {
    div()
        .px(px(6.))
        .rounded(px(4.))
        .bg(colors.fill)
        .text_size(px(11.5))
        .line_height(px(17.))
        .text_color(colors.text_secondary)
        .child(text)
}

/// Whether a plugin that is off for `why` can be tried again.
fn can_try_again(why: &Option<Disabled>) -> bool {
    matches!(
        why,
        Some(Disabled::Crashed | Disabled::FailedToStart | Disabled::Refused)
    )
}

/// Whether the plugin can change notes or the open text, so that asking
/// before it does is a choice that matters.
fn can_change(installed: &Installed) -> bool {
    let p = &installed.manifest.permissions;
    !p.notes.write.is_empty()
        || p.notes.create
        || p.editor.contains(&EditorPermission::SelectionReplace)
}

fn notify_error(message: String, window: &mut Window, cx: &mut App) {
    window.push_notification(Notification::error(message), cx);
}

/// Asks for a folder or a file and hands the first path to `then`.
fn pick(
    files: bool,
    prompt: &'static str,
    window: &mut Window,
    cx: &mut App,
    then: fn(std::path::PathBuf, &mut Window, &mut App),
) {
    let picked = cx.prompt_for_paths(PathPromptOptions {
        files,
        directories: !files,
        multiple: false,
        prompt: Some(prompt.into()),
    });
    window
        .spawn(cx, async move |cx| {
            let Ok(Ok(Some(paths))) = picked.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            cx.update(|window, cx| then(path, window, cx)).ok();
        })
        .detach();
}

// MARK: Render

impl Render for PluginsPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let Some(plugins) = cx.try_global::<Plugins>() else {
            return v_flex().w_full().into_any_element();
        };
        let shared = plugins.shared.clone();
        let directory = plugins.directory.clone();
        let offers = plugins.offers.clone();
        let dev_requests = plugins.dev_requests.clone();
        let installed = crate::plugins::installed(cx);
        let safe_mode = shared.safe_mode();

        let status_line_ = status(installed.len(), safe_mode);
        let mut page = v_flex().w_full().child(status_row(
            &status_line_,
            status_line_.label.clone(),
            None,
            colors,
        ));

        // Installed.
        if installed.is_empty() {
            page =
                page.child(section(
                    "Installed",
                    vec![Row::new(Lucide::Puzzle, "Nothing installed yet").detail(
                        "Find a plugin in Browse below, or add a folder in developer mode.",
                    )],
                    colors,
                ));
        } else {
            let names = book_names(cx);
            let mut rows = Vec::new();
            for (ix, plugin) in installed.iter().enumerate() {
                rows.push(self.plugin_row(ix, plugin, &names, safe_mode, window, cx));
            }
            page = page.child(group("Installed", rows, colors));
        }

        // Updates.
        if !offers.is_empty() {
            let rows = offers
                .iter()
                .enumerate()
                .map(|(ix, offer)| {
                    let have = installed
                        .iter()
                        .find(|p| p.id() == offer.entry.id)
                        .map(|p| p.manifest.version.clone());
                    let entry = offer.entry.clone();
                    let detail = match have {
                        Some(have) => format!("{have} to {}", entry.version),
                        None => format!("Version {}", entry.version),
                    };
                    Row::new(Lucide::RefreshCw, entry.name.clone())
                        .detail(detail)
                        .trailing(
                            Button::new(("plugin-update", ix))
                                .outline()
                                .small()
                                .label("Update…")
                                .on_click(move |_, window, cx| {
                                    crate::plugins::install_from_directory(
                                        entry.clone(),
                                        window,
                                        cx,
                                    )
                                }),
                        )
                })
                .collect();
            page = page.child(section("Updates", rows, colors));
        }

        // Browse.
        page = page.child(section(
            "Browse",
            self.browse_rows(&directory, &installed, cx),
            colors,
        ));

        // Developer mode.
        page = page.child(section(
            "Developer mode",
            self.developer_rows(&dev_requests),
            colors,
        ));
        page.into_any_element()
    }
}

/// Notebook names by ID, for the lines about what a plugin may read.
fn book_names(cx: &App) -> HashMap<String, String> {
    cx.global::<AppState>()
        .store
        .books()
        .map(|books| books.into_iter().map(|b| (b.id, b.name)).collect())
        .unwrap_or_default()
}

impl PluginsPage {
    /// One installed plugin: its row, and its details when it is open.
    fn plugin_row(
        &mut self,
        ix: usize,
        plugin: &Installed,
        names: &HashMap<String, String>,
        safe_mode: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = Colors::get(cx);
        let shared = cx.global::<Plugins>().shared.clone();
        let id = plugin.id().to_owned();
        let open = self.expanded.as_deref() == Some(id.as_str());
        let running = shared.is_running(&id);
        let on = shared.may_run(&id);
        let state_line = status_line(plugin, running, safe_mode);
        let detail = format!("{} · {state_line}", plugin.manifest.publisher.name);
        let developer = plugin.is_developer();

        let head = h_flex()
            .px(px(16.))
            .py(px(9.))
            .gap(px(14.))
            .child(
                h_flex()
                    .id(("plugin-row", ix))
                    .flex_1()
                    .min_w_0()
                    .gap(px(14.))
                    .cursor_pointer()
                    .on_click({
                        let id = id.clone();
                        cx.listener(move |this, _, _, cx| this.toggle(&id, cx))
                    })
                    .child(
                        Icon::new(Lucide::Puzzle)
                            .size(px(18.))
                            .flex_none()
                            .text_color(colors.text_secondary),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap(px(1.))
                            .child(
                                h_flex()
                                    .gap(px(8.))
                                    .child(
                                        div()
                                            .text_size(px(14.))
                                            .line_height(px(20.))
                                            .text_color(colors.text)
                                            .child(format!(
                                                "{} {}",
                                                plugin.manifest.name, plugin.manifest.version
                                            )),
                                    )
                                    .when(developer, |el| el.child(pill("Developer", colors))),
                            )
                            .child(
                                div()
                                    .text_size(px(12.5))
                                    .line_height(px(17.))
                                    .text_color(colors.text_secondary)
                                    .child(detail),
                            ),
                    )
                    .child(
                        Icon::new(if open {
                            Lucide::ChevronDown
                        } else {
                            Lucide::ChevronRight
                        })
                        .size(px(16.))
                        .flex_none()
                        .text_color(colors.text_muted),
                    ),
            )
            .child(
                div()
                    .flex_none()
                    .debug_selector(move || format!("plugin-on-{ix}"))
                    .child(Switch::new(("plugin-on", ix)).checked(on).on_click({
                        let shared = shared.clone();
                        let id = id.clone();
                        cx.listener(move |_, on: &bool, _, cx| {
                            shared.set_enabled(&id, *on);
                            cx.notify();
                        })
                    })),
            );

        v_flex()
            .child(head)
            .when(open, |el| {
                el.child(self.details(ix, plugin, names, window, cx))
            })
            .into_any_element()
    }

    /// What an open plugin shows: its permissions, tokens, network use,
    /// log and buttons.
    fn details(
        &mut self,
        ix: usize,
        plugin: &Installed,
        names: &HashMap<String, String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = Colors::get(cx);
        let shared = cx.global::<Plugins>().shared.clone();
        let id = plugin.id().to_owned();
        let manifest = plugin.manifest.clone();
        let mut body = v_flex().pl(px(48.)).pr(px(16.)).pb(px(14.));
        if let Some(description) = &manifest.description {
            body = body.child(line(description.clone(), colors));
        }

        // What it may do.
        body = body.child(heading("What it may do", colors));
        let name_of = |book: &str| names.get(book).cloned();
        for text in grant_lines(&plugin.grant, &name_of) {
            body = body.child(line(format!("• {text}"), colors));
        }
        let reads = manifest.permissions.notes.read.is_some();
        let changes = can_change(plugin);
        if changes || reads {
            let mut controls = h_flex().mt(px(8.)).gap(px(14.)).items_center();
            if changes {
                controls = controls.child(
                    h_flex()
                        .gap(px(8.))
                        .items_center()
                        .child(
                            Switch::new(("plugin-ask", ix))
                                .checked(plugin.grant.ask_before_changing)
                                .on_click({
                                    let shared = shared.clone();
                                    let id = id.clone();
                                    let grant = plugin.grant.clone();
                                    cx.listener(move |_, on: &bool, window, cx| {
                                        let mut grant = grant.clone();
                                        grant.ask_before_changing = *on;
                                        if let Err(why) = shared.set_grant(&id, grant) {
                                            notify_error(why, window, cx);
                                        }
                                        cx.notify();
                                    })
                                }),
                        )
                        .child(line("Ask before changing notes", colors)),
                );
            }
            if reads {
                controls = controls.child(
                    Button::new(("plugin-books", ix))
                        .outline()
                        .small()
                        .label("Choose notebooks…")
                        .on_click({
                            let plugin = plugin.clone();
                            let page = cx.entity();
                            move |_, window, cx| {
                                open_books_sheet(plugin.clone(), page.clone(), window, cx);
                            }
                        }),
                );
            }
            body = body.child(controls);
        }

        // Tokens.
        if !manifest.permissions.secrets.is_empty() {
            body = body.child(heading("Tokens", colors));
            for (n, name) in manifest.permissions.secrets.iter().enumerate() {
                let saved = shared.has_secret(&id, name);
                let field = self.token_field(&id, name, window, cx);
                body = body.child(
                    h_flex()
                        .gap(px(10.))
                        .py(px(3.))
                        .items_center()
                        .child(
                            div()
                                .w(px(120.))
                                .text_size(px(12.5))
                                .text_color(colors.text)
                                .child(name.clone()),
                        )
                        .child(div().w(px(220.)).child(Input::new(&field)))
                        .child(
                            Button::new(("plugin-token-save", ix * 100 + n))
                                .outline()
                                .small()
                                .label("Save")
                                .on_click({
                                    let shared = shared.clone();
                                    let (id, name, field) =
                                        (id.clone(), name.clone(), field.clone());
                                    cx.listener(move |_, _, window, cx| {
                                        let value = field.read(cx).value().trim().to_owned();
                                        if value.is_empty() {
                                            return;
                                        }
                                        match shared.set_secret(&id, &name, &value) {
                                            Ok(()) => field
                                                .update(cx, |f, cx| f.set_value("", window, cx)),
                                            Err(why) => notify_error(why, window, cx),
                                        }
                                        cx.notify();
                                    })
                                }),
                        )
                        .child(line(if saved { "Saved" } else { "Not saved" }, colors)),
                );
            }
        }

        // Network.
        if !manifest.permissions.network.is_empty() {
            body = body.child(heading("Network", colors));
            let hosts: Vec<&str> = plugin.grant.network.iter().map(String::as_str).collect();
            body = body.child(line(
                if hosts.is_empty() {
                    "It may not connect to anything.".to_owned()
                } else {
                    format!("It may connect to {}.", hosts.join(", "))
                },
                colors,
            ));
            let requests = recent_egress(shared.egress(&id));
            body = body.child(
                div()
                    .mt(px(6.))
                    .text_size(px(12.))
                    .font_medium()
                    .text_color(colors.text_secondary)
                    .child("Recent requests"),
            );
            if requests.is_empty() {
                body = body.child(line("None yet.", colors));
            }
            for entry in &requests {
                let when = Clock::edited_label(cx, entry.at_ms);
                body = body.child(line(egress_line(entry, &when), colors));
            }
        }

        // Log.
        let log = shared.log(&id);
        if (plugin.is_developer() || plugin.last_error.is_some())
            && (!log.is_empty() || plugin.last_error.is_some())
        {
            body = body.child(heading("Log", colors));
            if let Some(error) = &plugin.last_error {
                body = body.child(line(format!("Last error: {error}"), colors));
            }
            let mono = crate::theme::mono_font(cx);
            if !log.is_empty() {
                body = body.child(
                    v_flex()
                        .id(("plugin-log", ix))
                        .mt(px(4.))
                        .p(px(8.))
                        .max_h(px(140.))
                        .overflow_y_scroll()
                        .rounded(px(6.))
                        .bg(colors.code)
                        .text_size(px(12.))
                        .line_height(px(17.))
                        .text_color(colors.code_text)
                        .when_some(mono, |el, font| el.font_family(font))
                        .children(log_tail(&log).into_iter().map(|l| div().child(l))),
                );
            }
        }

        // Buttons.
        let mut buttons = h_flex().mt(px(14.)).gap(px(8.));
        if can_try_again(&plugin.disabled) {
            buttons = buttons.child(
                Button::new(("plugin-retry", ix))
                    .outline()
                    .small()
                    .label("Try again")
                    .on_click({
                        let shared = shared.clone();
                        let id = id.clone();
                        cx.listener(move |_, _, _, cx| {
                            shared.set_enabled(&id, true);
                            cx.notify();
                        })
                    }),
            );
        }
        if !matches!(plugin.disabled, Some(Disabled::Blocked)) {
            buttons = buttons.child(
                Button::new(("plugin-block", ix))
                    .outline()
                    .small()
                    .label("Block on this Mac")
                    .on_click({
                        let shared = shared.clone();
                        let id = id.clone();
                        cx.listener(move |_, _, _, cx| {
                            shared.block(&id);
                            cx.notify();
                        })
                    }),
            );
        }
        buttons = buttons.child(
            Button::new(("plugin-remove", ix))
                .outline()
                .small()
                .label("Remove…")
                .on_click({
                    let page = cx.entity();
                    let (id, name) = (id.clone(), manifest.name.clone());
                    move |_, window, cx| {
                        open_remove_sheet(id.clone(), name.clone(), page.clone(), window, cx);
                    }
                }),
        );
        body.child(buttons).into_any_element()
    }

    /// The Browse section's rows for the directory's state.
    fn browse_rows(
        &self,
        directory: &Directory,
        installed: &[Installed],
        cx: &mut Context<Self>,
    ) -> Vec<Row> {
        match directory {
            Directory::NotSetUp => vec![
                Row::new(Lucide::Globe, "Plugin directory")
                    .detail("The plugin directory isn't set up in this build of Kanso. You can still add a plugin folder in developer mode."),
            ],
            Directory::Idle => vec![
                Row::new(Lucide::Globe, "Plugin directory")
                    .detail("Kanso only contacts the directory when you ask.")
                    .trailing(
                        Button::new("plugin-directory-load")
                            .outline()
                            .small()
                            .label("Load the directory")
                            .on_click(cx.listener(|_, _, _, cx| {
                                crate::plugins::refresh_directory(cx);
                                cx.notify();
                            })),
                    ),
            ],
            Directory::Loading => vec![Row::new(Lucide::Globe, "Plugin directory").detail("Loading…")],
            Directory::Failed(why) => vec![
                Row::new(Lucide::TriangleAlert, "The directory didn't load")
                    .detail(why.clone())
                    .trailing(
                        Button::new("plugin-directory-retry")
                            .outline()
                            .small()
                            .label("Try again")
                            .on_click(cx.listener(|_, _, _, cx| {
                                crate::plugins::refresh_directory(cx);
                                cx.notify();
                            })),
                    ),
            ],
            Directory::Loaded(_) => {
                let query = self.search.read(cx).value().to_string();
                let entries = directory.search(&query);
                let mut rows = vec![Row::new(Lucide::Search, "Search").trailing(
                    h_flex()
                        .gap_2()
                        .child(div().w(px(260.)).child(Input::new(&self.search)))
                        .child(
                            Button::new("plugin-directory-refresh")
                                .outline()
                                .small()
                                .icon(Lucide::RefreshCw)
                                .tooltip("Check the directory for new plugins")
                                .on_click(cx.listener(|_, _, _, cx| {
                                    crate::plugins::refresh_directory(cx);
                                    cx.notify();
                                })),
                        ),
                )];
                if entries.is_empty() {
                    rows.push(Row::new(Lucide::Puzzle, "No plugins match").detail(
                        if query.trim().is_empty() {
                            "The directory has no plugins yet."
                        } else {
                            "Try a different word."
                        },
                    ));
                }
                for (ix, entry) in entries.into_iter().enumerate() {
                    let have = installed.iter().any(|p| p.id() == entry.id);
                    rows.push(entry_row(ix, entry, have));
                }
                rows
            }
        }
    }

    /// The Developer mode section's rows.
    fn developer_rows(&self, requests: &[std::path::PathBuf]) -> Vec<Row> {
        let mut rows = vec![
            Row::new(Lucide::Code, "Run a plugin from a folder")
                .detail("Developer mode runs an unsigned folder on this Mac only; it is never updated or shared.")
                .trailing(
                    Button::new("plugin-add-folder")
                        .outline()
                        .small()
                        .label("Add folder…")
                        .on_click(|_, window, cx| {
                            pick(false, "Add", window, cx, crate::plugins::add_developer_folder)
                        }),
                ),
            Row::new(Lucide::Download, "Install from a file")
                .detail("A .kansoplugin file the plugin directory lists.")
                .trailing(
                    Button::new("plugin-open-file")
                        .outline()
                        .small()
                        .label("Install from file…")
                        .on_click(|_, window, cx| {
                            pick(true, "Open", window, cx, crate::plugins::open_file)
                        }),
                ),
        ];
        for (ix, path) in requests.iter().enumerate() {
            let shown = path.to_string_lossy().into_owned();
            let path = path.clone();
            rows.push(
                Row::new(Lucide::FolderOpen, "A folder wants to run as a plugin")
                    .value(shown)
                    .trailing(
                        Button::new(("plugin-dev-review", ix))
                            .outline()
                            .small()
                            .label("Review…")
                            .on_click(move |_, window, cx| {
                                cx.global_mut::<Plugins>()
                                    .dev_requests
                                    .retain(|p| p != &path);
                                crate::plugins::add_developer_folder(path.clone(), window, cx);
                                cx.refresh_windows();
                            }),
                    ),
            );
        }
        rows
    }
}

/// One directory entry: what it is, and Install… or "Installed".
// no-prop-test: layout.
fn entry_row(ix: usize, entry: Entry, installed: bool) -> Row {
    let detail = if entry.description.is_empty() {
        format!("{} · {}", entry.publisher, entry.version)
    } else {
        format!(
            "{} · {} · {}",
            entry.publisher, entry.version, entry.description
        )
    };
    let row = Row::new(Lucide::Puzzle, entry.name.clone()).detail(detail);
    if installed {
        row.trailing(div().text_size(px(12.5)).child("Installed"))
    } else {
        row.trailing(
            Button::new(("plugin-install", ix))
                .outline()
                .small()
                .label("Install…")
                .on_click(move |_, window, cx| {
                    crate::plugins::install_from_directory(entry.clone(), window, cx)
                }),
        )
    }
}

// MARK: Notebook picker

/// The sheet that picks the notebooks a plugin may read and change.
pub struct BooksSheet {
    books: Vec<ScopeBook>,
    /// The manifest asks to read every notebook.
    can_all: bool,
    /// The manifest asks to change notes.
    can_write: bool,
    read_all: bool,
    read: BTreeSet<String>,
    write: BTreeSet<String>,
}

impl BooksSheet {
    /// A sheet starting from what `plugin` has now.
    pub fn new(plugin: &Installed, books: Vec<ScopeBook>) -> BooksSheet {
        let asks = &plugin.manifest.permissions.notes;
        let notes = &plugin.grant.notes;
        BooksSheet {
            can_all: asks.read.as_ref().is_some_and(|r| r.all),
            can_write: !asks.write.is_empty(),
            read_all: notes.read_all,
            read: notes.read_books.clone(),
            write: notes.write_books.clone(),
            books,
        }
    }

    fn set_read(&mut self, id: &str, on: bool, cx: &mut Context<Self>) {
        if on {
            self.read.insert(id.to_owned());
        } else {
            self.read.remove(id);
            self.write.remove(id);
        }
        cx.notify();
    }

    fn set_write(&mut self, id: &str, on: bool, cx: &mut Context<Self>) {
        if on {
            self.write.insert(id.to_owned());
            self.read.insert(id.to_owned());
        } else {
            self.write.remove(id);
        }
        cx.notify();
    }
}

impl Render for BooksSheet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let all = self.read_all;
        let rows = self.books.iter().enumerate().map(|(ix, book)| {
            let (read_id, write_id) = (book.id.clone(), book.id.clone());
            h_flex()
                .pl(px(8. + 16. * book.depth as f32))
                .py(px(3.))
                .gap(px(12.))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_size(px(13.))
                        .text_color(colors.text)
                        .child(book.name.clone()),
                )
                .child(
                    Checkbox::new(ElementId::NamedInteger(
                        "plugin-book-read".into(),
                        ix as u64,
                    ))
                    .checked(all || self.read.contains(&book.id))
                    .disabled(all)
                    .label("Read")
                    .on_click(
                        cx.listener(move |this, on: &bool, _, cx| this.set_read(&read_id, *on, cx)),
                    ),
                )
                .when(self.can_write, |el| {
                    el.child(
                        Checkbox::new(ElementId::NamedInteger(
                            "plugin-book-write".into(),
                            ix as u64,
                        ))
                        .checked(self.write.contains(&book.id))
                        .label("Change")
                        .on_click(cx.listener(
                            move |this, on: &bool, _, cx| this.set_write(&write_id, *on, cx),
                        )),
                    )
                })
        });
        v_flex()
            .gap(px(6.))
            .child(
                div()
                    .text_size(px(12.5))
                    .line_height(px(18.))
                    .text_color(colors.text_secondary)
                    .child("The plugin may use notes in the notebooks you pick, and in the notebooks inside them. Locked notebooks aren't listed and never reach it, nor does the trash."),
            )
            .when(self.can_all, |el| {
                el.child(
                    Checkbox::new("plugin-books-all")
                        .checked(all)
                        .label("Every notebook that isn't locked")
                        .on_click(cx.listener(|this, on: &bool, _, cx| {
                            this.read_all = *on;
                            cx.notify();
                        })),
                )
            })
            .child(
                v_flex()
                    .id("plugin-books")
                    .max_h(px(280.))
                    .overflow_y_scroll()
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

/// Opens the sheet that picks the notebooks `plugin` may use. Done keeps
/// the grant within what the manifest asks.
pub fn open_books_sheet(
    plugin: Installed,
    page: Entity<PluginsPage>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<BooksSheet> {
    let books = scope_books(&cx.global::<AppState>().store);
    let sheet = cx.new(|_| BooksSheet::new(&plugin, books));
    let view = sheet.clone();
    let title = format!("Notebooks for {}", plugin.manifest.name);
    window.open_alert_dialog(cx, move |alert, _, _| {
        let (done, plugin, page) = (view.clone(), plugin.clone(), page.clone());
        alert
            .title(title.clone())
            .width(px(SHEET_WIDTH))
            .child(view.clone())
            .show_cancel(true)
            .ok_text("Done")
            .on_ok(move |_, window, cx| {
                let shared = cx.global::<Plugins>().shared.clone();
                let locked = cx
                    .global::<AppState>()
                    .store
                    .locked_books()
                    .unwrap_or_default();
                let sheet = done.read(cx);
                let grant = with_books(
                    &plugin.grant,
                    &plugin.manifest,
                    &sheet.read,
                    &sheet.write,
                    sheet.read_all,
                    &locked,
                );
                if let Err(why) = shared.set_grant(plugin.id(), grant) {
                    notify_error(why, window, cx);
                }
                page.update(cx, |_, cx| cx.notify());
                true
            })
    });
    sheet
}

// MARK: Remove

/// The remove question's one choice.
pub struct RemoveSheet {
    name: String,
    sweep: bool,
}

impl Render for RemoveSheet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        v_flex()
            .gap(px(10.))
            .child(
                div()
                    .text_size(px(13.))
                    .line_height(px(19.))
                    .text_color(colors.text_secondary)
                    .child(format!(
                        "{} stops, and its files and any tokens you saved for it are deleted from this Mac.",
                        self.name
                    )),
            )
            .child(
                Checkbox::new("plugin-remove-sweep")
                    .checked(self.sweep)
                    .label("Also remove its data from notes")
                    .on_click(cx.listener(|this, on: &bool, _, cx| {
                        this.sweep = *on;
                        cx.notify();
                    })),
            )
    }
}

/// Asks before removing plugin `id`.
pub fn open_remove_sheet(
    id: String,
    name: String,
    page: Entity<PluginsPage>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<RemoveSheet> {
    let title = format!("Remove {name}?");
    let sheet = cx.new(|_| RemoveSheet { name, sweep: false });
    let view = sheet.clone();
    window.open_alert_dialog(cx, move |alert, _, _| {
        let (ask, id, page) = (view.clone(), id.clone(), page.clone());
        alert
            .title(title.clone())
            .width(px(SHEET_WIDTH))
            .child(view.clone())
            .show_cancel(true)
            .ok_text("Remove")
            .on_ok(move |_, _, cx| {
                let sweep = ask.read(cx).sweep;
                crate::plugins::remove(&id, sweep, cx);
                page.update(cx, |page, cx| {
                    page.expanded = None;
                    cx.notify();
                });
                true
            })
    });
    sheet
}

#[cfg(test)]
mod tests {
    use kanso_plugins::model::Disabled;

    use super::can_try_again;

    #[test]
    fn only_a_stopped_plugin_offers_try_again() {
        assert!(can_try_again(&Some(Disabled::Crashed)));
        assert!(can_try_again(&Some(Disabled::FailedToStart)));
        assert!(can_try_again(&Some(Disabled::Refused)));
        assert!(!can_try_again(&Some(Disabled::ByYou)));
        assert!(!can_try_again(&Some(Disabled::Blocked)));
        assert!(!can_try_again(&None));
    }
}

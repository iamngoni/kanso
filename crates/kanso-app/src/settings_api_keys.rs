//! Settings › Local API › API keys (issue #243, step 7): the keys that are
//! live, New key…, the setup shown once, Revoke, and the notice for a
//! request that was refused.
//!
//! The page and its sheets use the one `KeyStore` the running server
//! checks keys against (`services::key_store`), so a revoke here stops the
//! server's very next request. The page never keeps a key: the sheet that
//! made one holds it until it closes.

use std::collections::{BTreeSet, HashMap};

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Disableable as _, Icon, Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_server::{KeyAccess, KeyId, KeyStore};
use kanso_store::ApiKeyRow;
use zeroize::Zeroizing;

use crate::agent_scope::{ScopeBook, scope_books};
use crate::api_keys_model::{
    Client, GROUP_DETAIL, REVOKE_DETAIL, SHOWN_ONCE, access_choice, access_short, books_summary,
    created_label, new_key, refused_notice, revoke_question, used_label,
};
use crate::clock::Clock;
use crate::mcp_setup::Setup;
use crate::settings::{Row, mcp_endpoint, section};
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;
use crate::{dialogs, services};

/// The width of the New key sheet.
pub const SHEET_WIDTH: f32 = 560.;

/// The API keys group of the Local API page.
pub struct ApiKeysPage {
    /// The server's own `KeyStore`; `None` before the page first shows or
    /// when the library's keys can't be read.
    keys: Option<KeyStore>,
    /// The live keys, newest first.
    rows: Vec<ApiKeyRow>,
    /// Notebook names, for the notebooks each key may read.
    book_names: HashMap<String, String>,
    /// Why the keys can't be listed or changed.
    error: Option<String>,
    /// The key the sheet last made, so a test can use it as an agent would.
    #[cfg(test)]
    pub(crate) minted_for_test: Option<String>,
    /// The last New key sheet, held weakly, so a test can see it is gone
    /// once it closes.
    #[cfg(test)]
    pub(crate) sheet_for_test: Option<WeakEntity<NewKeySheet>>,
}

impl ApiKeysPage {
    pub fn new() -> Self {
        ApiKeysPage {
            keys: None,
            rows: Vec::new(),
            book_names: HashMap::new(),
            error: None,
            #[cfg(test)]
            minted_for_test: None,
            #[cfg(test)]
            sheet_for_test: None,
        }
    }

    /// Brings the page up to date each time it shows.
    pub fn shown(&mut self, cx: &mut Context<Self>) {
        self.refresh(cx);
    }

    /// Reads the live keys and the notebook names again.
    fn refresh(&mut self, cx: &mut Context<Self>) {
        self.error = None;
        if !cx.has_global::<services::Services>() || !cx.has_global::<AppState>() {
            cx.notify();
            return;
        }
        match services::key_store(cx) {
            Ok(keys) => {
                match keys.list() {
                    Ok(rows) => {
                        self.rows = rows
                            .into_iter()
                            .filter(|row| row.revoked_at.is_none())
                            .collect();
                    }
                    Err(err) => self.error = Some(format!("Couldn't list the API keys: {err}")),
                }
                self.keys = Some(keys);
            }
            Err(reason) => self.error = Some(reason),
        }
        self.book_names = cx
            .global::<AppState>()
            .store
            .books()
            .map(|books| books.into_iter().map(|b| (b.id, b.name)).collect())
            .unwrap_or_default();
        cx.notify();
    }

    /// Revokes key `id`. It is refused from the next request on.
    pub(crate) fn revoke(&mut self, id: &KeyId, cx: &mut Context<Self>) {
        let Some(keys) = self.keys.clone() else {
            return;
        };
        if let Err(err) = keys.revoke(id) {
            self.error = Some(format!("Couldn't revoke the key: {err}"));
        }
        self.refresh(cx);
    }

    fn ask_revoke(
        &mut self,
        id: String,
        name: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let page = cx.entity();
        dialogs::confirm(
            revoke_question(&name),
            REVOKE_DETAIL,
            "Revoke",
            window,
            cx,
            move |_, cx| {
                let id = KeyId(id.clone());
                page.update(cx, |page, cx| page.revoke(&id, cx));
            },
        );
    }

    /// The question before a revoke, as the Revoke button asks it.
    #[cfg(test)]
    pub(crate) fn snapshot_ask_revoke(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let row = self.rows[index].clone();
        self.ask_revoke(row.id, row.name, window, cx);
    }

    /// The keys shown, for tests.
    #[cfg(test)]
    pub(crate) fn rows(&self) -> &[ApiKeyRow] {
        &self.rows
    }
}

// no-prop-test: drawing; the snapshots show the list empty and with keys,
// and `api_keys_model` is tested.
impl Render for ApiKeysPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let now = Clock::now_ms(cx);
        let local = |ms: i64| Clock::local_ms(cx, ms);
        let refused = refused_notice(
            self.keys.as_ref().and_then(KeyStore::last_refused),
            now,
            local,
        );
        let texts: Vec<(String, String, String)> = self
            .rows
            .iter()
            .map(|row| {
                let books =
                    books_summary(row.books.as_deref(), |id| self.book_names.get(id).cloned());
                let detail = format!(
                    "{} · {} · {} · {}",
                    Client::from_id(&row.client).name(),
                    access_short(row.access),
                    books,
                    created_label(row.created_at, now, local),
                );
                (row.id.clone(), row.name.clone(), detail)
            })
            .collect();
        let used: Vec<String> = self
            .rows
            .iter()
            .map(|row| used_label(row.last_used_at, now, local))
            .collect();

        let mut rows = Vec::new();
        if let Some(error) = self.error.clone() {
            rows.push(Row::new(Lucide::TriangleAlert, error));
        }
        if let Some(notice) = refused {
            rows.push(Row::new(Lucide::ShieldAlert, notice));
        }
        rows.push(
            Row::new(Lucide::KeyRound, "Give something its own key")
                .detail(GROUP_DETAIL)
                .trailing(
                    Button::new("new-key")
                        .debug_selector(|| "new-key".into())
                        .primary()
                        .small()
                        .label("New key…")
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.open_new_key(window, cx);
                        })),
                ),
        );
        if texts.is_empty() {
            rows.push(
                Row::new(Lucide::Info, "No keys yet")
                    .detail("Choose New key… to give an agent, script or app a key of its own."),
            );
        }
        for (ix, ((id, name, detail), used)) in texts.into_iter().zip(used).enumerate() {
            let revoke_name = name.clone();
            rows.push(
                Row::new(Lucide::Key, name)
                    .detail(detail)
                    .trailing(
                        div()
                            .text_size(px(12.5))
                            .text_color(colors.text_secondary)
                            .child(used),
                    )
                    .trailing(
                        Button::new(ElementId::NamedInteger("revoke-key".into(), ix as u64))
                            .debug_selector(move || format!("revoke-key-{ix}"))
                            .outline()
                            .small()
                            .label("Revoke")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.ask_revoke(id.clone(), revoke_name.clone(), window, cx);
                            })),
                    ),
            );
        }
        section("API keys", rows, colors)
    }
}

impl ApiKeysPage {
    fn open_new_key(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(keys) = self.keys.clone() else {
            return;
        };
        let page = cx.entity();
        let _sheet = open_new_key_sheet(page, keys, window, cx);
        #[cfg(test)]
        {
            self.sheet_for_test = Some(_sheet);
        }
    }
}

// MARK: The New key sheet

/// The setup a key just made is shown with. It lives only in the sheet, and
/// goes (wiped) when the sheet closes: nothing else keeps the key or its
/// setup.
struct Made {
    client: Client,
    /// The snippet for the client, with the key in it.
    text: Zeroizing<String>,
    copied: bool,
}

/// The New key sheet: the form, then the setup with the key, shown once.
pub struct NewKeySheet {
    page: Entity<ApiKeysPage>,
    keys: KeyStore,
    client: Client,
    name: Entity<InputState>,
    name_text: String,
    /// The person typed in the name: it no longer follows the client.
    name_edited: bool,
    access: KeyAccess,
    books: Vec<ScopeBook>,
    all_books: bool,
    ticked: BTreeSet<String>,
    endpoint: String,
    program: String,
    made: Option<Made>,
    error: Option<String>,
    _typing: Subscription,
}

impl NewKeySheet {
    fn new(
        page: Entity<ApiKeysPage>,
        keys: KeyStore,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let client = Client::Agent(crate::mcp_setup::Agent::Codex);
        let initial = client.default_name();
        let name = cx.new(|cx| InputState::new(window, cx).default_value(initial.clone()));
        let typing = cx.subscribe_in(&name, window, |this, input, event: &InputEvent, _, cx| {
            if let InputEvent::Change = event {
                this.name_text = input.read(cx).value().to_string();
                this.name_edited = this.name_text != this.client.default_name();
                cx.notify();
            }
        });
        let state = cx.global::<AppState>();
        let server = &state.config.server;
        NewKeySheet {
            page,
            keys,
            client,
            name,
            name_text: initial,
            name_edited: false,
            access: KeyAccess::Read,
            books: scope_books(&state.store),
            all_books: true,
            ticked: BTreeSet::new(),
            endpoint: mcp_endpoint(&server.bind_address, server.port),
            program: std::env::current_exe()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|_| "kanso".to_owned()),
            made: None,
            error: None,
            _typing: typing,
        }
    }

    fn set_client(&mut self, client: Client, window: &mut Window, cx: &mut Context<Self>) {
        self.client = client;
        if !self.name_edited {
            let name = client.default_name();
            self.name
                .update(cx, |input, cx| input.set_value(name, window, cx));
        }
        cx.notify();
    }

    fn toggle(&mut self, id: &str, cx: &mut Context<Self>) {
        if !self.ticked.remove(id) {
            self.ticked.insert(id.to_owned());
        }
        cx.notify();
    }

    fn ready(&self) -> bool {
        self.made.is_none()
            && new_key(
                self.client,
                &self.name_text,
                self.access,
                self.all_books,
                &self.ticked,
            )
            .is_some()
    }

    /// Makes the key and swaps the form for its setup.
    fn create(&mut self, cx: &mut Context<Self>) {
        let Some(new) = new_key(
            self.client,
            &self.name_text,
            self.access,
            self.all_books,
            &self.ticked,
        ) else {
            return;
        };
        match self.keys.mint(new) {
            Ok(minted) => {
                #[cfg(test)]
                {
                    let key = minted.key.clone();
                    self.page
                        .update(cx, |page, _| page.minted_for_test = Some(key));
                }
                let setup = Setup {
                    endpoint: self.endpoint.clone(),
                    key: Zeroizing::new(minted.key),
                    program: self.program.clone(),
                };
                let text = Zeroizing::new(match self.client {
                    Client::Agent(agent) => agent.snippet(&setup),
                    Client::Other => setup.rest_snippet(),
                });
                self.made = Some(Made {
                    client: self.client,
                    text,
                    copied: false,
                });
                self.error = None;
                self.page.update(cx, |page, cx| page.refresh(cx));
            }
            Err(err) => self.error = Some(format!("Couldn't make the key: {err}")),
        }
        cx.notify();
    }

    /// Copies the setup as a secret, which clears in a minute.
    fn copy(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(made) = &mut self.made else {
            return;
        };
        if crate::clipboard::copy_secret(&made.text, cx) {
            made.copied = true;
            window.push_notification(Notification::info(made.client.copied_message()), cx);
        } else {
            window.push_notification(Notification::error("Couldn't copy the setup."), cx);
        }
        cx.notify();
    }

    /// A sheet that already shows `text` for `client`, for the snapshots.
    #[cfg(test)]
    pub(crate) fn show_made_for_test(&mut self, client: Client, text: String) {
        self.client = client;
        self.made = Some(Made {
            client,
            text: Zeroizing::new(text),
            copied: false,
        });
    }
}

fn label(text: &'static str, colors: &Colors) -> Div {
    div()
        .text_size(px(12.5))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(colors.text_secondary)
        .child(text)
}

// no-prop-test: drawing; the api_keys snapshots show each state.
impl Render for NewKeySheet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        if let Some(made) = &self.made {
            return self.render_made(made, colors, cx).into_any_element();
        }
        // Weak: the menu outlives this frame, and must not keep the sheet
        // (and the key in it) alive after it closes.
        let view = cx.weak_entity();
        let picker = Button::new("key-client")
            .debug_selector(|| "key-client".into())
            .outline()
            .small()
            .label(self.client.name())
            .dropdown_caret(true)
            .dropdown_menu({
                let current = self.client;
                move |mut menu, _, _| {
                    for option in Client::all() {
                        let view = view.clone();
                        menu = menu.item(
                            PopupMenuItem::new(option.name())
                                .checked(option == current)
                                .on_click(move |_, window, cx| {
                                    view.update(cx, |this, cx| this.set_client(option, window, cx))
                                        .ok();
                                }),
                        );
                    }
                    menu
                }
            });
        let access = KeyAccess::ALL.into_iter().enumerate().map(|(ix, access)| {
            let (title, detail) = access_choice(access);
            let selected = self.access == access;
            h_flex()
                .id(ElementId::NamedInteger("key-access".into(), ix as u64))
                .gap(px(8.))
                .py(px(4.))
                .items_start()
                .cursor_pointer()
                .child(
                    div().mt(px(2.)).flex_none().child(
                        Icon::new(if selected {
                            Lucide::CircleDot
                        } else {
                            Lucide::Circle
                        })
                        .size(px(15.))
                        .text_color(if selected {
                            Hsla::from(colors.accent)
                        } else {
                            Hsla::from(colors.text_muted)
                        }),
                    ),
                )
                .child(
                    v_flex()
                        .child(
                            div()
                                .text_size(px(13.))
                                .text_color(colors.text)
                                .child(title),
                        )
                        .when_some(detail, |el, detail| {
                            el.child(
                                div()
                                    .text_size(px(12.))
                                    .text_color(colors.text_muted)
                                    .child(detail),
                            )
                        }),
                )
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.access = access;
                    cx.notify();
                }))
        });
        let book_rows = self.books.iter().enumerate().map(|(ix, book)| {
            let id = book.id.clone();
            h_flex()
                .pl(px(4. + 16. * book.depth as f32))
                .py(px(2.))
                .child(
                    Checkbox::new(ElementId::NamedInteger("key-book".into(), ix as u64))
                        .checked(self.ticked.contains(&book.id))
                        .label(book.name.clone())
                        .on_click(cx.listener(move |this, _: &bool, _, cx| this.toggle(&id, cx))),
                )
        });
        v_flex()
            .gap(px(14.))
            .child(
                v_flex()
                    .gap(px(6.))
                    .child(label("For", colors))
                    .child(h_flex().child(picker)),
            )
            .child(
                v_flex()
                    .gap(px(6.))
                    .child(label("Name", colors))
                    .child(Input::new(&self.name)),
            )
            .child(
                v_flex()
                    .gap(px(2.))
                    .child(label("Can", colors))
                    .children(access),
            )
            .child(
                v_flex()
                    .gap(px(6.))
                    .child(label("Notebooks", colors))
                    .child(
                        h_flex()
                            .gap(px(8.))
                            .child(
                                Switch::new("key-all-books")
                                    .checked(self.all_books)
                                    .on_click(cx.listener(|this, on: &bool, _, cx| {
                                        this.all_books = *on;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                div()
                                    .text_size(px(13.))
                                    .text_color(colors.text)
                                    .child("All notebooks"),
                            ),
                    )
                    .when(!self.all_books, |el| {
                        el.child(
                            v_flex()
                                .id("key-books")
                                .max_h(px(140.))
                                .overflow_y_scroll()
                                .children(book_rows),
                        )
                        .when(self.books.is_empty(), |el| {
                            el.child(
                                div()
                                    .text_size(px(12.5))
                                    .text_color(colors.text_muted)
                                    .child("There are no unlocked notebooks."),
                            )
                        })
                    })
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(colors.text_muted)
                            .child("Locked notes are never shown to a key."),
                    ),
            )
            .when_some(self.error.clone(), |el, error| {
                el.child(
                    div()
                        .text_size(px(12.5))
                        .text_color(colors.danger)
                        .child(error),
                )
            })
            .into_any_element()
    }
}

impl NewKeySheet {
    /// The setup for the key just made, with the key in it.
    fn render_made(
        &self,
        made: &Made,
        colors: &Colors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mono = crate::theme::mono_font(cx);
        let intro = match made.client {
            Client::Agent(agent) => format!("Setup for {}. {}", agent.name(), agent_where(agent)),
            Client::Other => "Send the key with every request to the local API.".to_owned(),
        };
        v_flex()
            .gap(px(12.))
            .child(
                div()
                    .text_size(px(13.))
                    .line_height(px(19.))
                    .text_color(colors.text)
                    .child(intro),
            )
            .child(
                v_flex()
                    .id("key-setup")
                    .max_h(px(260.))
                    .overflow_y_scroll()
                    .p(px(10.))
                    .rounded(px(8.))
                    .bg(colors.fill)
                    .text_size(px(12.))
                    .line_height(px(17.))
                    .text_color(colors.text)
                    .when_some(mono, |el, family| el.font_family(family))
                    .children(made.text.lines().map(|line| {
                        div().child(if line.is_empty() { " " } else { line }.to_owned())
                    })),
            )
            .child(
                h_flex()
                    .gap(px(10.))
                    .items_center()
                    .child(
                        Button::new("copy-key-setup")
                            .debug_selector(|| "copy-key-setup".into())
                            .primary()
                            .small()
                            .label(if made.copied { "Copied" } else { "Copy" })
                            .on_click(cx.listener(|this, _, window, cx| this.copy(window, cx))),
                    )
                    .child(
                        div()
                            .text_size(px(12.5))
                            .text_color(colors.text_secondary)
                            .child(SHOWN_ONCE),
                    ),
            )
    }
}

/// Where an agent's setup goes, from its own instructions.
fn agent_where(agent: crate::mcp_setup::Agent) -> &'static str {
    agent.instructions()
}

/// The sheet's buttons: Cancel and Create key, then Done.
struct KeyFooter {
    sheet: Entity<NewKeySheet>,
    _observing: Subscription,
}

// no-prop-test: drawing; `ready` follows `new_key`, which is tested.
impl Render for KeyFooter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sheet = self.sheet.read(cx);
        let ready = sheet.ready();
        let made = sheet.made.is_some();
        let target = self.sheet.clone();
        if made {
            return h_flex()
                .w_full()
                .gap_2()
                .justify_end()
                .child(
                    Button::new("key-done")
                        .debug_selector(|| "key-done".into())
                        .primary()
                        .label("Done")
                        .on_click(|_, window, cx| window.close_dialog(cx)),
                )
                .into_any_element();
        }
        h_flex()
            .w_full()
            .gap_2()
            .justify_end()
            .child(
                Button::new("key-cancel")
                    .label("Cancel")
                    .on_click(|_, window, cx| window.close_dialog(cx)),
            )
            .child(
                Button::new("key-create")
                    .debug_selector(|| "key-create".into())
                    .primary()
                    .label("Create key")
                    .disabled(!ready)
                    .on_click(move |_, _, cx| target.update(cx, |sheet, cx| sheet.create(cx))),
            )
            .into_any_element()
    }
}

/// Opens the New key sheet. Returns it, held weakly: the dialog owns it.
pub fn open_new_key_sheet(
    page: Entity<ApiKeysPage>,
    keys: KeyStore,
    window: &mut Window,
    cx: &mut App,
) -> WeakEntity<NewKeySheet> {
    let sheet = cx.new(|cx| NewKeySheet::new(page, keys, window, cx));
    let weak = sheet.downgrade();
    open_sheet(sheet, window, cx);
    weak
}

fn open_sheet(sheet: Entity<NewKeySheet>, window: &mut Window, cx: &mut App) {
    let footer = cx.new(|cx| KeyFooter {
        _observing: cx.observe(&sheet, |_, _, cx| cx.notify()),
        sheet: sheet.clone(),
    });
    window.open_alert_dialog(cx, move |alert, _, _| {
        alert
            .title("New key")
            .width(px(SHEET_WIDTH))
            .child(sheet.clone())
            .footer(footer.clone())
    });
}

/// The sheet showing a key's setup, for the snapshots.
#[cfg(test)]
pub(crate) fn open_made_sheet_for_test(
    page: Entity<ApiKeysPage>,
    keys: KeyStore,
    client: Client,
    text: String,
    window: &mut Window,
    cx: &mut App,
) {
    let sheet = cx.new(|cx| {
        let mut sheet = NewKeySheet::new(page, keys, window, cx);
        sheet.show_made_for_test(client, text);
        sheet
    });
    open_sheet(sheet, window, cx);
}

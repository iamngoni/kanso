//! The Publish sheet for a notebook: its notes as a small site at one link,
//! sealed on this device like a note's page, so the sync server keeps only a
//! copy it can't read (docs/ACCOUNTS.md › Published notebooks).
//!
//! `kanso_engine::publish::site` reads what would be published and builds the
//! site (the mobile app calls it too); the sheet
//! shows that first (the notes, the locked notes left out, the images and
//! files), then publishes, updates or unpublishes the link. The link and
//! its key are kept in the notebook (`kansoPublish`), so the account's
//! other devices can update or unpublish it too.

use std::sync::Arc;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, StyledExt as _, WindowExt as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_engine::publish::site::{
    PublishError, SitePlan, contents_line, error_text, locked_line, lowercase_first,
};
use kanso_engine::publish::{Busy, Phase, expiry_choice, needs_update, phase};
use kanso_store::Store;
use kanso_sync::share::{self, Expiry, KansoPublish};

use crate::clock::Clock;
use crate::editor::NoteEditor;
use crate::publish::{expiry_row, paragraph};
use crate::services::Services;
use crate::theme::Colors;
use crate::workspace::Workspace;
use crate::zoom::px;

actions!(kanso, [PublishNotebook]);

/// The sheet's width.
pub const SHEET_WIDTH: f32 = 480.;

pub struct PublishSiteView {
    editor: Entity<NoteEditor>,
    store: Arc<Store>,
    plan: SitePlan,
    signed_in: bool,
    /// The server the link points at.
    server: String,
    expiry: Expiry,
    busy: Option<Busy>,
    error: Option<SharedString>,
    copied: bool,
    /// Unpublish was pressed once; the next press unpublishes.
    confirm_unpublish: bool,
    scroll: ScrollHandle,
}

impl PublishSiteView {
    pub fn phase(&self) -> Phase {
        phase(self.signed_in, self.plan.publish.as_ref())
    }

    pub fn is_busy(&self) -> bool {
        self.busy.is_some()
    }

    #[cfg(test)]
    pub fn plan(&self) -> &SitePlan {
        &self.plan
    }

    #[cfg(test)]
    pub fn error(&self) -> Option<&SharedString> {
        self.error.as_ref()
    }

    fn link(&self) -> Option<String> {
        self.plan.publish.as_ref().map(|p| p.link(&self.server))
    }

    fn outdated(&self, now_ms: i64) -> bool {
        self.plan.publish.as_ref().is_some_and(|p| {
            needs_update(
                p,
                &self.plan.name,
                &self.plan.fingerprint(),
                self.expiry,
                now_ms,
            )
        })
    }

    fn set_expiry(&mut self, expiry: Expiry, cx: &mut Context<Self>) {
        self.expiry = expiry;
        cx.notify();
    }

    fn copy(&mut self, cx: &mut Context<Self>) {
        if let Some(link) = self.link() {
            cx.write_to_clipboard(ClipboardItem::new_string(link));
            self.copied = true;
            cx.notify();
        }
    }

    /// Reads the notebook again, with the open note as typed.
    fn reread(&mut self, cx: &mut Context<Self>) -> bool {
        self.editor.update(cx, |editor, cx| editor.flush(cx));
        match kanso_engine::publish::site::plan(&self.store, &self.plan.book_id) {
            Ok(plan) => {
                self.plan = plan;
                true
            }
            Err(_) => {
                self.error = Some("The notebook is gone. It may have been deleted.".into());
                false
            }
        }
    }

    /// Publishes the notebook as it is now, or updates its link in place.
    pub fn run_publish(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy.is_some() {
            return;
        }
        let services = cx.global::<Services>();
        let (Some(session), Some(api)) = (services.session(), services.shares.clone()) else {
            self.error = Some("Publishing needs your Kanso account. Sign in, then publish.".into());
            cx.notify();
            return;
        };
        if !self.reread(cx) {
            cx.notify();
            return;
        }
        if self.plan.notes.is_empty() {
            self.error = Some("This notebook has no notes to publish.".into());
            cx.notify();
            return;
        }
        let palette = kanso_engine::export::print_palette(crate::preview_rich::palette(cx));
        let style = crate::export_style::for_palette(cx, &palette);
        let now = Clock::now_ms(cx);
        let expires_at = self.expiry.at(now);
        let store = self.store.clone();
        let plan = self.plan.clone();
        self.busy = Some(if plan.publish.is_some() {
            Busy::Updating
        } else {
            Busy::Publishing
        });
        self.error = None;
        self.confirm_unpublish = false;
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    kanso_engine::publish::site::publish(
                        &*api, &session, &plan, &store, &palette, &style, expires_at, now,
                    )
                })
                .await;
            this.update(cx, |this, cx| this.finish(result.map(Some), cx))
                .ok();
        })
        .detach();
    }

    /// Unpublishes the link: the first press asks, the second does it.
    pub fn run_unpublish(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy.is_some() {
            return;
        }
        if !self.confirm_unpublish {
            self.confirm_unpublish = true;
            cx.notify();
            return;
        }
        let Some(publish) = self.plan.publish.clone() else {
            return;
        };
        let services = cx.global::<Services>();
        let (Some(session), Some(api)) = (services.session(), services.shares.clone()) else {
            self.error = Some("Unpublishing needs your Kanso account. Sign in first.".into());
            cx.notify();
            return;
        };
        self.busy = Some(Busy::Unpublishing);
        self.error = None;
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { share::unpublish(&*api, &session, &publish) })
                .await;
            this.update(cx, |this, cx| {
                this.finish(result.map(|()| None).map_err(PublishError::from), cx)
            })
            .ok();
        })
        .detach();
    }

    /// Keeps what the server now holds in the notebook.
    fn finish(
        &mut self,
        result: Result<Option<KansoPublish>, PublishError>,
        cx: &mut Context<Self>,
    ) {
        self.busy = None;
        self.confirm_unpublish = false;
        match result {
            Ok(publish) => {
                if let Err(err) = kanso_engine::publish::site::store_book_publish(
                    &self.store,
                    &self.plan.book_id,
                    publish.as_ref(),
                ) {
                    self.error =
                        Some(format!("Couldn't save the link in the notebook: {err}").into());
                }
                if let Some(p) = &publish {
                    self.expiry = expiry_choice(p);
                }
                self.plan.publish = publish;
                self.copied = false;
            }
            Err(err) => self.error = Some(error_text(&err).into()),
        }
        cx.notify();
    }

    /// What the link holds, or will: the notes, then what is left out.
    fn contents(&self, colors: &Colors) -> Div {
        let plan = &self.plan;
        let rows = plan.notes.iter().enumerate().map(|(ix, note)| {
            h_flex()
                .id(("publish-site-note", ix))
                .gap_2()
                .px(px(10.))
                .py(px(4.))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_size(px(13.))
                        .text_color(colors.text)
                        .whitespace_nowrap()
                        .overflow_hidden()
                        .text_ellipsis()
                        .child(note.title.clone()),
                )
                .when_some(note.group.clone(), |el, group| {
                    el.child(
                        div()
                            .flex_shrink_0()
                            .max_w(px(180.))
                            .text_size(px(12.))
                            .text_color(colors.text_muted)
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(group),
                    )
                })
        });
        let locked = locked_line(plan);
        v_flex()
            .gap(px(8.))
            .child(
                div()
                    .text_size(px(13.))
                    .font_medium()
                    .text_color(colors.text)
                    .debug_selector(|| "publish-site-contents".into())
                    .child(if plan.notes.is_empty() {
                        "This notebook has no notes to publish.".to_owned()
                    } else if plan.publish.is_some() {
                        format!(
                            "The notebook now has {}",
                            lowercase_first(&contents_line(plan))
                        )
                    } else {
                        format!(
                            "The site will hold {}",
                            lowercase_first(&contents_line(plan))
                        )
                    }),
            )
            .when(!plan.notes.is_empty(), |el| {
                el.child(
                    div()
                        .id("publish-site-notes")
                        .max_h(px(168.))
                        .overflow_y_scroll()
                        .track_scroll(&self.scroll)
                        .rounded(px(6.))
                        .border_1()
                        .border_color(colors.divider)
                        .bg(colors.list)
                        .py(px(4.))
                        .children(rows),
                )
            })
            .when(!locked.is_empty(), |el| {
                el.child(
                    div()
                        .text_size(px(12.5))
                        .line_height(px(18.))
                        .text_color(colors.text_muted)
                        .debug_selector(|| "publish-site-locked".into())
                        .child(locked),
                )
            })
    }
}

// no-prop-test: drawing; the notebook publish snapshots show each phase,
// and `contents_line`, `locked_line` and `error_text` carry the wording.
impl Render for PublishSiteView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let now = Clock::now_ms(cx);
        let body = v_flex()
            .gap(px(14.))
            .debug_selector(|| "publish-site-sheet".into());
        let body = match self.phase() {
            Phase::SignedOut => body
                .child(paragraph(
                    "Publishing puts this notebook on the web as a small site, a page for each note, at a link you can share. Your sync server keeps only an encrypted copy it can't read.",
                    colors,
                ))
                .child(paragraph(
                    "It needs a Kanso account, so the copy has somewhere to live. Sign in, or create an account, then publish.",
                    colors,
                ))
                .child(self.contents(colors)),
            Phase::Unpublished => body
                .child(paragraph(
                    "Anyone with the link can read these notes, as a site with a page for each one and links between them. The key that opens it is in the link itself, so your sync server keeps only an encrypted copy it can't read.",
                    colors,
                ))
                .child(self.contents(colors))
                .child(expiry_row(self.expiry, self.busy.is_some(), colors, cx, Self::set_expiry))
                .child(
                    div()
                        .text_size(px(12.5))
                        .line_height(px(18.))
                        .text_color(colors.text_muted)
                        .child("The site shows the notebook as it is now. Later edits stay on your devices until you update it. The notebook's name is sent unencrypted, so link previews can show it."),
                ),
            Phase::Published => {
                let publish = self.plan.publish.clone().expect("published");
                let link = self.link().unwrap_or_default();
                let mono = cx.theme().mono_font_family.clone();
                let published = kanso_engine::publish::when_label(Clock::local_ms(cx, publish.published_at));
                let expires = match publish.expires_at {
                    None => "The link doesn't expire.".to_owned(),
                    Some(at) if at <= now => format!("The link expired on {}.", kanso_engine::publish::when_label(Clock::local_ms(cx, at))),
                    Some(at) => format!("The link stops working on {}.", kanso_engine::publish::when_label(Clock::local_ms(cx, at))),
                };
                let changed = self.plan.is_outdated(&publish);
                body.child(paragraph(
                    "Anyone with this link can read the notebook's site. Your sync server keeps only an encrypted copy it can't read.",
                    colors,
                ))
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .id("publish-site-link")
                                .debug_selector(|| "publish-site-link".into())
                                .flex_1()
                                .min_w_0()
                                .px(px(10.))
                                .py(px(7.))
                                .rounded(px(6.))
                                .border_1()
                                .border_color(colors.divider)
                                .bg(colors.list)
                                .font_family(mono)
                                .text_size(px(12.))
                                .text_color(colors.text)
                                .whitespace_nowrap()
                                .overflow_hidden()
                                .text_ellipsis()
                                .child(link),
                        )
                        .child(
                            Button::new("publish-site-copy")
                                .outline()
                                .icon(if self.copied { Lucide::Check } else { Lucide::Copy })
                                .label(if self.copied { "Copied" } else { "Copy" })
                                .debug_selector(|| "publish-site-copy".into())
                                .on_click(cx.listener(|this, _, _, cx| this.copy(cx))),
                        ),
                )
                .child(
                    div()
                        .text_size(px(12.5))
                        .line_height(px(18.))
                        .text_color(colors.text_muted)
                        .child(format!("Published {published}. {expires}")),
                )
                .when(changed, |el| {
                    el.child(
                        div()
                            .px(px(12.))
                            .py(px(8.))
                            .rounded(px(6.))
                            .bg(Hsla::from(colors.warning).opacity(0.14))
                            .text_size(px(13.))
                            .line_height(px(19.))
                            .text_color(colors.text)
                            .debug_selector(|| "publish-site-outdated".into())
                            .child("This notebook has changed since you published it. The link still shows the earlier version."),
                    )
                })
                .child(self.contents(colors))
                .child(expiry_row(self.expiry, self.busy.is_some(), colors, cx, Self::set_expiry))
                .when(self.confirm_unpublish, |el| {
                    el.child(
                        div()
                            .text_size(px(13.))
                            .line_height(px(19.))
                            .text_color(colors.danger)
                            .child("Unpublishing stops the link working for everyone who has it. Publishing again makes a new link."),
                    )
                })
            }
        };
        body.when_some(self.error.clone(), |el, error| {
            el.child(
                div()
                    .text_size(px(13.))
                    .line_height(px(19.))
                    .text_color(colors.danger)
                    .debug_selector(|| "publish-site-error".into())
                    .child(error),
            )
        })
    }
}

/// The sheet's buttons, which follow its phase and task.
pub struct PublishSiteFooter {
    view: Entity<PublishSiteView>,
    _observing: Subscription,
}

// no-prop-test: drawing; the notebook publish snapshots show each phase's
// buttons.
impl Render for PublishSiteFooter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = self.view.read(cx);
        let busy = view.busy;
        let now = Clock::now_ms(cx);
        let empty = view.plan.notes.is_empty();
        let close = Button::new("publish-site-close")
            .label(if view.phase() == Phase::Published {
                "Done"
            } else {
                "Cancel"
            })
            .disabled(busy.is_some())
            .debug_selector(|| "publish-site-close".into())
            .on_click(|_, window, cx| window.close_dialog(cx));
        let target = self.view.clone();
        let row = h_flex().w_full().gap_2().items_center();
        match view.phase() {
            Phase::SignedOut => row.child(div().flex_1()).child(close).child(
                Button::new("publish-site-sign-in")
                    .primary()
                    .label("Sign in…")
                    .debug_selector(|| "publish-site-sign-in".into())
                    .on_click(|_, window, cx| {
                        window.close_dialog(cx);
                        window.dispatch_action(Box::new(crate::workspace::ShowSignIn), cx);
                    }),
            ),
            Phase::Unpublished => {
                let running = busy == Some(Busy::Publishing);
                row.child(div().flex_1()).child(close).child(
                    Button::new("publish-site-go")
                        .primary()
                        .label(if running { "Publishing…" } else { "Publish" })
                        .when(running, |b| b.icon(Lucide::RefreshCcw))
                        .loading(running)
                        .disabled(empty)
                        .debug_selector(|| "publish-site-go".into())
                        .on_click(move |_, window, cx| {
                            target.update(cx, |view, cx| view.run_publish(window, cx));
                        }),
                )
            }
            Phase::Published => {
                let unpublishing = busy == Some(Busy::Unpublishing);
                let updating = busy == Some(Busy::Updating);
                let confirm = view.confirm_unpublish;
                let outdated = view.outdated(now);
                let unpublish_target = self.view.clone();
                row.child(
                    Button::new("publish-site-unpublish")
                        .ghost()
                        .when(confirm || unpublishing, |b| b.danger())
                        .label(if unpublishing {
                            "Unpublishing…"
                        } else if confirm {
                            "Unpublish link"
                        } else {
                            "Unpublish"
                        })
                        .loading(unpublishing)
                        .disabled(updating)
                        .debug_selector(|| "publish-site-unpublish".into())
                        .on_click(move |_, window, cx| {
                            unpublish_target.update(cx, |view, cx| view.run_unpublish(window, cx));
                        }),
                )
                .child(div().flex_1())
                .child(close)
                .when((outdated && !empty) || updating, |row| {
                    row.child(
                        Button::new("publish-site-update")
                            .primary()
                            .label(if updating {
                                "Updating…"
                            } else {
                                "Update published version"
                            })
                            .when(updating, |b| b.icon(Lucide::RefreshCcw))
                            .loading(updating)
                            .disabled(unpublishing)
                            .debug_selector(|| "publish-site-update".into())
                            .on_click(move |_, window, cx| {
                                target.update(cx, |view, cx| view.run_publish(window, cx));
                            }),
                    )
                })
            }
        }
    }
}

/// Opens the Publish sheet for notebook `book_id`. Returns the sheet, for
/// tests and snapshots to drive.
pub fn open_publish_site(
    workspace: &mut Workspace,
    book_id: &str,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> Option<Entity<PublishSiteView>> {
    workspace.flush(cx);
    let store = workspace.store.clone();
    let plan = match kanso_engine::publish::site::plan(&store, book_id) {
        Ok(plan) => plan,
        Err(err) => {
            workspace.notify_error(format!("Couldn't read the notebook: {err}"), window, cx);
            return None;
        }
    };
    let services = cx.global::<Services>();
    let signed_in = services.signed_in();
    let server = services
        .session()
        .map(|s| s.server.clone())
        .unwrap_or_else(|| kanso_sync::DEFAULT_SERVER.to_owned());
    let expiry = plan.publish.as_ref().map_or(Expiry::Never, expiry_choice);
    let editor = workspace.editor.clone();
    let title = format!("Publish “{}” to the web", plan.name);
    let view = cx.new(|_| PublishSiteView {
        editor,
        store,
        plan,
        signed_in,
        server,
        expiry,
        busy: None,
        error: None,
        copied: false,
        confirm_unpublish: false,
        scroll: ScrollHandle::new(),
    });
    let footer = cx.new(|cx| PublishSiteFooter {
        view: view.clone(),
        _observing: cx.observe(&view, |_, _, cx| cx.notify()),
    });
    let dialog_view = view.clone();
    window.open_alert_dialog(cx, move |alert, _, _| {
        let running = dialog_view.clone();
        alert
            .title(title.clone())
            .width(px(SHEET_WIDTH))
            .child(dialog_view.clone())
            .footer(footer.clone())
            // Escape closes, but not while a task runs.
            .on_cancel(move |_, _, cx| !running.read(cx).is_busy())
    });
    Some(view)
}

impl Workspace {
    /// The notebook "Publish notebook…" means: the one the list shows, else
    /// the open note's.
    pub(crate) fn notebook_to_publish(&self, cx: &App) -> Option<String> {
        if let crate::library::Filter::Book(id) = &self.library.read(cx).filter {
            return Some(id.clone());
        }
        let id = self.editor.read(cx).note_id()?.to_owned();
        let note = self.store.note(&id).ok().flatten()?;
        (note.book_id != kanso_core::TRASH_BOOK_ID).then_some(note.book_id)
    }

    pub(crate) fn publish_notebook(
        &mut self,
        book_id: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.close_settings(window, cx);
        match book_id.or_else(|| self.notebook_to_publish(cx)) {
            Some(id) => {
                open_publish_site(self, &id, window, cx);
            }
            None => window.push_notification(
                gpui_kit::component::notification::Notification::info(
                    "Choose a notebook in the sidebar, then publish it.",
                ),
                cx,
            ),
        }
    }
}

pub fn on_actions(el: Div, cx: &mut Context<Workspace>) -> Div {
    el.on_action(cx.listener(|this, _: &PublishNotebook, window, cx| {
        this.publish_notebook(None, window, cx);
    }))
}

#[cfg(test)]
mod tests {
    mod flow {
        use std::sync::Arc;

        use gpui_kit::component::WindowExt as _;
        use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};
        use kanso_core::DocType;
        use kanso_core::links::NOTE_LINK_PREFIX;
        use kanso_store::{Origin, Put, Store};
        use kanso_sync::share::{KansoPublish, MemoryShares, PUBLISH_KEY, open_share};
        use kanso_sync::site::SiteBundle;
        use kanso_sync::{DeviceIdentity, MemorySecrets, MemoryServer, Password};
        use serde_json::json;

        use super::super::{PublishSiteView, open_publish_site};
        use crate::config::AppConfig;
        use crate::services::Services;
        use crate::workspace::{AppState, Workspace};
        use kanso_engine::publish::Phase;

        struct Fixture {
            _dir: tempfile::TempDir,
            store: Arc<Store>,
            window: AnyWindowHandle,
            workspace: Entity<Workspace>,
            shares: Arc<MemoryShares>,
            book: String,
            soup: String,
        }

        fn note(store: &Store, title: &str, body: &str, book: &str) -> String {
            store
                .put(
                    DocType::Note,
                    json!({"title": title, "body": body, "bookId": book}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap()
                .id
        }

        fn setup(cx: &mut TestAppContext) -> Fixture {
            cx.executor().allow_parking();
            let dir = tempfile::tempdir().unwrap();
            let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
            let book = store
                .put(
                    DocType::Book,
                    json!({"name": "Recipes"}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap()
                .id;
            let other = store
                .put(
                    DocType::Book,
                    json!({"name": "Private"}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap()
                .id;
            let soup = note(&store, "Soup", "Warm. Serve with [[Bread]].", &book);
            let soup_link = format!("{NOTE_LINK_PREFIX}{}", soup.trim_start_matches("note:"));
            note(
                &store,
                "Bread",
                &format!("Flour and water. Back to [soup]({soup_link}). See [[Diary]]."),
                &book,
            );
            note(&store, "Diary", "The secret diary.", &other);
            let secret = note(&store, "Family secret", "Never share this.", &book);
            let mut locked = store.note(&secret).unwrap().unwrap();
            locked.body = String::new();
            locked.extra.insert(
                kanso_core::model::LOCK_FIELD.into(),
                json!({"v": 1, "sealed": "c2VjcmV0"}),
            );
            store
                .lock_note(serde_json::to_value(&locked).unwrap())
                .unwrap();
            let shares = Arc::new(MemoryShares::new());
            let mut services = Services::default();
            services.shares = Some(shares.clone());
            let server = MemoryServer::new();
            let username = "alice";
            let session = kanso_sync::sign_up(
                &server,
                &MemorySecrets::new(),
                kanso_sync::DEFAULT_SERVER,
                username,
                &Password::new("correct horse battery", username).unwrap(),
                &DeviceIdentity::from_parts(&format!("l_{:032x}", 1), "Mac", [1; 32]),
                &|_| {},
            )
            .unwrap()
            .session;
            services.set_signed_in(Arc::new(session));
            let (data_dir, app_store) = (dir.path().to_path_buf(), store.clone());
            let (window, workspace) = cx.update(|cx| {
                gpui_kit::init(cx);
                cx.set_global(AppState {
                    store: app_store,
                    data_dir,
                    config: AppConfig::default(),
                });
                cx.set_global(services);
                gpui_kit::open_window(Default::default(), cx, |window, cx| {
                    cx.new(|cx| Workspace::new(window, cx))
                })
                .unwrap()
            });
            cx.run_until_parked();
            Fixture {
                _dir: dir,
                store,
                window,
                workspace,
                shares,
                book,
                soup,
            }
        }

        impl Fixture {
            fn open(&self, cx: &mut TestAppContext) -> Entity<PublishSiteView> {
                let book = self.book.clone();
                self.window
                    .update(cx, |_, window, cx| {
                        self.workspace
                            .update(cx, |ws, cx| open_publish_site(ws, &book, window, cx))
                    })
                    .unwrap()
                    .expect("the notebook exists")
            }

            fn close(&self, cx: &mut TestAppContext) {
                self.window
                    .update(cx, |_, window, cx| window.close_dialog(cx))
                    .unwrap();
                cx.run_until_parked();
            }

            fn act(
                &self,
                cx: &mut TestAppContext,
                view: &Entity<PublishSiteView>,
                f: impl FnOnce(
                    &mut PublishSiteView,
                    &mut gpui_kit::Window,
                    &mut gpui_kit::Context<PublishSiteView>,
                ),
            ) {
                self.window
                    .update(cx, |_, window, cx| {
                        view.update(cx, |v, cx| f(v, window, cx))
                    })
                    .unwrap();
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
                loop {
                    cx.run_until_parked();
                    if !cx.read(|cx| view.read(cx).is_busy()) {
                        break;
                    }
                    assert!(std::time::Instant::now() < deadline, "the task finished");
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
            }

            fn kept(&self) -> Option<KansoPublish> {
                let book = self
                    .store
                    .books()
                    .unwrap()
                    .into_iter()
                    .find(|b| b.id == self.book)
                    .unwrap();
                KansoPublish::from_extra(&book.extra)
            }

            fn site(&self, publish: &KansoPublish) -> SiteBundle {
                let stored = self.shares.get(&publish.share_id).unwrap();
                assert!(!String::from_utf8_lossy(&stored.sealed).contains("Flour"));
                let plain = open_share(
                    &publish.share_key().unwrap(),
                    &publish.share_id,
                    &stored.sealed,
                )
                .unwrap();
                SiteBundle::from_bytes(&plain).expect("a site")
            }
        }

        // no-prop-test: an end-to-end flow in a window; the properties are
        // site.rs's, kanso-sync's site and share tests, and the viewer's.
        #[gpui_kit::test]
        fn publish_a_notebook_update_it_and_unpublish(cx: &mut TestAppContext) {
            let f = setup(cx);
            let view = f.open(cx);
            assert_eq!(cx.read(|cx| view.read(cx).phase()), Phase::Unpublished);
            let plan = cx.read(|cx| view.read(cx).plan().clone());
            let titles: Vec<&str> = plan.notes.iter().map(|n| n.title.as_str()).collect();
            assert_eq!(titles, ["Bread", "Soup"], "the locked note stays out");
            assert_eq!(plan.locked, ["Family secret"]);

            f.act(cx, &view, |v, window, cx| v.run_publish(window, cx));
            assert_eq!(cx.read(|cx| view.read(cx).error().cloned()), None);
            assert_eq!(cx.read(|cx| view.read(cx).phase()), Phase::Published);
            let first = f.kept().expect("the notebook keeps its link");
            let stored = f.shares.get(&first.share_id).unwrap();
            assert_eq!(stored.title, "Recipes", "the name goes with the upload");
            let site = f.site(&first);
            assert_eq!(site.title, "Recipes");
            assert_eq!(site.pages.len(), 2);
            let all: String = std::iter::once(site.index.clone())
                .chain(site.pages.values().map(|p| p.html.clone()))
                .collect();
            assert!(!all.contains("Family secret") && !all.contains("Never share"));
            assert!(!all.contains("secret diary"), "other notebooks stay out");
            let soup = site.page_html("soup").unwrap();
            assert!(soup.contains(r#"href="kanso-page:bread""#), "{soup}");
            let bread = site.page_html("bread").unwrap();
            assert!(bread.contains(r#"href="kanso-page:soup""#), "{bread}");
            assert!(
                !bread.contains(r#"kanso-page:diary"#),
                "unpublished notes are plain text"
            );
            let soup_note = f.store.note(&f.soup).unwrap().unwrap();
            assert!(
                !soup_note.extra.contains_key(PUBLISH_KEY),
                "the link is the notebook's"
            );
            f.close(cx);

            // Changed since: the sheet offers an update, which keeps the link.
            f.store
                .put(
                    DocType::Note,
                    json!({"_id": f.soup, "body": "Hot. Serve with [[Bread]]."}),
                    Put::Merge,
                    Origin::Ui,
                )
                .unwrap();
            let view = f.open(cx);
            assert!(cx.read(|cx| view.read(cx).plan().is_outdated(&first)));
            f.act(cx, &view, |v, window, cx| v.run_publish(window, cx));
            let second = f.kept().unwrap();
            assert_eq!(
                (&second.share_id, &second.key),
                (&first.share_id, &first.key)
            );
            assert!(!cx.read(|cx| view.read(cx).plan().is_outdated(&second)));
            assert!(f.site(&second).page_html("soup").unwrap().contains("Hot."));

            // Unpublish asks first, then takes the link out of the notebook.
            f.act(cx, &view, |v, window, cx| v.run_unpublish(window, cx));
            assert!(
                f.shares.get(&first.share_id).is_some(),
                "the first press only asks"
            );
            f.act(cx, &view, |v, window, cx| v.run_unpublish(window, cx));
            assert!(f.shares.is_empty());
            assert_eq!(f.kept(), None);
            assert_eq!(cx.read(|cx| view.read(cx).phase()), Phase::Unpublished);
        }

        // no-prop-test: one flow in a window.
        #[gpui_kit::test]
        fn an_expired_notebook_link_is_published_anew(cx: &mut TestAppContext) {
            let f = setup(cx);
            let view = f.open(cx);
            f.act(cx, &view, |v, window, cx| v.run_publish(window, cx));
            let first = f.kept().unwrap();
            assert!(f.shares.take_down(&first.share_id));
            f.close(cx);
            let view = f.open(cx);
            f.act(cx, &view, |v, window, cx| v.run_publish(window, cx));
            let second = f.kept().unwrap();
            assert_ne!(second.share_id, first.share_id);
            assert_ne!(second.key, first.key);
            assert_eq!(f.site(&second).pages.len(), 2);
        }
    }
}

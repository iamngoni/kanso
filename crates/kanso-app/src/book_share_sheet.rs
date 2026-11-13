//! The Share notebook sheet (issue #390 › Shared studio notebooks): who a
//! notebook is shared with and at what role. It works like the note Share
//! sheet. The owner, and managers below their rank, invite people by
//! username with a role, change roles and remove people. Anyone else sees
//! who is in the notebook and can leave it; the owner can stop sharing it.
//!
//! The first share with someone shows their fingerprint to compare, as the
//! note sheet does. Removing someone, lowering a role, leaving and stopping
//! each change the notebook's key; the engine does that, and the sheet says
//! what is running and shows any error. Sharing a notebook for the first
//! time sends every note up, with a progress line.

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, Sizable as _, StyledExt as _, WindowExt as _, h_flex,
    v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_collab::book_copy::{leave_title, remove_body, remove_title, stop_title};
use kanso_collab::{BookMemberView, BookProgress, BookView, Person, Role};

use crate::book_share_model::{
    BookBusy, BookStep, busy_words, first_step, initial, invite_roles, locked_words,
    member_actions, member_detail, status_words,
};
use crate::collab_model::{Collab, Lookup};
use crate::publish::paragraph;
use crate::share_sheet::{choices, fingerprint_block, heading, look_up, quiet};
use crate::theme::{Colors, cursor_colour};
use crate::zoom::px;

/// The sheet's width.
pub const BOOK_SHEET_WIDTH: f32 = 540.;

pub struct BookShareSheet {
    book_id: String,
    pub(crate) name: String,
    pub(crate) step: BookStep,
    pub(crate) share_id: Option<String>,
    pub(crate) view: Option<BookView>,
    username: Entity<InputState>,
    /// The role the next invitation gives.
    pub(crate) role: Role,
    /// Locked notes that would stay out if it were shared now.
    locked: usize,
    pub(crate) busy: Option<BookBusy>,
    pub(crate) error: Option<SharedString>,
    /// What the last step did, when it is worth saying.
    pub(crate) done: Option<SharedString>,
    _subscriptions: Vec<Subscription>,
}

impl BookShareSheet {
    pub fn is_busy(&self) -> bool {
        self.busy.is_some()
    }

    fn my_role(&self) -> Option<Role> {
        match &self.view {
            Some(view) => view.my_role,
            // Not shared yet: it is this account's to share.
            None => self.share_id.is_none().then_some(Role::Owner),
        }
    }

    /// Reads the notebook's people from the server again.
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        let Some(share) = self.share_id.clone() else {
            return;
        };
        self.busy = Some(BookBusy::Loading);
        let this = cx.entity().downgrade();
        crate::collab::run(
            cx,
            move |engine| engine.book_view_fresh(&share),
            move |result, cx| {
                this.update(cx, |this, cx| this.finish(result.map(Some), cx))
                    .ok();
            },
        );
        cx.notify();
    }

    /// Keeps what a step brought back: the notebook's people as they are now.
    pub(crate) fn finish(
        &mut self,
        result: Result<Option<BookView>, String>,
        cx: &mut Context<Self>,
    ) {
        self.busy = None;
        match result {
            Ok(Some(view)) => {
                self.name = view.name.clone();
                self.view = Some(view);
            }
            Ok(None) => {}
            Err(words) => self.error = Some(words.into()),
        }
        if !matches!(
            self.step,
            BookStep::Verify { .. } | BookStep::KeyChanged { .. }
        ) {
            self.step = BookStep::Main;
        }
        cx.notify();
    }

    /// Looks the typed username up, then invites them, or first shows their
    /// fingerprint (a first share) or their changed keys.
    pub fn invite(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        if self.busy.is_some() {
            return;
        }
        let name = self.username.read(cx).value().trim().to_owned();
        if name.is_empty() {
            self.error = Some("Type the username of the person to invite.".into());
            cx.notify();
            return;
        }
        let role = self.role;
        self.busy = Some(BookBusy::LookingUp);
        self.error = None;
        self.done = None;
        let this = cx.entity().downgrade();
        crate::collab::run(
            cx,
            move |engine| look_up(engine, &name),
            move |result, cx| {
                this.update(cx, |this, cx| {
                    this.busy = None;
                    match result {
                        Ok(Lookup::Found(person)) if person.verified => {
                            this.send_invite(person.username, role, false, cx)
                        }
                        Ok(Lookup::Found(person)) => {
                            this.step = BookStep::Verify { person, role };
                            cx.notify();
                        }
                        Ok(Lookup::Changed { pinned, offered }) => {
                            this.step = BookStep::KeyChanged {
                                pinned,
                                offered,
                                role,
                            };
                            cx.notify();
                        }
                        Err(words) => {
                            this.error = Some(words.into());
                            cx.notify();
                        }
                    }
                })
                .ok();
            },
        );
        cx.notify();
    }

    /// Invites someone whose keys are pinned, marking their fingerprint
    /// verified first when it was compared. The first invitation shares
    /// the notebook: every note goes up, with a progress line.
    pub(crate) fn send_invite(
        &mut self,
        username: String,
        role: Role,
        verify: bool,
        cx: &mut Context<Self>,
    ) {
        let Some(engine) = crate::collab::engine(cx) else {
            self.error = Some("Sharing needs your Kanso account. Sign in first.".into());
            cx.notify();
            return;
        };
        self.step = BookStep::Main;
        self.error = None;
        self.done = None;
        self.busy = Some(match self.share_id {
            Some(_) => BookBusy::Inviting,
            None => BookBusy::Sharing { done: 0, total: 0 },
        });
        let (book_id, share) = (self.book_id.clone(), self.share_id.clone());
        let invited = username.clone();
        let (tx, rx) = async_channel::unbounded::<BookProgress>();
        let task = cx.background_executor().spawn(async move {
            let work = || -> kanso_collab::api::Result<(String, BookView, bool)> {
                if verify {
                    let person = engine.person(&username)?;
                    engine.verify_person(&person.account_id)?;
                }
                let (share, first) = match share {
                    Some(share) => {
                        engine.invite_to_book(&share, &[(username, role)])?;
                        (share, false)
                    }
                    None => {
                        let shared =
                            engine.share_book(&book_id, &[(username, role)], true, &mut |p| {
                                tx.try_send(p).ok();
                            })?;
                        (shared.share_id, true)
                    }
                };
                // The people as the server has them now; the invitation
                // stands even if this read fails.
                let view = engine
                    .book_view_fresh(&share)
                    .or_else(|_| engine.book_view(&share))?;
                Ok((share, view, first))
            };
            work().map_err(|err| crate::collab::error_text(&err))
        });
        cx.spawn(async move |this, cx| {
            while let Ok(progress) = rx.recv().await {
                let alive = this.update(cx, |this, cx| {
                    this.busy = Some(BookBusy::Sharing {
                        done: progress.done,
                        total: progress.total,
                    });
                    cx.notify();
                });
                if alive.is_err() {
                    break;
                }
            }
            let result = task.await;
            this.update(cx, |this, cx| match result {
                Ok((share, view, first)) => {
                    crate::metrics::record(cx, crate::metrics::Feature::SharedNote);
                    this.share_id = Some(share);
                    this.done = Some(
                        if first {
                            format!(
                                "Shared \"{}\" with {invited}. It shows for them once they accept.",
                                view.name
                            )
                        } else {
                            format!(
                                "Invited {invited}. The notebook shows for them once they accept."
                            )
                        }
                        .into(),
                    );
                    this.finish(Ok(Some(view)), cx);
                    crate::shared_books::refresh_local(cx);
                    crate::collab::sync_soon(cx);
                }
                Err(words) => this.finish(Err(words), cx),
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    /// Runs a step on the notebook, then shows its people as they are now.
    fn act(
        &mut self,
        busy: BookBusy,
        done: Option<String>,
        cx: &mut Context<Self>,
        work: impl FnOnce(&kanso_collab::Engine, &str) -> kanso_collab::api::Result<()> + Send + 'static,
    ) {
        let Some(share) = self.share_id.clone() else {
            return;
        };
        if self.busy.is_some() {
            return;
        }
        self.busy = Some(busy);
        self.step = BookStep::Main;
        self.error = None;
        self.done = None;
        let this = cx.entity().downgrade();
        crate::collab::run(
            cx,
            move |engine| {
                work(engine, &share)?;
                engine
                    .book_view_fresh(&share)
                    .or_else(|_| engine.book_view(&share))
            },
            move |result, cx| {
                this.update(cx, |this, cx| {
                    if result.is_ok() {
                        this.done = done.map(Into::into);
                    }
                    this.finish(result.map(Some), cx);
                })
                .ok();
                crate::shared_books::refresh_local(cx);
                crate::collab::sync_soon(cx);
            },
        );
        cx.notify();
    }

    /// Gives a member another role: a lower one changes the notebook's key.
    pub(crate) fn set_role(&mut self, account: String, role: Role, cx: &mut Context<Self>) {
        let lower = self
            .view
            .as_ref()
            .and_then(|v| v.members.iter().find(|m| m.account == account))
            .is_some_and(|m| role < m.role);
        let busy = if lower {
            BookBusy::Rotating
        } else {
            BookBusy::Raising
        };
        self.act(busy, None, cx, move |engine, share| {
            engine.set_book_role(share, &account, role)
        });
    }

    /// Asks before removing someone, in the notebook's words.
    pub(crate) fn ask_remove(&mut self, account: String, username: String, cx: &mut Context<Self>) {
        self.error = None;
        self.step = BookStep::ConfirmRemove { account, username };
        cx.notify();
    }

    pub(crate) fn remove(&mut self, account: String, username: String, cx: &mut Context<Self>) {
        let done = format!("Removed {username}. The notebook has a new key.");
        self.act(BookBusy::Rotating, Some(done), cx, move |engine, share| {
            engine.remove_book_member(share, &account)
        });
    }

    pub(crate) fn ask_end(&mut self, cx: &mut Context<Self>) {
        self.error = None;
        self.step = BookStep::ConfirmEnd;
        cx.notify();
    }

    /// Stops sharing (the owner) or leaves (anyone else). Everyone keeps a
    /// private copy; the engine says what it is called.
    pub(crate) fn end(&mut self, cx: &mut Context<Self>) {
        let Some(share) = self.share_id.clone() else {
            return;
        };
        if self.busy.is_some() {
            return;
        }
        let owner = self.my_role() == Some(Role::Owner);
        self.busy = Some(if owner {
            BookBusy::Stopping
        } else {
            BookBusy::Leaving
        });
        self.step = BookStep::Main;
        self.error = None;
        let this = cx.entity().downgrade();
        crate::collab::run(
            cx,
            move |engine| {
                if owner {
                    engine.stop_book(&share)
                } else {
                    engine.leave_book(&share)
                }
            },
            move |result, cx| {
                this.update(cx, |this, cx| {
                    this.busy = None;
                    match result {
                        Ok(ended) => {
                            this.share_id = None;
                            this.view = None;
                            this.done = Some(ended.message.into());
                        }
                        Err(words) => this.error = Some(words.into()),
                    }
                    cx.notify();
                })
                .ok();
                crate::shared_books::refresh_local(cx);
                crate::collab::sync_soon(cx);
            },
        );
        cx.notify();
    }

    /// Trusts someone's new keys, after their new fingerprint was compared;
    /// then they are invited as a first share would be.
    fn trust_new_keys(&mut self, offered: Person, role: Role, cx: &mut Context<Self>) {
        self.busy = Some(BookBusy::LookingUp);
        self.error = None;
        let this = cx.entity().downgrade();
        crate::collab::run(
            cx,
            move |engine| engine.accept_new_key(&offered.username, &offered.fingerprint),
            move |result, cx| {
                this.update(cx, |this, cx| {
                    this.busy = None;
                    match result {
                        Ok(person) => this.step = BookStep::Verify { person, role },
                        Err(words) => this.error = Some(words.into()),
                    }
                    cx.notify();
                })
                .ok();
            },
        );
        cx.notify();
    }

    fn back(&mut self, cx: &mut Context<Self>) {
        self.step = BookStep::Main;
        self.error = None;
        cx.notify();
    }
}

impl BookShareSheet {
    fn render_member(
        &self,
        ix: usize,
        member: &BookMemberView,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = Colors::get(cx);
        let busy = self.busy.is_some();
        let actions = member_actions(self.my_role(), member);
        let detail = member_detail(member);
        let gone = member.standing == kanso_collab::MemberStanding::Left;
        let control: AnyElement = if actions.any() {
            let view = cx.entity();
            let current = member.role;
            let (account, username) = (member.account.clone(), member.username.clone());
            Button::new(("book-member-role", ix))
                .outline()
                .small()
                .label(current.label())
                .dropdown_caret(true)
                .disabled(busy)
                .debug_selector(move || format!("book-member-role-{ix}"))
                .dropdown_menu(move |menu, _, _| {
                    let menu = actions.roles.iter().fold(menu, |menu, &role| {
                        let view = view.clone();
                        let account = account.clone();
                        menu.item(
                            PopupMenuItem::new(role.label())
                                .checked(role == current)
                                .on_click(move |_, _, cx| {
                                    if role != current {
                                        view.update(cx, |this, cx| {
                                            this.set_role(account.clone(), role, cx)
                                        });
                                    }
                                }),
                        )
                    });
                    if actions.remove {
                        let view = view.clone();
                        let (account, username) = (account.clone(), username.clone());
                        menu.separator()
                            .item(PopupMenuItem::new("Remove from notebook…").on_click(
                                move |_, _, cx| {
                                    view.update(cx, |this, cx| {
                                        this.ask_remove(account.clone(), username.clone(), cx)
                                    });
                                },
                            ))
                    } else {
                        menu
                    }
                })
                .into_any_element()
        } else {
            div()
                .text_size(px(12.5))
                .text_color(colors.text_secondary)
                .child(member.role.label())
                .into_any_element()
        };
        let dark = cx.theme().is_dark();
        h_flex()
            .id(("book-member", ix))
            .debug_selector(move || format!("book-member-{ix}"))
            .gap_3()
            .py(px(4.))
            .when(gone, |el| el.opacity(0.5))
            .child(
                div()
                    .flex_none()
                    .size(px(24.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .bg(cursor_colour(
                        kanso_collab::engine::colour_of(&member.account),
                        dark,
                    ))
                    .text_size(px(11.5))
                    .font_semibold()
                    .text_color(colors.editor)
                    .child(initial(&member.username)),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .text_size(px(13.5))
                            .text_color(colors.text)
                            .truncate()
                            .child(member.username.clone()),
                    )
                    .when(!detail.is_empty(), |el| {
                        el.child(quiet(detail.join(" · "), colors))
                    }),
            )
            .child(control)
            .into_any_element()
    }

    fn render_main(&mut self, cx: &mut Context<Self>) -> Div {
        let colors = Colors::get(cx);
        let busy = self.busy.is_some();
        let roles = invite_roles(self.my_role());
        let mut body = v_flex().gap(px(12.));
        match &self.view {
            None => {
                body = body.child(paragraph(
                    "Invite people by their Kanso username. Everyone here sees every note in this notebook, its board, schedule and templates. It stays end-to-end encrypted: your sync server passes it on without reading it.",
                    colors,
                ));
                if let Some(words) = locked_words(self.locked, &self.name) {
                    body = body.child(
                        h_flex()
                            .gap_2()
                            .px(px(12.))
                            .py(px(8.))
                            .rounded(px(6.))
                            .bg(colors.fill)
                            .text_size(px(13.))
                            .line_height(px(19.))
                            .text_color(colors.text)
                            .debug_selector(|| "book-share-locked".into())
                            .child(
                                Icon::new(Lucide::Lock)
                                    .size(px(15.))
                                    .text_color(colors.text_secondary),
                            )
                            .child(div().flex_1().child(words)),
                    );
                }
            }
            Some(view) if !view.my_role.is_some_and(|r| r.may_manage()) => {
                body = body.child(paragraph(
                    format!(
                        "{} shared this notebook with you. You {}.",
                        view.owner_name,
                        match view.my_role {
                            Some(Role::View) => "can read its notes",
                            Some(Role::Comment) => "can read its notes and comment on them",
                            _ => "can edit its notes",
                        }
                    ),
                    colors,
                ));
            }
            Some(_) => {}
        }
        if self.view.is_some() {
            let collab = cx.try_global::<Collab>();
            let offline = collab.is_some_and(|c| c.offline);
            let syncing = collab.is_some_and(|c| c.syncing);
            let waiting = collab
                .and_then(|c| c.book(&self.book_id))
                .map_or(0, |b| b.waiting);
            body = body.child(
                h_flex()
                    .gap(px(6.))
                    .text_size(px(12.5))
                    .text_color(if offline {
                        colors.warning.into()
                    } else {
                        Hsla::from(colors.text_secondary)
                    })
                    .debug_selector(|| "book-share-status".into())
                    .child(
                        Icon::new(if offline {
                            Lucide::CloudOff
                        } else {
                            Lucide::CloudCheck
                        })
                        .size(px(14.)),
                    )
                    .child(status_words(offline, syncing, waiting)),
            );
        }
        if !roles.is_empty() {
            let options: Vec<(Role, String)> =
                roles.iter().map(|r| (*r, r.label().to_owned())).collect();
            body = body
                .child(
                    h_flex()
                        .gap_2()
                        .child(div().flex_1().child(Input::new(&self.username).small()))
                        .child(
                            Button::new("book-invite")
                                .primary()
                                .small()
                                .label(match self.busy {
                                    Some(BookBusy::LookingUp) => "Looking up…",
                                    Some(BookBusy::Inviting) => "Inviting…",
                                    Some(BookBusy::Sharing { .. }) => "Sharing…",
                                    _ if self.view.is_none() => "Share",
                                    _ => "Invite",
                                })
                                .loading(matches!(
                                    self.busy,
                                    Some(
                                        BookBusy::LookingUp
                                            | BookBusy::Inviting
                                            | BookBusy::Sharing { .. }
                                    )
                                ))
                                .disabled(busy)
                                .debug_selector(|| "book-invite".into())
                                .on_click(
                                    cx.listener(|this, _, window, cx| this.invite(window, cx)),
                                ),
                        ),
                )
                .child(
                    h_flex()
                        .gap_3()
                        .items_center()
                        .child(
                            div()
                                .text_size(px(13.))
                                .text_color(colors.text)
                                .child("They"),
                        )
                        .child(choices(
                            "book-role",
                            &options,
                            self.role,
                            busy,
                            colors,
                            cx,
                            |this: &mut BookShareSheet, role, cx| {
                                this.role = role;
                                cx.notify();
                            },
                        )),
                );
        }
        if let Some(view) = self.view.clone() {
            body = body.child(heading("People", colors)).child(
                v_flex().children(
                    view.members
                        .iter()
                        .enumerate()
                        .map(|(ix, m)| self.render_member(ix, m, cx))
                        .collect::<Vec<_>>(),
                ),
            );
        }
        body
    }

    fn render_confirm(&self, title: String, words: String, cx: &App) -> Div {
        let colors = Colors::get(cx);
        v_flex()
            .gap(px(8.))
            .debug_selector(|| "book-share-confirm".into())
            .child(
                div()
                    .text_size(px(14.))
                    .font_semibold()
                    .text_color(colors.text)
                    .child(title),
            )
            .child(paragraph(words, colors))
    }
}

// no-prop-test: drawing; the book share snapshots show each step, and
// `book_share_model` carries the logic.
impl Render for BookShareSheet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let body = match self.step.clone() {
            BookStep::SignedOut => v_flex().gap(px(14.)).child(paragraph(
                "Sharing a notebook lets people you choose read, comment on or edit its notes with you, end-to-end encrypted. It needs a Kanso account: sign in, or create one, then share.",
                colors,
            )),
            BookStep::NotShareable(reason) => {
                v_flex().gap(px(14.)).child(paragraph(reason, colors))
            }
            BookStep::Main => self.render_main(cx),
            BookStep::Verify { person, .. } => v_flex()
                .gap(px(12.))
                .child(paragraph(
                    format!(
                        "You're sharing with {} for the first time. Compare this fingerprint with the one they see in Settings › Account, in person or on a call. If it matches, nobody is standing between you.",
                        person.username
                    ),
                    colors,
                ))
                .child(fingerprint_block(&person.fingerprint, cx)),
            BookStep::KeyChanged {
                pinned, offered, ..
            } => v_flex()
                .gap(px(12.))
                .child(
                    h_flex()
                        .gap_2()
                        .px(px(12.))
                        .py(px(8.))
                        .rounded(px(6.))
                        .bg(Hsla::from(colors.warning).opacity(0.14))
                        .text_size(px(13.))
                        .line_height(px(19.))
                        .text_color(colors.text)
                        .child(
                            Icon::new(Lucide::ShieldAlert)
                                .size(px(16.))
                                .text_color(colors.warning),
                        )
                        .child(div().flex_1().child(format!(
                            "{}'s keys changed since you last shared with them. That happens when they set up their account again, and also when someone tries to read what you share. Ask them which fingerprint they see now before you trust it.",
                            offered.username
                        ))),
                )
                .when_some(pinned, |el, pinned| {
                    el.child(quiet("Before", colors))
                        .child(fingerprint_block(&pinned.fingerprint, cx))
                })
                .child(quiet("Now", colors))
                .child(fingerprint_block(&offered.fingerprint, cx)),
            BookStep::ConfirmRemove { username, .. } => self.render_confirm(
                remove_title(&username, &self.name),
                remove_body(&username),
                cx,
            ),
            BookStep::ConfirmEnd => {
                if self.my_role() == Some(Role::Owner) {
                    self.render_confirm(
                        stop_title(&self.name),
                        "Everyone in it, you included, keeps a private copy of its notes. Nobody's edits reach the others any more.".into(),
                        cx,
                    )
                } else {
                    self.render_confirm(
                        leave_title(&self.name),
                        "Your copy is a private notebook from now on. To come back, someone in it invites you again.".into(),
                        cx,
                    )
                }
            }
        };
        body.debug_selector(|| "book-share-sheet".into())
            .when_some(self.busy, |el, busy| {
                el.child(
                    h_flex()
                        .gap(px(6.))
                        .text_size(px(13.))
                        .text_color(colors.text_secondary)
                        .debug_selector(|| "book-share-busy".into())
                        .child(
                            Icon::new(Lucide::RefreshCw)
                                .size(px(13.))
                                .text_color(colors.text_secondary),
                        )
                        .child(busy_words(busy, &self.name)),
                )
            })
            .when_some(self.done.clone(), |el, done| {
                el.child(
                    div()
                        .text_size(px(13.))
                        .line_height(px(19.))
                        .text_color(colors.text_secondary)
                        .debug_selector(|| "book-share-done".into())
                        .child(done),
                )
            })
            .when_some(self.error.clone(), |el, error| {
                el.child(
                    div()
                        .text_size(px(13.))
                        .line_height(px(19.))
                        .text_color(colors.danger)
                        .debug_selector(|| "book-share-error".into())
                        .child(error),
                )
            })
    }
}

/// The sheet's buttons, which follow its step.
pub struct BookShareFooter {
    view: Entity<BookShareSheet>,
    _observing: Subscription,
}

// no-prop-test: drawing; the book share snapshots show each step's buttons.
impl Render for BookShareFooter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sheet = self.view.read(cx);
        let busy = sheet.busy.is_some();
        let target = self.view.clone();
        let row = h_flex().w_full().gap_2().items_center();
        let close = |label: &'static str| {
            Button::new("book-share-close")
                .label(label)
                .disabled(busy)
                .debug_selector(|| "book-share-close".into())
                .on_click(|_, window, cx| window.close_dialog(cx))
        };
        let owner = sheet.my_role() == Some(Role::Owner);
        match sheet.step.clone() {
            BookStep::SignedOut => row.child(div().flex_1()).child(close("Cancel")).child(
                Button::new("book-share-sign-in")
                    .primary()
                    .label("Sign in…")
                    .on_click(|_, window, cx| {
                        window.close_dialog(cx);
                        window.dispatch_action(Box::new(crate::workspace::ShowSignIn), cx);
                    }),
            ),
            BookStep::NotShareable(_) => row.child(div().flex_1()).child(close("Close")),
            BookStep::Main => {
                let shared = sheet.share_id.is_some() && sheet.view.is_some();
                let ending = matches!(sheet.busy, Some(BookBusy::Leaving | BookBusy::Stopping));
                row.when(shared, |row| {
                    row.child(
                        Button::new("book-share-end")
                            .ghost()
                            .icon(if owner { Lucide::Ban } else { Lucide::LogOut })
                            .label(if owner { "Stop sharing" } else { "Leave" })
                            .loading(ending)
                            .disabled(busy)
                            .debug_selector(|| "book-share-end".into())
                            .on_click(move |_, _, cx| {
                                target.update(cx, |view, cx| view.ask_end(cx))
                            }),
                    )
                })
                .child(div().flex_1())
                .child(close("Done"))
            }
            BookStep::Verify { person, role } => {
                let (verify, plain) = (person.username.clone(), person.username.clone());
                let other = self.view.clone();
                let marked = self.view.clone();
                row.child(
                    Button::new("book-share-back")
                        .ghost()
                        .label("Back")
                        .disabled(busy)
                        .on_click(move |_, _, cx| target.update(cx, |view, cx| view.back(cx))),
                )
                .child(div().flex_1())
                .child(
                    Button::new("book-invite-unverified")
                        .label("Invite")
                        .disabled(busy)
                        .debug_selector(|| "book-invite-unverified".into())
                        .on_click(move |_, _, cx| {
                            other.update(cx, |view, cx| {
                                view.send_invite(plain.clone(), role, false, cx)
                            })
                        }),
                )
                .child(
                    Button::new("book-verify")
                        .primary()
                        .label("Mark as verified and invite")
                        .disabled(busy)
                        .debug_selector(|| "book-verify".into())
                        .on_click(move |_, _, cx| {
                            marked.update(cx, |view, cx| {
                                view.send_invite(verify.clone(), role, true, cx)
                            })
                        }),
                )
            }
            BookStep::KeyChanged { offered, role, .. } => {
                let trust = self.view.clone();
                row.child(div().flex_1())
                    .child(
                        Button::new("book-share-back")
                            .label("Don't invite")
                            .disabled(busy)
                            .on_click(move |_, _, cx| target.update(cx, |view, cx| view.back(cx))),
                    )
                    .child(
                        Button::new("book-trust")
                            .primary()
                            .label("Trust the new keys")
                            .disabled(busy)
                            .on_click(move |_, _, cx| {
                                trust.update(cx, |view, cx| {
                                    view.trust_new_keys(offered.clone(), role, cx)
                                })
                            }),
                    )
            }
            BookStep::ConfirmRemove { account, username } => {
                let remove = self.view.clone();
                row.child(div().flex_1())
                    .child(
                        Button::new("book-share-back")
                            .label("Cancel")
                            .disabled(busy)
                            .on_click(move |_, _, cx| target.update(cx, |view, cx| view.back(cx))),
                    )
                    .child(
                        Button::new("book-remove")
                            .danger()
                            .label("Remove")
                            .disabled(busy)
                            .debug_selector(|| "book-remove".into())
                            .on_click(move |_, _, cx| {
                                remove.update(cx, |view, cx| {
                                    view.remove(account.clone(), username.clone(), cx)
                                })
                            }),
                    )
            }
            BookStep::ConfirmEnd => {
                let end = self.view.clone();
                row.child(div().flex_1())
                    .child(
                        Button::new("book-share-back")
                            .label("Cancel")
                            .disabled(busy)
                            .on_click(move |_, _, cx| target.update(cx, |view, cx| view.back(cx))),
                    )
                    .child(
                        Button::new("book-end-confirm")
                            .danger()
                            .label(if owner { "Stop sharing" } else { "Leave" })
                            .disabled(busy)
                            .debug_selector(|| "book-end-confirm".into())
                            .on_click(move |_, _, cx| end.update(cx, |view, cx| view.end(cx))),
                    )
            }
        }
    }
}

/// Opens the Share notebook sheet for `book_id`. Returns the sheet, for
/// tests and snapshots to drive.
pub fn open_book_share(
    book_id: String,
    window: &mut Window,
    cx: &mut App,
) -> Option<Entity<BookShareSheet>> {
    let store = cx.try_global::<crate::workspace::AppState>()?.store.clone();
    let name = store
        .books()
        .ok()
        .and_then(|books| books.into_iter().find(|b| b.id == book_id))
        .map(|b| b.name)
        .unwrap_or_default();
    let engine = crate::collab::engine(cx);
    let share_id = engine.as_ref().and_then(|e| {
        e.shared_books().ok().and_then(|books| {
            books
                .into_iter()
                .find(|b| b.book_id == book_id && b.status == kanso_store::SharedStatus::Active)
                .map(|b| b.share_id)
        })
    });
    let (unshareable, locked) = match (&engine, &share_id) {
        (Some(engine), None) => match engine.check_book_shareable(&book_id) {
            Ok(check) if check.eligible.is_empty() && check.locked.is_empty() => (
                Some("This notebook has no notes to share yet. Write one first.".to_owned()),
                0,
            ),
            Ok(check) => (None, check.locked.len()),
            Err(err) => (Some(crate::collab::error_text(&err)), 0),
        },
        _ => (None, 0),
    };
    let step = first_step(engine.is_some(), unshareable);
    let view = share_id
        .as_ref()
        .and_then(|share| engine.as_ref().and_then(|e| e.book_view(share).ok()));
    let username = cx.new(|cx| InputState::new(window, cx).placeholder("Username"));
    let sheet = cx.new(|cx| {
        let subscriptions = vec![cx.subscribe_in(
            &username,
            window,
            |this: &mut BookShareSheet, _, event: &InputEvent, window, cx| {
                if let InputEvent::PressEnter { .. } = event {
                    this.invite(window, cx);
                }
            },
        )];
        let mut sheet = BookShareSheet {
            book_id,
            name,
            step,
            share_id,
            view,
            username,
            role: Role::Edit,
            locked,
            busy: None,
            error: None,
            done: None,
            _subscriptions: subscriptions,
        };
        sheet.reload(cx);
        sheet
    });
    let footer = cx.new(|cx| BookShareFooter {
        view: sheet.clone(),
        _observing: cx.observe(&sheet, |_, _, cx| cx.notify()),
    });
    let dialog_view = sheet.clone();
    let title = sheet.read(cx).name.clone();
    window.open_alert_dialog(cx, move |alert, _, _| {
        let running = dialog_view.clone();
        alert
            .title(if title.trim().is_empty() {
                "Share notebook".to_owned()
            } else {
                format!("Share “{}”", title.trim())
            })
            .width(px(BOOK_SHEET_WIDTH))
            .child(dialog_view.clone())
            .footer(footer.clone())
            // Escape closes, but not while a step runs.
            .on_cancel(move |_, _, cx| !running.read(cx).is_busy())
    });
    Some(sheet)
}

#[cfg(test)]
impl BookShareSheet {
    /// Types a username into the invite field, as a person would.
    pub fn type_username(&mut self, name: &str, window: &mut Window, cx: &mut Context<Self>) {
        let name = name.to_owned();
        self.username
            .update(cx, |input, cx| input.set_value(name, window, cx));
    }
}

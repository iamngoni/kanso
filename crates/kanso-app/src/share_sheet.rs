//! The Share sheet: who a note is shared with and how (docs/ACCOUNTS.md ›
//! Collaboration). The owner invites people by username with a role,
//! changes roles, removes people, lets in those who asked through a link,
//! makes and cancels invite links, and stops sharing; anyone else sees who
//! is in the note and can leave it.
//!
//! The first share with someone shows their fingerprint to compare, and
//! marks them verified when it matches. When someone's keys changed since
//! they were pinned, the sheet shows both fingerprints and waits for the
//! new ones to be trusted. Every step that reaches the server runs off the
//! main thread; its error shows in the sheet.

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonGroup, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, Selectable as _, Sizable as _, StyledExt as _,
    WindowExt as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_collab::{CollabError, Engine, LinkView, MemberView, Person, RequestView, Role};

use crate::clock::Clock;
use crate::collab_model::{Lookup, MadeLink, SheetBusy, SheetStep};
use crate::collab_text::{days_until, fingerprint_groups, link_label};
use crate::editor::NoteEditor;
use crate::publish::paragraph;
use crate::theme::Colors;
use crate::zoom::px;

/// The sheet's width.
pub const SHEET_WIDTH: f32 = 520.;
/// The roles a member can be given.
pub const ROLES: [Role; 3] = [Role::View, Role::Comment, Role::Edit];
/// How long an invite link can last, in days.
pub const LINK_DAYS: [i64; 3] = [1, 7, 30];

/// What the sheet first shows for a note. Pure.
pub fn first_step(signed_in: bool, unshareable: Option<String>) -> SheetStep {
    match (signed_in, unshareable) {
        (_, Some(reason)) => SheetStep::NotShareable(reason),
        (false, None) => SheetStep::SignedOut,
        (true, None) => SheetStep::Main,
    }
}

/// How a member's row reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemberTone {
    Normal,
    /// They left or declined: dimmed until the owner's device removes them.
    Gone,
    /// Their keys couldn't be checked: a warning tint.
    Unchecked,
}

/// The words under a member's name, and how the row reads. Pure.
pub fn member_status(member: &MemberView, mine: bool) -> (Vec<&'static str>, MemberTone) {
    let mut detail = Vec::new();
    if member.left {
        // Someone who never accepted declined; anyone else left.
        detail.push(if member.invited { "Declined" } else { "Left" });
        return (detail, MemberTone::Gone);
    }
    if member.invited {
        detail.push("Invited, hasn't accepted yet");
    }
    if mine {
        detail.push("You");
    } else if member.unverified {
        detail.push("Can't be checked");
        return (detail, MemberTone::Unchecked);
    } else if member.person.verified {
        detail.push("Fingerprint verified");
    }
    (detail, MemberTone::Normal)
}

/// Whether this account's role lets it invite, change roles and make
/// links: the owner's, or a note not shared yet. Pure.
pub fn manages(role: Option<Role>) -> bool {
    role.is_none_or(|role| role == Role::Owner)
}

pub struct ShareSheet {
    editor: Entity<NoteEditor>,
    note_id: String,
    title: String,
    pub(crate) step: SheetStep,
    pub(crate) share_id: Option<String>,
    pub(crate) view: Option<kanso_collab::ShareView>,
    username: Entity<InputState>,
    /// The role the next invitation gives.
    role: Role,
    link_role: Role,
    link_days: i64,
    pub(crate) made_link: Option<MadeLink>,
    pub(crate) busy: Option<SheetBusy>,
    pub(crate) error: Option<SharedString>,
    /// What the last step did, when it is worth saying.
    pub(crate) done: Option<SharedString>,
    copied: bool,
    /// Stop sharing or Leave was pressed once; the next press does it.
    confirm_end: bool,
    _subscriptions: Vec<Subscription>,
}

impl ShareSheet {
    pub fn is_busy(&self) -> bool {
        self.busy.is_some()
    }

    fn role(&self) -> Option<Role> {
        self.view.as_ref().map(|v| v.role)
    }

    /// Reads the note's sharing from the server again.
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        let Some(share) = self.share_id.clone() else {
            return;
        };
        self.busy = Some(SheetBusy::Loading);
        let this = cx.entity().downgrade();
        crate::collab::run(
            cx,
            move |engine| {
                engine.refresh()?;
                Ok(engine.share_view(&share))
            },
            move |result, cx| {
                this.update(cx, |this, cx| this.finish(result.map(Some), cx))
                    .ok();
            },
        );
        cx.notify();
    }

    /// Looks the typed username up, then invites them, or first shows
    /// their fingerprint (a first share) or their changed keys.
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
        // Shared first from the text as typed now.
        self.editor.update(cx, |editor, cx| editor.flush(cx));
        let role = self.role;
        self.busy = Some(SheetBusy::LookingUp);
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
                            this.step = SheetStep::Verify { person, role };
                            cx.notify();
                        }
                        Ok(Lookup::Changed { pinned, offered }) => {
                            this.step = SheetStep::KeyChanged {
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

    /// Shares the note with someone whose keys are pinned (the first share
    /// makes the note shared), marking their fingerprint verified first
    /// when it was compared.
    fn send_invite(&mut self, username: String, role: Role, verify: bool, cx: &mut Context<Self>) {
        let note_id = self.note_id.clone();
        let share = self.share_id.clone();
        self.busy = Some(SheetBusy::Inviting);
        self.error = None;
        self.step = SheetStep::Main;
        let this = cx.entity().downgrade();
        let invited = username.clone();
        crate::collab::run(
            cx,
            move |engine| {
                if verify {
                    let person = engine.person(&username)?;
                    engine.verify_person(&person.account_id)?;
                }
                let share = match share {
                    Some(share) => {
                        engine.invite(&share, &username, role)?;
                        share
                    }
                    None => engine.share(&note_id, &[(username, role)])?,
                };
                // The list as the server has it now; the invite stands
                // even if this read fails.
                let _ = engine.refresh();
                Ok((share.clone(), engine.share_view(&share)))
            },
            move |result, cx| {
                this.update(cx, |this, cx| match result {
                    Ok((share, view)) => {
                        crate::metrics::record(cx, crate::metrics::Feature::SharedNote);
                        this.share_id = Some(share);
                        this.done = Some(
                            format!("Invited {invited}. The note shows for them once they accept.")
                                .into(),
                        );
                        this.finish(Ok(Some(view)), cx);
                        crate::collab::sync_soon(cx);
                    }
                    Err(words) => this.finish(Err(words), cx),
                })
                .ok();
            },
        );
        cx.notify();
    }

    /// Runs a step on the shared note, then shows its sharing as it is now.
    fn act(
        &mut self,
        busy: SheetBusy,
        cx: &mut Context<Self>,
        work: impl FnOnce(&Engine, &str) -> kanso_collab::api::Result<()> + Send + 'static,
    ) {
        let Some(share) = self.share_id.clone() else {
            return;
        };
        if self.busy.is_some() {
            return;
        }
        self.busy = Some(busy);
        self.error = None;
        self.done = None;
        let this = cx.entity().downgrade();
        crate::collab::run(
            cx,
            move |engine| {
                work(engine, &share)?;
                let _ = engine.refresh();
                Ok(engine.share_view(&share))
            },
            move |result, cx| {
                this.update(cx, |this, cx| this.finish(result.map(Some), cx))
                    .ok();
            },
        );
        cx.notify();
    }

    /// Keeps what a step brought back: the note's sharing as it is now.
    pub(crate) fn finish(
        &mut self,
        result: Result<Option<Option<kanso_collab::ShareView>>, String>,
        cx: &mut Context<Self>,
    ) {
        self.busy = None;
        self.confirm_end = false;
        match result {
            Ok(Some(view)) => {
                if view.is_some() {
                    self.view = view;
                }
            }
            Ok(None) => {}
            Err(words) => self.error = Some(words.into()),
        }
        cx.notify();
    }

    fn set_role(&mut self, account: String, role: Role, cx: &mut Context<Self>) {
        self.act(SheetBusy::Changing, cx, move |engine, share| {
            engine.set_role(share, &account, role)
        });
    }

    fn remove(&mut self, account: String, cx: &mut Context<Self>) {
        self.act(SheetBusy::Changing, cx, move |engine, share| {
            engine.remove(share, &account)
        });
    }

    fn approve(&mut self, account: String, cx: &mut Context<Self>) {
        self.act(SheetBusy::Changing, cx, move |engine, share| {
            engine.approve(share, &account)
        });
    }

    fn decline_request(&mut self, account: String, cx: &mut Context<Self>) {
        self.act(SheetBusy::Changing, cx, move |engine, share| {
            engine.decline_request(share, &account)
        });
    }

    fn cancel_link(&mut self, link_id: String, cx: &mut Context<Self>) {
        self.act(SheetBusy::Changing, cx, move |engine, share| {
            engine.cancel_link(share, &link_id)
        });
    }

    /// Makes an invite link with the chosen role and expiry. Its secret is
    /// only known here: the link shows until the sheet closes.
    fn make_link(&mut self, cx: &mut Context<Self>) {
        let Some(share) = self.share_id.clone() else {
            return;
        };
        if self.busy.is_some() {
            return;
        }
        let (role, days) = (self.link_role, self.link_days);
        self.busy = Some(SheetBusy::MakingLink);
        self.error = None;
        self.copied = false;
        let this = cx.entity().downgrade();
        crate::collab::run(
            cx,
            move |engine| {
                let url = engine.create_link(&share, role, days)?;
                Ok((url, engine.share_view(&share)))
            },
            move |result, cx| {
                this.update(cx, |this, cx| match result {
                    Ok((url, view)) => {
                        this.made_link = Some(MadeLink { url, role, days });
                        this.finish(Ok(Some(view)), cx);
                    }
                    Err(words) => this.finish(Err(words), cx),
                })
                .ok();
            },
        );
        cx.notify();
    }

    fn copy_link(&mut self, cx: &mut Context<Self>) {
        if let Some(link) = &self.made_link {
            cx.write_to_clipboard(ClipboardItem::new_string(link.url.clone()));
            self.copied = true;
            cx.notify();
        }
    }

    /// Stops sharing (the owner) or leaves the note (anyone else): the
    /// first press asks, the second does it. Everyone keeps a private copy.
    pub fn end(&mut self, cx: &mut Context<Self>) {
        let Some(share) = self.share_id.clone() else {
            return;
        };
        if self.busy.is_some() {
            return;
        }
        if !self.confirm_end {
            self.confirm_end = true;
            cx.notify();
            return;
        }
        let owner = self.role() == Some(Role::Owner);
        self.busy = Some(SheetBusy::Stopping);
        self.error = None;
        let this = cx.entity().downgrade();
        crate::collab::run(
            cx,
            move |engine| {
                if owner {
                    engine.stop(&share)
                } else {
                    engine.leave(&share)
                }
            },
            move |result, cx| {
                this.update(cx, |this, cx| {
                    this.busy = None;
                    this.confirm_end = false;
                    match result {
                        Ok(()) => {
                            this.share_id = None;
                            this.view = None;
                            this.made_link = None;
                            this.done = Some(if owner {
                                "Sharing stopped. Everyone keeps a private copy of the note as it is."
                            } else {
                                "You left the note. Your copy of it is private now."
                            }.into());
                        }
                        Err(words) => this.error = Some(words.into()),
                    }
                    cx.notify();
                })
                .ok();
            },
        );
        cx.notify();
    }

    /// Trusts someone's new keys, after their new fingerprint was
    /// compared; then they are invited as a first share would be.
    fn trust_new_keys(&mut self, offered: Person, role: Role, cx: &mut Context<Self>) {
        self.busy = Some(SheetBusy::LookingUp);
        self.error = None;
        let this = cx.entity().downgrade();
        crate::collab::run(
            cx,
            move |engine| engine.accept_new_key(&offered.username, &offered.fingerprint),
            move |result, cx| {
                this.update(cx, |this, cx| {
                    this.busy = None;
                    match result {
                        Ok(person) => this.step = SheetStep::Verify { person, role },
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
        self.step = SheetStep::Main;
        self.error = None;
        cx.notify();
    }
}

/// Looks someone up by username: their pinned keys, or the keys the
/// server offers now that they changed.
fn look_up(engine: &Engine, name: &str) -> kanso_collab::api::Result<Lookup> {
    match engine.person(name) {
        Ok(person) => Ok(Lookup::Found(person)),
        Err(CollabError::KeyChanged(username)) => {
            let offered = engine.offered_key(&username)?;
            let pinned = engine.person_of(&offered.account_id).ok();
            Ok(Lookup::Changed { pinned, offered })
        }
        Err(err) => Err(err),
    }
}

/// A fingerprint as six groups, in the monospaced face.
fn fingerprint_block(fingerprint: &str, cx: &App) -> Div {
    let colors = Colors::get(cx);
    h_flex()
        .flex_wrap()
        .gap(px(10.))
        .px(px(12.))
        .py(px(9.))
        .rounded(px(6.))
        .bg(colors.fill)
        .font_family(cx.theme().mono_font_family.clone())
        .text_size(px(14.))
        .text_color(colors.text)
        .debug_selector(|| "share-fingerprint".into())
        .children(
            fingerprint_groups(fingerprint)
                .into_iter()
                .map(|g| div().child(g)),
        )
}

/// A row's quiet second line.
fn quiet(text: impl Into<SharedString>, colors: &Colors) -> Div {
    div()
        .text_size(px(12.))
        .line_height(px(16.))
        .text_color(colors.text_muted)
        .child(text.into())
}

fn heading(text: &'static str, colors: &Colors) -> Div {
    div()
        .pt(px(4.))
        .text_size(px(11.5))
        .font_semibold()
        .text_color(colors.text_secondary)
        .child(text)
}

/// A row of choices, one selected.
fn choices<V: 'static, T: Copy + PartialEq + 'static>(
    id: &'static str,
    options: &[(T, String)],
    current: T,
    disabled: bool,
    colors: &Colors,
    cx: &mut Context<V>,
    pick: fn(&mut V, T, &mut Context<V>),
) -> impl IntoElement {
    let values: Vec<T> = options.iter().map(|(v, _)| *v).collect();
    ButtonGroup::new(id)
        .outline()
        .small()
        .disabled(disabled)
        .children(options.iter().enumerate().map(|(ix, (value, label))| {
            let selected = *value == current;
            Button::new((id, ix))
                .selected(selected)
                .debug_selector(move || format!("{id}-{ix}"))
                .child(
                    div()
                        .px_1()
                        .text_size(px(12.5))
                        .when(selected, |el| el.font_medium())
                        .text_color(if selected {
                            colors.text
                        } else {
                            colors.text_secondary
                        })
                        .child(label.clone()),
                )
        }))
        .on_click(cx.listener(move |this, clicked: &Vec<usize>, _, cx| {
            if let Some(&ix) = clicked.first() {
                pick(this, values[ix], cx);
            }
        }))
}

fn role_options() -> Vec<(Role, String)> {
    ROLES.iter().map(|r| (*r, r.label().to_owned())).collect()
}

impl ShareSheet {
    fn render_member(
        &self,
        ix: usize,
        member: &MemberView,
        manage: bool,
        me: &str,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = Colors::get(cx);
        let busy = self.busy.is_some();
        let person = &member.person;
        let mine = person.username == me;
        let (detail, tone) = member_status(member, mine);
        let account = person.account_id.clone();
        let role_control: AnyElement =
            if manage && member.role != Role::Owner && !mine && tone != MemberTone::Gone {
                let view = cx.entity();
                let current = member.role;
                let account = account.clone();
                Button::new(("member-role", ix))
                    .outline()
                    .small()
                    .label(current.label())
                    .dropdown_caret(true)
                    .disabled(busy)
                    .dropdown_menu(move |menu, _, _| {
                        ROLES
                            .iter()
                            .fold(menu, |menu, &role| {
                                let view = view.clone();
                                let account = account.clone();
                                menu.item(
                                    PopupMenuItem::new(role.label())
                                        .checked(role == current)
                                        .on_click(move |_, _, cx| {
                                            view.update(cx, |this, cx| {
                                                this.set_role(account.clone(), role, cx)
                                            });
                                        }),
                                )
                            })
                            .separator()
                            .item({
                                let view = view.clone();
                                let account = account.clone();
                                PopupMenuItem::new("Remove from note").on_click(move |_, _, cx| {
                                    view.update(cx, |this, cx| this.remove(account.clone(), cx));
                                })
                            })
                    })
                    .into_any_element()
            } else {
                div()
                    .text_size(px(12.5))
                    .text_color(colors.text_secondary)
                    .child(member.role.label())
                    .into_any_element()
            };
        let warning = Hsla::from(colors.warning);
        h_flex()
            .id(("member", ix))
            .gap_3()
            .py(px(4.))
            .when(tone == MemberTone::Gone, |el| el.opacity(0.5))
            .child(
                Icon::new(match tone {
                    MemberTone::Unchecked => Lucide::ShieldAlert,
                    _ if person.verified || mine => Lucide::ShieldCheck,
                    _ => Lucide::CircleUser,
                })
                .size(px(16.))
                .text_color(if tone == MemberTone::Unchecked {
                    warning
                } else {
                    colors.text_secondary.into()
                }),
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
                            .child(person.username.clone()),
                    )
                    .when(!detail.is_empty(), |el| {
                        let words = detail.join(" · ");
                        if tone == MemberTone::Unchecked {
                            el.child(quiet(words, colors).text_color(warning))
                        } else {
                            el.child(quiet(words, colors))
                        }
                    }),
            )
            .child(role_control)
            .into_any_element()
    }

    fn render_request(
        &self,
        ix: usize,
        request: &RequestView,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = Colors::get(cx);
        let busy = self.busy.is_some();
        let (approve, decline) = (
            request.person.account_id.clone(),
            request.person.account_id.clone(),
        );
        h_flex()
            .id(("request", ix))
            .gap_3()
            .py(px(4.))
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .text_size(px(13.5))
                            .text_color(colors.text)
                            .child(format!("{} asked to join", request.person.username)),
                    )
                    .child(quiet(
                        format!(
                            "{} · Fingerprint {}",
                            request.role.label(),
                            request.person.fingerprint
                        ),
                        colors,
                    )),
            )
            .child(
                Button::new(("request-decline", ix))
                    .ghost()
                    .small()
                    .label("Decline")
                    .disabled(busy)
                    .on_click(
                        cx.listener(move |this, _, _, cx| {
                            this.decline_request(decline.clone(), cx)
                        }),
                    ),
            )
            .child(
                Button::new(("request-approve", ix))
                    .outline()
                    .small()
                    .label("Approve")
                    .disabled(busy)
                    .debug_selector(move || format!("request-approve-{ix}"))
                    .on_click(cx.listener(move |this, _, _, cx| this.approve(approve.clone(), cx))),
            )
            .into_any_element()
    }

    fn render_link(
        &self,
        ix: usize,
        link: &LinkView,
        now: i64,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = Colors::get(cx);
        let id = link.link_id.clone();
        h_flex()
            .id(("link", ix))
            .gap_3()
            .py(px(2.))
            .child(
                Icon::new(Lucide::Link2)
                    .size(px(15.))
                    .text_color(colors.text_secondary),
            )
            .child(
                div()
                    .flex_1()
                    .text_size(px(13.))
                    .text_color(colors.text)
                    .child(link_label(link.role, days_until(link.expires_at, now))),
            )
            .child(
                Button::new(("link-cancel", ix))
                    .ghost()
                    .small()
                    .label("Cancel link")
                    .disabled(self.busy.is_some())
                    .on_click(cx.listener(move |this, _, _, cx| this.cancel_link(id.clone(), cx))),
            )
            .into_any_element()
    }

    fn render_main(&mut self, cx: &mut Context<Self>) -> Div {
        let colors = Colors::get(cx);
        let busy = self.busy.is_some();
        let manage = manages(self.role());
        let me = crate::collab::current(cx)
            .map(|e| e.caller().username.clone())
            .unwrap_or_default();
        let now = Clock::now_ms(cx);
        let mut body = v_flex().gap(px(12.));
        if self.share_id.is_none() {
            body = body.child(paragraph(
                "Invite people by their Kanso username to read, comment on or edit this note with you. It stays end-to-end encrypted: your sync server passes it on without reading it.",
                colors,
            ));
        } else if let Some(view) = &self.view
            && view.role != Role::Owner
        {
            body = body.child(paragraph(
                format!(
                    "{} shared this note with you. You {}.",
                    view.owner.username,
                    match view.role {
                        Role::View => "can read it",
                        Role::Comment => "can read it and comment on it",
                        _ => "can edit it",
                    }
                ),
                colors,
            ));
        }
        if manage {
            body = body
                .child(
                    h_flex()
                        .gap_2()
                        .child(div().flex_1().child(Input::new(&self.username).small()))
                        .child(
                            Button::new("share-invite")
                                .primary()
                                .small()
                                .label(match self.busy {
                                    Some(SheetBusy::LookingUp) => "Looking up…",
                                    Some(SheetBusy::Inviting) => "Inviting…",
                                    _ => "Invite",
                                })
                                .loading(matches!(
                                    self.busy,
                                    Some(SheetBusy::LookingUp | SheetBusy::Inviting)
                                ))
                                .disabled(busy)
                                .debug_selector(|| "share-invite".into())
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
                            "share-role",
                            &role_options(),
                            self.role,
                            busy,
                            colors,
                            cx,
                            |this: &mut ShareSheet, role, cx| {
                                this.role = role;
                                cx.notify();
                            },
                        )),
                );
        }
        if let Some(view) = self.view.clone() {
            let mut people: Vec<MemberView> = vec![MemberView {
                person: view.owner.clone(),
                role: Role::Owner,
                invited: false,
                left: false,
                unverified: false,
            }];
            people.extend(
                view.members
                    .iter()
                    .filter(|m| m.role != Role::Owner)
                    .cloned(),
            );
            body = body.child(heading("People", colors)).child(
                v_flex().children(
                    people
                        .iter()
                        .enumerate()
                        .map(|(ix, m)| self.render_member(ix, m, manage, &me, cx))
                        .collect::<Vec<_>>(),
                ),
            );
            if manage && !view.requests.is_empty() {
                body = body.child(heading("Asked to join", colors)).child(
                    v_flex().children(
                        view.requests
                            .iter()
                            .enumerate()
                            .map(|(ix, r)| self.render_request(ix, r, cx))
                            .collect::<Vec<_>>(),
                    ),
                );
            }
            if manage {
                body = body
                    .child(heading("Invite links", colors))
                    .children(
                        view.links
                            .iter()
                            .enumerate()
                            .map(|(ix, l)| self.render_link(ix, l, now, cx))
                            .collect::<Vec<_>>(),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .flex_wrap()
                            .items_center()
                            .child(choices(
                                "link-role",
                                &role_options(),
                                self.link_role,
                                busy,
                                colors,
                                cx,
                                |this: &mut ShareSheet, role, cx| {
                                    this.link_role = role;
                                    cx.notify();
                                },
                            ))
                            .child(choices(
                                "link-days",
                                &LINK_DAYS
                                    .iter()
                                    .map(|d| (*d, if *d == 1 { "1 day".to_owned() } else { format!("{d} days") }))
                                    .collect::<Vec<_>>(),
                                self.link_days,
                                busy,
                                colors,
                                cx,
                                |this: &mut ShareSheet, days, cx| {
                                    this.link_days = days;
                                    cx.notify();
                                },
                            ))
                            .child(
                                Button::new("link-make")
                                    .outline()
                                    .small()
                                    .label("Make link")
                                    .loading(self.busy == Some(SheetBusy::MakingLink))
                                    .disabled(busy)
                                    .debug_selector(|| "link-make".into())
                                    .on_click(cx.listener(|this, _, _, cx| this.make_link(cx))),
                            ),
                    )
                    .when_some(self.made_link.clone(), |el, link| {
                        el.child(
                            h_flex()
                                .gap_2()
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .px(px(10.))
                                        .py(px(7.))
                                        .rounded(px(6.))
                                        .border_1()
                                        .border_color(colors.divider)
                                        .bg(colors.list)
                                        .font_family(cx.theme().mono_font_family.clone())
                                        .text_size(px(12.))
                                        .text_color(colors.text)
                                        .whitespace_nowrap()
                                        .overflow_hidden()
                                        .text_ellipsis()
                                        .debug_selector(|| "link-url".into())
                                        .child(link.url.clone()),
                                )
                                .child(
                                    Button::new("link-copy")
                                        .outline()
                                        .icon(if self.copied { Lucide::Check } else { Lucide::Copy })
                                        .label(if self.copied { "Copied" } else { "Copy" })
                                        .on_click(cx.listener(|this, _, _, cx| this.copy_link(cx))),
                                ),
                        )
                        .child(quiet(
                            "Anyone with the link can ask to join; you approve each of them here. Copy it now: it can't be shown again.",
                            colors,
                        ))
                    });
            }
        }
        if self.confirm_end {
            let words = if self.role() == Some(Role::Owner) {
                "Stopping sharing ends the note for everyone in it. Each of them keeps a private copy of it as it is now."
            } else {
                "Leaving takes you out of the note. You keep a private copy of it as it is now."
            };
            body = body.child(
                div()
                    .text_size(px(13.))
                    .line_height(px(19.))
                    .text_color(colors.danger)
                    .child(words),
            );
        }
        body
    }
}

// no-prop-test: drawing; the share snapshots show each step, and
// `first_step` and `manages` carry the logic.
impl Render for ShareSheet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let body = match self.step.clone() {
            SheetStep::SignedOut => v_flex().gap(px(14.)).child(paragraph(
                "Sharing lets people you choose read, comment on or edit a note with you, end-to-end encrypted. It needs a Kanso account: sign in, or create one, then share.",
                colors,
            )),
            SheetStep::NotShareable(reason) => v_flex().gap(px(14.)).child(paragraph(reason, colors)),
            SheetStep::Main => self.render_main(cx),
            SheetStep::Verify { person, .. } => v_flex()
                .gap(px(12.))
                .child(paragraph(
                    format!(
                        "You're sharing with {} for the first time. Compare this fingerprint with the one they see in Settings › Account, in person or on a call. If it matches, nobody is standing between you.",
                        person.username
                    ),
                    colors,
                ))
                .child(fingerprint_block(&person.fingerprint, cx)),
            SheetStep::KeyChanged {
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
                        .debug_selector(|| "share-key-changed".into())
                        .child(Icon::new(Lucide::ShieldAlert).size(px(16.)).text_color(colors.warning))
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
        };
        body.debug_selector(|| "share-sheet".into())
            .when_some(self.done.clone(), |el, done| {
                el.child(
                    div()
                        .text_size(px(13.))
                        .line_height(px(19.))
                        .text_color(colors.text_secondary)
                        .debug_selector(|| "share-done".into())
                        .child(done),
                )
            })
            .when_some(self.error.clone(), |el, error| {
                el.child(
                    div()
                        .text_size(px(13.))
                        .line_height(px(19.))
                        .text_color(colors.danger)
                        .debug_selector(|| "share-error".into())
                        .child(error),
                )
            })
    }
}

/// The sheet's buttons, which follow its step.
pub struct ShareFooter {
    view: Entity<ShareSheet>,
    _observing: Subscription,
}

// no-prop-test: drawing; the share snapshots show each step's buttons.
impl Render for ShareFooter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = self.view.read(cx);
        let busy = view.busy.is_some();
        let target = self.view.clone();
        let row = h_flex().w_full().gap_2().items_center();
        let close = |label: &'static str| {
            Button::new("share-close")
                .label(label)
                .disabled(busy)
                .debug_selector(|| "share-close".into())
                .on_click(|_, window, cx| window.close_dialog(cx))
        };
        match view.step.clone() {
            SheetStep::SignedOut => row.child(div().flex_1()).child(close("Cancel")).child(
                Button::new("share-sign-in")
                    .primary()
                    .label("Sign in…")
                    .on_click(|_, window, cx| {
                        window.close_dialog(cx);
                        window.dispatch_action(Box::new(crate::workspace::ShowSignIn), cx);
                    }),
            ),
            SheetStep::NotShareable(_) => row.child(div().flex_1()).child(close("Close")),
            SheetStep::Main => {
                let shared = view.share_id.is_some();
                let owner = view.role() == Some(Role::Owner);
                let confirm = view.confirm_end;
                let stopping = view.busy == Some(SheetBusy::Stopping);
                row.when(shared && view.view.is_some(), |row| {
                    row.child(
                        Button::new("share-end")
                            .ghost()
                            .when(confirm || stopping, |b| b.danger())
                            .label(match (owner, confirm) {
                                (true, false) => "Stop sharing",
                                (true, true) => "Stop sharing for everyone",
                                (false, false) => "Leave",
                                (false, true) => "Leave the note",
                            })
                            .loading(stopping)
                            .disabled(busy && !stopping)
                            .debug_selector(|| "share-end".into())
                            .on_click(move |_, _, cx| target.update(cx, |view, cx| view.end(cx))),
                    )
                })
                .child(div().flex_1())
                .child(close("Done"))
            }
            SheetStep::Verify { person, role } => {
                let (verify, plain) = (person.username.clone(), person.username.clone());
                let other = self.view.clone();
                row.child(
                    Button::new("share-back")
                        .ghost()
                        .label("Back")
                        .disabled(busy)
                        .on_click(move |_, _, cx| target.update(cx, |view, cx| view.back(cx))),
                )
                .child(div().flex_1())
                .child(
                    Button::new("share-invite-unverified")
                        .label("Invite")
                        .disabled(busy)
                        .debug_selector(|| "share-invite-unverified".into())
                        .on_click(move |_, _, cx| {
                            other.update(cx, |view, cx| {
                                view.send_invite(plain.clone(), role, false, cx)
                            })
                        }),
                )
                .child({
                    let view = self.view.clone();
                    Button::new("share-verify")
                        .primary()
                        .label("Mark as verified and invite")
                        .disabled(busy)
                        .debug_selector(|| "share-verify".into())
                        .on_click(move |_, _, cx| {
                            view.update(cx, |view, cx| {
                                view.send_invite(verify.clone(), role, true, cx)
                            })
                        })
                })
            }
            SheetStep::KeyChanged { offered, role, .. } => {
                let trust = self.view.clone();
                row.child(div().flex_1())
                    .child(
                        Button::new("share-back")
                            .label("Don't invite")
                            .disabled(busy)
                            .on_click(move |_, _, cx| target.update(cx, |view, cx| view.back(cx))),
                    )
                    .child(
                        Button::new("share-trust")
                            .primary()
                            .label("Trust the new keys")
                            .disabled(busy)
                            .debug_selector(|| "share-trust".into())
                            .on_click(move |_, _, cx| {
                                trust.update(cx, |view, cx| {
                                    view.trust_new_keys(offered.clone(), role, cx)
                                })
                            }),
                    )
            }
        }
    }
}

/// Opens the Share sheet for the note in `editor`. Returns the sheet, for
/// tests and snapshots to drive.
pub fn open_share(
    editor: Entity<NoteEditor>,
    window: &mut Window,
    cx: &mut App,
) -> Option<Entity<ShareSheet>> {
    let note_id = editor.read(cx).note_id()?.to_owned();
    editor.update(cx, |editor, cx| editor.flush(cx));
    let title = editor
        .read(cx)
        .current_note(cx)
        .map(|n| n.title)
        .unwrap_or_default();
    let engine = crate::collab::engine(cx);
    let share_id = engine.as_ref().and_then(|e| e.share_of(&note_id));
    // A locked note's text stays sealed: sharing it would hand it out.
    let locked = editor.read(cx).lock_view() != crate::locked::LockView::Plain;
    let unshareable = if locked {
        Some(
            "Locked notes, and notes in locked notebooks, can't be shared. Remove the lock first."
                .to_owned(),
        )
    } else if share_id.is_none() {
        engine
            .as_ref()
            .and_then(|e| e.check_shareable(&note_id).err())
            .map(|err| crate::collab::error_text(&err))
    } else {
        None
    };
    let step = first_step(engine.is_some(), unshareable);
    let view = share_id
        .as_ref()
        .and_then(|share| engine.as_ref().and_then(|e| e.share_view(share)));
    let username = cx.new(|cx| InputState::new(window, cx).placeholder("Username"));
    let sheet = cx.new(|cx| {
        let subscriptions = vec![cx.subscribe_in(
            &username,
            window,
            |this: &mut ShareSheet, _, event: &InputEvent, window, cx| {
                if let InputEvent::PressEnter { .. } = event {
                    this.invite(window, cx);
                }
            },
        )];
        let mut sheet = ShareSheet {
            editor,
            note_id,
            title,
            step,
            share_id,
            view,
            username,
            role: Role::Edit,
            link_role: Role::Comment,
            link_days: 7,
            made_link: None,
            busy: None,
            error: None,
            done: None,
            copied: false,
            confirm_end: false,
            _subscriptions: subscriptions,
        };
        sheet.reload(cx);
        sheet
    });
    let footer = cx.new(|cx| ShareFooter {
        view: sheet.clone(),
        _observing: cx.observe(&sheet, |_, _, cx| cx.notify()),
    });
    let dialog_view = sheet.clone();
    let title = sheet.read(cx).title.clone();
    window.open_alert_dialog(cx, move |alert, _, _| {
        let running = dialog_view.clone();
        alert
            .title(if title.trim().is_empty() {
                "Share".to_owned()
            } else {
                format!("Share “{}”", title.trim())
            })
            .width(px(SHEET_WIDTH))
            .child(dialog_view.clone())
            .footer(footer.clone())
            // Escape closes, but not while a step runs.
            .on_cancel(move |_, _, cx| !running.read(cx).is_busy())
    });
    Some(sheet)
}

#[cfg(test)]
mod tests {
    use super::{MemberTone, first_step, manages, member_status};
    use crate::collab_model::SheetStep;
    use kanso_collab::{MemberView, Person, Role};
    use proptest::prelude::*;

    fn member(invited: bool, left: bool, unverified: bool, verified: bool) -> MemberView {
        MemberView {
            person: Person {
                account_id: "a_2".into(),
                username: "user2".into(),
                fingerprint: "1 2 3 4 5 6".into(),
                verified,
            },
            role: Role::Edit,
            invited,
            left,
            unverified,
        }
    }

    #[test]
    fn members_who_went_or_cant_be_checked_say_so() {
        assert_eq!(
            member_status(&member(true, true, false, false), false),
            (vec!["Declined"], MemberTone::Gone)
        );
        assert_eq!(
            member_status(&member(false, true, true, true), false),
            (vec!["Left"], MemberTone::Gone)
        );
        assert_eq!(
            member_status(&member(false, false, true, true), false),
            (vec!["Can't be checked"], MemberTone::Unchecked)
        );
        assert_eq!(
            member_status(&member(true, false, false, true), false),
            (
                vec!["Invited, hasn't accepted yet", "Fingerprint verified"],
                MemberTone::Normal
            )
        );
    }

    #[test]
    fn steps() {
        assert_eq!(first_step(false, None), SheetStep::SignedOut);
        assert_eq!(first_step(true, None), SheetStep::Main);
        assert_eq!(
            first_step(false, Some("Locked.".into())),
            SheetStep::NotShareable("Locked.".into())
        );
        assert!(manages(None));
        assert!(manages(Some(Role::Owner)));
        assert!(!manages(Some(Role::Edit)));
    }

    proptest! {
        /// A reason a note can't be shared always shows, signed in or not;
        /// without one, the sheet opens on sharing when signed in.
        #[test]
        fn prop_a_reason_always_shows(signed_in in any::<bool>(), reason in prop::option::of("[a-z ]{1,20}")) {
            match first_step(signed_in, reason.clone()) {
                SheetStep::NotShareable(shown) => prop_assert_eq!(Some(shown), reason),
                SheetStep::SignedOut => prop_assert!(!signed_in && reason.is_none()),
                SheetStep::Main => prop_assert!(signed_in && reason.is_none()),
                _ => prop_assert!(false),
            }
        }

        /// A member who left or declined is always dimmed and says which;
        /// one whose keys can't be checked (and is still in) is always
        /// warned about, unless it is this account; nobody else is.
        #[test]
        fn prop_member_rows_read_their_state(invited: bool, left: bool, unverified: bool, verified: bool, mine: bool) {
            let (detail, tone) = member_status(&member(invited, left, unverified, verified), mine);
            if left {
                prop_assert_eq!(tone, MemberTone::Gone);
                prop_assert_eq!(detail, vec![if invited { "Declined" } else { "Left" }]);
            } else if unverified && !mine {
                prop_assert_eq!(tone, MemberTone::Unchecked);
                prop_assert!(detail.contains(&"Can't be checked"));
                prop_assert!(!detail.contains(&"Fingerprint verified"));
            } else {
                prop_assert_eq!(tone, MemberTone::Normal);
                prop_assert!(!detail.contains(&"Can't be checked"));
            }
        }

        /// Only the owner (or the sharer of a note not shared yet) manages.
        #[test]
        fn prop_only_owners_manage(role in prop::sample::select(vec![Role::View, Role::Comment, Role::Edit, Role::Owner])) {
            prop_assert_eq!(manages(Some(role)), role == Role::Owner);
        }
    }
}

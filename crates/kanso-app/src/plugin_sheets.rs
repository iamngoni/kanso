//! The plugin sheets: installing or updating, and reviewing a batch of
//! changes before Kanso applies it. A developer folder is trusted here too:
//! its path is typed out and a box ticked before Install works.

use std::collections::BTreeSet;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Disableable as _, Icon, WindowExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_plugins::batch::{LineKind, ReviewNote};
use kanso_plugins::consent::{self, Sheet};
use kanso_plugins::model::Source;

use crate::agent_scope::{ScopeBook, scope_books};
use crate::plugin_sheets_model::{
    chosen_grant, install_enabled, install_label, note_summary, starting_ticks,
};
use crate::plugins_model::InstallPlan;
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

/// The width of both sheets.
pub const SHEET_WIDTH: f32 = 520.;

/// A small heading over a list.
fn heading(text: &'static str, colors: &Colors) -> Div {
    div()
        .text_size(px(11.5))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(colors.text_muted)
        .child(text)
}

/// A list of lines, each behind a mark.
fn list(lines: &[String], icon: Lucide, tint: Hsla, colors: &Colors) -> Div {
    v_flex().gap(px(3.)).children(lines.iter().map(|line| {
        h_flex()
            .items_start()
            .gap(px(8.))
            .child(
                div()
                    .mt(px(3.))
                    .flex_none()
                    .child(Icon::new(icon).size(px(13.)).text_color(tint)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_size(px(13.))
                    .line_height(px(19.))
                    .text_color(colors.text)
                    .child(capital(line)),
            )
    }))
}

/// `text` with its first letter capital.
fn capital(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// The install sheet's state: the plan and what the person has chosen.
pub struct InstallSheet {
    plan: InstallPlan,
    sheet_books: Vec<ScopeBook>,
    ticked: BTreeSet<String>,
    all: bool,
    ask: bool,
    typed: Entity<InputState>,
    typed_text: String,
    trusted: bool,
    error: Option<String>,
    _typing: Subscription,
}

impl InstallSheet {
    fn new(plan: InstallPlan, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let books = scope_books(&cx.global::<AppState>().store);
        let offered: Vec<String> = books.iter().map(|b| b.id.clone()).collect();
        let ticked = starting_ticks(&plan.grant, &offered);
        // The notebooks it starts with come first, so they show.
        let mut books = books;
        books.sort_by_key(|book| !ticked.contains(&book.id));
        let typed = cx.new(|cx| InputState::new(window, cx).placeholder("Type the folder's path"));
        let typing = cx.subscribe_in(&typed, window, |this, input, event: &InputEvent, _, cx| {
            if let InputEvent::Change = event {
                this.typed_text = input.read(cx).value().to_string();
                cx.notify();
            }
        });
        InstallSheet {
            all: plan.grant.notes.read_all,
            ask: plan.grant.ask_before_changing,
            sheet_books: books,
            ticked,
            typed,
            typed_text: String::new(),
            trusted: false,
            error: None,
            _typing: typing,
            plan,
        }
    }

    /// The developer folder, when this is one.
    fn folder(&self) -> Option<&str> {
        match &self.plan.source {
            Source::Developer { path } => Some(path),
            _ => None,
        }
    }

    /// Whether the manifest asks for notes in notebooks.
    fn asks_for_books(&self) -> bool {
        let asks = &self.plan.bundle.manifest.permissions.notes;
        asks.read.is_some()
    }

    /// The grant the choices make.
    fn grant(&self) -> kanso_plugins::model::Grant {
        let offered: Vec<String> = self.sheet_books.iter().map(|b| b.id.clone()).collect();
        chosen_grant(
            &self.plan.grant,
            &self.plan.bundle.manifest,
            &offered,
            &self.ticked,
            self.all,
            self.ask,
        )
    }

    fn ready(&self) -> bool {
        install_enabled(self.folder(), &self.typed_text, self.trusted)
    }

    fn label(&self) -> &'static str {
        install_label(self.plan.offer.is_some(), self.plan.key_changed())
    }

    /// Installs with the choices, or shows why it couldn't.
    fn install(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let grant = self.grant();
        let locked = cx
            .global::<AppState>()
            .store
            .locked_books()
            .unwrap_or_default();
        let manifest = &self.plan.bundle.manifest;
        if let Err(error) = kanso_plugins::grant::check(&grant, manifest, &locked) {
            self.error = Some(error.to_string());
            cx.notify();
            return;
        }
        match crate::plugins::confirm_install(self.plan.clone(), grant, cx) {
            Ok(()) => window.close_dialog(cx),
            Err(message) => {
                self.error = Some(message);
                cx.notify();
            }
        }
    }

    fn toggle(&mut self, id: &str, cx: &mut Context<Self>) {
        if !self.ticked.remove(id) {
            self.ticked.insert(id.to_owned());
        }
        cx.notify();
    }
}

// no-prop-test: drawing; the plugins_sheets snapshots show each state.
impl Render for InstallSheet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let grant = self.grant();
        let store = cx.global::<AppState>().store.clone();
        let name_of = move |id: &str| {
            store
                .books()
                .ok()?
                .into_iter()
                .find(|book| book.id == id)
                .map(|book| book.name)
        };
        let sheet: Sheet = consent::sheet(
            &self.plan.bundle.manifest,
            &grant,
            self.plan.signed,
            &name_of,
        );
        let manifest = &self.plan.bundle.manifest;
        let developer = self.folder().map(str::to_owned);
        let success = Hsla::from(colors.success);
        let danger = Hsla::from(colors.danger);
        let warning = Hsla::from(colors.warning);
        let secondary = Hsla::from(colors.text_secondary);
        let asks_all = manifest
            .permissions
            .notes
            .read
            .as_ref()
            .is_some_and(|r| r.all);
        let all = self.all;

        let book_rows = self.sheet_books.iter().enumerate().map(|(ix, book)| {
            let id = book.id.clone();
            h_flex()
                .pl(px(4. + 16. * book.depth as f32))
                .py(px(2.))
                .child(
                    Checkbox::new(ElementId::NamedInteger("plugin-book".into(), ix as u64))
                        .checked(self.ticked.contains(&book.id))
                        .disabled(all)
                        .label(book.name.clone())
                        .on_click(cx.listener(move |this, _: &bool, _, cx| this.toggle(&id, cx))),
                )
        });

        let notice = |text: String, tint: Hsla| {
            h_flex()
                .items_start()
                .gap(px(8.))
                .p(px(10.))
                .rounded(px(8.))
                .bg(tint.opacity(0.12))
                .child(
                    div().mt(px(2.)).flex_none().child(
                        Icon::new(Lucide::TriangleAlert)
                            .size(px(14.))
                            .text_color(tint),
                    ),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_size(px(12.5))
                        .line_height(px(18.))
                        .text_color(colors.text)
                        .child(text),
                )
        };

        v_flex()
            .id("plugin-install-body")
            .max_h(px(560.))
            .overflow_y_scroll()
            .gap(px(12.))
            .child(
                v_flex()
                    .gap(px(2.))
                    .child(
                        div()
                            .text_size(px(12.5))
                            .text_color(colors.text_secondary)
                            .child(sheet.byline.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(12.5))
                            .text_color(if developer.is_some() {
                                colors.warning
                            } else {
                                colors.text_secondary
                            })
                            .child(capital(&sheet.signed_by)),
                    ),
            )
            .when_some(manifest.description.clone(), |el, text| {
                el.child(
                    div()
                        .text_size(px(13.))
                        .line_height(px(19.))
                        .text_color(colors.text)
                        .child(text),
                )
            })
            .children(self.plan.lookalikes.iter().map(|id| {
                notice(
                    format!("Its ID looks like {id}, which is a different plugin."),
                    warning,
                )
            }))
            .when(self.plan.key_changed(), |el| {
                el.child(notice(
                    "The author's signing key changed since the last version. That can mean someone else published this one."
                        .to_owned(),
                    danger,
                ))
            })
            .when_some(self.plan.offer.clone(), |el, offer| {
                el.when(!offer.added.is_empty(), |el| {
                    el.child(
                        v_flex()
                            .gap(px(4.))
                            .child(heading("New permissions", colors))
                            .child(list(&offer.added, Lucide::Plus, warning, colors)),
                    )
                })
                .when(!offer.removed.is_empty(), |el| {
                    el.child(
                        v_flex()
                            .gap(px(4.))
                            .child(heading("No longer asks for", colors))
                            .child(list(&offer.removed, Lucide::Minus, secondary, colors)),
                    )
                })
            })
            .child(
                v_flex()
                    .gap(px(4.))
                    .child(heading("It can", colors))
                    .child(list(&sheet.can, Lucide::Check, success, colors)),
            )
            .child(
                v_flex()
                    .gap(px(4.))
                    .child(heading("It can't", colors))
                    .child(list(&sheet.cannot, Lucide::CircleX, secondary, colors)),
            )
            .when_some(sheet.warning.clone(), |el, text| el.child(notice(text, warning)))
            .when(self.asks_for_books(), |el| {
                el.child(
                    v_flex()
                        .gap(px(4.))
                        .child(
                            h_flex()
                                .justify_between()
                                .child(heading("Notebooks to give it", colors))
                                .when(!all && self.sheet_books.len() > 1, |el| {
                                    let every = self.sheet_books.iter().all(|b| self.ticked.contains(&b.id));
                                    el.child(
                                        div()
                                            .id("plugin-books-select-all")
                                            .cursor_pointer()
                                            .text_size(px(11.5))
                                            .text_color(colors.accent)
                                            .hover(|s| s.underline())
                                            .child(if every { "Clear" } else { "Select all" })
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                if every {
                                                    this.ticked.clear();
                                                } else {
                                                    this.ticked = this.sheet_books.iter().map(|b| b.id.clone()).collect();
                                                }
                                                cx.notify();
                                            })),
                                    )
                                }),
                        )
                        .when(asks_all, |el| {
                            el.child(
                                h_flex()
                                    .gap(px(8.))
                                    .py(px(2.))
                                    .child(
                                        Switch::new("plugin-all-books")
                                            .checked(all)
                                            .on_click(cx.listener(|this, on: &bool, _, cx| {
                                                this.all = *on;
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(13.))
                                            .text_color(colors.text)
                                            .child("Every notebook that isn't locked"),
                                    ),
                            )
                        })
                        .child(
                            v_flex()
                                .id("plugin-books")
                                .max_h(px(110.))
                                .overflow_y_scroll()
                                .when(all, |el| el.opacity(0.55))
                                .children(book_rows),
                        )
                        .when(self.sheet_books.is_empty(), |el| {
                            el.child(
                                div()
                                    .text_size(px(12.5))
                                    .text_color(colors.text_muted)
                                    .child("There are no unlocked notebooks."),
                            )
                        }),
                )
            })
            .child(
                h_flex()
                    .gap(px(8.))
                    .child(
                        Switch::new("plugin-ask")
                            .checked(self.ask)
                            .on_click(cx.listener(|this, on: &bool, _, cx| {
                                this.ask = *on;
                                cx.notify();
                            })),
                    )
                    .child(
                        v_flex()
                            .child(
                                div()
                                    .text_size(px(13.))
                                    .text_color(colors.text)
                                    .child("Ask before changing notes"),
                            )
                            .child(
                                div()
                                    .text_size(px(12.))
                                    .text_color(colors.text_muted)
                                    .child("Kanso shows what would change and waits for you."),
                            ),
                    ),
            )
            .when_some(developer, |el, path| {
                el.child(
                    v_flex()
                        .gap(px(6.))
                        .child(heading("Developer folder", colors))
                        .child(
                            div()
                                .text_size(px(12.5))
                                .text_color(colors.text)
                                .child(path.clone()),
                        )
                        .child(
                            div()
                                .text_size(px(12.5))
                                .text_color(colors.text_secondary)
                                .child("Code in this folder is not signed. Type its path to say this is the one."),
                        )
                        .child(Input::new(&self.typed))
                        .child(
                            Checkbox::new("plugin-trust")
                                .checked(self.trusted)
                                .label("I trust this code")
                                .on_click(cx.listener(|this, on: &bool, _, cx| {
                                    this.trusted = *on;
                                    cx.notify();
                                })),
                        ),
                )
            })
            .when_some(self.error.clone(), |el, error| {
                el.child(
                    div()
                        .text_size(px(12.5))
                        .text_color(colors.danger)
                        .child(error),
                )
            })
    }
}

/// The install sheet's buttons.
struct InstallFooter {
    sheet: Entity<InstallSheet>,
    _observing: Subscription,
}

// no-prop-test: drawing; the install label and readiness are tested in
// plugin_sheets_model.
impl Render for InstallFooter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sheet = self.sheet.read(cx);
        let ready = sheet.ready();
        let label = sheet.label();
        let danger = sheet.plan.key_changed();
        let target = self.sheet.clone();
        let install = Button::new("plugin-install")
            .debug_selector(|| "plugin-install".into())
            .label(label)
            .disabled(!ready)
            .on_click(move |_, window, cx| {
                target.update(cx, |sheet, cx| sheet.install(window, cx));
            });
        let install = if danger {
            install.danger()
        } else {
            install.primary()
        };
        h_flex()
            .w_full()
            .gap_2()
            .justify_end()
            .child(
                Button::new("plugin-install-cancel")
                    .label("Cancel")
                    .on_click(|_, window, cx| window.close_dialog(cx)),
            )
            .child(install)
    }
}

/// The install (or update) sheet for `plan`.
pub fn open_install(plan: InstallPlan, window: &mut Window, cx: &mut App) {
    let update = plan.offer.is_some();
    let title = {
        let name = plan.bundle.manifest.name.clone();
        if update {
            format!("Update {name}")
        } else {
            format!("Install {name}")
        }
    };
    let sheet = cx.new(|cx| InstallSheet::new(plan, window, cx));
    let footer = cx.new(|cx| InstallFooter {
        _observing: cx.observe(&sheet, |_, _, cx| cx.notify()),
        sheet: sheet.clone(),
    });
    window.open_alert_dialog(cx, move |alert, _, _| {
        alert
            .title(title.clone())
            .width(px(SHEET_WIDTH))
            .child(sheet.clone())
            .footer(footer.clone())
    });
}

/// One note on the review sheet: its title and summary, and its changed
/// lines when open.
fn review_note(
    ix: usize,
    note: &ReviewNote,
    open: bool,
    colors: &Colors,
    mono: Option<&'static str>,
) -> impl IntoElement {
    let title = if note.title.trim().is_empty() {
        "Untitled".to_owned()
    } else {
        note.title.clone()
    };
    let lines = note.lines.iter().map(|(kind, text)| {
        let (tint, sign) = match kind {
            LineKind::Added => (Hsla::from(colors.success), "+"),
            LineKind::Removed => (Hsla::from(colors.danger), "−"),
        };
        h_flex()
            .items_start()
            .min_h(px(20.))
            .bg(tint.opacity(0.13))
            .child(
                div()
                    .w(px(18.))
                    .flex_none()
                    .text_center()
                    .text_color(tint)
                    .child(sign),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .pr(px(8.))
                    .text_color(colors.text)
                    .when(*kind == LineKind::Removed, |el| el.line_through())
                    .child(if text.is_empty() {
                        " ".to_owned()
                    } else {
                        text.clone()
                    }),
            )
    });
    v_flex()
        .id(ElementId::NamedInteger("review-note".into(), ix as u64))
        .child(
            h_flex()
                .gap(px(8.))
                .py(px(6.))
                .px(px(4.))
                .child(
                    Icon::new(if open {
                        Lucide::ChevronDown
                    } else {
                        Lucide::ChevronRight
                    })
                    .size(px(14.))
                    .text_color(colors.text_muted),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .text_size(px(13.))
                        .text_color(colors.text)
                        .child(title),
                )
                .child(
                    div()
                        .flex_none()
                        .text_size(px(12.))
                        .text_color(colors.text_muted)
                        .child(note_summary(note)),
                ),
        )
        .when(open, |el| {
            el.child(
                v_flex()
                    .ml(px(22.))
                    .mb(px(6.))
                    .rounded(px(6.))
                    .overflow_hidden()
                    .border_1()
                    .border_color(colors.divider)
                    .text_size(px(12.5))
                    .when_some(mono, |el, family| el.font_family(family))
                    .children(lines)
                    .when(note.lines.is_empty(), |el| {
                        el.child(
                            div()
                                .p(px(8.))
                                .text_color(colors.text_muted)
                                .child("No lines to show."),
                        )
                    }),
            )
        })
}

/// The review sheet's state: which notes are open.
pub struct ReviewSheet {
    notes: Vec<ReviewNote>,
    open: BTreeSet<usize>,
}

// no-prop-test: drawing; the summary is tested in plugin_sheets_model.
impl Render for ReviewSheet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let mono = crate::theme::mono_font(cx);
        let rows = self.notes.iter().enumerate().map(|(ix, note)| {
            let open = self.open.contains(&ix);
            review_note(ix, note, open, colors, mono).into_any_element()
        });
        let rows: Vec<AnyElement> = rows
            .enumerate()
            .map(|(ix, row)| {
                div()
                    .id(ElementId::NamedInteger("review-row".into(), ix as u64))
                    .cursor_pointer()
                    .child(row)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if !this.open.remove(&ix) {
                            this.open.insert(ix);
                        }
                        cx.notify();
                    }))
                    .into_any_element()
            })
            .collect();
        v_flex()
            .gap(px(8.))
            .child(
                div()
                    .text_size(px(12.5))
                    .line_height(px(18.))
                    .text_color(colors.text_secondary)
                    .child("Nothing changes until you apply. Open a note to see its lines."),
            )
            .child(
                v_flex()
                    .id("review-notes")
                    .max_h(px(380.))
                    .overflow_y_scroll()
                    .children(rows),
            )
    }
}

/// The review sheet for the batch waiting in [`crate::plugins::Plugins`].
pub fn open_review(window: &mut Window, cx: &mut App) {
    open_review_with(&BTreeSet::new(), window, cx);
}

/// The review sheet with the notes at `open` already expanded.
pub fn open_review_with(open: &BTreeSet<usize>, window: &mut Window, cx: &mut App) {
    let Some(review) = cx.global::<crate::plugins::Plugins>().review.clone() else {
        return;
    };
    let title = review.heading();
    let sheet = cx.new(|_| ReviewSheet {
        notes: review.notes,
        open: open.clone(),
    });
    window.open_alert_dialog(cx, move |alert, _, _| {
        alert
            .title(title.clone())
            .width(px(SHEET_WIDTH))
            .child(sheet.clone())
            .footer(
                h_flex()
                    .w_full()
                    .gap_2()
                    .justify_end()
                    .child(
                        Button::new("plugin-review-cancel")
                            .label("Cancel")
                            .on_click(|_, window, cx| {
                                crate::plugins::cancel_review(cx);
                                window.close_dialog(cx);
                            }),
                    )
                    .child(
                        Button::new("plugin-review-apply")
                            .debug_selector(|| "plugin-review-apply".into())
                            .primary()
                            .label("Apply")
                            .on_click(|_, window, cx| {
                                crate::plugins::apply_review(cx);
                                window.close_dialog(cx);
                            }),
                    ),
            )
            .on_cancel(|_, _, cx| {
                crate::plugins::cancel_review(cx);
                true
            })
    });
}

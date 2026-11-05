//! Small modal prompts: ask for a name, confirm something destructive, fill
//! in a few fields while a task runs ([`form`]), pick one of a few described
//! choices ([`choose`]), or confirm by typing a word ([`confirm_typed`]).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::{Disableable as _, WindowExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_sync::Step;

use crate::account_words::step_label;
use crate::theme::Colors;
use crate::zoom::px;

type TextHandler = Rc<dyn Fn(String, &mut Window, &mut App) -> Result<(), String>>;
type ConfirmHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// Asks for a line of text. `on_ok` gets the trimmed, non-empty answer.
pub fn prompt_text(
    title: impl Into<SharedString>,
    initial: &str,
    ok_label: impl Into<SharedString>,
    window: &mut Window,
    cx: &mut App,
    on_ok: impl Fn(String, &mut Window, &mut App) + 'static,
) {
    prompt_checked(
        title,
        initial,
        ok_label,
        window,
        cx,
        move |value, window, cx| {
            on_ok(value, window, cx);
            Ok(())
        },
    );
}

/// Asks for a line of text until `on_ok` accepts it. `on_ok` gets the
/// trimmed, non-empty answer; an error is shown and the dialog stays open.
pub fn prompt_checked(
    title: impl Into<SharedString>,
    initial: &str,
    ok_label: impl Into<SharedString>,
    window: &mut Window,
    cx: &mut App,
    on_ok: impl Fn(String, &mut Window, &mut App) -> Result<(), String> + 'static,
) {
    prompt(title, initial, false, ok_label, window, cx, on_ok);
}

fn prompt(
    title: impl Into<SharedString>,
    initial: &str,
    masked: bool,
    ok_label: impl Into<SharedString>,
    window: &mut Window,
    cx: &mut App,
    on_ok: impl Fn(String, &mut Window, &mut App) -> Result<(), String> + 'static,
) {
    let title = title.into();
    let ok_label = ok_label.into();
    let initial = initial.to_owned();
    let input = cx.new(|cx| {
        InputState::new(window, cx)
            .masked(masked)
            .default_value(initial)
    });
    let on_ok: TextHandler = Rc::new(on_ok);

    // Enter in the field submits, like the OK button.
    let handle = window.window_handle();
    let submit = on_ok.clone();
    let enter = cx.subscribe(&input, move |input, event: &InputEvent, cx| {
        if !matches!(event, InputEvent::PressEnter { .. }) {
            return;
        }
        let value = input.read(cx).value().trim().to_owned();
        if value.is_empty() {
            return;
        }
        let submit = submit.clone();
        handle
            .update(cx, move |_, window, cx| match submit(value, window, cx) {
                Ok(()) => window.close_dialog(cx),
                Err(message) => window.push_notification(Notification::error(message), cx),
            })
            .ok();
    });
    let enter = RefCell::new(Some(enter));

    let first_build = Cell::new(true);
    window.open_alert_dialog(cx, move |alert, window, cx| {
        // Keeps the Enter subscription alive exactly as long as the dialog.
        let _ = &enter;
        if first_build.replace(false) {
            input.update(cx, |input, cx| input.focus(window, cx));
        }
        let input_for_ok = input.clone();
        let on_ok = on_ok.clone();
        alert
            .title(title.clone())
            .child(Input::new(&input).when(masked, |input| input.mask_toggle()))
            .show_cancel(true)
            .ok_text(ok_label.clone())
            .on_ok(move |_, window, cx| {
                let value = input_for_ok.read(cx).value().trim().to_owned();
                if value.is_empty() {
                    return false;
                }
                match on_ok(value, window, cx) {
                    Ok(()) => true,
                    Err(message) => {
                        window.push_notification(Notification::error(message), cx);
                        false
                    }
                }
            })
    });
}

/// Asks before doing something that can't be undone.
pub fn confirm(
    title: impl Into<SharedString>,
    description: impl Into<SharedString>,
    ok_label: impl Into<SharedString>,
    window: &mut Window,
    cx: &mut App,
    on_ok: impl Fn(&mut Window, &mut App) + 'static,
) {
    let title = title.into();
    let description = description.into();
    let ok_label = ok_label.into();
    let on_ok: ConfirmHandler = Rc::new(on_ok);
    window.open_alert_dialog(cx, move |alert, _, _| {
        let on_ok = on_ok.clone();
        alert
            .title(title.clone())
            .description(description.clone())
            .show_cancel(true)
            .ok_text(ok_label.clone())
            .on_ok(move |_, window, cx| {
                on_ok(window, cx);
                true
            })
    });
}

// MARK: Forms

/// The width of [`form`] and [`choose`] dialogs.
pub const FORM_WIDTH: f32 = 460.;

/// One field of a [`form`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    pub label: SharedString,
    pub hint: Option<SharedString>,
    /// Shown as dots, never trimmed, and emptied as soon as a task starts.
    pub masked: bool,
    pub initial: String,
}

impl Field {
    pub fn text(label: impl Into<SharedString>) -> Self {
        Field {
            label: label.into(),
            hint: None,
            masked: false,
            initial: String::new(),
        }
    }

    pub fn secret(label: impl Into<SharedString>) -> Self {
        Field {
            masked: true,
            ..Field::text(label)
        }
    }

    pub fn hint(mut self, hint: impl Into<SharedString>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn initial(mut self, value: impl Into<String>) -> Self {
        self.initial = value.into();
        self
    }
}

/// What a running task tells the form that started it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Progress {
    Step(Step),
    /// Finished: closes the form, or shows why not.
    Done(Result<(), String>),
}

/// What a form does with what was typed.
pub enum Submit {
    /// Not accepted: the words show under the fields and the dialog stays.
    Invalid(String),
    /// A task started: its steps show on the button until it is done.
    Running(async_channel::Receiver<Progress>),
}

/// A form's values as its submit gets them: plain fields trimmed, masked
/// ones exactly as typed.
pub fn submitted_values(fields: &[Field], typed: &[String]) -> Vec<String> {
    fields
        .iter()
        .zip(typed)
        .map(|(field, value)| {
            if field.masked {
                value.clone()
            } else {
                value.trim().to_owned()
            }
        })
        .collect()
}

/// Whether `typed` is `word`, as a confirmation asks: spaces around it and
/// letter case don't matter.
pub fn typed_matches(typed: &str, word: &str) -> bool {
    !word.trim().is_empty() && typed.trim().eq_ignore_ascii_case(word.trim())
}

type SubmitHandler = Rc<dyn Fn(Vec<String>, &mut Window, &mut App) -> Submit>;
type ReadyCheck = Rc<dyn Fn(&[String]) -> bool>;

/// What a [`form`] shows besides its fields.
pub struct FormSpec {
    pub title: SharedString,
    pub description: Option<SharedString>,
    pub fields: Vec<Field>,
    pub submit: SharedString,
    /// The submit button warns: the task can't be undone.
    pub danger: bool,
    /// Whether the submit button can be pressed for these values.
    pub ready: Option<ReadyCheck>,
}

impl FormSpec {
    pub fn new(title: impl Into<SharedString>, submit: impl Into<SharedString>) -> Self {
        FormSpec {
            title: title.into(),
            description: None,
            fields: Vec::new(),
            submit: submit.into(),
            danger: false,
            ready: None,
        }
    }

    pub fn description(mut self, text: impl Into<SharedString>) -> Self {
        self.description = Some(text.into());
        self
    }

    pub fn field(mut self, field: Field) -> Self {
        self.fields.push(field);
        self
    }

    pub fn danger(mut self) -> Self {
        self.danger = true;
        self
    }
}

/// The body of a [`form`] dialog: its fields and an inline error. Its
/// buttons are a [`FormFooter`] in the dialog's footer.
pub struct FormView {
    description: Option<SharedString>,
    fields: Vec<(Field, Entity<InputState>)>,
    submit_label: SharedString,
    danger: bool,
    ready: Option<ReadyCheck>,
    on_submit: SubmitHandler,
    error: Option<SharedString>,
    /// A task runs, at its step once it has said one.
    running: Option<Option<Step>>,
    _subscriptions: Vec<Subscription>,
    _listening: Option<Task<()>>,
}

impl FormView {
    fn new(
        spec: FormSpec,
        on_submit: SubmitHandler,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut subscriptions = Vec::new();
        let fields: Vec<(Field, Entity<InputState>)> = spec
            .fields
            .into_iter()
            .map(|field| {
                let input = cx.new(|cx| {
                    InputState::new(window, cx)
                        .masked(field.masked)
                        .default_value(field.initial.clone())
                });
                subscriptions.push(cx.subscribe_in(
                    &input,
                    window,
                    |this: &mut FormView, _, event: &InputEvent, window, cx| match event {
                        InputEvent::PressEnter { .. } => this.submit(window, cx),
                        InputEvent::Change => cx.notify(),
                        _ => {}
                    },
                ));
                (field, input)
            })
            .collect();
        FormView {
            description: spec.description,
            fields,
            submit_label: spec.submit,
            danger: spec.danger,
            ready: spec.ready,
            on_submit,
            error: None,
            running: None,
            _subscriptions: subscriptions,
            _listening: None,
        }
    }

    fn field_specs(&self) -> Vec<Field> {
        self.fields.iter().map(|(field, _)| field.clone()).collect()
    }

    fn values(&self, cx: &App) -> Vec<String> {
        let typed: Vec<String> = self
            .fields
            .iter()
            .map(|(_, input)| input.read(cx).value().to_string())
            .collect();
        submitted_values(&self.field_specs(), &typed)
    }

    /// Every field has something in it, and the form's own check passes.
    fn is_ready(&self, cx: &App) -> bool {
        let values = self.values(cx);
        values.iter().all(|value| !value.is_empty())
            && self.ready.as_ref().is_none_or(|ready| ready(&values))
    }

    /// Whether a task is running: the fields are disabled and the dialog
    /// stays open.
    pub fn is_running(&self) -> bool {
        self.running.is_some()
    }

    /// Fills the fields as if typed (tests and snapshots).
    #[cfg(test)]
    pub fn fill(&mut self, values: &[&str], window: &mut Window, cx: &mut Context<Self>) {
        for ((_, input), value) in self.fields.iter().zip(values) {
            let value = value.to_string();
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        cx.notify();
    }

    /// Shows `error` as a failed task would (snapshots).
    #[cfg(test)]
    /// Tests: the error under the fields.
    #[cfg(test)]
    pub fn error_text(&self) -> Option<&str> {
        self.error.as_ref().map(|error| error.as_ref())
    }

    pub fn show_error(&mut self, error: &str, cx: &mut Context<Self>) {
        self.error = Some(error.to_owned().into());
        cx.notify();
    }

    /// Shows the button spinning at `step` (snapshots).
    #[cfg(test)]
    pub fn show_step(&mut self, step: Step, cx: &mut Context<Self>) {
        self.running = Some(Some(step));
        cx.notify();
    }

    /// Presses the submit button.
    pub fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.running.is_some() || !self.is_ready(cx) {
            return;
        }
        let values = self.values(cx);
        self.error = None;
        match (self.on_submit)(values, window, cx) {
            Submit::Invalid(words) => {
                self.error = Some(words.into());
                cx.notify();
            }
            Submit::Running(progress) => {
                // Secrets don't stay on screen while the task runs.
                for (field, input) in &self.fields {
                    if field.masked {
                        input.update(cx, |input, cx| input.set_value("", window, cx));
                    }
                }
                self.running = Some(None);
                cx.notify();
                self._listening = Some(cx.spawn_in(window, async move |this, cx| {
                    let mut done = None;
                    while let Ok(event) = progress.recv().await {
                        match event {
                            Progress::Step(step) => {
                                let shown = this.update(cx, |this, cx| {
                                    this.running = Some(Some(step));
                                    cx.notify();
                                });
                                if shown.is_err() {
                                    return;
                                }
                            }
                            Progress::Done(result) => {
                                done = Some(result);
                                break;
                            }
                        }
                    }
                    let result =
                        done.unwrap_or_else(|| Err("Something went wrong. Try again.".to_owned()));
                    this.update_in(cx, |this, window, cx| {
                        this.running = None;
                        match result {
                            Ok(()) => window.close_dialog(cx),
                            Err(words) => {
                                this.error = Some(words.into());
                                cx.notify();
                            }
                        }
                    })
                    .ok();
                }));
            }
        }
    }
}

// no-prop-test: drawing; the account snapshots show each form, and
// `submitted_values` and `typed_matches` are tested.
impl Render for FormView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let running = self.running.is_some();
        let fields = self.fields.iter().enumerate().map(|(ix, (field, input))| {
            v_flex()
                .gap(px(6.))
                .child(
                    div()
                        .text_size(px(13.))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors.text)
                        .child(field.label.clone()),
                )
                .child(
                    div()
                        .debug_selector(move || format!("form-field-{ix}"))
                        // Masked fields can be shown while typing, to check them.
                        .child(
                            Input::new(input)
                                .disabled(running)
                                .when(field.masked, |input| input.mask_toggle()),
                        ),
                )
                .when_some(field.hint.clone(), |el, hint| {
                    el.child(
                        div()
                            .text_size(px(12.))
                            .line_height(px(16.))
                            .text_color(colors.text_muted)
                            .child(hint),
                    )
                })
        });
        v_flex()
            .gap(px(14.))
            .pb(px(4.))
            .when_some(self.description.clone(), |el, text| {
                el.child(
                    div()
                        .text_size(px(13.5))
                        .line_height(px(19.))
                        .text_color(colors.text_secondary)
                        .child(text),
                )
            })
            .children(fields)
            .when_some(self.error.clone(), |el, error| {
                el.child(
                    div()
                        .debug_selector(|| "form-error".into())
                        .text_size(px(13.))
                        .line_height(px(18.))
                        .text_color(colors.danger)
                        .child(error),
                )
            })
    }
}

/// A [`form`]'s buttons, in the dialog's footer like every other dialog's.
/// A view of its own that redraws with the form, so the submit button can
/// spin with the task's step.
struct FormFooter {
    form: Entity<FormView>,
    _observing: Subscription,
}

impl FormFooter {
    fn new(form: Entity<FormView>, cx: &mut Context<Self>) -> Self {
        let observing = cx.observe(&form, |_, _, cx| cx.notify());
        FormFooter {
            form,
            _observing: observing,
        }
    }
}

// no-prop-test: drawing; the account snapshots show the buttons at rest,
// failed and running.
impl Render for FormFooter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let form = self.form.read(cx);
        let running = form.running.is_some();
        let ready = form.is_ready(cx);
        let label = match form.running {
            Some(Some(step)) => SharedString::from(step_label(step)),
            _ => form.submit_label.clone(),
        };
        let danger = form.danger;
        let target = self.form.clone();
        let submit = Button::new("form-submit")
            .label(label)
            // The spinner takes the icon's place; a loading button ignores
            // clicks, and keeps its colors so the step reads.
            .when(running, |button| {
                button.icon(gpui_kit::assets::IconName::RefreshCcw)
            })
            .loading(running)
            .disabled(!ready && !running)
            .debug_selector(|| "form-submit".into())
            .on_click(move |_, window, cx| {
                target.update(cx, |form, cx| form.submit(window, cx));
            });
        let submit = if danger {
            submit.danger()
        } else {
            submit.primary()
        };
        h_flex()
            .w_full()
            .gap_2()
            .justify_end()
            .child(
                Button::new("form-cancel")
                    .label("Cancel")
                    .disabled(running)
                    .debug_selector(|| "form-cancel".into())
                    .on_click(|_, window, cx| window.close_dialog(cx)),
            )
            .child(submit)
    }
}

/// Asks for a few fields and runs a task with them. The dialog stays open
/// while the task runs, with its step on the button, and shows an error
/// under the fields when it fails. Returns the form, for tests to fill.
pub fn form(
    spec: FormSpec,
    window: &mut Window,
    cx: &mut App,
    on_submit: impl Fn(Vec<String>, &mut Window, &mut App) -> Submit + 'static,
) -> Entity<FormView> {
    let title = spec.title.clone();
    let view = cx.new(|cx| FormView::new(spec, Rc::new(on_submit), window, cx));
    let footer = cx.new(|cx| FormFooter::new(view.clone(), cx));
    let first = view.read(cx).fields.first().map(|(_, input)| input.clone());
    let first_build = Cell::new(true);
    let dialog_view = view.clone();
    window.open_alert_dialog(cx, move |alert, window, cx| {
        if first_build.replace(false)
            && let Some(first) = &first
        {
            first.update(cx, |input, cx| input.focus(window, cx));
        }
        let running = dialog_view.clone();
        alert
            .title(title.clone())
            .width(px(FORM_WIDTH))
            .child(dialog_view.clone())
            .footer(footer.clone())
            // Escape cancels, but not while the task runs.
            .on_cancel(move |_, _, cx| !running.read(cx).is_running())
    });
    view
}

/// A confirmation that needs `word` typed first, then any other fields of
/// `spec` (such as the password), before its button works.
pub fn confirm_typed(
    spec: FormSpec,
    word: &str,
    window: &mut Window,
    cx: &mut App,
    on_submit: impl Fn(Vec<String>, &mut Window, &mut App) -> Submit + 'static,
) -> Entity<FormView> {
    let word = word.to_owned();
    let mut spec = spec;
    spec.fields
        .insert(0, Field::text(format!("Type {word} to confirm")));
    spec.ready = Some(Rc::new(move |values: &[String]| {
        values
            .first()
            .is_some_and(|typed| typed_matches(typed, &word))
    }));
    form(spec, window, cx, on_submit)
}

// MARK: Choices

/// One of the choices [`choose`] offers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Described {
    pub label: SharedString,
    pub description: SharedString,
}

/// Offers up to three described choices, and Cancel. `on_choice` gets the
/// index of the one picked, or `None` for Cancel (or Escape), once.
pub fn choose(
    title: impl Into<SharedString>,
    body: impl Into<SharedString>,
    choices: Vec<Described>,
    window: &mut Window,
    cx: &mut App,
    on_choice: impl Fn(Option<usize>, &mut Window, &mut App) + 'static,
) {
    let title = title.into();
    let body = body.into();
    type ChoiceHandler = Rc<dyn Fn(Option<usize>, &mut Window, &mut App)>;
    let answered = Rc::new(Cell::new(false));
    let on_choice: ChoiceHandler = Rc::new(on_choice);
    let answer: ChoiceHandler = Rc::new(move |choice, window, cx| {
        if !answered.replace(true) {
            on_choice(choice, window, cx);
        }
    });
    let choices: Vec<Described> = choices.into_iter().take(3).collect();
    window.open_alert_dialog(cx, move |alert, _, cx| {
        let colors = Colors::get(cx);
        let rows = choices.iter().enumerate().map(|(ix, choice)| {
            let answer = answer.clone();
            v_flex()
                .id(("choice", ix))
                .debug_selector(move || format!("choice-{ix}"))
                .px(px(14.))
                .py(px(10.))
                .gap(px(3.))
                .rounded(px(8.))
                .border_1()
                .border_color(colors.divider)
                .cursor_pointer()
                .hover(|el| el.bg(colors.row_hover))
                .child(
                    div()
                        .text_size(px(14.))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors.text)
                        .child(choice.label.clone()),
                )
                .child(
                    div()
                        .text_size(px(12.5))
                        .line_height(px(17.))
                        .text_color(colors.text_secondary)
                        .child(choice.description.clone()),
                )
                .on_click(move |_, window, cx| {
                    window.close_dialog(cx);
                    answer(Some(ix), window, cx);
                })
        });
        let cancel = answer.clone();
        let escape = answer.clone();
        alert
            .title(title.clone())
            .description(body.clone())
            .width(px(FORM_WIDTH))
            .child(v_flex().gap(px(8.)).children(rows))
            .footer(
                h_flex().w_full().justify_end().child(
                    Button::new("choice-cancel")
                        .label("Cancel")
                        .debug_selector(|| "choice-cancel".into())
                        .on_click(move |_, window, cx| {
                            window.close_dialog(cx);
                            cancel(None, window, cx);
                        }),
                ),
            )
            .on_cancel(move |_, window, cx| {
                escape(None, window, cx);
                true
            })
    });
}

#[cfg(test)]
mod tests {
    use super::{Field, submitted_values, typed_matches};
    use proptest::prelude::*;

    #[test]
    fn examples() {
        let fields = [Field::text("Username"), Field::secret("Password")];
        let typed = [" ngoni ".to_owned(), " pass phrase ".to_owned()];
        assert_eq!(
            submitted_values(&fields, &typed),
            ["ngoni", " pass phrase "]
        );
        assert!(typed_matches(" Ngoni", "ngoni"));
        assert!(!typed_matches("ngon", "ngoni"));
        assert!(!typed_matches("", ""));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Secrets reach the task exactly as typed; other fields lose only
        /// the spaces around them.
        #[test]
        fn prop_only_plain_fields_are_trimmed(values in prop::collection::vec(("\\PC{0,12}", any::<bool>()), 0..6)) {
            let fields: Vec<Field> = values
                .iter()
                .map(|(_, masked)| if *masked { Field::secret("s") } else { Field::text("t") })
                .collect();
            let typed: Vec<String> = values.iter().map(|(v, _)| v.clone()).collect();
            let out = submitted_values(&fields, &typed);
            prop_assert_eq!(out.len(), typed.len());
            for ((field, typed), out) in fields.iter().zip(&typed).zip(&out) {
                if field.masked {
                    prop_assert_eq!(out, typed);
                } else {
                    prop_assert_eq!(out.as_str(), typed.trim());
                }
            }
        }

        /// A confirmation word matches itself however it is spaced or
        /// cased, and nothing that differs.
        #[test]
        fn prop_typed_words_match_only_themselves(word in "[a-z0-9_-]{3,12}", pad in " {0,3}", other in "[a-z0-9_-]{3,12}") {
            let typed = format!("{pad}{}{pad}", word.to_uppercase());
            prop_assert!(typed_matches(&typed, &word));
            prop_assert_eq!(typed_matches(&other, &word), other == word);
        }
    }
}

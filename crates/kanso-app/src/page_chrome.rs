//! The chrome the full pages share (the morning brief, the weekly review,
//! open loops and inbox triage): a header whose actions never clip, the
//! centred reading column, empty states that say what the page is for, and
//! a friendly inline card for a model that failed.
//!
//! The header measures its actions' labels and fits them to the width it
//! has: every label when they fit, then icons only (each with its label as
//! a tooltip), then as many icons as fit and a "More" menu with the rest.
//! Which of the three is [`fit`]'s decision, a pure function.

use std::rc::Rc;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, Sizable as _, StyledExt as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::layout::TOP_BAND_HEIGHT;
use crate::theme::Colors;
use crate::zoom::px;

/// How wide the reading column of a full page gets.
pub const COLUMN_WIDTH: f32 = 720.;
/// Space between header actions.
const GAP: f32 = 4.;
/// A small button's padding and icon, around its label.
const BUTTON_CHROME: f32 = 34.;
/// A small icon-only button.
const ICON_BUTTON: f32 = 28.;
/// The header's padding on either side, past any leading inset.
const BAND_PAD: f32 = 16.;
/// The least the title keeps before actions start collapsing.
const TITLE_MIN: f32 = 96.;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionStyle {
    Ghost,
    Outline,
    Primary,
}

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

/// One action in a page's header.
#[derive(Clone)]
pub struct HeaderAction {
    pub id: &'static str,
    pub icon: Lucide,
    pub label: SharedString,
    pub tooltip: SharedString,
    pub style: ActionStyle,
    pub disabled: bool,
    /// Working: the button spins and says so.
    pub loading: bool,
    on_click: Handler,
}

impl HeaderAction {
    pub fn new(
        id: &'static str,
        icon: Lucide,
        label: impl Into<SharedString>,
        on_click: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        let label = label.into();
        HeaderAction {
            id,
            icon,
            tooltip: label.clone(),
            label,
            style: ActionStyle::Ghost,
            disabled: false,
            loading: false,
            on_click: Rc::new(on_click),
        }
    }

    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = tooltip.into();
        self
    }

    pub fn style(mut self, style: ActionStyle) -> Self {
        self.style = style;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }
}

/// How a header's actions fit its width.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fit {
    /// Every action with its label.
    Full,
    /// Every action as an icon.
    Icons,
    /// The last `n` as icons, the rest in the More menu before them.
    Overflow(usize),
}

/// How actions `full` wide with labels, `icon` wide as icons, fit in
/// `room`, with `gap` between them and `more` for the More button. Always
/// an answer that fits when one can: a More button alone is the least.
pub fn fit(full: &[f32], icon: &[f32], gap: f32, more: f32, room: f32) -> Fit {
    let row = |widths: &[f32]| -> f32 {
        widths.iter().sum::<f32>() + gap * widths.len().saturating_sub(1) as f32
    };
    if full.is_empty() || row(full) <= room {
        return Fit::Full;
    }
    if row(icon) <= room {
        return Fit::Icons;
    }
    let mut kept = 0;
    while kept < icon.len() && row(&icon[..=kept]) + gap + more <= room {
        kept += 1;
    }
    Fit::Overflow(kept.min(icon.len().saturating_sub(1)))
}

fn text_width(text: &str, size: Pixels, weight: FontWeight, window: &Window) -> Pixels {
    let mut font = window.text_style().font();
    font.weight = weight;
    let run = TextRun {
        len: text.len(),
        font,
        color: Hsla::default(),
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    window
        .text_system()
        .shape_line(text.to_owned().into(), size, &[run], None)
        .width
}

fn action_button(action: &HeaderAction, labelled: bool, colors: &Colors) -> Button {
    let handler = action.on_click.clone();
    let icon_color = match action.style {
        ActionStyle::Primary => colors.button_text,
        _ => colors.text_secondary,
    };
    let button = Button::new(action.id)
        .small()
        .icon(Icon::new(action.icon).size(px(14.)).text_color(icon_color))
        .disabled(action.disabled || action.loading)
        .loading(action.loading)
        .tooltip(if labelled {
            action.tooltip.clone()
        } else {
            action.label.clone()
        })
        .on_click(move |_, window, cx| {
            cx.stop_propagation();
            handler(window, cx);
        });
    let button = match action.style {
        ActionStyle::Ghost => button.ghost(),
        ActionStyle::Outline => button.outline(),
        ActionStyle::Primary => button.primary(),
    };
    if labelled {
        button.label(action.label.clone())
    } else {
        button
    }
}

/// A full page's header band: `title` at the left past `leading_inset`
/// (the traffic lights' room), the actions at the right fitted to `width`,
/// the band's whole width. `pinned` (such as Close) always shows, last.
pub fn page_header(
    title: impl Into<SharedString>,
    leading_inset: Pixels,
    width: Pixels,
    actions: Vec<HeaderAction>,
    pinned: Option<HeaderAction>,
    window: &Window,
    cx: &App,
) -> Stateful<Div> {
    let colors = Colors::get(cx);
    let title: SharedString = title.into();
    // Buttons set their label at `text_sm`: the theme's size less an eighth.
    let label_size = cx.theme().font_size * 0.875;
    let title_width = text_width(&title, px(14.), FontWeight::SEMIBOLD, window).min(px(240.));
    let pinned_width = if pinned.is_some() {
        px(ICON_BUTTON + GAP)
    } else {
        px(0.)
    };
    let room = width
        - leading_inset
        - px(2. * BAND_PAD)
        - title_width.max(px(TITLE_MIN))
        - px(12.)
        - pinned_width;
    let full: Vec<f32> = actions
        .iter()
        .map(|a| {
            f32::from(text_width(&a.label, label_size, FontWeight::NORMAL, window))
                + f32::from(px(BUTTON_CHROME))
        })
        .collect();
    let icons = vec![f32::from(px(ICON_BUTTON)); actions.len()];
    let shape = fit(
        &full,
        &icons,
        f32::from(px(GAP)),
        f32::from(px(ICON_BUTTON)),
        f32::from(room),
    );
    let (shown, labelled, rest): (&[HeaderAction], bool, &[HeaderAction]) = match shape {
        Fit::Full => (&actions, true, &[]),
        Fit::Icons => (&actions, false, &[]),
        // The last actions are the page's main ones: they stay out.
        Fit::Overflow(n) => {
            let cut = actions.len() - n;
            (&actions[cut..], false, &actions[..cut])
        }
    };
    let more = (!rest.is_empty()).then(|| {
        let items: Vec<(SharedString, SharedString, Lucide, bool, Handler)> = rest
            .iter()
            .map(|a| {
                (
                    a.label.clone(),
                    a.tooltip.clone(),
                    a.icon,
                    a.disabled || a.loading,
                    a.on_click.clone(),
                )
            })
            .collect();
        Button::new("page-more")
            .ghost()
            .small()
            .icon(
                Icon::new(Lucide::Ellipsis)
                    .size(px(15.))
                    .text_color(colors.text_secondary),
            )
            .tooltip("More")
            .debug_selector(|| "page-more".into())
            .dropdown_menu(move |mut menu, _, _| {
                for (label, tooltip, icon, disabled, handler) in &items {
                    let handler = handler.clone();
                    menu = menu.item(
                        PopupMenuItem::element({
                            let label = label.clone();
                            let tooltip = tooltip.clone();
                            move |_, _| {
                                let tooltip = tooltip.clone();
                                div()
                                    .id(SharedString::from(format!("more-{label}")))
                                    .child(label.clone())
                                    .tooltip(move |window, cx| {
                                        gpui_kit::component::tooltip::Tooltip::new(tooltip.clone())
                                            .build(window, cx)
                                    })
                            }
                        })
                        .icon(Icon::new(*icon))
                        .disabled(*disabled)
                        .on_click(move |_, window, cx| handler(window, cx)),
                    );
                }
                menu
            })
    });
    h_flex()
        .id("page-header")
        .debug_selector(|| "page-header".into())
        .h(px(TOP_BAND_HEIGHT))
        .w_full()
        .flex_shrink_0()
        .pl(px(BAND_PAD) + leading_inset)
        .pr(px(BAND_PAD - 6.))
        .gap(px(GAP))
        .overflow_hidden()
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .debug_selector(|| "page-title".into())
                .text_size(px(14.))
                .font_semibold()
                .text_color(colors.text)
                .child(title),
        )
        .children(more)
        .children(shown.iter().map(|a| {
            action_button(a, labelled, colors)
                .debug_selector({
                    let id = a.id;
                    move || id.into()
                })
                .flex_shrink_0()
        }))
        .children(pinned.map(|a| action_button(&a, false, colors).flex_shrink_0()))
}

/// The page's scrolling body: a centred column at most [`COLUMN_WIDTH`]
/// wide, with room either side.
pub fn column() -> Div {
    v_flex()
        .w_full()
        .max_w(px(COLUMN_WIDTH))
        .px(px(24.))
        .pt(px(12.))
        .pb(px(48.))
}

/// The column, centred in the page's width.
pub fn centred(column: impl IntoElement) -> Div {
    h_flex().w_full().justify_center().child(column)
}

/// The page's own heading: a large title, then a quiet line under it.
pub fn hero(
    title: impl Into<SharedString>,
    detail: Option<SharedString>,
    trailing: Option<AnyElement>,
    cx: &App,
) -> Div {
    let colors = Colors::get(cx);
    h_flex()
        .w_full()
        .flex_wrap()
        .items_end()
        .gap(px(12.))
        .pt(px(16.))
        .pb(px(8.))
        .child(
            v_flex()
                .flex_1()
                .min_w(px(200.))
                .gap(px(4.))
                .child(
                    div()
                        .text_size(px(26.))
                        .line_height(px(32.))
                        .font_semibold()
                        .text_color(colors.text)
                        .child(title.into()),
                )
                .when_some(detail, |el, detail| {
                    el.child(
                        div()
                            .text_size(px(14.))
                            .line_height(px(20.))
                            .text_color(colors.text_secondary)
                            .child(detail),
                    )
                }),
        )
        .children(trailing)
}

/// A section of a page: an icon, a heading and a count, then its rows in a
/// quiet card.
pub fn section(
    icon: Lucide,
    title: impl Into<SharedString>,
    count: Option<usize>,
    rows: Vec<AnyElement>,
    empty: Option<&'static str>,
    cx: &App,
) -> Div {
    let colors = Colors::get(cx);
    let body = if rows.is_empty() {
        div()
            .px(px(14.))
            .py(px(10.))
            .text_size(px(13.))
            .text_color(colors.text_muted)
            .children(empty)
            .into_any_element()
    } else {
        v_flex().py(px(4.)).children(rows).into_any_element()
    };
    v_flex()
        .w_full()
        .mt(px(22.))
        .gap(px(8.))
        .child(
            h_flex()
                .gap(px(8.))
                .px(px(2.))
                .child(Icon::new(icon).size(px(14.)).text_color(colors.text_muted))
                .child(
                    div()
                        .text_size(px(13.))
                        .font_semibold()
                        .text_color(colors.text)
                        .child(title.into()),
                )
                .when_some(count.filter(|n| *n > 0), |el, n| {
                    el.child(
                        div()
                            .px(px(6.))
                            .rounded(px(8.))
                            .bg(colors.fill)
                            .text_size(px(11.5))
                            .text_color(colors.text_secondary)
                            .child(n.to_string()),
                    )
                }),
        )
        .child(
            div()
                .w_full()
                .rounded(px(10.))
                .border_1()
                .border_color(colors.divider)
                .bg(colors.editor)
                .child(body),
        )
}

/// A page with nothing to show: what the page is for and what to do next.
pub fn page_empty(
    icon: Lucide,
    title: impl Into<SharedString>,
    body: impl Into<SharedString>,
    actions: Vec<AnyElement>,
    cx: &App,
) -> Div {
    let colors = Colors::get(cx);
    v_flex()
        .debug_selector(|| "page-empty".into())
        .w_full()
        .items_center()
        .pt(px(56.))
        .pb(px(32.))
        .child(
            div()
                .size(px(56.))
                .rounded_full()
                .bg(colors.fill)
                .flex()
                .items_center()
                .justify_center()
                .child(Icon::new(icon).size(px(24.)).text_color(colors.accent)),
        )
        .child(
            div()
                .mt(px(18.))
                .text_size(px(19.))
                .line_height(px(26.))
                .font_semibold()
                .text_color(colors.text)
                .text_center()
                .child(title.into()),
        )
        .child(
            div()
                .mt(px(8.))
                .max_w(px(440.))
                .text_size(px(14.))
                .line_height(px(21.))
                .text_color(colors.text_secondary)
                .text_center()
                .child(body.into()),
        )
        .when(!actions.is_empty(), |el| {
            el.child(
                h_flex()
                    .mt(px(22.))
                    .gap_2()
                    .flex_wrap()
                    .justify_center()
                    .children(actions),
            )
        })
}

/// What went wrong with a model, in words a person can act on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiTrouble {
    pub title: String,
    pub detail: String,
    /// The error as it came, for anyone who wants it.
    pub raw: String,
}

/// The words for model error `raw`: never the raw text as the title; the
/// detail says what to do. The reading is `kanso_ai::classify_trouble`'s, the
/// one every AI feature and the phone share.
pub fn ai_trouble(raw: &str) -> AiTrouble {
    let words = kanso_ai::classify_trouble(raw).words();
    AiTrouble {
        title: words.title.to_owned(),
        detail: words.detail,
        raw: raw.trim().to_owned(),
    }
}

/// The card shown where an AI action failed: what happened, Try Again and
/// AI Settings, and a way to dismiss it.
pub fn ai_error_card(
    trouble: &AiTrouble,
    on_retry: impl Fn(&mut Window, &mut App) + 'static,
    on_dismiss: impl Fn(&mut Window, &mut App) + 'static,
    window: &mut Window,
    cx: &mut App,
) -> Div {
    let colors = Colors::get(cx);
    let details = window.use_keyed_state("ai-error-details", cx, |_, _| false);
    let details_open = *details.read(cx);
    h_flex()
        .debug_selector(|| "ai-error".into())
        .w_full()
        .mt(px(12.))
        .p(px(14.))
        .gap(px(12.))
        .items_start()
        .rounded(px(10.))
        .border_1()
        .border_color(colors.warning)
        .bg(colors.fill)
        .child(
            Icon::new(Lucide::TriangleAlert)
                .size(px(16.))
                .mt(px(2.))
                .text_color(colors.warning),
        )
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .gap(px(4.))
                .child(
                    div()
                        .text_size(px(13.5))
                        .font_semibold()
                        .text_color(colors.text)
                        .child(trouble.title.clone()),
                )
                .child(
                    div()
                        .text_size(px(13.))
                        .line_height(px(19.))
                        .text_color(colors.text_secondary)
                        .child(trouble.detail.clone()),
                )
                .child(
                    h_flex()
                        .mt(px(8.))
                        .gap_2()
                        .flex_wrap()
                        .child(
                            Button::new("ai-retry")
                                .outline()
                                .small()
                                .icon(Lucide::RefreshCw)
                                .label("Try Again")
                                .on_click(move |_, window, cx| on_retry(window, cx)),
                        )
                        .child(
                            Button::new("ai-settings")
                                .ghost()
                                .small()
                                .label("AI Settings")
                                .on_click(|_, window, cx| {
                                    window.dispatch_action(
                                        Box::new(crate::settings::OpenAiSettings),
                                        cx,
                                    )
                                }),
                        )
                        .child(
                            Button::new("ai-details")
                                .ghost()
                                .small()
                                .icon(if details_open {
                                    Lucide::ChevronDown
                                } else {
                                    Lucide::ChevronRight
                                })
                                .label("Details")
                                .debug_selector(|| "ai-details".into())
                                .on_click(move |_, _, cx| {
                                    details.update(cx, |open, cx| {
                                        *open = !*open;
                                        cx.notify();
                                    })
                                }),
                        ),
                )
                .when(details_open, |el| {
                    el.child(
                        div()
                            .debug_selector(|| "ai-details-text".into())
                            .mt(px(6.))
                            .p(px(8.))
                            .rounded(px(6.))
                            .bg(colors.elevated)
                            .text_size(px(12.))
                            .text_color(colors.text_secondary)
                            .child(trouble.raw.clone()),
                    )
                }),
        )
        .child(
            Button::new("ai-error-dismiss")
                .ghost()
                .xsmall()
                .icon(Icon::new(Lucide::X).text_color(colors.text_muted))
                .tooltip("Dismiss")
                .on_click(move |_, window, cx| on_dismiss(window, cx)),
        )
}

#[cfg(test)]
mod tests {
    use super::{Fit, GAP, ICON_BUTTON, ai_trouble, fit};
    use proptest::prelude::*;

    const TITLES: &[&str] = &[
        "No API key is saved",
        "Your model turned the key down",
        "No model is chosen",
        "No server address is set",
        "No agent is set up",
        "Your agent isn't installed",
        "Your agent needs a newer Node",
        "This model can't use tools",
        "Your model is busy",
        "Your model took too long",
        "Your model wasn't found",
        "Your model's server had a problem",
        "Couldn't reach your model",
        "AI isn't set up yet",
        "Your model ran into a problem",
    ];

    fn row(widths: &[f32], gap: f32) -> f32 {
        widths.iter().sum::<f32>() + gap * widths.len().saturating_sub(1) as f32
    }

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 256,
            rng_seed: proptest::test_runner::RngSeed::Fixed(194),
            ..ProptestConfig::default()
        }
    }

    proptest! {
        #![proptest_config(config())]

        /// Whatever the width, every action is reachable (shown or in the
        /// menu), and what is shown fits whenever a More button alone does.
        #[test]
        fn every_action_stays_reachable_and_fits(
            full in prop::collection::vec(40f32..200.0, 0..7),
            room in 0f32..1200.0,
        ) {
            let icon = vec![ICON_BUTTON; full.len()];
            match fit(&full, &icon, GAP, ICON_BUTTON, room) {
                Fit::Full => prop_assert!(full.is_empty() || row(&full, GAP) <= room),
                Fit::Icons => prop_assert!(row(&icon, GAP) <= room),
                Fit::Overflow(n) => {
                    prop_assert!(n < full.len());
                    if ICON_BUTTON <= room {
                        prop_assert!(row(&icon[..n], GAP) + GAP + ICON_BUTTON <= room + GAP);
                    }
                }
            }
        }

        /// More room never shows fewer actions.
        #[test]
        fn more_room_never_hides_more(
            full in prop::collection::vec(40f32..200.0, 1..7),
            a in 0f32..1200.0,
            b in 0f32..1200.0,
        ) {
            let icon = vec![ICON_BUTTON; full.len()];
            let (small, big) = if a < b { (a, b) } else { (b, a) };
            let rank = |f: Fit| match f {
                Fit::Full => full.len() * 2 + 1,
                Fit::Icons => full.len() * 2,
                Fit::Overflow(n) => n,
            };
            prop_assert!(rank(fit(&full, &icon, GAP, ICON_BUTTON, small))
                <= rank(fit(&full, &icon, GAP, ICON_BUTTON, big)));
        }

        /// The title is never the raw error, and always says something.
        #[test]
        fn trouble_is_never_raw(raw in ".{0,120}") {
            let t = ai_trouble(&raw);
            prop_assert!(!t.detail.is_empty());
            prop_assert!(TITLES.contains(&t.title.as_str()), "{}", t.title);
        }
    }

    #[test]
    fn a_missing_key_and_a_missing_agent_say_so_and_how_to_fix_it() {
        use kanso_ai::AiError;
        let key = ai_trouble(&AiError::MissingKey.to_string());
        assert_eq!(key.title, "No API key is saved");
        assert!(key.detail.contains("Settings › AI"), "{}", key.detail);
        let agent = ai_trouble(
            &AiError::Agent("Set the agent's command in Settings › AI.".into()).to_string(),
        );
        assert_eq!(agent.title, "No agent is set up");
        assert!(agent.detail.contains("Choose an agent in Settings › AI"));
    }

    #[test]
    fn status_codes_match_as_numbers_not_substrings() {
        for raw in ["status 401", "HTTP 401: nope", "error (401)", "401"] {
            assert_eq!(
                ai_trouble(raw).title,
                "Your model turned the key down",
                "{raw}"
            );
        }
        for raw in ["port 14010 closed", "id a401b", "took 4290 ms"] {
            assert_eq!(
                ai_trouble(raw).title,
                "Your model ran into a problem",
                "{raw}"
            );
        }
        assert_eq!(ai_trouble("status code: 429").title, "Your model is busy");
    }

    #[test]
    fn the_ollama_transport_error_reads_as_a_person_would_say_it() {
        let t = ai_trouble(
            "The model reported an error: ollama transport error: failed to call chat completions for streamed message",
        );
        assert_eq!(t.title, "Couldn't reach your model");
        assert_eq!(t.detail, "Is Ollama running? Start it, then try again.");
    }

    #[test]
    fn a_narrow_header_overflows_and_a_wide_one_shows_labels() {
        let full = [90., 120., 100.];
        let icon = [ICON_BUTTON; 3];
        assert_eq!(fit(&full, &icon, GAP, ICON_BUTTON, 400.), Fit::Full);
        assert_eq!(fit(&full, &icon, GAP, ICON_BUTTON, 100.), Fit::Icons);
        assert_eq!(fit(&full, &icon, GAP, ICON_BUTTON, 64.), Fit::Overflow(1));
        assert_eq!(fit(&full, &icon, GAP, ICON_BUTTON, 10.), Fit::Overflow(0));
    }
}

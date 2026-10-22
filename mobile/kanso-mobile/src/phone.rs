//! The phone layout: a list of notes, and one note opened in an editor or a preview.

use std::path::PathBuf;
use std::time::Duration;

use gpui_kit::component::input::{Editor, EditorState, InputEvent};
use gpui_kit::component::text::TextView;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::{div, prelude::*, px, Context, Entity, IntoElement, Window};
use kanso_core::{DocType, roles::Role};
use kanso_store::{NoteSummary, Origin, Put, SearchParams, Store};
use serde_json::json;

/// A long note, to feel scrolling in the editor and the preview.
fn long_note() -> String {
    let mut body = String::from("# Long note\n\nThirty paragraphs, to scroll through.\n");
    for i in 1..=30 {
        body.push_str(&format!(
            "\n## Section {i}\n\nParagraph {i}: the quick brown fox jumps over the lazy dog, and **bold {i}** and `code {i}` sit in the same line as a [link](https://example.com/{i}).\n\n- item {i}.1\n- item {i}.2\n"
        ));
    }
    body
}

const SEED: &[(&str, &str)] = &[
    (
        "Welcome to Kanso",
        "# Welcome to Kanso\n\nA **quiet** place for notes. This is the phone spike: the editor and the preview are the same GPUI Kit widgets the desktop app uses.\n\n- [x] Open a store in the app container\n- [ ] Type with the software keyboard\n- [ ] Select text with a finger\n\n> Notes stay on this device until you sign in.\n",
    ),
    (
        "Shopping",
        "# Shopping\n\n- oat milk\n- coffee beans\n- *good* bread\n\n```rust\nfn main() {\n    println!(\"hello from a code block\");\n}\n```\n",
    ),
    (
        "Meeting notes",
        "# Meeting notes\n\n## Agenda\n\n1. Mobile spike\n2. Sync\n3. Release\n\n| Who | What |\n|---|---|\n| Ada | Store |\n| Lin | Sync |\n\nSee [[Welcome to Kanso]].\n",
    ),
    ("Ideas", "# Ideas\n\nA note to type into. Try autocorrect: teh, recieve, definately. Try dictation too.\n"),
];

struct OpenNote {
    id: String,
    body: Entity<EditorState>,
    text: String,
    preview: bool,
}

pub struct Phone {
    store: Store,
    notes: Vec<NoteSummary>,
    open: Option<OpenNote>,
    keyboard: f32,
}

fn db_path() -> PathBuf {
    // The app container's own Application Support: never ~/Library/Application Support/Kanso.
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join("Library/Application Support/KansoSpike/kanso.db")
}

fn title_of(text: &str) -> String {
    text.lines()
        .find(|l| !l.trim().is_empty())
        .map(|l| l.trim_start_matches('#').trim().to_owned())
        .unwrap_or_default()
}

impl Phone {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let started = std::time::Instant::now();
        let store = Store::open(db_path()).expect("open the store");
        let inbox = store.role_book(Role::Inbox, Origin::Ui).expect("inbox");
        let mut notes = list(&store);
        if notes.is_empty() {
            let long = long_note();
            for (title, body) in SEED.iter().copied().chain(std::iter::once(("Long note", long.as_str()))) {
                let _ = store.put(
                    DocType::Note,
                    json!({ "title": title, "body": body, "bookId": inbox }),
                    Put::Upsert,
                    Origin::Ui,
                );
            }
            notes = list(&store);
        }
        log::info!("kanso-mobile: store open, {} notes, {} ms", notes.len(), started.elapsed().as_millis());

        // The keyboard's height is a global the platform sets; poll it so the view re-lays out.
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_millis(60)).await;
                let Ok(()) = this.update(cx, |this, cx| {
                    let h = gpui_mobile::keyboard_height();
                    if (h - this.keyboard).abs() > 0.5 {
                        this.keyboard = h;
                        cx.notify();
                    }
                }) else {
                    break;
                };
            }
        })
        .detach();

        let mut phone = Phone { store, notes, open: None, keyboard: 0.0 };
        // Screenshots and smoke tests: KANSO_SPIKE_OPEN=<n> opens the nth note, KANSO_SPIKE_PREVIEW=1 its preview.
        if let Some(n) = std::env::var("KANSO_SPIKE_OPEN").ok().and_then(|n| n.parse::<usize>().ok()) {
            phone.open_note(n, window, cx);
            if std::env::var("KANSO_SPIKE_PREVIEW").is_ok() {
                if let Some(open) = phone.open.as_mut() {
                    open.preview = true;
                }
            }
        }
        phone
    }

    fn open_note(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(summary) = self.notes.get(index) else { return };
        let Ok(Some(note)) = self.store.note(&summary.id) else { return };
        let id = note.id.clone();
        let body = cx.new(|cx| {
            EditorState::new(window, cx)
                .language("markdown")
                .line_number(false)
                .soft_wrap(true)
                .auto_close(false)
                .default_value(note.body.clone())
        });
        cx.subscribe_in(&body, window, move |this, state, event, _window, cx| {
            match event {
                InputEvent::Change => {
                    let text = state.read(cx).value().to_string();
                    this.save(&text, cx);
                }
                // gpui-mobile never raises the keyboard itself: the app asks for it.
                InputEvent::Focus => gpui_mobile::show_keyboard(),
                InputEvent::Blur => gpui_mobile::hide_keyboard(),
                _ => {}
            }
        })
        .detach();
        self.open = Some(OpenNote { id, body, text: note.body, preview: false });
        cx.notify();
    }

    fn save(&mut self, text: &str, cx: &mut Context<Self>) {
        let Some(open) = self.open.as_mut() else { return };
        open.text = text.to_owned();
        let _ = self.store.put(
            DocType::Note,
            json!({ "_id": open.id, "title": title_of(text), "body": text }),
            Put::Merge,
            Origin::Ui,
        );
        self.notes = list(&self.store);
        cx.notify();
    }

    fn close_note(&mut self, cx: &mut Context<Self>) {
        self.open = None;
        self.notes = list(&self.store);
        cx.notify();
    }
}

fn list(store: &Store) -> Vec<NoteSummary> {
    store.search_summaries(&SearchParams::default()).unwrap_or_default()
}

impl Render for Phone {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (top, bottom, _, _) = gpui_mobile::safe_area_insets();
        let top = if top > 0.0 { top } else { 59.0 };
        let bottom = if self.keyboard > 0.0 { self.keyboard } else if bottom > 0.0 { bottom } else { 34.0 };
        let theme = cx.theme();
        let root = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.background)
            .text_color(theme.foreground)
            .pt(px(top))
            .pb(px(bottom));

        match &self.open {
            None => root.child(self.list_view(cx)),
            Some(open) => root.child(self.note_view(open, cx)),
        }
    }
}

impl Phone {
    fn list_view(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let (border, muted) = (theme.border, theme.muted_foreground);
        let rows = self.notes.iter().enumerate().map(|(i, n)| {
            div()
                .id(("note", i))
                .px_4()
                .py_3()
                .border_b_1()
                .border_color(border)
                .on_click(cx.listener(move |this, _, window, cx| this.open_note(i, window, cx)))
                .child(div().text_base().font_weight(gpui_kit::FontWeight::SEMIBOLD).child(n.title.clone()))
                .child(div().text_sm().text_color(muted).line_clamp(2).child(n.snippet.replace('\n', " ")))
        });
        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .child(div().px_4().py_3().text_2xl().font_weight(gpui_kit::FontWeight::BOLD).child("Notes"))
            .child(div().id("notes").flex_1().overflow_y_scroll().children(rows))
    }

    fn note_view(&self, open: &OpenNote, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let (border, accent, accent_fg) = (theme.border, theme.primary, theme.primary_foreground);
        let preview = open.preview;
        let tab = |label: &'static str, on: bool, which: bool| {
            let el = div()
                .id(label)
                .px_3()
                .py_1()
                .rounded_md()
                .child(label)
                .on_click(cx.listener(move |this, _, _, cx| {
                    if let Some(o) = this.open.as_mut() {
                        o.preview = which;
                    }
                    cx.notify();
                }));
            if on { el.bg(accent).text_color(accent_fg) } else { el }
        };
        let content = if preview {
            div()
                .id("preview")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .p_4()
                .child(TextView::markdown("preview", open.text.clone()).selectable(true).scrollable(false))
                .into_any_element()
        } else {
            div()
                .flex_1()
                .min_h_0()
                .child(Editor::new(&open.body).bordered(false).h_full().text_size(px(17.)))
                .into_any_element()
        };
        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(border)
                    .child(div().id("back").px_2().py_1().child("‹ Notes").on_click(cx.listener(|this, _, _, cx| this.close_note(cx))))
                    .child(div().flex().gap_1().child(tab("Edit", !preview, false)).child(tab("Preview", preview, true))),
            )
            .child(content)
    }
}

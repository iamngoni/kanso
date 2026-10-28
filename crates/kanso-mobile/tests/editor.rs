//! The editing session: what is typed is never lost, the colors are those of
//! the whole text, commands and undo are the engine's, typing is saved by
//! the autosave rule, and a note changed elsewhere while it is typed in
//! never loses either version.

mod common;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use common::*;
use kanso_core::properties;
use kanso_mobile::{
    EditCommand, EditorEvent, EditorSession, FormatKind, KansoError, Library, MemorySecrets,
    Range16, Renderer, SaveState, TextChange,
};
use kanso_sync::MemoryServer;
use proptest::prelude::*;

const THEME: &str = "kanso-light";

// MARK: Helpers

fn units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn string(units: &[u16]) -> String {
    String::from_utf16(units).expect("the model holds whole characters")
}

fn caret(at: usize) -> Range16 {
    Range16 {
        start: at as u32,
        end: at as u32,
    }
}

fn sel(a: usize, b: usize) -> Range16 {
    Range16 {
        start: a.min(b) as u32,
        end: a.max(b) as u32,
    }
}

/// A session and what the test knows of its text: a model in UTF-16 units.
struct Typed {
    session: EditorSession,
    model: Vec<u16>,
    version: u64,
}

impl Typed {
    fn open(phone: &Phone, body: &str) -> (Typed, String) {
        let id = note(&phone.library, "Title", body);
        let session = phone.library.edit(&id).unwrap();
        let snapshot = session.snapshot().unwrap();
        (
            Typed {
                session,
                model: units(&snapshot.text),
                version: snapshot.version,
            },
            id,
        )
    }

    /// Types `inserted` over units `a..b` as a text field would, and the
    /// same to the model: a range that cuts a surrogate pair takes in the
    /// whole character, an insertion point inside one goes before it.
    fn type_over(&mut self, a: usize, b: usize, inserted: &str) {
        let (a, b) = (a.min(b), a.max(b));
        let low = |model: &[u16], at: usize| {
            at > 0 && at < model.len() && (0xDC00..0xE000).contains(&model[at])
        };
        let start = if low(&self.model, a) { a - 1 } else { a };
        let end = if a == b {
            start
        } else if low(&self.model, b) {
            b + 1
        } else {
            b
        };
        let new = units(inserted);
        self.model.splice(start..end, new.iter().copied());
        let out = self
            .session
            .apply(TextChange {
                base_version: self.version,
                start: a as u32,
                end: b as u32,
                text: inserted.to_owned(),
                selection: caret(start + new.len()),
                composing: None,
            })
            .unwrap();
        assert_eq!(out.replace, None);
        assert!(out.version > self.version || (a == b && inserted.is_empty()));
        self.version = out.version;
    }

    fn text(&self) -> String {
        self.session.snapshot().unwrap().text
    }

    /// Applies an outcome's replacement to the model.
    fn follow(&mut self, out: &kanso_mobile::EditOutcome) {
        if let Some(replace) = &out.replace {
            self.model.splice(
                replace.start as usize..replace.end as usize,
                units(&replace.text),
            );
        }
        self.version = out.version;
    }
}

fn wait_until(what: &str, mut ready: impl FnMut() -> bool) {
    let end = Instant::now() + Duration::from_secs(10);
    while !ready() {
        assert!(Instant::now() < end, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// The events of a session, collected.
fn listen(session: &EditorSession) -> Arc<Mutex<Vec<EditorEvent>>> {
    let heard = Arc::new(Mutex::new(Vec::new()));
    let log = heard.clone();
    session.events(move |event| {
        log.lock().unwrap().push(event);
        true
    });
    heard
}

fn stored_body(library: &Library, id: &str) -> String {
    library.note(id).unwrap().body.unwrap()
}

fn string_edits() -> impl Strategy<Value = Vec<(u32, u32, String)>> {
    prop::collection::vec((0u32..60, 0u32..60, "[a-zé日🙂\n #*\\[]{0,5}"), 0..16)
}

// MARK: No edit loses text

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// Whatever ranges are typed over (cutting surrogate pairs too), the
    /// session's text is a `Vec<u16>` model of the same edits.
    #[test]
    fn prop_no_edit_loses_text(initial in "[a-zé日🙂\n #*]{0,24}", edits in string_edits()) {
        let phone = lone_phone();
        let (mut typed, _) = Typed::open(&phone, &format!("t{initial}"));
        for (a, b, inserted) in edits {
            let len = typed.model.len() as u32;
            typed.type_over((a % (len + 1)) as usize, (b % (len + 1)) as usize, &inserted);
            prop_assert_eq!(units(&typed.text()), typed.model.clone());
        }
    }
}

// no-prop-test: a change against an old version, and one outside the text.
#[test]
fn a_change_against_an_old_text_or_outside_it_is_refused() {
    let phone = lone_phone();
    let (mut typed, _) = Typed::open(&phone, "hello");
    typed.type_over(5, 5, "!");
    let stale = TextChange {
        base_version: typed.version - 1,
        start: 0,
        end: 0,
        text: "x".into(),
        selection: caret(1),
        composing: None,
    };
    assert!(matches!(
        typed.session.apply(stale),
        Err(KansoError::Invalid { .. })
    ));
    let outside = TextChange {
        base_version: typed.version,
        start: 3,
        end: 99,
        text: "x".into(),
        selection: caret(1),
        composing: None,
    };
    assert!(matches!(
        typed.session.apply(outside),
        Err(KansoError::Invalid { .. })
    ));
    assert_eq!(typed.text(), "hello!");
}

// MARK: Incremental equals full

/// A run as the tests compare them: its range, color, slant and weight.
type Style = (Range16, Option<u32>, Option<bool>, Option<u16>);

/// The runs of the full text between `a` and `b`, as the renderer gives them.
fn full_runs(text: &str, a: u32, b: u32) -> Vec<Style> {
    Renderer::styled_runs(text.to_owned(), a, b, THEME.into())
        .unwrap()
        .iter()
        .map(|run| (run.range, run.color, run.italic, run.weight))
        .collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// After any sequence of edits, with runs asked for after each one (so
    /// the cache is used across them), the runs of the session equal the
    /// runs the renderer computes on the whole text, in the whole text and
    /// in a window; and they are in bounds, in order, apart, and never cut a
    /// surrogate pair.
    #[test]
    fn prop_incremental_runs_equal_a_full_recompute(
        initial in "[#*`\\[\\]()a-z0-9 \n_>é日🙂|-]{0,60}",
        edits in prop::collection::vec((0u32..80, 0u32..80, "[#*`\\[\\]()a-z \n_>é日🙂|-]{0,5}"), 0..12),
        window in (0u32..100, 0u32..100),
    ) {
        let phone = lone_phone();
        let (mut typed, _) = Typed::open(&phone, &format!("t{initial}"));
        for (a, b, inserted) in edits {
            let len = typed.model.len() as u32;
            typed.type_over((a % (len + 1)) as usize, (b % (len + 1)) as usize, &inserted);
            let text = typed.text();
            let total = typed.model.len() as u32;
            for (from, to) in [(0, total), (window.0.min(total), window.1.min(total))] {
                let (from, to) = (from.min(to), from.max(to));
                let batch = typed.session.styled_runs(from, to, THEME.into()).unwrap();
                prop_assert_eq!(batch.version, typed.version);
                let styled: Vec<_> = batch
                    .runs
                    .iter()
                    .filter(|run| run.color.is_some() || run.italic.is_some() || run.weight.is_some())
                    .map(|run| (run.range, run.color, run.italic, run.weight))
                    .collect();
                prop_assert_eq!(styled, full_runs(&text, from, to));
                let mut at = 0;
                for run in &batch.runs {
                    let (s, e) = (run.range.start as usize, run.range.end as usize);
                    prop_assert!(s < e && e <= typed.model.len() && s >= at);
                    at = e;
                    for edge in [s, e] {
                        let inside_pair = edge > 0
                            && edge < typed.model.len()
                            && (0xD800..0xDC00).contains(&typed.model[edge - 1])
                            && (0xDC00..0xE000).contains(&typed.model[edge]);
                        prop_assert!(!inside_pair, "a run cuts a pair at {edge}");
                    }
                }
            }
        }
    }
}

// MARK: Commands and history

#[derive(Clone, Debug)]
enum Step {
    Select(u32, u32),
    Enter(bool),
    Tab(bool),
    Format(usize),
    Heading,
    Wiki,
    Insert(u32, u32, String),
    Type(u32, u32, String),
    Undo,
    Redo,
}

const FORMATS: [FormatKind; 19] = [
    FormatKind::Bold,
    FormatKind::Italic,
    FormatKind::Strikethrough,
    FormatKind::Code,
    FormatKind::Bulleted,
    FormatKind::Numbered,
    FormatKind::Task,
    FormatKind::Quote,
    FormatKind::Link,
    FormatKind::Image,
    FormatKind::Table,
    FormatKind::Rule,
    FormatKind::Heading1,
    FormatKind::Heading2,
    FormatKind::Heading3,
    FormatKind::Heading4,
    FormatKind::Heading5,
    FormatKind::Heading6,
    FormatKind::CodeBlock,
];

fn steps() -> impl Strategy<Value = Vec<Step>> {
    prop::collection::vec(
        prop_oneof![
            3 => (0u32..80, 0u32..80).prop_map(|(a, b)| Step::Select(a, b)),
            2 => any::<bool>().prop_map(Step::Enter),
            2 => any::<bool>().prop_map(Step::Tab),
            3 => (0..FORMATS.len()).prop_map(Step::Format),
            1 => Just(Step::Heading),
            1 => Just(Step::Wiki),
            1 => (0u32..80, 0u32..80, "[a-z\\[\\]]{0,4}").prop_map(|(a, b, t)| Step::Insert(a, b, t)),
            4 => (0u32..80, 0u32..80, "[a-zé日🙂\n ]{0,3}").prop_map(|(a, b, t)| Step::Type(a, b, t)),
            3 => Just(Step::Undo),
            2 => Just(Step::Redo),
        ],
        0..24,
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Commands, typing, undo and redo in any order: every outcome applied
    /// to the model gives the session's text, the outcome's selection is the
    /// session's, and after the last step undoing everything gives the
    /// first text and redoing everything the last.
    #[test]
    fn prop_outcomes_applied_to_the_model_are_the_text(
        initial in "[a-z\n #*|0-9\\[\\]x -]{0,40}",
        steps in steps(),
    ) {
        let phone = lone_phone();
        let first = format!("t{initial}");
        let (mut typed, _) = Typed::open(&phone, &first);
        for step in steps {
            let len = typed.model.len() as u32;
            let at = |n: u32| (n % (len + 1)) as usize;
            let outcome = match step {
                Step::Select(a, b) => {
                    typed.session.select(sel(at(a), at(b)), false).unwrap();
                    None
                }
                Step::Type(a, b, inserted) => {
                    typed.type_over(at(a), at(b), &inserted);
                    None
                }
                Step::Enter(plain) => typed.session.command(EditCommand::Enter { plain }).unwrap(),
                Step::Tab(out) => typed.session.command(EditCommand::Tab { out }).unwrap(),
                Step::Format(ix) => typed.session.command(EditCommand::Format { format: FORMATS[ix] }).unwrap(),
                Step::Heading => typed.session.command(EditCommand::CycleHeading).unwrap(),
                Step::Wiki => typed.session.command(EditCommand::WikiLink).unwrap(),
                Step::Insert(a, b, text) => typed.session.command(EditCommand::Insert { range: sel(at(a), at(b)), text }).unwrap(),
                Step::Undo => typed.session.undo().unwrap(),
                Step::Redo => typed.session.redo().unwrap(),
            };
            if let Some(out) = outcome {
                typed.follow(&out);
                prop_assert!(out.selection.start <= out.selection.end && out.selection.end as usize <= typed.model.len());
            }
            prop_assert_eq!(units(&typed.text()), typed.model.clone());
        }
        let last = typed.model.clone();
        let redo_waits = typed.session.snapshot().unwrap().history.can_redo;
        while let Some(out) = typed.session.undo().unwrap() {
            typed.follow(&out);
        }
        prop_assert_eq!(string(&typed.model), first);
        let history = typed.session.snapshot().unwrap().history;
        prop_assert!(!history.can_undo);
        while let Some(out) = typed.session.redo().unwrap() {
            typed.follow(&out);
        }
        // With nothing waiting to be redone before, everything undone is
        // done again.
        if !redo_waits {
            prop_assert_eq!(typed.model, last);
        }
        prop_assert!(!typed.session.snapshot().unwrap().history.can_redo);
    }
}

// no-prop-test: the examples the desktop's rules are known by; the rules
// themselves are property-tested in kanso_engine::edit.
// no-prop-test: the formats are property-tested in kanso_engine::edit; this
// is the facade's mapping of each heading level and the code block onto them.
#[test]
fn every_heading_level_and_the_code_block_format_the_line() {
    let phone = lone_phone();
    let headings = [
        FormatKind::Heading1,
        FormatKind::Heading2,
        FormatKind::Heading3,
        FormatKind::Heading4,
        FormatKind::Heading5,
        FormatKind::Heading6,
    ];
    for (ix, format) in headings.into_iter().enumerate() {
        let (mut typed, _) = Typed::open(&phone, "word");
        typed.session.select(caret(4), false).unwrap();
        let out = typed
            .session
            .command(EditCommand::Format { format })
            .unwrap()
            .unwrap();
        typed.follow(&out);
        assert_eq!(typed.text(), format!("{} word", "#".repeat(ix + 1)));
    }
    let (mut typed, _) = Typed::open(&phone, "word");
    typed.session.select(sel(0, 4), false).unwrap();
    let out = typed
        .session
        .command(EditCommand::Format {
            format: FormatKind::CodeBlock,
        })
        .unwrap()
        .unwrap();
    typed.follow(&out);
    assert!(typed.text().starts_with("```"), "{}", typed.text());
    assert!(typed.text().contains("word"));
}

#[test]
fn enter_continues_a_list_tab_moves_cells_and_indents() {
    let phone = lone_phone();
    let (mut typed, _) = Typed::open(&phone, "- one");
    typed.session.select(caret(5), false).unwrap();
    let out = typed
        .session
        .command(EditCommand::Enter { plain: false })
        .unwrap()
        .unwrap();
    typed.follow(&out);
    assert_eq!(typed.text(), "- one\n- ");
    assert_eq!(out.selection, caret(8));
    // Shift-Enter is a plain newline: nothing for the command to do.
    assert_eq!(
        typed
            .session
            .command(EditCommand::Enter { plain: true })
            .unwrap(),
        None
    );
    // The context knows the caret is in a list.
    let context = typed.session.select(caret(8), false).unwrap();
    assert!(context.in_list && !context.in_table && !context.in_code);
    // Tab with a list item in it moves the item; with nothing to move it
    // indents by the desktop editor's two spaces.
    let out = typed
        .session
        .command(EditCommand::Tab { out: false })
        .unwrap()
        .unwrap();
    typed.follow(&out);
    assert_eq!(typed.text(), "- one\n  - ");
    let (mut plain, _) = Typed::open(&phone, "word");
    plain.session.select(caret(4), false).unwrap();
    let out = plain
        .session
        .command(EditCommand::Tab { out: false })
        .unwrap()
        .unwrap();
    plain.follow(&out);
    assert_eq!(plain.text(), "word  ");
    // Undo takes the indent back whole.
    let out = plain.session.undo().unwrap().unwrap();
    plain.follow(&out);
    assert_eq!(plain.text(), "word");
    // A table: Tab goes to the next cell.
    let (table, _) = Typed::open(&phone, "| a | b |\n| --- | --- |\n| c | d |");
    table.session.select(caret(2), false).unwrap();
    assert!(table.session.select(caret(2), false).unwrap().in_table);
    let out = table
        .session
        .command(EditCommand::Tab { out: false })
        .unwrap()
        .unwrap();
    // The next cell's text is selected (the table is lined up on the way).
    let text = table.text();
    let selected = &units(&text)[out.selection.start as usize..out.selection.end as usize];
    assert_eq!(string(selected), "b");
}

// no-prop-test: the grouping rule is property-tested in
// kanso_engine::edit::history; this is the facade's mapping of a field's
// changes onto it.
#[test]
fn typed_words_are_one_undo_step_and_a_composition_is_one_too() {
    let phone = lone_phone();
    let (mut typed, _) = Typed::open(&phone, "");
    for (ix, ch) in "hello".chars().enumerate() {
        typed.type_over(ix, ix, &ch.to_string());
    }
    let out = typed.session.undo().unwrap().unwrap();
    typed.follow(&out);
    assert_eq!(typed.text(), "");
    assert!(!out.history.can_undo && out.history.can_redo);
    let out = typed.session.redo().unwrap().unwrap();
    typed.follow(&out);
    assert_eq!(typed.text(), "hello");
    // Kana composed: each keystroke replaces the composing text.
    let (mut kana, _) = Typed::open(&phone, "");
    let mut version = kana.version;
    let mut composed = String::new();
    for next in ["k", "ka", "か", "かん", "かんじ", "漢字"] {
        let before = units(&composed).len() as u32;
        let out = kana
            .session
            .apply(TextChange {
                base_version: version,
                start: 0,
                end: before,
                text: next.to_owned(),
                selection: caret(units(next).len()),
                composing: (next != "漢字").then(|| sel(0, units(next).len())),
            })
            .unwrap();
        version = out.version;
        composed = next.to_owned();
    }
    kana.version = version;
    kana.model = units("漢字");
    assert_eq!(kana.text(), "漢字");
    let out = kana.session.undo().unwrap().unwrap();
    kana.follow(&out);
    assert_eq!(kana.text(), "");
    assert!(!out.history.can_undo);
}

// no-prop-test: a table of what the completion offers in a small library.
#[test]
fn completion_offers_titles_and_tags_and_links_resolve() {
    let phone = lone_phone();
    let library = &phone.library;
    let parent = library.create_notebook("Projects".into(), None).unwrap();
    let child = library
        .create_notebook("Kanso".into(), Some(parent.id.clone()))
        .unwrap();
    let target = library
        .create(kanso_mobile::NewNote {
            notebook_id: Some(child.id.clone()),
            title: "Budget plan".into(),
            body: "SECRET words #budget".into(),
            ..Default::default()
        })
        .unwrap()
        .id;
    let (mut typed, _) = Typed::open(&phone, "see ");
    typed.type_over(4, 4, "[[Bud");
    typed.session.select(caret(9), false).unwrap();
    assert!(
        typed
            .session
            .select(caret(9), false)
            .unwrap()
            .completion_here
    );
    let completion = typed.session.completion().unwrap().unwrap();
    assert_eq!(completion.kind, kanso_mobile::CompletionKind::Wiki);
    assert_eq!(completion.replace, sel(4, 9));
    let first = &completion.items[0];
    assert_eq!(first.label, "Budget plan");
    assert_eq!(first.detail.as_deref(), Some("Projects / Kanso"));
    assert_eq!(first.highlight, Some(sel(0, 3)));
    assert!(first.insert.contains(&target["note:".len()..]));
    // Titles only: not a word of the note.
    assert!(!format!("{completion:?}").contains("SECRET"));
    assert!(completion.items.last().unwrap().new_note);
    // Choosing one is one step; the link then resolves to the note.
    let out = typed
        .session
        .command(EditCommand::Insert {
            range: completion.replace,
            text: first.insert.clone(),
        })
        .unwrap()
        .unwrap();
    typed.follow(&out);
    let end = typed.model.len() - 1;
    assert_eq!(
        typed.session.link_at(end as u32).unwrap(),
        Some(kanso_mobile::LinkTarget::Note { id: target.clone() })
    );
    // A `[[title]]` no note has is a new note; an address is an address.
    typed.type_over(
        typed.model.len(),
        typed.model.len(),
        " [[Nope]] https://kanso.app/x",
    );
    let text = typed.text();
    let at = |needle: &str| units(&text[..text.find(needle).unwrap() + 2]).len() as u32;
    assert_eq!(
        typed.session.link_at(at("[[Nope")).unwrap(),
        Some(kanso_mobile::LinkTarget::NewNote {
            title: "Nope".into()
        })
    );
    assert_eq!(
        typed.session.link_at(at("https")).unwrap(),
        Some(kanso_mobile::LinkTarget::Web {
            url: "https://kanso.app/x".into()
        })
    );
    // Tags.
    typed.type_over(typed.model.len(), typed.model.len(), "\n#bud");
    let end = typed.model.len();
    typed.session.select(caret(end), false).unwrap();
    let completion = typed.session.completion().unwrap().unwrap();
    assert_eq!(completion.kind, kanso_mobile::CompletionKind::Tag);
    assert_eq!(completion.items[0].label, "#budget");
    assert_eq!(completion.replace, sel(end - 4, end));
}

// MARK: The front matter and the properties

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// The text shown plus the hidden front matter is always the stored
    /// text, and editing the properties never changes the text shown.
    #[test]
    fn prop_the_visible_text_and_the_hidden_front_matter_are_the_stored_body(
        entries in prop::collection::vec(("k[a-z]{1,4}", "[a-z0-9]{1,6}"), 0..4),
        text in "t[a-z\n ]{0,20}",
        ops in prop::collection::vec(
            prop_oneof![
                ("n[a-z]{1,4}", "[a-z0-9]{1,6}").prop_map(|(k, v)| (0u8, 0u32, k, v)),
                (0u32..6, "[a-z0-9]{1,6}").prop_map(|(i, v)| (1u8, i, String::new(), v)),
                (0u32..6).prop_map(|i| (2u8, i, String::new(), String::new())),
                "[a-z]{1,3}".prop_map(|v| (3u8, 0u32, String::new(), v)),
            ],
            0..10,
        ),
    ) {
        let mut seen = std::collections::HashSet::new();
        let entries: Vec<_> = entries.into_iter().filter(|(k, _)| seen.insert(k.clone())).collect();
        let front: String = entries.iter().map(|(k, v)| format!("{k}: {v}\n")).collect();
        let body = if entries.is_empty() { text.clone() } else { format!("---\n{front}---\n{text}") };
        let phone = lone_phone();
        let (mut typed, id) = Typed::open(&phone, &body);
        prop_assert_eq!(typed.text(), text.clone());
        prop_assert_eq!(typed.session.snapshot().unwrap().properties.len(), entries.len());
        for (kind, index, key, value) in ops {
            let before = typed.text();
            let count = typed.session.snapshot().unwrap().properties.len() as u32;
            match kind {
                0 => { typed.session.set_property(None, key, value).unwrap(); }
                1 if count > 0 => { typed.session.set_property(Some(index % count), String::new(), value).unwrap(); }
                2 if count > 0 => {
                    let after = typed.session.remove_property(index % count).unwrap();
                    prop_assert_eq!(after.len() as u32, count - 1);
                }
                3 => {
                    let end = typed.model.len();
                    typed.type_over(end, end, &value);
                    continue;
                }
                _ => {}
            }
            prop_assert_eq!(typed.text(), before, "a property edit changed the text");
        }
        typed.session.flush().unwrap();
        let snapshot = typed.session.snapshot().unwrap();
        let stored = stored_body(&phone.library, &id);
        let shown = properties::split(&stored).map_or(stored.as_str(), |front| &stored[front.body_start..]);
        prop_assert_eq!(shown, snapshot.text.as_str());
        prop_assert_eq!(phone.library.properties(&id).unwrap(), snapshot.properties);
    }
}

// MARK: Autosave

// no-prop-test: the rule is property-tested in kanso_engine::saving; this
// is the session following it with the clock the test passes.
#[test]
fn typing_is_saved_600ms_after_the_last_change_and_not_before() {
    let phone = lone_phone();
    let (mut typed, id) = Typed::open(&phone, "base");
    let before = Instant::now();
    typed.type_over(4, 4, "a");
    typed.session.tick(before + Duration::from_millis(599));
    assert_eq!(stored_body(&phone.library, &id), "base");
    assert_eq!(typed.session.snapshot().unwrap().save, SaveState::Pending);
    // Another change pushes the save out again.
    let again = Instant::now();
    typed.type_over(5, 5, "b");
    let after = Instant::now();
    typed.session.tick(again + Duration::from_millis(599));
    assert_eq!(stored_body(&phone.library, &id), "base");
    let events = listen(&typed.session);
    typed.session.tick(after + Duration::from_millis(601));
    assert_eq!(stored_body(&phone.library, &id), "baseab");
    assert!(matches!(
        typed.session.snapshot().unwrap().save,
        SaveState::Saved { .. }
    ));
    assert!(
        events
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e, EditorEvent::Saved { .. }))
    );
}

// no-prop-test: a thread and the clock; the rule is property-tested.
#[test]
fn a_library_saves_typing_by_the_clock() {
    let dir = tempfile::tempdir().unwrap();
    let library = Library::open_with(
        &dir.path().join("library.db"),
        Arc::new(MemorySecrets::new()),
        Arc::new(MemoryServer::new()),
    )
    .unwrap();
    let id = note(&library, "T", "base");
    let session = library.edit(&id).unwrap();
    let started = Instant::now();
    session
        .apply(TextChange {
            base_version: session.snapshot().unwrap().version,
            start: 4,
            end: 4,
            text: "!".into(),
            selection: caret(5),
            composing: None,
        })
        .unwrap();
    wait_until("the autosave", || stored_body(&library, &id) == "base!");
    assert!(
        started.elapsed() >= Duration::from_millis(550),
        "saved too soon: {:?}",
        started.elapsed()
    );
}

// no-prop-test: threads and the clock; the rule is property-tested and the
// conflict logic is property-tested above with no write under way.
#[test]
fn typing_goes_on_while_the_library_is_written_and_loses_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let library = Library::open_with(
        &dir.path().join("library.db"),
        Arc::new(MemorySecrets::new()),
        Arc::new(MemoryServer::new()),
    )
    .unwrap();
    let id = note(&library, "T", "");
    let session = library.edit(&id).unwrap();
    let events = listen(&session);
    let mut typed = Typed {
        model: Vec::new(),
        version: session.snapshot().unwrap().version,
        session,
    };
    // A keystroke lands just as the autosave of the one before is due, so
    // that some of them are typed while its write is under way.
    for round in 0..10usize {
        for _ in 0..3 {
            let end = typed.model.len();
            typed.type_over(end, end, "x");
        }
        std::thread::sleep(Duration::from_millis(596 + (round as u64 % 5) * 2));
        let end = typed.model.len();
        typed.type_over(end, end, "y");
    }
    // Everything is saved, and the session never asked about its own
    // writes.
    let save = typed.session.flush().unwrap();
    assert!(matches!(save, SaveState::Saved { .. }), "{save:?}");
    assert_eq!(stored_body(&library, &id), string(&typed.model));
    assert_eq!(typed.text(), string(&typed.model));
    let heard = events.lock().unwrap();
    assert!(
        heard.iter().all(|e| matches!(e, EditorEvent::Saved { .. })),
        "{heard:?}"
    );
    assert!(!heard.is_empty());
}

// no-prop-test: the half-typed tag; apply_inline_tags is tested in
// kanso_engine::links::tags.
#[test]
fn a_tag_still_being_typed_waits_for_a_pause() {
    let phone = lone_phone();
    let (mut typed, id) = Typed::open(&phone, "x ");
    typed.type_over(2, 2, "#hal");
    typed.session.tick(Instant::now() + Duration::from_secs(5));
    let names = |phone: &Phone| {
        phone
            .library
            .tags()
            .unwrap()
            .into_iter()
            .map(|t| t.name)
            .collect::<Vec<_>>()
    };
    assert_eq!(stored_body(&phone.library, &id), "x #hal");
    assert!(names(&phone).is_empty(), "{:?}", names(&phone));
    // The next word after it is the pause: the tag is the note's.
    typed.type_over(6, 6, " ");
    typed.session.tick(Instant::now() + Duration::from_secs(5));
    assert_eq!(names(&phone), vec!["hal".to_owned()]);
}

// MARK: Conflicts

fn revision_bodies(phone: &Phone, id: &str) -> Vec<String> {
    phone
        .library
        .store()
        .revisions(id)
        .unwrap()
        .into_iter()
        .filter_map(|r| {
            r.fields
                .get("body")
                .and_then(|b| b.as_str())
                .map(str::to_owned)
        })
        .collect()
}

fn write_outside(phone: &Phone, id: &str, body: &str) {
    let rev = phone.library.note(id).unwrap().rev;
    phone.library.save(id, &rev, body.to_owned()).unwrap();
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// Typing meets a change made elsewhere: Keep mine writes the typed
    /// text and theirs is in history; Use theirs shows theirs and the typed
    /// text is in history. Either way neither version is lost.
    #[test]
    fn prop_a_conflict_loses_neither_version(
        mine in "[a-z]{1,8}",
        theirs in "[A-Z]{1,8}",
        keep_mine: bool,
    ) {
        let phone = lone_phone();
        let (mut typed, id) = Typed::open(&phone, "base");
        let events = listen(&typed.session);
        typed.type_over(4, 4, &mine);
        write_outside(&phone, &id, &theirs);
        wait_until("the question", || {
            events.lock().unwrap().iter().any(|e| matches!(e, EditorEvent::Incoming { deleted: false }))
        });
        prop_assert_eq!(typed.session.snapshot().unwrap().save, SaveState::Conflict { deleted: false });
        // Nothing is written while the question is open.
        typed.session.tick(Instant::now() + Duration::from_secs(60));
        prop_assert_eq!(stored_body(&phone.library, &id), theirs.clone());
        let mine_text = format!("base{mine}");
        if keep_mine {
            let save = typed.session.keep_mine().unwrap();
            let is_saved = matches!(save, SaveState::Saved { .. });
            prop_assert!(is_saved, "{:?}", save);
            prop_assert_eq!(stored_body(&phone.library, &id), mine_text);
            prop_assert!(revision_bodies(&phone, &id).contains(&theirs));
        } else {
            let shown = typed.session.use_theirs().unwrap();
            prop_assert_eq!(shown.text, theirs.clone());
            prop_assert_eq!(stored_body(&phone.library, &id), theirs);
            prop_assert!(revision_bodies(&phone, &id).contains(&mine_text));
            prop_assert!(!shown.history.can_undo);
        }
    }

    /// Typing, outside writes, saves and choices in any order: whenever a
    /// question is answered, the typed text and the other version are both
    /// somewhere (the library's text, or its history), and a note with no
    /// typing waiting reloads to the other version.
    #[test]
    fn prop_interleavings_never_lose_either_version(
        ops in prop::collection::vec(
            prop_oneof![
                4 => "[a-z]{1,3}".prop_map(|t| (0u8, t)),
                3 => Just((1u8, String::new())),
                2 => Just((2u8, String::new())),
                2 => Just((3u8, String::new())),
                2 => Just((4u8, String::new())),
                2 => Just((5u8, String::new())),
            ],
            1..14,
        ),
    ) {
        let phone = lone_phone();
        let (mut typed, id) = Typed::open(&phone, "base");
        let events = listen(&typed.session);
        let mut outside = 0;
        for (kind, text) in ops {
            let heard = |events: &Arc<Mutex<Vec<EditorEvent>>>| {
                events.lock().unwrap().iter()
                    .filter(|e| matches!(e, EditorEvent::Incoming { .. } | EditorEvent::Reloaded { .. }))
                    .count()
            };
            match kind {
                0 => {
                    // Typing at the end, when the model still matches (a
                    // reload may have moved it).
                    let snapshot = typed.session.snapshot().unwrap();
                    typed.model = units(&snapshot.text);
                    typed.version = snapshot.version;
                    let end = typed.model.len();
                    typed.type_over(end, end, &text);
                }
                1 => {
                    let seen = heard(&events);
                    outside += 1;
                    write_outside(&phone, &id, &format!("OUT{outside}:{}", "x".repeat(outside)));
                    wait_until("the change", || heard(&events) > seen);
                }
                2 => { typed.session.tick(Instant::now() + Duration::from_secs(60)); }
                3 => { typed.session.flush().unwrap(); }
                4 | 5 => {
                    let snapshot = typed.session.snapshot().unwrap();
                    if let SaveState::Conflict { .. } = snapshot.save {
                        let mine = snapshot.text.clone();
                        let theirs = stored_body(&phone.library, &id);
                        if kind == 4 {
                            typed.session.keep_mine().unwrap();
                            prop_assert_eq!(stored_body(&phone.library, &id), mine);
                            prop_assert!(revision_bodies(&phone, &id).contains(&theirs));
                        } else {
                            let shown = typed.session.use_theirs().unwrap();
                            prop_assert_eq!(shown.text, theirs.clone());
                            prop_assert!(revision_bodies(&phone, &id).contains(&mine));
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        // Leaving with a question open keeps what was typed.
        let snapshot = typed.session.snapshot().unwrap();
        let mine = snapshot.text.clone();
        let theirs = stored_body(&phone.library, &id);
        let asked = matches!(snapshot.save, SaveState::Conflict { .. });
        typed.session.close().unwrap();
        let after = stored_body(&phone.library, &id);
        if asked {
            prop_assert_eq!(&after, &mine);
            prop_assert!(revision_bodies(&phone, &id).contains(&theirs));
        } else {
            prop_assert_eq!(after, mine);
        }
    }
}

// no-prop-test: the deleted case, one note and its three answers.
#[test]
fn a_note_deleted_elsewhere_can_be_kept_or_let_go() {
    for keep in [true, false] {
        let phone = lone_phone();
        let (mut typed, id) = Typed::open(&phone, "base");
        let events = listen(&typed.session);
        typed.type_over(4, 4, " mine");
        phone
            .library
            .store()
            .delete(&id, kanso_store::Origin::Ui)
            .unwrap();
        wait_until("the question", || {
            events
                .lock()
                .unwrap()
                .iter()
                .any(|e| matches!(e, EditorEvent::Incoming { deleted: true }))
        });
        assert_eq!(
            typed.session.snapshot().unwrap().save,
            SaveState::Conflict { deleted: true }
        );
        if keep {
            typed.session.keep_mine().unwrap();
            assert_eq!(stored_body(&phone.library, &id), "base mine");
        } else {
            assert_eq!(typed.session.use_theirs(), Err(KansoError::NotFound));
            assert!(events.lock().unwrap().contains(&EditorEvent::Closed));
            assert_eq!(typed.session.snapshot(), Err(KansoError::NotFound));
        }
    }
}

// no-prop-test: a sync that only moved the revision, and a note with
// nothing typed that reloads.
#[test]
fn a_note_changed_elsewhere_with_nothing_typed_reloads_and_starts_its_history_over() {
    let phone = lone_phone();
    let (mut typed, id) = Typed::open(&phone, "base");
    typed.type_over(4, 4, "!");
    typed.session.flush().unwrap();
    assert!(typed.session.snapshot().unwrap().history.can_undo);
    let events = listen(&typed.session);
    write_outside(&phone, &id, "from elsewhere");
    wait_until("the reload", || {
        events
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e, EditorEvent::Reloaded { .. }))
    });
    let snapshot = typed.session.snapshot().unwrap();
    assert_eq!(snapshot.text, "from elsewhere");
    assert!(!snapshot.history.can_undo, "the history starts over");
    assert!(snapshot.version > typed.version);
    // A change typed against the text from before the reload is refused.
    let late = typed.session.apply(TextChange {
        base_version: typed.version,
        start: 0,
        end: 0,
        text: "x".into(),
        selection: caret(1),
        composing: None,
    });
    assert!(matches!(late, Err(KansoError::Invalid { .. })));
}

// MARK: A long note

// no-prop-test: one keystroke in a fixed 50,000-word note; the highlighter's
// equality with a full recompute is property-tested above and in
// kanso_engine::highlight.
#[test]
fn one_keystroke_in_a_50000_word_note_parses_at_most_two_blocks() {
    let text = long_note(50_000);
    assert!(text.split_whitespace().count() >= 50_000);
    let phone = lone_phone();
    let (mut typed, _) = Typed::open(&phone, &text);
    let total = typed.model.len() as u32;
    // Everything highlighted once, as scrolling through it would.
    typed.session.styled_runs(0, total, THEME.into()).unwrap();
    let parsed = typed.session.blocks_parsed();
    assert!(parsed > 1000, "{parsed}");
    // A keystroke in the middle of a paragraph.
    let needle = "Paragraph 1351 has";
    let at = units(&text[..text.find(needle).unwrap() + needle.len()]).len();
    typed.type_over(at, at, "x");
    let total = typed.model.len() as u32;
    let batch = typed.session.styled_runs(0, total, THEME.into()).unwrap();
    let reparsed = typed.session.blocks_parsed() - parsed;
    assert!(reparsed <= 2, "{reparsed} blocks parsed for one keystroke");
    // The runs are those of the whole text, computed from nothing: a second
    // session on the same text, with an empty cache. (The renderer's own
    // recompute of a note this long takes close to a minute in a debug build; its
    // equality with the cache is the property above.)
    let styled: Vec<_> = batch
        .runs
        .iter()
        .map(|run| (run.range, run.color, run.italic, run.weight, run.quiet))
        .collect();
    let id = note(&phone.library, "Again", &typed.text());
    let cold = phone.library.edit(&id).unwrap();
    assert_eq!(cold.blocks_parsed(), 0);
    let fresh: Vec<_> = cold
        .styled_runs(0, total, THEME.into())
        .unwrap()
        .runs
        .iter()
        .map(|run| (run.range, run.color, run.italic, run.weight, run.quiet))
        .collect();
    assert!(cold.blocks_parsed() > 1000);
    assert_eq!(styled, fresh);
}

// no-prop-test: a fixed 50,000-word note with a syntax error in it, and the
// cost of one keystroke for the window the editor asks for; the equality
// with a full parse is property H6 in kanso_engine::highlight and the
// property above.
#[test]
fn a_keystroke_in_a_50000_word_note_with_an_error_parses_a_bounded_window() {
    let text = long_note_with_error(50_000);
    let phone = lone_phone();
    let (mut typed, _) = Typed::open(&phone, &text);
    let window = |typed: &Typed, at: usize| {
        let start = at.saturating_sub(4096) as u32;
        (start, (start + 8192).min(typed.model.len() as u32))
    };
    // Everything highlighted once, as scrolling through it would.
    let total = typed.model.len() as u32;
    typed.session.styled_runs(0, total, THEME.into()).unwrap();
    for (label, at) in [
        ("before the error", text.len() * 2 / 5),
        ("after the error", text.len() * 9 / 10),
    ] {
        let at = units(&text[..(0..=at).rev().find(|b| text.is_char_boundary(*b)).unwrap()]).len();
        let mut newest = None;
        let parsed = typed.session.blocks_parsed();
        for i in 0..5 {
            typed.type_over(at + i, at + i, "x");
            let (start, end) = window(&typed, at + i);
            newest = Some(typed.session.styled_runs(start, end, THEME.into()).unwrap());
        }
        let reparsed = typed.session.blocks_parsed() - parsed;
        // Five keystrokes, each in one block, and after the error that
        // block is the error's parse, made for the window: at most two
        // parses a keystroke, never the 2,800 blocks of the note.
        assert!(reparsed <= 10, "{label}: {reparsed} blocks parsed");
        // The runs are in the window, in order, apart, and those a cold
        // session on the same text gives for it.
        let batch = newest.unwrap();
        let (start, end) = window(&typed, at + 4);
        let mut last = start;
        for run in &batch.runs {
            assert!(run.range.start >= last && run.range.end > run.range.start);
            assert!(run.range.end <= end, "{label}: {run:?} past {end}");
            last = run.range.end;
        }
        assert!(
            batch.runs.len() < 2_000,
            "{label}: {} runs",
            batch.runs.len()
        );
        let id = note(&phone.library, "Cold", &typed.text());
        let cold = phone.library.edit(&id).unwrap();
        let fresh = cold.styled_runs(start, end, THEME.into()).unwrap();
        assert_eq!(batch.runs, fresh.runs, "{label}");
    }
}

// MARK: Locked notes

// no-prop-test: the guarantee is property-tested in tests/locked.rs; this is
// the sequence it is about, step by step, with the auto-lock as well.
#[test]
fn locking_saves_what_is_typed_sealed_then_ends_the_session() {
    for by_idle in [false, true] {
        let phone = lone_phone();
        let library = &phone.library;
        let id = note(library, "Secret", "the plan SECRET-WORDS");
        let locks = library.locks();
        locks.lock_note(&id, LOCK_PASSWORD.into()).unwrap();
        let (mut typed, _) = {
            let session = library.edit(&id).unwrap();
            let snapshot = session.snapshot().unwrap();
            assert_eq!(snapshot.lock, kanso_mobile::LockState::Open);
            (
                Typed {
                    session,
                    model: units(&snapshot.text),
                    version: snapshot.version,
                },
                (),
            )
        };
        let events = listen(&typed.session);
        typed.type_over(0, 0, "TYPED ");
        // Saving a locked note seals it and keeps it open for the auto-lock
        // time; the library has none of it in the clear.
        typed.session.flush().unwrap();
        let raw = library.store().note(&id).unwrap().unwrap();
        assert!(raw.is_locked() && raw.body.is_empty());
        // More typing, not yet saved, when the lock closes.
        typed.type_over(6, 6, "AND MORE ");
        if by_idle {
            let later = Instant::now() + kanso_engine::lock::AUTO_LOCK + Duration::from_secs(1);
            assert!(locks.lock_if_idle(later));
        } else {
            locks.lock().unwrap();
        }
        assert_eq!(
            typed.session.snapshot().map(|_| ()),
            Err(KansoError::Locked)
        );
        assert_eq!(typed.session.undo().map(|_| ()), Err(KansoError::Locked));
        assert_eq!(
            typed.session.completion().map(|_| ()),
            Err(KansoError::Locked)
        );
        assert_eq!(library.edit(&id).map(|_| ()), Err(KansoError::Locked));
        let heard = events.lock().unwrap();
        assert_eq!(heard.last(), Some(&EditorEvent::Closed));
        assert!(!format!("{heard:?}").contains("SECRET-WORDS"));
        drop(heard);
        // Sealed in the store, and all of it saved.
        let raw = library.store().note(&id).unwrap().unwrap();
        assert!(raw.is_locked() && raw.body.is_empty());
        assert!(locks.unlock(LOCK_PASSWORD.into()).unwrap());
        assert_eq!(
            library.note(&id).unwrap().body.as_deref(),
            Some("TYPED AND MORE the plan SECRET-WORDS")
        );
    }
}

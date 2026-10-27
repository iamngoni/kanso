//! Themes: listing, fitting three together, importing, and the diagram
//! renderer; and that an imported theme gives the colors on the phone that
//! the desktop's fitting gives for the same files.

mod common;

use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};

use kanso_engine::theme::Tokens;
use kanso_engine::theme::builtin::{BUILTIN, id_of};
use kanso_engine::theme::file::ThemeFile;
use kanso_mobile::{
    DiagramRenderer, KansoError, Library, MemorySecrets, Renderer, ThemeOrigin, ThemeTokens, Themes,
};
use kanso_sync::MemoryServer;
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

/// A Zed theme family with two themes, comments and a trailing comma as Zed
/// allows.
const ZED_FAMILY: &str = r##"{ // Zed allows comments
    "name": "Zed Fam", "author": "Me",
    "themes": [
      {"name": "Zed Night", "appearance": "dark", "style": {"editor.background": "#101820", "text": "#e6edf3", "syntax": {"keyword": {"color": "#ff7b72"}, "string": {"color": "#a5d6ff"}}}},
      {"name": "Zed Day", "appearance": "light", "style": {"editor.background": "#fafafa", "text": "#202020",}},
    ]
}"##;

/// A Kanso theme file written by hand.
const KANSO_FILE: &str = r##"{"name": "Hand Made", "appearance": "light", "ui": {"editor": "#ffffff", "text": "#000000", "accent": "#0000ff"}}"##;

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

/// A library on a folder of its own.
struct Fixture {
    dir: tempfile::TempDir,
    themes: Themes,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let library = Library::open_for_test(
        &dir.path().join("library.db"),
        Arc::new(MemorySecrets::new()),
        Arc::new(MemoryServer::new()),
    )
    .unwrap();
    Fixture {
        themes: library.themes(),
        dir,
    }
}

impl Fixture {
    fn file(&self, name: &str, text: &str) -> PathBuf {
        let path = self.dir.path().join(name);
        std::fs::write(&path, text).unwrap();
        path
    }

    fn own(&self) -> PathBuf {
        self.dir.path().join("themes")
    }
}

/// A theme package packed as npm packs it, as a `.tgz`.
fn package_tgz(dir: &Path, name: &str, accent: &str) -> PathBuf {
    let pack = |archive: &mut tar::Builder<flate2::write::GzEncoder<std::fs::File>>,
                path: &str,
                text: &str| {
        let mut header = tar::Header::new_gnu();
        header.set_size(text.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        archive
            .append_data(&mut header, path, text.as_bytes())
            .unwrap();
    };
    let tgz = dir.join(format!("{name}-1.0.0.tgz"));
    let file = std::fs::File::create(&tgz).unwrap();
    let mut archive = tar::Builder::new(flate2::write::GzEncoder::new(
        file,
        flate2::Compression::default(),
    ));
    pack(
        &mut archive,
        "package/package.json",
        &format!(
            r#"{{"name": "{name}", "theme": true, "themeAppearance": "light", "styleSheets": ["ui.css"], "author": "A <a@b.c>", "license": "MIT"}}"#
        ),
    );
    pack(
        &mut archive,
        "package/styles/ui.css",
        &format!(":root {{ --primary-color: {accent}; }}"),
    );
    archive.into_inner().unwrap().finish().unwrap();
    tgz
}

fn builtin_ids() -> Vec<&'static str> {
    BUILTIN.iter().map(|(file, _)| id_of(file)).collect()
}

// no-prop-test: the order and content of the built-in rows, fixed files; the
// fitting is held to the desktop's by `prop_fits_match_the_desktops`.
#[test]
fn the_list_starts_with_the_built_in_themes_in_the_desktops_order() {
    let f = fixture();
    let rows = f.themes.list().unwrap();
    let ids: Vec<&str> = rows.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids, builtin_ids());
    assert_eq!(&ids[..2], ["kanso-light", "kanso-dark"]);
    assert!(rows.iter().all(|r| r.origin == ThemeOrigin::Builtin));
    assert!(f.themes.notices().unwrap().is_empty());
    let kanso = &rows[1];
    assert!(kanso.dark && kanso.tokens.dark);
    assert_eq!(kanso.name, "Kanso Dark");
}

// no-prop-test: files written once, listed.
#[test]
fn an_imported_zed_theme_and_package_are_listed_as_the_persons_own() {
    let f = fixture();
    let zed = f.file("family.json", ZED_FAMILY);
    let imported = f.themes.import(zed.to_string_lossy().into_owned()).unwrap();
    assert_eq!(imported.names, ["Zed Night", "Zed Day"]);
    assert_eq!(imported.ids, ["zed-night", "zed-day"]);
    assert!(f.own().join("zed-night.json").is_file());

    let tgz = package_tgz(f.dir.path(), "tiny-light", "#0a58ca");
    let imported = f.themes.import(tgz.to_string_lossy().into_owned()).unwrap();
    assert_eq!(imported.ids, ["tiny-light"]);
    assert_eq!(imported.names, ["Tiny Light"]);

    let rows = f.themes.list().unwrap();
    let own: Vec<(&str, ThemeOrigin)> = rows[builtin_ids().len()..]
        .iter()
        .map(|r| (r.name.as_str(), r.origin))
        .collect();
    // The person's own come after the built-in ones, by name.
    assert_eq!(
        own,
        [
            ("Tiny Light", ThemeOrigin::Own),
            ("Zed Day", ThemeOrigin::Own),
            ("Zed Night", ThemeOrigin::Own),
        ]
    );
    let night = rows.iter().find(|r| r.id == "zed-night").unwrap();
    assert!(night.dark);
    assert_eq!(night.author, "Me");
    // Fitted alone, as the desktop fits a theme chosen for all three.
    assert_eq!(
        night.tokens,
        f.themes.fit("zed-night".into(), None, None).unwrap()
    );
}

// no-prop-test: files written once, read back.
#[test]
fn a_file_that_fails_is_a_notice_and_a_taken_name_is_a_notice() {
    let f = fixture();
    std::fs::create_dir_all(f.own()).unwrap();
    std::fs::write(f.own().join("broken.json"), "{ not json").unwrap();
    std::fs::write(
        f.own().join("clash.json"),
        KANSO_FILE.replace("Hand Made", "nord"),
    )
    .unwrap();
    std::fs::write(f.own().join("fine.json"), KANSO_FILE).unwrap();
    let rows = f.themes.list().unwrap();
    assert!(rows.iter().all(|r| r.id != "broken" && r.id != "clash"));
    assert!(
        rows.iter()
            .any(|r| r.id == "fine" && r.origin == ThemeOrigin::Own)
    );
    let notices = f.themes.notices().unwrap();
    let files: Vec<&str> = notices.iter().map(|n| n.file.as_str()).collect();
    assert_eq!(files, ["broken.json", "clash.json"]);
    assert!(
        notices[0]
            .sentence
            .starts_with("broken.json isn't valid JSON")
    );
    assert!(notices[0].sentence.ends_with('.'));
    assert_eq!(
        notices[1].sentence,
        "clash.json is named \"nord\", like another theme; give it a name of its own."
    );
}

// no-prop-test: a table of cases for the ids.
#[test]
fn ids_stay_unique_and_gallery_themes_follow_the_persons_own() {
    let f = fixture();
    std::fs::create_dir_all(f.own()).unwrap();
    // A file named like a built-in's id, with a name of its own.
    std::fs::write(f.own().join("nord.json"), KANSO_FILE).unwrap();
    let gallery = f.dir.path().join("gallery");
    std::fs::create_dir_all(gallery.join("themes")).unwrap();
    std::fs::write(
        gallery.join("themes/from-gallery.json"),
        KANSO_FILE.replace("Hand Made", "Gallery One"),
    )
    .unwrap();
    // Disabled by the gallery, so not loaded.
    std::fs::write(
        gallery.join("themes/off.json"),
        KANSO_FILE.replace("Hand Made", "Gallery Off"),
    )
    .unwrap();
    std::fs::write(gallery.join("state.json"), r#"{"disabled": ["off"]}"#).unwrap();
    // The same name as the person's own: theirs wins.
    std::fs::write(gallery.join("themes/dup.json"), KANSO_FILE).unwrap();
    let rows = f.themes.list().unwrap();
    let mut ids: Vec<&str> = rows.iter().map(|r| r.id.as_str()).collect();
    let count = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), count, "ids are unique");
    let own = rows.iter().find(|r| r.name == "Hand Made").unwrap();
    assert_eq!(
        (own.id.as_str(), own.origin),
        ("own-nord", ThemeOrigin::Own)
    );
    let last = rows.last().unwrap();
    assert_eq!(
        (last.id.as_str(), last.origin),
        ("from-gallery", ThemeOrigin::Gallery)
    );
    assert!(rows.iter().all(|r| r.id != "off" && r.id != "dup"));
}

// no-prop-test: error cases, each with its sentence.
#[test]
fn unknown_ids_are_not_found_and_import_errors_say_why() {
    let f = fixture();
    assert_eq!(
        f.themes.fit("nope".into(), None, None).unwrap_err(),
        KansoError::NotFound
    );
    assert_eq!(
        f.themes
            .fit("nord".into(), Some("nope".into()), None)
            .unwrap_err(),
        KansoError::NotFound
    );
    assert_eq!(
        f.themes
            .fit("nord".into(), None, Some("nope".into()))
            .unwrap_err(),
        KansoError::NotFound
    );
    assert_eq!(
        f.themes
            .styled_runs("# a".into(), 0, 3, "nord".into(), Some("nope".into()))
            .unwrap_err(),
        KansoError::NotFound
    );
    let message = |path: &Path| match f.themes.import(path.to_string_lossy().into_owned()) {
        Err(KansoError::Invalid { message }) => message,
        other => panic!("{other:?}"),
    };
    assert!(message(&f.dir.path().join("missing.json")).contains("doesn't exist"));
    let txt = f.file("notes.txt", "hi");
    assert!(message(&txt).contains("neither"));
    let bad = f.file("bad.tgz", "not gzip");
    assert!(message(&bad).contains("couldn't be unpacked"));
    let json = f.file("other.json", r#"{"hello": 1}"#);
    assert!(message(&json).contains("isn't a theme Kanso can import"));
    // Nothing was written for any of them.
    assert!(f.themes.list().unwrap().len() == builtin_ids().len());
}

// no-prop-test: the built-in themes' runs are `Renderer::styled_runs`'s
// own, compared for each of the first few themes.
#[test]
fn styled_runs_with_a_built_in_theme_match_the_renderers() {
    let f = fixture();
    let text = "# Title\n\nSome *emphasis* and `code`.\n";
    for id in builtin_ids().into_iter().take(6) {
        let end = text.encode_utf16().count() as u32;
        let mine = f
            .themes
            .styled_runs(text.into(), 0, end, id.into(), None)
            .unwrap();
        assert_eq!(
            mine,
            Renderer::styled_runs(text.into(), 0, end, id.into()).unwrap(),
            "{id}"
        );
        assert!(!mine.is_empty(), "{id}");
    }
}

// no-prop-test: the diagram renderer is a process-wide switch, so one test
// sets and restores it.
#[test]
fn the_diagram_renderer_is_set_and_read_back() {
    let before = Renderer::diagram_renderer();
    Renderer::set_diagram_renderer(DiagramRenderer::Kanso);
    assert_eq!(Renderer::diagram_renderer(), DiagramRenderer::Kanso);
    Renderer::set_diagram_renderer(DiagramRenderer::Pele);
    assert_eq!(Renderer::diagram_renderer(), DiagramRenderer::Pele);
    Renderer::set_diagram_renderer(before);
}

// no-prop-test: both calls fail the same way until the icon themes arrive.
#[test]
fn icon_themes_say_they_are_not_here_yet() {
    let f = fixture();
    let invalid = KansoError::invalid("Icon theme import arrives with the icon themes.");
    assert_eq!(f.themes.icon_themes().unwrap_err(), invalid);
    assert_eq!(
        f.themes.import_icon_theme("/nowhere".into()).unwrap_err(),
        invalid
    );
}

// MARK: Parity with the desktop

/// A library whose themes folder holds what an import wrote, shared by the
/// property test.
static PARITY: LazyLock<Fixture> = LazyLock::new(|| {
    let f = fixture();
    let zed = f.file("family.json", ZED_FAMILY);
    f.themes.import(zed.to_string_lossy().into_owned()).unwrap();
    let tgz = package_tgz(f.dir.path(), "tiny-light", "#0a58ca");
    f.themes.import(tgz.to_string_lossy().into_owned()).unwrap();
    let hand = f.file("hand.json", KANSO_FILE);
    f.themes
        .import(hand.to_string_lossy().into_owned())
        .unwrap();
    f
});

/// Every id the parity test chooses from: built-in themes (a spread of
/// them) and the imported ones.
const IMPORTED: [&str; 4] = ["zed-night", "zed-day", "tiny-light", "hand-made"];

fn choices() -> Vec<String> {
    let mut ids: Vec<String> = builtin_ids()
        .into_iter()
        .step_by(3)
        .map(str::to_owned)
        .collect();
    ids.extend(["kanso-dark", "kanso-light"].map(str::to_owned));
    ids.extend(IMPORTED.map(str::to_owned));
    ids
}

/// The theme file as the desktop loads it: a built-in one from the compiled-in
/// text, an imported one from the file in the themes folder.
fn desktop_file(id: &str) -> ThemeFile {
    let text = match BUILTIN.iter().find(|(file, _)| id_of(file) == id) {
        Some((_, text)) => (*text).to_owned(),
        None => std::fs::read_to_string(PARITY.own().join(format!("{id}.json"))).unwrap(),
    };
    ThemeFile::parse(&text).unwrap()
}

/// What the desktop yields for `ui`, `syntax` and `preview`: the engine's own
/// fit of the three files, the call its `themes::apply` makes.
fn desktop_tokens(ui: &str, syntax: &str, preview: &str) -> Tokens {
    Tokens::fit(
        &desktop_file(ui).resolve(),
        &desktop_file(syntax).resolve(),
        &desktop_file(preview).resolve(),
    )
}

/// Every color and style of `got` is `want`'s.
fn assert_same(want: &Tokens, got: &ThemeTokens) {
    assert_eq!(got.name, want.name);
    assert_eq!(got.dark, want.dark);
    assert_eq!(got.caret, want.caret.0);
    assert_eq!(got.selection, want.selection.rgb.0);
    assert_eq!(got.selection_alpha, want.selection.alpha);
    // The interface colors, by the names the shell reads.
    let ui = |name: &str| got.ui.iter().find(|c| c.name == name).unwrap().rgb;
    let c = &want.ui;
    for (name, rgb) in [
        ("app", c.app),
        ("sidebar", c.sidebar),
        ("list", c.list),
        ("editor", c.editor),
        ("elevated", c.elevated),
        ("fill", c.fill),
        ("text", c.text),
        ("text_secondary", c.text_secondary),
        ("text_muted", c.text_muted),
        ("divider", c.divider),
        ("accent", c.accent),
        ("link", c.link),
        ("row_active", c.row_active),
        ("row_hover", c.row_hover),
        ("sidebar_active", c.sidebar_active),
        ("sidebar_hover", c.sidebar_hover),
        ("hover", c.hover),
        ("selected", c.selected),
        ("active_line", c.active_line),
        ("button", c.button),
        ("button_text", c.button_text),
        ("code", c.code),
        ("code_text", c.code_text),
        ("syntax_keyword", c.syntax_keyword),
        ("syntax_string", c.syntax_string),
        ("success", c.success),
        ("warning", c.warning),
        ("danger", c.danger),
    ] {
        assert_eq!(ui(name), rgb.0, "ui {name}");
    }
    assert_eq!(got.ui.len(), 28);
    // The syntax styles, every key.
    assert_eq!(got.syntax.len(), want.syntax.len());
    for row in &got.syntax {
        let style = &want.syntax[row.token.as_str()];
        assert_eq!(row.color, style.color.map(|c| c.0), "syntax {}", row.token);
        assert_eq!(row.italic, style.italic, "syntax {}", row.token);
        assert_eq!(row.weight, style.weight, "syntax {}", row.token);
    }
    // The preview's palette.
    let p = &want.preview;
    let palette = &got.palette;
    assert_eq!(palette.dark, want.dark);
    assert_eq!(palette.background, c.editor.0);
    assert_eq!(palette.text, p.text.0);
    assert_eq!(palette.muted, c.text_muted.0);
    assert_eq!(palette.accent, c.link.0);
    assert_eq!(palette.border, p.border.0);
    assert_eq!(palette.surface, p.code_background.0);
    assert_eq!(palette.heading, p.heading.map(|x| x.0));
    assert_eq!(palette.quote, p.quote.0);
    assert_eq!(palette.task_done, p.task_done.0);
    assert_eq!(palette.code_text, p.code_text.map(|x| x.0));
    assert_eq!(palette.inline_code_background, p.inline_code_background.0);
    assert_eq!(palette.inline_code_text, p.inline_code_text.map(|x| x.0));
    assert_eq!(
        palette.table_header_background,
        p.table_header_background.map(|x| x.0)
    );
}

// no-prop-test: the imported fixtures are in the parity test's choices, and
// this example names the ones a person is likeliest to use.
#[test]
fn an_imported_theme_gives_the_desktops_colors_in_an_example() {
    let themes = &PARITY.themes;
    for (ui, syntax, preview) in [
        ("zed-night", "zed-night", "zed-night"),
        ("zed-day", "kanso-dark", "tiny-light"),
        ("kanso-light", "zed-night", "hand-made"),
        ("tiny-light", "nord", "kanso-dark"),
    ] {
        let got = themes
            .fit(ui.into(), Some(syntax.into()), Some(preview.into()))
            .unwrap();
        assert_same(&desktop_tokens(ui, syntax, preview), &got);
    }
    // None is the ui theme.
    let alone = themes.fit("zed-night".into(), None, None).unwrap();
    assert_same(
        &desktop_tokens("zed-night", "zed-night", "zed-night"),
        &alone,
    );
}

proptest! {
    #![proptest_config(config(64))]

    /// Whatever three themes are chosen, built-in or imported, the phone's
    /// colors, syntax styles and preview palette are the desktop's for the
    /// same files; `None` is the interface theme.
    #[test]
    fn prop_fits_match_the_desktops(
        ui in prop::sample::select(choices()),
        syntax in prop::sample::select(choices()),
        preview in prop::sample::select(choices()),
        syntax_given in any::<bool>(),
        preview_given in any::<bool>(),
    ) {
        let got = PARITY
            .themes
            .fit(
                ui.clone(),
                syntax_given.then(|| syntax.clone()),
                preview_given.then(|| preview.clone()),
            )
            .unwrap();
        let syntax = if syntax_given { &syntax } else { &ui };
        let preview = if preview_given { &preview } else { &ui };
        assert_same(&desktop_tokens(&ui, syntax, preview), &got);
    }

    /// The runs of a note's text in a fitted pair are the engine's styles
    /// of that pair applied to the highlighter's spans, whatever the text.
    #[test]
    fn prop_styled_runs_use_the_fitted_pairs_syntax(
        ui in prop::sample::select(choices()),
        syntax in prop::sample::select(choices()),
        text in "(#{1,3} [a-z ]{0,10}\n|\\*[a-z]{1,5}\\*|`[a-z]{1,5}`|[a-z \n]{1,12}){0,8}",
    ) {
        let end = text.encode_utf16().count() as u32;
        let runs = PARITY
            .themes
            .styled_runs(text.clone(), 0, end, ui.clone(), Some(syntax.clone()))
            .unwrap();
        let want = desktop_tokens(&ui, &syntax, &ui).syntax;
        let spans = kanso_engine::highlight::spans(&text, 0..text.len());
        let expected: Vec<_> = spans
            .iter()
            .filter_map(|span| {
                let style = kanso_engine::theme::style(&span.tokens, &want);
                (!style.is_empty()).then(|| style.color.map(|c| c.0))
            })
            .collect();
        let got: Vec<_> = runs.iter().map(|run| run.color).collect();
        prop_assert_eq!(got, expected);
        // Runs are inside the text, in order.
        for run in &runs {
            prop_assert!(run.range.start <= run.range.end && run.range.end <= end);
        }
    }
}

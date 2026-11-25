//! How the editor is set up, as Settings › Editor and the view modes keep it
//! (`config.json`'s `appearance` and `editor` objects): the typeface and
//! which view mode sets it in monospace, when the caret's line stays
//! centred, whether focus mode shows notes in Live, and the writing aids
//! that are on. The desktop app reads and writes them in its config file;
//! the mobile app keeps the same values and hands them to the engine, so
//! the two agree on every default and every name in the file.

use serde::{Deserialize, Serialize};

/// How a note is shown: the source, the source drawn in place (Live),
/// both side by side, or the preview.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ViewMode {
    /// The source alone.
    Edit,
    /// The source alone, its Markdown drawn in place: marks hidden off the
    /// caret's lines and headings larger (see [`crate::live`]).
    Live,
    /// The preview alone.
    Preview,
    /// Source and preview side by side; also what a mode this version
    /// doesn't know reads as, so a newer app's file still loads. (Serde
    /// wants the catch-all last.)
    #[default]
    #[serde(other)]
    Split,
}

impl ViewMode {
    /// Whether the mode shows the preview, alone or beside the source.
    pub fn shows_preview(self) -> bool {
        matches!(self, ViewMode::Split | ViewMode::Preview)
    }

    /// Whether the mode shows the source on its own (Edit and Live).
    pub fn source_alone(self) -> bool {
        matches!(self, ViewMode::Edit | ViewMode::Live)
    }
}

/// The view mode focus mode set aside to show Live, with
/// [`EditorSettings::live_preview_in_focus`] on, to go back to on leaving.
/// The switch is the editor's alone: the view mode kept for next time
/// doesn't change.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FocusLive {
    before: Option<ViewMode>,
}

impl FocusLive {
    /// Focus mode starts while `mode` shows: the mode to show now, Live
    /// when `live` (the setting) is on.
    pub fn enter(&mut self, mode: ViewMode, live: bool) -> ViewMode {
        if live && mode != ViewMode::Live {
            self.before = Some(mode);
            ViewMode::Live
        } else {
            self.before = None;
            mode
        }
    }

    /// Focus mode ends while `mode` shows: the mode it was in before, if
    /// focus mode switched to Live and Live still shows; otherwise `mode`.
    pub fn leave(&mut self, mode: ViewMode) -> ViewMode {
        match self.before.take() {
            Some(before) if mode == ViewMode::Live => before,
            _ => mode,
        }
    }

    /// The person picked a view mode: it stands, and leaving focus mode
    /// keeps it.
    pub fn chosen(&mut self) {
        self.before = None;
    }
}

/// The editor's typeface.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EditorFont {
    /// Proportional for writing (Edit and Live), monospace beside the
    /// preview.
    #[default]
    Auto,
    /// Proportional everywhere.
    Sans,
    /// Monospace everywhere.
    Mono,
}

/// How the editor sets its text in one view mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorFace {
    /// The text is set monospace.
    pub mono: bool,
    /// The source's lines are numbered.
    pub line_numbers: bool,
}

impl EditorFont {
    /// Line numbers come with the monospace face, where source lines and
    /// their numbers line up.
    pub fn face(self, mode: ViewMode) -> EditorFace {
        let mono = match self {
            EditorFont::Auto => mode == ViewMode::Split,
            EditorFont::Sans => false,
            EditorFont::Mono => true,
        };
        EditorFace {
            mono,
            line_numbers: mono,
        }
    }
}

impl EditorFace {
    /// The face with line numbers only where the setting allows them.
    pub fn with_line_numbers(self, allowed: bool) -> EditorFace {
        EditorFace {
            line_numbers: self.line_numbers && allowed,
            ..self
        }
    }
}

/// The source side's share of the split before it is first dragged.
pub const DEFAULT_SPLIT_RATIO: f32 = 0.5;

/// When the caret's line stays in the middle of the editor while typing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Typewriter {
    /// Never.
    Off,
    /// Only in focus mode.
    #[default]
    FocusMode,
    /// In focus mode or out of it.
    Always,
}

impl Typewriter {
    /// Whether the caret's line stays centred, in focus mode or out of it.
    pub fn centres(self, focus_mode: bool) -> bool {
        match self {
            Typewriter::Off => false,
            Typewriter::FocusMode => focus_mode,
            Typewriter::Always => true,
        }
    }
}

/// Whether ticking a task also ticks the tasks nested under it, and the
/// parent once its last child is done (`kanso_core::tick::tick_task_linked`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TickNested {
    /// Ask the first time it would matter, each session.
    #[default]
    Ask,
    /// Always tick them together.
    Always,
    /// Tick the one task only.
    Never,
}

impl TickNested {
    /// The word the setting is kept under outside `config.json`.
    pub fn key(self) -> &'static str {
        match self {
            TickNested::Ask => "ask",
            TickNested::Always => "always",
            TickNested::Never => "never",
        }
    }

    /// The setting a [`TickNested::key`] names.
    pub fn from_key(key: &str) -> Option<TickNested> {
        [TickNested::Ask, TickNested::Always, TickNested::Never]
            .into_iter()
            .find(|setting| setting.key() == key)
    }

    /// Whether a tick links nested tasks: `Always` does, and so does `Ask`
    /// once `agreed` (the person said yes to this very tick).
    pub fn links(self, agreed: bool) -> bool {
        match self {
            TickNested::Always => true,
            TickNested::Ask => agreed,
            TickNested::Never => false,
        }
    }
}

/// How the editor helps with writing (Settings › Editor).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct EditorSettings {
    /// Brackets, quotes and Markdown markers get their closer as they're
    /// typed, and wrap a selection.
    pub auto_pair: bool,
    /// Tab and Enter in a table line its pipes up.
    pub format_tables: bool,
    /// A link pasted over words links them, and formatted text pastes as
    /// Markdown.
    pub smart_paste: bool,
    pub typewriter: Typewriter,
    /// The word goal of notes that don't set their own.
    pub word_goal: Option<u32>,
    /// `#tags` in a note's text become its tags when it is saved
    /// (`kanso_core::tags::inline_tags`). Off unless turned on.
    pub inline_tags: bool,
    /// Vim keys in the note's text: Normal, Insert and Visual modes.
    pub vim: bool,
    /// Misspelled words in the note's text are underlined (`spellcheck`).
    /// On unless turned off, also for files from before the setting.
    pub spellcheck: bool,
    /// Focus mode shows the note in Live while it is on, then goes back to
    /// the mode it was in, without changing the view mode kept for next
    /// time. Off unless turned on.
    pub live_preview_in_focus: bool,
    /// Ticking a task ticks the tasks nested under it too (and the parent
    /// when its last child is done). Asks first, by default.
    pub tick_nested: TickNested,
}

impl Default for EditorSettings {
    fn default() -> Self {
        EditorSettings {
            auto_pair: true,
            format_tables: true,
            smart_paste: true,
            typewriter: Typewriter::FocusMode,
            word_goal: None,
            inline_tags: false,
            vim: false,
            spellcheck: true,
            live_preview_in_focus: false,
            tick_nested: TickNested::Ask,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use serde_json::{Value, json};

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 256,
            failure_persistence: None,
            ..ProptestConfig::default()
        }
    }

    fn gen_settings() -> impl Strategy<Value = EditorSettings> {
        (
            any::<[bool; 6]>(),
            prop::sample::select(vec![
                Typewriter::Off,
                Typewriter::FocusMode,
                Typewriter::Always,
            ]),
            prop::option::of(any::<u32>()),
            prop::sample::select(vec![TickNested::Ask, TickNested::Always, TickNested::Never]),
        )
            .prop_map(
                |(
                    [
                        auto_pair,
                        format_tables,
                        smart_paste,
                        inline_tags,
                        vim,
                        live_preview_in_focus,
                    ],
                    typewriter,
                    word_goal,
                    tick_nested,
                )| {
                    EditorSettings {
                        auto_pair,
                        format_tables,
                        smart_paste,
                        typewriter,
                        word_goal,
                        inline_tags,
                        vim,
                        spellcheck: !vim,
                        live_preview_in_focus,
                        tick_nested,
                    }
                },
            )
    }

    /// The `editor` object as `config.json` has always written it, built by
    /// hand so that it doesn't depend on the derive being tested.
    fn written(settings: &EditorSettings) -> Value {
        json!({
            "autoPair": settings.auto_pair,
            "formatTables": settings.format_tables,
            "smartPaste": settings.smart_paste,
            "typewriter": match settings.typewriter {
                Typewriter::Off => "off",
                Typewriter::FocusMode => "focusMode",
                Typewriter::Always => "always",
            },
            "wordGoal": settings.word_goal,
            "inlineTags": settings.inline_tags,
            "vim": settings.vim,
            "spellcheck": settings.spellcheck,
            "livePreviewInFocus": settings.live_preview_in_focus,
            "tickNested": match settings.tick_nested {
                TickNested::Ask => "ask",
                TickNested::Always => "always",
                TickNested::Never => "never",
            },
        })
    }

    proptest! {
        #![proptest_config(config())]

        /// Round trip: the config file's format is unchanged. The `editor`
        /// object as the app has always written it parses to the same
        /// settings, and the settings write it back equal.
        #[test]
        fn prop_the_config_format_is_unchanged(settings in gen_settings()) {
            let old = written(&settings);
            let parsed: EditorSettings = serde_json::from_value(old.clone()).unwrap();
            prop_assert_eq!(parsed, settings);
            prop_assert_eq!(serde_json::to_value(parsed).unwrap(), old);
        }

        /// Defaults: a key missing from an older file is the default for it,
        /// and the others are kept.
        #[test]
        fn prop_tick_nested_keys_round_trip(setting in prop::sample::select(vec![TickNested::Ask, TickNested::Always, TickNested::Never]), junk in "[a-z]{0,8}") {
            prop_assert_eq!(TickNested::from_key(setting.key()), Some(setting));
            // The key is the word the config file writes.
            let written = serde_json::to_value(setting).unwrap();
            prop_assert_eq!(written.as_str(), Some(setting.key()));
            if !["ask", "always", "never"].contains(&junk.as_str()) {
                prop_assert_eq!(TickNested::from_key(&junk), None);
            }
        }

        #[test]
        fn prop_missing_keys_take_their_defaults(settings in gen_settings(), drop in 0usize..10) {
            let mut old = written(&settings);
            let object = old.as_object_mut().unwrap();
            let key = object.keys().nth(drop).unwrap().clone();
            object.remove(&key);
            let parsed: EditorSettings = serde_json::from_value(old).unwrap();
            let mut expected = written(&settings);
            expected[&key] = written(&EditorSettings::default())[&key].clone();
            prop_assert_eq!(serde_json::to_value(parsed).unwrap(), expected);
        }

        /// The typeface and view mode keep their names in the file.
        #[test]
        fn prop_fonts_and_modes_keep_their_names(
            font in prop::sample::select(vec![EditorFont::Auto, EditorFont::Sans, EditorFont::Mono]),
            mode in prop::sample::select(vec![ViewMode::Edit, ViewMode::Live, ViewMode::Split, ViewMode::Preview]),
        ) {
            let font_name = match font {
                EditorFont::Auto => "auto",
                EditorFont::Sans => "sans",
                EditorFont::Mono => "mono",
            };
            let mode_name = match mode {
                ViewMode::Edit => "edit",
                ViewMode::Live => "live",
                ViewMode::Split => "split",
                ViewMode::Preview => "preview",
            };
            prop_assert_eq!(serde_json::to_value(font).unwrap(), json!(font_name));
            prop_assert_eq!(serde_json::from_value::<EditorFont>(json!(font_name)).unwrap(), font);
            prop_assert_eq!(serde_json::to_value(mode).unwrap(), json!(mode_name));
            prop_assert_eq!(serde_json::from_value::<ViewMode>(json!(mode_name)).unwrap(), mode);
        }

        /// Line numbers only ever come with the monospace face, and turning
        /// the setting off removes them without changing the face.
        #[test]
        fn prop_line_numbers_come_with_the_monospace_face(
            font in prop::sample::select(vec![EditorFont::Auto, EditorFont::Sans, EditorFont::Mono]),
            mode in prop::sample::select(vec![ViewMode::Edit, ViewMode::Live, ViewMode::Split, ViewMode::Preview]),
            allowed in any::<bool>(),
        ) {
            let face = font.face(mode);
            let shown = face.with_line_numbers(allowed);
            prop_assert_eq!(shown.mono, face.mono);
            prop_assert_eq!(shown.line_numbers, face.mono && allowed);
            prop_assert_eq!(face.with_line_numbers(true), face);
        }
    }

    fn gen_mode() -> impl Strategy<Value = ViewMode> {
        prop::sample::select(vec![
            ViewMode::Edit,
            ViewMode::Live,
            ViewMode::Split,
            ViewMode::Preview,
        ])
    }

    proptest! {
        #![proptest_config(config())]

        /// Focus mode with the setting off leaves the mode alone, going in
        /// and coming out.
        #[test]
        fn prop_focus_mode_keeps_the_mode_with_the_setting_off(mode in gen_mode()) {
            let mut focus = FocusLive::default();
            prop_assert_eq!(focus.enter(mode, false), mode);
            prop_assert_eq!(focus.leave(mode), mode);
        }

        /// With the setting on, focus mode shows Live and leaving it shows
        /// the mode from before, once.
        #[test]
        fn prop_focus_mode_shows_live_and_goes_back(mode in gen_mode()) {
            let mut focus = FocusLive::default();
            prop_assert_eq!(focus.enter(mode, true), ViewMode::Live);
            prop_assert_eq!(focus.leave(ViewMode::Live), mode);
            // Nothing is left to go back to.
            prop_assert_eq!(focus.leave(ViewMode::Live), ViewMode::Live);
        }

        /// A mode picked while in focus mode stands when it ends, whether
        /// it is Live or another.
        #[test]
        fn prop_a_mode_picked_in_focus_mode_stands(
            mode in gen_mode(),
            picked in gen_mode(),
            live in any::<bool>(),
        ) {
            let mut focus = FocusLive::default();
            focus.enter(mode, live);
            focus.chosen();
            prop_assert_eq!(focus.leave(picked), picked);
        }

        /// A mode left some other way than picking it (Live gone from
        /// under focus mode) isn't overridden on leaving.
        #[test]
        fn prop_focus_mode_only_undoes_its_own_live(mode in gen_mode(), now in gen_mode()) {
            prop_assume!(now != ViewMode::Live);
            let mut focus = FocusLive::default();
            focus.enter(mode, true);
            prop_assert_eq!(focus.leave(now), now);
        }
    }

    #[test]
    fn the_defaults_are_the_apps() {
        let settings = EditorSettings::default();
        assert_eq!(
            written(&settings),
            json!({
                "autoPair": true, "formatTables": true, "smartPaste": true,
                "typewriter": "focusMode", "wordGoal": null, "inlineTags": false,
                "vim": false, "spellcheck": true, "livePreviewInFocus": false,
                "tickNested": "ask",
            })
        );
        assert_eq!(
            serde_json::from_value::<EditorSettings>(json!({})).unwrap(),
            settings
        );
        assert_eq!(ViewMode::default(), ViewMode::Split);
        assert_eq!(EditorFont::default(), EditorFont::Auto);
        assert_eq!(DEFAULT_SPLIT_RATIO, 0.5);
    }

    #[test]
    fn modes_show_the_source_or_the_preview() {
        use ViewMode::{Edit, Live, Preview, Split};
        for mode in [Edit, Live, Split, Preview] {
            assert_eq!(mode.source_alone(), matches!(mode, Edit | Live));
            assert_eq!(mode.shows_preview(), matches!(mode, Split | Preview));
            // A mode either has the preview or shows the source alone.
            assert!(mode.source_alone() != mode.shows_preview());
        }
    }

    /// Live sets its text like Edit: proportional unless the user picks
    /// monospace.
    #[test]
    fn live_sets_its_text_like_edit() {
        for font in [EditorFont::Auto, EditorFont::Sans, EditorFont::Mono] {
            assert_eq!(font.face(ViewMode::Live), font.face(ViewMode::Edit));
        }
        assert!(!EditorFont::Auto.face(ViewMode::Live).mono);
    }

    /// A view mode this version doesn't know opens beside the preview
    /// rather than failing the whole file.
    #[test]
    fn an_unknown_view_mode_reads_as_split() {
        assert_eq!(
            serde_json::from_value::<ViewMode>(json!("outline")).unwrap(),
            ViewMode::Split
        );
        assert_eq!(
            serde_json::from_value::<ViewMode>(json!("live")).unwrap(),
            ViewMode::Live
        );
    }

    #[test]
    fn the_typewriter_centres_by_mode() {
        assert!(Typewriter::FocusMode.centres(true) && !Typewriter::FocusMode.centres(false));
        assert!(Typewriter::Always.centres(false) && !Typewriter::Off.centres(true));
    }
}

//! How the editor is set up, as Settings › Editor and the view modes keep it
//! (`config.json`'s `appearance` and `editor` objects): the typeface and
//! which view mode sets it in monospace, when the caret's line stays
//! centred, and the writing aids that are on. The desktop app reads and
//! writes them in its config file; the mobile app keeps the same values and
//! hands them to the engine, so the two agree on every default and every
//! name in the file.

use serde::{Deserialize, Serialize};

/// How a note is shown: the source, both side by side, or the preview.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ViewMode {
    /// The source alone.
    Edit,
    /// Source and preview side by side.
    #[default]
    Split,
    /// The preview alone.
    Preview,
}

/// The editor's typeface.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EditorFont {
    /// Proportional for writing, monospace beside the preview.
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
            any::<[bool; 5]>(),
            prop::sample::select(vec![
                Typewriter::Off,
                Typewriter::FocusMode,
                Typewriter::Always,
            ]),
            prop::option::of(any::<u32>()),
        )
            .prop_map(
                |(
                    [auto_pair, format_tables, smart_paste, inline_tags, vim],
                    typewriter,
                    word_goal,
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
        fn prop_missing_keys_take_their_defaults(settings in gen_settings(), drop in 0usize..8) {
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
            mode in prop::sample::select(vec![ViewMode::Edit, ViewMode::Split, ViewMode::Preview]),
        ) {
            let font_name = match font {
                EditorFont::Auto => "auto",
                EditorFont::Sans => "sans",
                EditorFont::Mono => "mono",
            };
            let mode_name = match mode {
                ViewMode::Edit => "edit",
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
            mode in prop::sample::select(vec![ViewMode::Edit, ViewMode::Split, ViewMode::Preview]),
            allowed in any::<bool>(),
        ) {
            let face = font.face(mode);
            let shown = face.with_line_numbers(allowed);
            prop_assert_eq!(shown.mono, face.mono);
            prop_assert_eq!(shown.line_numbers, face.mono && allowed);
            prop_assert_eq!(face.with_line_numbers(true), face);
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
                "vim": false, "spellcheck": true,
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
    fn the_typewriter_centres_by_mode() {
        assert!(Typewriter::FocusMode.centres(true) && !Typewriter::FocusMode.centres(false));
        assert!(Typewriter::Always.centres(false) && !Typewriter::Off.centres(true));
    }
}

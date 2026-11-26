//! Settings › Editor › Google Fonts: a field for a family name and a button
//! that downloads it, with the step it is at or what went wrong in words
//! under them, as the phones' Google Fonts section does.
//!
//! This owns the field, the download's task and the words. What the download
//! does is `kanso_engine::google_fonts::load`; handing the file to the text
//! system is `crate::google_fonts`; choosing the new font and adding it to
//! the pickers is the settings view's, which this tells with
//! [`GoogleFontAdded`]. One download runs at a time, and the font is only
//! reported once the text system lists it under its name.
//!
//! Each downloaded family also has a button that removes it: this deletes
//! the kept file and tells the settings view with [`GoogleFontRemoved`],
//! which forgets the family in the settings and reloads the pickers.
//! The text system can't unload a font it has taken (GPUI has no way to),
//! so a removed family may stay installed, and show in the pickers, until
//! Kanso restarts; it is no longer listed as downloaded, no setting keeps
//! it, and the next launch neither registers nor downloads it.

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::Disableable as _;
use gpui_kit::component::Sizable as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::*;
use kanso_engine::google_fonts::{Family, HttpSource, Origin, Stage, load};

use crate::settings::{Row, section};
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

/// The field's width: a family name and a few more words.
const FIELD_WIDTH: f32 = 200.;

/// The phones' words (`googleFontFooter`, `googleFontHint`).
const FOOTER: &str = "Enter a family from Google Fonts. Once downloaded it is used for writing and reading, shows up in every font list above, and works offline.";
const HINT: &str = "A family, like Lora";

/// Where the download is, for the words under the field.
#[derive(Clone, Debug, PartialEq, Eq)]
enum State {
    Idle,
    Working {
        family: Family,
        stage: Stage,
    },
    Failed(String),
    Added(Family),
    /// The named family was removed (its name as the settings kept it).
    Removed(String),
}

impl State {
    /// The line under the field: the footer until something happens.
    fn words(&self) -> String {
        match self {
            State::Idle => FOOTER.to_owned(),
            State::Working { family, stage } => stage.label(family),
            State::Failed(why) => why.clone(),
            State::Added(family) => {
                format!("{family} is downloaded. It is now the writing and preview font.")
            }
            State::Removed(name) => {
                format!("{name} is removed. It may stay in the font lists until Kanso restarts.")
            }
        }
    }

    fn working(&self) -> bool {
        matches!(self, State::Working { .. })
    }

    /// The same download at a later step; anything else stays as it is.
    fn at(self, stage: Stage) -> State {
        match self {
            State::Working { family, .. } => State::Working { family, stage },
            other => other,
        }
    }
}

/// A font was downloaded and registered: the settings choose it for
/// writing and the preview and list it in every picker.
pub struct GoogleFontAdded(pub Family);

/// A downloaded font was deleted: the settings forget it, put back any
/// font setting that chose it and list the families again. Holds the name
/// as the settings kept it.
pub struct GoogleFontRemoved(pub String);

/// The Google Fonts section's field and download.
pub struct GoogleFonts {
    input: Entity<InputState>,
    state: State,
    task: Option<Task<()>>,
    _typing: Subscription,
}

impl EventEmitter<GoogleFontAdded> for GoogleFonts {}
impl EventEmitter<GoogleFontRemoved> for GoogleFonts {}

impl GoogleFonts {
    // no-prop-test: builds controls; `State` and the engine's `load` have
    // the properties.
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| InputState::new(window, cx).placeholder(HINT));
        let typing = cx.subscribe_in(
            &input,
            window,
            |this: &mut GoogleFonts, _, event: &InputEvent, window, cx| {
                if let InputEvent::PressEnter { shift: false, .. } = event {
                    this.download(window, cx);
                }
            },
        );
        GoogleFonts {
            input,
            state: State::Idle,
            task: None,
            _typing: typing,
        }
    }

    /// Whether the Google Fonts text field currently has keyboard focus.
    pub(crate) fn is_focused(&self, window: &Window, cx: &App) -> bool {
        window.focused(cx) == Some(self.input.focus_handle(cx))
    }

    /// Downloads the family in the field, unless one is downloading.
    // no-prop-test: starts a task on the network; the steps are `State`'s
    // and the download is `kanso_engine::google_fonts::load`'s.
    pub fn download(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.state.working() {
            return;
        }
        let typed = self.input.read(cx).value();
        let family = match Family::parse(&typed) {
            Ok(family) => family,
            Err(err) => {
                self.state = State::Failed(err.to_string());
                cx.notify();
                return;
            }
        };
        self.state = State::Working {
            family: family.clone(),
            stage: Stage::Stylesheet,
        };
        cx.notify();
        let (steps, stages) = async_channel::unbounded::<Stage>();
        let cache = crate::google_fonts::cache(cx);
        let wanted = family.clone();
        let loading = cx.background_executor().spawn(async move {
            load(&HttpSource::default(), &cache, &wanted, &mut |stage| {
                // The receiver only goes away with this view.
                steps.try_send(stage).ok();
            })
        });
        self.task = Some(cx.spawn_in(window, async move |this, cx| {
            // `steps` is dropped when the download ends, which ends this.
            let mut shown = String::new();
            while let Ok(stage) = stages.recv().await {
                let words = stage.label(&family);
                if words == shown {
                    continue;
                }
                shown = words;
                if this
                    .update(cx, |this, cx| {
                        this.state = this.state.clone().at(stage);
                        cx.notify();
                    })
                    .is_err()
                {
                    return;
                }
            }
            let loaded = loading.await;
            this.update_in(cx, |this, window, cx| {
                this.task = None;
                this.finish(loaded, window, cx);
            })
            .ok();
        }));
    }

    /// Registers what was loaded, and tells the settings or says why not.
    // no-prop-test: hands bytes to the platform's text system.
    fn finish(
        &mut self,
        loaded: Result<kanso_engine::google_fonts::Loaded, kanso_engine::google_fonts::LoadError>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let loaded = match loaded {
            Ok(loaded) => loaded,
            Err(err) => {
                self.state = State::Failed(err.to_string());
                cx.notify();
                return;
            }
        };
        if let Err(err) = crate::google_fonts::register(&loaded.family, &loaded.file, cx) {
            // A file the system won't use is not kept to fail every launch.
            if loaded.origin == Origin::Download {
                crate::google_fonts::cache(cx).remove(&loaded.family).ok();
            }
            self.state = State::Failed(err.to_string());
            cx.notify();
            return;
        }
        self.input
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.state = State::Added(loaded.family.clone());
        cx.emit(GoogleFontAdded(loaded.family));
        cx.notify();
    }

    /// Deletes the kept font of the downloaded family `name` (as the settings
    /// name it) and tells the settings view. A file that can't be deleted
    /// is said in words and the family stays downloaded, so the settings
    /// never forget a font the next launch would find again. A name that
    /// isn't a family has no file, and is only forgotten.
    // no-prop-test: deletes a file and emits; the removal's rules are
    // `Cache::remove`'s and `Appearance::remove_google_font`'s.
    pub fn remove(&mut self, name: &str, cx: &mut Context<Self>) {
        if self.state.working() {
            return;
        }
        if let Ok(family) = Family::parse(name)
            && let Err(err) = crate::google_fonts::cache(cx).remove(&family)
        {
            self.state = State::Failed(format!("Kanso couldn't remove {name}: {err}"));
            cx.notify();
            return;
        }
        self.state = State::Removed(name.to_owned());
        cx.emit(GoogleFontRemoved(name.to_owned()));
        cx.notify();
    }

    /// The section: the field and its button, with the words under them,
    /// and the fonts already downloaded, each with a button to remove it.
    // no-prop-test: layout; the words are `State::words`.
    pub fn section(this: &Entity<GoogleFonts>, cx: &App, colors: &Colors) -> AnyElement {
        let me = this.read(cx);
        let working = me.state.working();
        let button = {
            let this = this.clone();
            Button::new("google-font-download")
                .debug_selector(|| "google-font-download".into())
                .small()
                .primary()
                .label(if working {
                    "Downloading…"
                } else {
                    "Download"
                })
                .disabled(working)
                .on_click(move |_, window, cx| {
                    this.update(cx, |this, cx| this.download(window, cx));
                })
        };
        let mut rows = vec![
            Row::new(Lucide::Globe, "Add a Google Font")
                .detail(me.state.words())
                .trailing(
                    div()
                        .w(px(FIELD_WIDTH))
                        .debug_selector(|| "google-font-field".into())
                        .child(Input::new(&me.input).disabled(working)),
                )
                .trailing(button),
        ];
        let downloaded = &cx.global::<AppState>().config.appearance.google_fonts;
        for (ix, name) in downloaded.iter().enumerate() {
            let (this, family) = (this.clone(), name.clone());
            rows.push(
                Row::new(Lucide::CaseSensitive, name.clone())
                    .detail("Downloaded")
                    .trailing(
                        Button::new(ElementId::NamedInteger(
                            "google-font-remove".into(),
                            ix as u64,
                        ))
                        .debug_selector(move || format!("google-font-remove-{ix}"))
                        .outline()
                        .small()
                        .icon(Lucide::Trash)
                        .tooltip(format!("Remove {name}"))
                        .disabled(working)
                        .on_click(move |_, _, cx| {
                            this.update(cx, |this, cx| this.remove(&family, cx));
                        }),
                    ),
            );
        }
        section("Google Fonts", rows, colors)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    // Not `super::*`: it would bring in gpui's `test` over the built-in one.
    use super::{FOOTER, Family, Stage, State};

    fn family(name: &str) -> Family {
        Family::parse(name).expect("a family")
    }

    #[test]
    fn the_words_follow_the_download() {
        let lora = family("Lora");
        let running = State::Working {
            family: lora.clone(),
            stage: Stage::Stylesheet,
        };
        let failed = State::Failed("Google Fonts has no family with that name.".to_owned());
        let added = State::Added(lora);
        let removed = State::Removed("Lora".to_owned());
        assert_eq!(State::Idle.words(), FOOTER);
        assert_eq!(running.words(), "Looking up Lora…");
        assert_eq!(failed.words(), "Google Fonts has no family with that name.");
        assert!(added.words().starts_with("Lora is downloaded"));
        assert!(removed.words().starts_with("Lora is removed"));
    }

    /// A later step moves a running download and nothing else.
    fn step_moves_only_a_running_download(
        family: Family,
        stage: Stage,
    ) -> Result<(), TestCaseError> {
        let running = State::Working {
            family: family.clone(),
            stage: Stage::Stylesheet,
        };
        let moved = State::Working {
            family: family.clone(),
            stage,
        };
        prop_assert_eq!(running.at(stage), moved);
        for ended in [
            State::Idle,
            State::Failed("no".to_owned()),
            State::Added(family),
            State::Removed("Lora".to_owned()),
        ] {
            prop_assert_eq!(ended.clone().at(stage), ended);
        }
        Ok(())
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::with_cases(128)
        })]

        /// A later step changes a running download's step and nothing else;
        /// a download that has ended ignores it.
        #[test]
        fn prop_a_step_only_moves_a_running_download(
            name in "[A-Za-z][A-Za-z ]{0,9}",
            received in any::<u64>(),
            total in prop::option::of(any::<u64>()),
        ) {
            if let Ok(family) = Family::parse(&name) {
                step_moves_only_a_running_download(family, Stage::Font { received, total })?;
            }
        }
    }
}

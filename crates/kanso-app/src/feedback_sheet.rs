//! The Send feedback sheet (issue #232): Help › Send Feedback… and the
//! palette's "Send feedback…".
//!
//! A kind, the words, an optional screenshot, the app info (on, and shown
//! exactly as sent under "Show what's sent") and an optional credit: a plain
//! name, or a GitHub username picked from a search (the issue then mentions
//! them, so GitHub notifies them).
//! **No note is ever read.** The payload comes from
//! `kanso_engine::feedback::build_payload`, and the screenshot is a file the
//! person chose or a capture of Kanso's window they can look at and remove
//! before sending.
//!
//! Send saves the report to the outbox first, then tries the network off the
//! main thread. Offline, the report waits and goes when it can.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::component::button::{Button, ButtonGroup, ButtonVariants as _};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::input::{Input, InputEvent, InputState, Textarea, TextareaState};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::{
    Disableable as _, Selectable as _, Sizable as _, StyledExt as _, WindowExt as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_engine::feedback::{
    self, AppInfo, Credit, Draft, DraftError, GithubUser, Kind, MAX_SCREENSHOT, MAX_TEXT, Screen,
    ScreenshotDecl, search_query,
};
use kanso_engine::feedback_flow::{self, Picture, Sent};
use kanso_sync::feedback::{FeedbackError, fetch_avatar};

use crate::feedback_app::{self, Env};
use crate::theme::Colors;
use crate::zoom::px;

actions!(kanso, [SendFeedback, MyFeedback]);

/// The sheet's width.
pub const SHEET_WIDTH: f32 = 520.;

/// An image the person chose or captured, without its metadata, and drawn
/// in the sheet so they see what goes before sending.
#[derive(Clone)]
struct Chosen {
    bytes: Arc<Vec<u8>>,
    decl: ScreenshotDecl,
    label: String,
    image: Arc<Image>,
}

/// The thumbnail's tallest size.
const THUMBNAIL_HEIGHT: f32 = 180.;

/// What the character counter says, once it shows. Pure.
pub fn counter(chars: usize) -> Option<String> {
    feedback::shows_counter(chars).then(|| format!("{chars} / {MAX_TEXT}"))
}

/// Which credit the person is choosing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreditKind {
    /// A plain name.
    Name,
    /// A GitHub username picked from the search.
    Github,
}

/// What the GitHub picker shows under its search box.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Lookup {
    /// Nothing typed yet, or too little to search.
    Idle,
    /// Waiting for the pause after typing, or for GitHub.
    Searching,
    /// The users found (possibly none).
    Found(Vec<GithubUser>),
    /// The search couldn't be made.
    Unavailable,
}

/// How long typing must pause before the search is made.
pub const SEARCH_DEBOUNCE: Duration = Duration::from_millis(300);

/// The credit the form asks for, or `None` when it is off. GitHub with
/// nobody picked is a credit that can't be sent yet (an empty login), so
/// Send stays off and the person is told to pick. Pure.
pub fn credit_of(on: bool, kind: CreditKind, name: &str, picked: Option<&str>) -> Option<Credit> {
    on.then(|| match kind {
        CreditKind::Name => Credit::Name(name.to_owned()),
        CreditKind::Github => Credit::Github(picked.unwrap_or_default().to_owned()),
    })
}

/// Whether Send is on: the text is long enough and not too long, and a
/// credit, if asked for, is valid for its kind. Pure.
pub fn can_send(chars: usize, credit: Option<&Credit>) -> bool {
    (feedback::MIN_TEXT..=MAX_TEXT).contains(&chars)
        && credit.is_none_or(|credit| feedback::credit_json(credit).is_ok())
}

/// A picked file as a screenshot, or the line that says why not. Pure.
pub fn chosen_from_file(
    bytes: Vec<u8>,
    name: &str,
) -> Result<(ScreenshotDecl, String), &'static str> {
    if bytes.len() as u64 > MAX_SCREENSHOT {
        return Err(feedback_app::SCREENSHOT_TOO_BIG);
    }
    match feedback::declare_screenshot(&bytes) {
        Ok(decl) => Ok((decl, name.to_owned())),
        Err(_) => Err(feedback_app::NOT_AN_IMAGE),
    }
}

/// Kanso's window as a PNG, or a JPEG when the PNG is too large. macOS:
/// `screencapture -l <window id>`, the id from the window list. Blocking.
/// Elsewhere Kanso has no window capture and the sheet only offers
/// "Choose image…".
#[cfg(target_os = "macos")]
// no-prop-test: runs system tools; the sheet shows the result for the person to check.
fn capture_window() -> Option<Vec<u8>> {
    let pid = std::process::id();
    let script = format!(
        "ObjC.import('CoreGraphics');\
         const list = ObjC.deepUnwrap(ObjC.castRefToObject($.CGWindowListCopyWindowInfo(17, 0)));\
         let best = 0, area = 0;\
         for (const w of list) {{\
           if (w.kCGWindowOwnerPID === {pid} && w.kCGWindowLayer === 0) {{\
             const b = w.kCGWindowBounds; const a = b.Width * b.Height;\
             if (a > area) {{ area = a; best = w.kCGWindowNumber; }}\
           }}\
         }}\
         best"
    );
    let out = std::process::Command::new("osascript")
        .args(["-l", "JavaScript", "-e", &script])
        .output()
        .ok()?;
    let id: u32 = String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse()
        .ok()
        .filter(|id| *id != 0)?;
    let dir = std::env::temp_dir().join(format!("kanso-capture-{pid}"));
    std::fs::create_dir_all(&dir).ok()?;
    let done = |bytes: Option<Vec<u8>>| {
        let _ = std::fs::remove_dir_all(&dir);
        bytes
    };
    for (kind, ext) in [("png", "png"), ("jpg", "jpg")] {
        let file = dir.join(format!("kanso.{ext}"));
        let ok = std::process::Command::new("screencapture")
            .args(["-l", &id.to_string(), "-o", "-x", "-t", kind])
            .arg(&file)
            .status()
            .ok()
            .is_some_and(|s| s.success());
        let Some(bytes) = ok.then(|| std::fs::read(&file).ok()).flatten() else {
            return done(None);
        };
        if bytes.len() as u64 <= MAX_SCREENSHOT {
            return done(Some(bytes));
        }
    }
    done(None)
}

#[cfg(not(target_os = "macos"))]
fn capture_window() -> Option<Vec<u8>> {
    None
}

/// Whether this build can capture Kanso's window.
pub const CAN_CAPTURE: bool = cfg!(target_os = "macos");

pub struct FeedbackSheet {
    env: Option<Env>,
    pub(crate) kind: Kind,
    /// The text area's placeholder now.
    pub(crate) hint: &'static str,
    pub(crate) text: Entity<TextareaState>,
    pub(crate) name: Entity<InputState>,
    /// The GitHub search box.
    pub(crate) github: Entity<InputState>,
    sent_view: Entity<TextareaState>,
    shown: String,
    include_app: bool,
    pub(crate) credit: bool,
    pub(crate) credit_kind: CreditKind,
    /// The GitHub user chosen from the list.
    pub(crate) picked: Option<GithubUser>,
    pub(crate) lookup: Lookup,
    /// The pending or running search; dropping it cancels the wait.
    search: Option<Task<()>>,
    /// Avatars by login; present once asked for, `None` while loading or when it failed.
    avatars: HashMap<String, Option<Arc<Image>>>,
    show_sent: bool,
    picture: Option<Chosen>,
    choosing: bool,
    app: Option<AppInfo>,
    pub(crate) busy: bool,
    pub(crate) error: Option<String>,
    _subscriptions: Vec<Subscription>,
}

impl FeedbackSheet {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> FeedbackSheet {
        let env = Env::capture(cx);
        let text = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder(feedback_app::placeholder(Kind::Bug))
        });
        let signed_in_name = env.as_ref().and_then(Env::username).unwrap_or_default();
        let name = cx.new(|cx| {
            let mut state = InputState::new(window, cx).placeholder(feedback_app::CREDIT_NAME_HINT);
            if !signed_in_name.is_empty() {
                state.set_value(signed_in_name.clone(), window, cx);
            }
            state
        });
        let github =
            cx.new(|cx| InputState::new(window, cx).placeholder(feedback_app::GITHUB_SEARCH_HINT));
        let sent_view = cx.new(|cx| TextareaState::new(window, cx));
        let subscriptions = vec![
            cx.subscribe_in(&github, window, |this, _, event: &InputEvent, _, cx| {
                if let InputEvent::Change = event {
                    this.query_changed(cx);
                }
            }),
            cx.subscribe_in(&text, window, |_, _, event: &InputEvent, _, cx| {
                if let InputEvent::Change = event {
                    cx.notify();
                }
            }),
            cx.subscribe_in(&name, window, |_, _, event: &InputEvent, _, cx| {
                if let InputEvent::Change = event {
                    cx.notify();
                }
            }),
        ];
        let mut sheet = FeedbackSheet {
            env,
            kind: Kind::Bug,
            hint: feedback_app::placeholder(Kind::Bug),
            text,
            name,
            github,
            sent_view,
            shown: String::new(),
            include_app: true,
            credit: false,
            credit_kind: CreditKind::Name,
            picked: None,
            lookup: Lookup::Idle,
            search: None,
            avatars: HashMap::new(),
            show_sent: false,
            picture: None,
            choosing: false,
            app: None,
            busy: false,
            error: None,
            _subscriptions: subscriptions,
        };
        sheet.collect_app_info(cx);
        sheet
    }

    /// Reads the app info off the main thread (it runs a few small system commands).
    fn collect_app_info(&mut self, cx: &mut Context<Self>) {
        let Some(env) = self.env.clone() else {
            return;
        };
        let task = cx.background_executor().spawn(async move {
            feedback_app::app_info(&env.data_dir, env.sync_mode(), Screen::Other)
        });
        cx.spawn(async move |this, cx| {
            let info = task.await;
            this.update(cx, |this, cx| {
                this.app = Some(info);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn chars(&self, cx: &App) -> usize {
        self.text.read(cx).value().chars().count()
    }

    /// The credit the form asks for now, if any.
    pub(crate) fn credit(&self, cx: &App) -> Option<Credit> {
        credit_of(
            self.credit,
            self.credit_kind,
            &self.name.read(cx).value(),
            self.picked.as_ref().map(|user| user.login.as_str()),
        )
    }

    /// The search box changed: wait out the typing, then ask the Worker. A
    /// newer keystroke drops this task, so only the last pause searches.
    pub(crate) fn query_changed(&mut self, cx: &mut Context<Self>) {
        self.search = None;
        let typed = self.github.read(cx).value().to_string();
        let (Some(query), Some(env)) = (search_query(&typed), self.env.clone()) else {
            self.lookup = Lookup::Idle;
            cx.notify();
            return;
        };
        self.lookup = Lookup::Searching;
        cx.notify();
        self.search = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(SEARCH_DEBOUNCE).await;
            let found = cx
                .background_executor()
                .spawn(async move {
                    let reporter = env.reporter().ok_or(FeedbackError::Offline)?;
                    env.client().search_users(&reporter, &query)
                })
                .await;
            this.update(cx, |this, cx| {
                this.lookup = match found {
                    Ok(users) => {
                        this.load_avatars(&users, cx);
                        Lookup::Found(users)
                    }
                    Err(_) => Lookup::Unavailable,
                };
                cx.notify();
            })
            .ok();
        }));
    }

    /// Fetches the avatars not asked for yet (the real network only: a
    /// test's stand-in server has no avatar host).
    fn load_avatars(&mut self, users: &[GithubUser], cx: &mut Context<Self>) {
        if !self.env.as_ref().is_some_and(Env::real_network) {
            return;
        }
        for user in users {
            if self.avatars.contains_key(&user.login) {
                continue;
            }
            self.avatars.insert(user.login.clone(), None);
            let (login, url) = (user.login.clone(), user.avatar_url.clone());
            cx.spawn(async move |this, cx| {
                let image = cx
                    .background_executor()
                    .spawn(async move {
                        let bytes = fetch_avatar(&url).ok()?;
                        let format = match feedback::sniff_image(&bytes)? {
                            "image/png" => ImageFormat::Png,
                            _ => ImageFormat::Jpeg,
                        };
                        Some(Arc::new(Image::from_bytes(format, bytes)))
                    })
                    .await;
                this.update(cx, |this, cx| {
                    this.avatars.insert(login, image);
                    cx.notify();
                })
                .ok();
            })
            .detach();
        }
    }

    pub(crate) fn set_credit_kind(&mut self, kind: CreditKind, cx: &mut Context<Self>) {
        self.credit_kind = kind;
        cx.notify();
    }

    /// Chooses `user` from the list: the search box gives way to their chip.
    pub(crate) fn pick(&mut self, user: GithubUser, window: &mut Window, cx: &mut Context<Self>) {
        self.picked = Some(user);
        self.github
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.search = None;
        self.lookup = Lookup::Idle;
        cx.notify();
    }

    /// Takes the chosen user back, to search again.
    pub(crate) fn unpick(&mut self, cx: &mut Context<Self>) {
        self.picked = None;
        cx.notify();
    }

    /// A user's avatar: their picture when it has loaded, else a circle with their initial.
    fn avatar(&self, login: &str, colors: &Colors) -> AnyElement {
        let size = px(20.);
        match self.avatars.get(login).and_then(Option::as_ref) {
            Some(image) => img(ImageSource::Image(image.clone()))
                .size(size)
                .rounded_full()
                .object_fit(ObjectFit::Cover)
                .into_any_element(),
            None => div()
                .size(size)
                .rounded_full()
                .bg(colors.divider)
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(11.))
                .text_color(colors.text_secondary)
                .child(
                    login
                        .chars()
                        .next()
                        .unwrap_or('?')
                        .to_uppercase()
                        .to_string(),
                )
                .into_any_element(),
        }
    }

    fn draft(&self, cx: &App) -> Draft {
        Draft {
            kind: self.kind,
            text: self.text.read(cx).value().to_string(),
            credit: self.credit(cx),
            screenshot: self.picture.as_ref().map(|p| p.decl.clone()),
        }
    }

    fn app_for_report(&self) -> Option<&AppInfo> {
        self.include_app.then_some(self.app.as_ref()).flatten()
    }

    /// The exact JSON "Show what's sent" displays, or why there isn't one yet.
    fn payload_text(&self, cx: &App) -> String {
        match feedback_flow::preview_payload(&self.draft(cx), self.app_for_report()) {
            Ok(json) => json,
            Err(err) => feedback_app::draft_error_text(&err),
        }
    }

    pub(crate) fn set_kind(&mut self, kind: Kind, window: &mut Window, cx: &mut Context<Self>) {
        self.kind = kind;
        let hint = feedback_app::placeholder(kind);
        self.hint = hint;
        self.text
            .update(cx, |text, cx| text.set_placeholder(hint, window, cx));
        cx.notify();
    }

    pub(crate) fn set_picture(&mut self, bytes: Vec<u8>, label: String, cx: &mut Context<Self>) {
        // A photo's EXIF (GPS, camera) and text chunks never leave the Mac.
        let bytes = feedback::strip_metadata(&bytes);
        match chosen_from_file(bytes.clone(), &label) {
            Ok((decl, label)) => {
                let format = if decl.mime == "image/png" {
                    ImageFormat::Png
                } else {
                    ImageFormat::Jpeg
                };
                self.picture = Some(Chosen {
                    image: Arc::new(Image::from_bytes(format, bytes.clone())),
                    bytes: Arc::new(bytes),
                    decl,
                    label,
                });
                self.error = None;
            }
            Err(message) => self.error = Some(message.to_owned()),
        }
        cx.notify();
    }

    fn capture(&mut self, cx: &mut Context<Self>) {
        self.choosing = false;
        let task = cx
            .background_executor()
            .spawn(async move { capture_window() });
        cx.spawn(async move |this, cx| {
            let bytes = task.await;
            this.update(cx, |this, cx| match bytes {
                Some(bytes) => this.set_picture(bytes, "Kanso window".into(), cx),
                None => {
                    this.error =
                        Some("Kanso couldn't capture its window. Choose an image instead.".into());
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    fn choose_image(&mut self, cx: &mut Context<Self>) {
        self.choosing = false;
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Choose".into()),
        });
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = picked.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "Image".into());
            // Check the size before reading it all.
            let too_big = std::fs::metadata(&path)
                .map(|m| m.len() > MAX_SCREENSHOT)
                .unwrap_or(false);
            let bytes = if too_big {
                None
            } else {
                std::fs::read(&path).ok()
            };
            this.update(cx, |this, cx| match bytes {
                Some(bytes) => this.set_picture(bytes, name, cx),
                None => {
                    this.error = Some(feedback_app::SCREENSHOT_TOO_BIG.into());
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    /// Saves the report and sends it.
    pub(crate) fn send(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let Some(env) = self.env.clone() else {
            self.error = Some("Kanso can't send feedback right now.".into());
            cx.notify();
            return;
        };
        let draft = self.draft(cx);
        if let Err(err) = feedback::clean_text(&draft.text) {
            self.error = Some(feedback_app::draft_error_text(&err));
            cx.notify();
            return;
        }
        let app = self.app_for_report().cloned();
        let picture = self.picture.clone();
        self.busy = true;
        self.error = None;
        cx.notify();
        let mut entropy = [0u8; 16];
        // A fixed id would collide with everyone else's: never send without one.
        if getrandom::fill(&mut entropy).is_err() {
            self.busy = false;
            self.error = Some("Kanso couldn't send this report. Try again.".into());
            cx.notify();
            return;
        }
        let (tx, rx) = async_channel::bounded::<Result<Sent, DraftError>>(1);
        std::thread::Builder::new()
            .name("kanso-feedback".into())
            .spawn(move || {
                let outcome = (|| {
                    let reporter = env
                        .reporter()
                        .ok_or(feedback_flow::Error::Draft(DraftError::BadId))?;
                    feedback_flow::send(
                        &env.store,
                        &env.client(),
                        &reporter,
                        &env.data_dir,
                        entropy,
                        draft,
                        picture.as_ref().map(|p| Picture { bytes: &p.bytes }),
                        app.as_ref(),
                        kanso_core::now_ms(),
                    )
                })();
                let _ = tx.send_blocking(match outcome {
                    Ok((_, sent)) => Ok(sent),
                    Err(feedback_flow::Error::Draft(err)) => Err(err),
                    // The outbox could not be written, or a file could not be kept.
                    Err(_) => Ok(Sent::Rejected {
                        field: String::new(),
                    }),
                });
            })
            .ok();
        cx.spawn_in(window, async move |this, cx| {
            let result = rx.recv().await;
            this.update_in(cx, |this, window, cx| {
                this.busy = false;
                match result {
                    Ok(Ok(Sent::Done)) => {
                        window.close_dialog(cx);
                        notify(window, feedback_app::SENT, cx);
                    }
                    Ok(Ok(Sent::Queued(wait))) => {
                        window.close_dialog(cx);
                        notify(window, feedback_app::wait_text(&wait), cx);
                        feedback_app::flush_soon(cx, Duration::from_secs(60));
                    }
                    Ok(Ok(Sent::Rejected { field })) => {
                        this.error = Some(feedback_app::rejected_text(&field));
                    }
                    Ok(Err(err)) => this.error = Some(feedback_app::draft_error_text(&err)),
                    Err(_) => {
                        this.error = Some("Kanso couldn't send this report. Try again.".into())
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}

/// "Thanks — feedback sent" and the like, with a shortcut to My feedback.
fn notify(window: &mut Window, message: &'static str, cx: &mut App) {
    window.push_notification(
        Notification::new().message(message).action(|_, _, _| {
            Button::new("feedback-mine")
                .small()
                .label("My feedback")
                .on_click(|_, window, cx| window.dispatch_action(Box::new(MyFeedback), cx))
        }),
        cx,
    );
}

impl Render for FeedbackSheet {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let chars = self.chars(cx);
        // Keep "Show what's sent" in step with the form, setting the view only when it changed.
        if self.show_sent {
            let now = self.payload_text(cx);
            if now != self.shown {
                self.shown = now.clone();
                self.sent_view
                    .update(cx, |view, cx| view.set_value(now, window, cx));
            }
        }
        let label = |text: &'static str| {
            div()
                .text_size(px(12.5))
                .font_medium()
                .text_color(colors.text_secondary)
                .child(text)
        };
        let quiet = |text: String| {
            div()
                .text_size(px(12.5))
                .line_height(px(18.))
                .text_color(colors.text_secondary)
                .child(text)
        };
        let kinds = ButtonGroup::new("feedback-kind")
            .outline()
            .small()
            .disabled(self.busy)
            .children(Kind::ALL.iter().enumerate().map(|(ix, kind)| {
                let selected = *kind == self.kind;
                Button::new(("feedback-kind", ix))
                    .selected(selected)
                    .debug_selector(move || format!("feedback-kind-{ix}"))
                    .child(
                        div()
                            .px_1()
                            .text_size(px(12.5))
                            .when(selected, |el| el.font_medium())
                            .child(feedback_app::kind_label(*kind)),
                    )
            }))
            .on_click(cx.listener(|this, clicked: &Vec<usize>, window, cx| {
                if let Some(&ix) = clicked.first() {
                    this.set_kind(Kind::ALL[ix], window, cx);
                }
            }));
        let screenshot = match &self.picture {
            Some(picture) => v_flex()
                .gap_2()
                .child(
                    div()
                        .debug_selector(|| "feedback-thumbnail".into())
                        .rounded(px(6.))
                        .border_1()
                        .border_color(colors.divider)
                        .overflow_hidden()
                        .child(
                            img(ImageSource::Image(picture.image.clone()))
                                .w_full()
                                .max_h(px(THUMBNAIL_HEIGHT))
                                .object_fit(ObjectFit::Contain),
                        ),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .text_size(px(13.))
                                .text_color(colors.text)
                                .child(format!(
                                    "{} · {}",
                                    picture.label,
                                    size_label(picture.decl.bytes)
                                )),
                        )
                        .child(
                            Button::new("feedback-remove-shot")
                                .small()
                                .label("Remove")
                                .disabled(self.busy)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.picture = None;
                                    cx.notify();
                                })),
                        ),
                )
                .into_any_element(),
            None if self.choosing => h_flex()
                .gap_2()
                .when(CAN_CAPTURE, |el| {
                    el.child(
                        Button::new("feedback-capture")
                            .small()
                            .label("Kanso window")
                            .on_click(cx.listener(|this, _, _, cx| this.capture(cx))),
                    )
                })
                .child(
                    Button::new("feedback-choose")
                        .small()
                        .label("Choose image…")
                        .on_click(cx.listener(|this, _, _, cx| this.choose_image(cx))),
                )
                .into_any_element(),
            None => h_flex()
                .child(
                    Button::new("feedback-add-shot")
                        .small()
                        .label("Add screenshot")
                        .disabled(self.busy)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.choosing = true;
                            cx.notify();
                        })),
                )
                .into_any_element(),
        };
        let counter_text = counter(chars);
        v_flex()
            .gap(px(12.))
            .debug_selector(|| "feedback-sheet".into())
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                if event.keystroke.key == "enter" && event.keystroke.modifiers.secondary() {
                    this.send(window, cx);
                }
            }))
            .child(kinds)
            .child(
                v_flex()
                    .gap(px(4.))
                    .child(Textarea::new(&self.text).h(px(130.)).disabled(self.busy))
                    .when_some(counter_text, |el, counter| {
                        el.child(
                            div()
                                .text_size(px(12.))
                                .text_color(if chars > MAX_TEXT {
                                    colors.warning
                                } else {
                                    colors.text_secondary
                                })
                                .child(counter),
                        )
                    }),
            )
            .child(
                v_flex()
                    .gap(px(6.))
                    .child(label("Screenshot"))
                    .child(screenshot)
                    .child(quiet(feedback_app::SCREENSHOT_WARNING.into())),
            )
            .child(
                v_flex()
                    .gap(px(6.))
                    .child(
                        h_flex()
                            .gap_3()
                            .items_center()
                            .child(
                                Checkbox::new("feedback-app-info")
                                    .checked(self.include_app)
                                    .label("Include app info (version, system, recent errors)")
                                    .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                        this.include_app = *checked;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("feedback-show-sent")
                                    .small()
                                    .ghost()
                                    .label(if self.show_sent {
                                        "Hide what's sent"
                                    } else {
                                        "Show what's sent"
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.show_sent = !this.show_sent;
                                        this.shown.clear();
                                        cx.notify();
                                    })),
                            ),
                    )
                    .when(self.show_sent, |el| {
                        el.child(
                            div()
                                .font_family("Menlo")
                                .text_size(px(11.5))
                                .child(Textarea::new(&self.sent_view).h(px(150.)).readonly(true)),
                        )
                    }),
            )
            .child(
                v_flex()
                    .gap(px(6.))
                    .child(
                        Checkbox::new("feedback-credit")
                            .checked(self.credit)
                            .label(feedback_app::CREDIT_LABEL)
                            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                this.credit = *checked;
                                cx.notify();
                            })),
                    )
                    .when(self.credit, |el| el.child(self.credit_choice(colors, cx))),
            )
            .child(quiet(feedback_app::FOOTER.into()))
            .when_some(self.error.clone(), |el, error| {
                el.child(
                    div()
                        .text_size(px(13.))
                        .line_height(px(19.))
                        .text_color(colors.warning)
                        .debug_selector(|| "feedback-error".into())
                        .child(error),
                )
            })
    }
}

impl FeedbackSheet {
    /// The two ways to be credited, and the field for the one chosen.
    fn credit_choice(&self, colors: &Colors, cx: &mut Context<Self>) -> impl IntoElement {
        let quiet = |text: &'static str| {
            div()
                .text_size(px(12.5))
                .line_height(px(18.))
                .text_color(colors.text_secondary)
                .child(text)
        };
        let kinds = [
            (CreditKind::Name, feedback_app::CREDIT_NAME_CHOICE),
            (CreditKind::Github, feedback_app::CREDIT_GITHUB_CHOICE),
        ];
        let choice = ButtonGroup::new("feedback-credit-kind")
            .outline()
            .small()
            .disabled(self.busy)
            .children(kinds.iter().enumerate().map(|(ix, (kind, label))| {
                let selected = *kind == self.credit_kind;
                Button::new(("feedback-credit-kind", ix))
                    .selected(selected)
                    .debug_selector(move || format!("feedback-credit-kind-{ix}"))
                    .child(
                        div()
                            .px_1()
                            .text_size(px(12.5))
                            .when(selected, |el| el.font_medium())
                            .child(*label),
                    )
            }))
            .on_click(cx.listener(|this, clicked: &Vec<usize>, _, cx| {
                if let Some(&ix) = clicked.first() {
                    this.set_credit_kind([CreditKind::Name, CreditKind::Github][ix.min(1)], cx);
                }
            }));
        let body = match self.credit_kind {
            CreditKind::Name => v_flex()
                .gap(px(6.))
                .child(Input::new(&self.name))
                .child(quiet(feedback_app::CREDIT_NAME_DETAIL))
                .into_any_element(),
            CreditKind::Github => self.github_picker(colors, cx).into_any_element(),
        };
        v_flex().gap(px(8.)).child(choice).child(body)
    }

    /// The chosen user's chip, or the search box with the users it found.
    fn github_picker(&self, colors: &Colors, cx: &mut Context<Self>) -> impl IntoElement {
        let note = |text: &'static str| {
            div()
                .text_size(px(12.5))
                .line_height(px(18.))
                .text_color(colors.text_secondary)
                .child(text)
        };
        if let Some(user) = &self.picked {
            return v_flex()
                .gap(px(6.))
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .debug_selector(|| "feedback-github-picked".into())
                        .child(self.avatar(&user.login, colors))
                        .child(
                            div()
                                .text_size(px(13.))
                                .text_color(colors.text)
                                .child(format!("@{}", user.login)),
                        )
                        .child(
                            Button::new("feedback-github-change")
                                .small()
                                .label("Change")
                                .disabled(self.busy)
                                .on_click(cx.listener(|this, _, _, cx| this.unpick(cx))),
                        ),
                )
                .child(note(feedback_app::CREDIT_GITHUB_DETAIL));
        }
        let results = match &self.lookup {
            Lookup::Idle => None,
            Lookup::Searching => Some(note(feedback_app::GITHUB_SEARCHING).into_any_element()),
            Lookup::Unavailable => Some(
                note(feedback_app::GITHUB_UNAVAILABLE)
                    .debug_selector(|| "feedback-github-unavailable".into())
                    .into_any_element(),
            ),
            Lookup::Found(users) if users.is_empty() => {
                Some(note(feedback_app::GITHUB_NONE_FOUND).into_any_element())
            }
            Lookup::Found(users) => Some(
                v_flex()
                    .debug_selector(|| "feedback-github-results".into())
                    .rounded(px(6.))
                    .border_1()
                    .border_color(colors.divider)
                    .children(users.iter().enumerate().map(|(ix, user)| {
                        let chosen = user.clone();
                        Button::new(("feedback-github-user", ix))
                            .ghost()
                            .w_full()
                            .debug_selector(move || format!("feedback-github-user-{ix}"))
                            .child(
                                h_flex()
                                    .w_full()
                                    .gap_2()
                                    .items_center()
                                    .child(self.avatar(&user.login, colors))
                                    .child(
                                        div()
                                            .text_size(px(13.))
                                            .text_color(colors.text)
                                            .child(user.login.clone()),
                                    ),
                            )
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.pick(chosen.clone(), window, cx)
                            }))
                    }))
                    .into_any_element(),
            ),
        };
        v_flex()
            .gap(px(6.))
            .child(Input::new(&self.github))
            .children(results)
            .child(note(feedback_app::CREDIT_GITHUB_DETAIL))
    }
}

/// A byte count in words: "482 KB".
pub fn size_label(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{} KB", bytes.div_ceil(1024).max(1))
    }
}

/// The sheet's buttons.
pub struct FeedbackFooter {
    view: Entity<FeedbackSheet>,
    _observing: Subscription,
}

impl Render for FeedbackFooter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = self.view.read(cx);
        let busy = view.busy;
        let ready = can_send(view.chars(cx), view.credit(cx).as_ref());
        let target = self.view.clone();
        h_flex()
            .w_full()
            .gap_2()
            .items_center()
            .child(div().flex_1())
            .child(
                Button::new("feedback-cancel")
                    .label("Cancel")
                    .disabled(busy)
                    .on_click(|_, window, cx| window.close_dialog(cx)),
            )
            .child(
                Button::new("feedback-send")
                    .primary()
                    .label(if busy { "Sending…" } else { "Send" })
                    .disabled(busy || !ready)
                    .debug_selector(|| "feedback-send".into())
                    .on_click(move |_, window, cx| {
                        target.update(cx, |sheet, cx| sheet.send(window, cx));
                    }),
            )
    }
}

/// The workspace's handlers for Send Feedback… and My Feedback.
// no-prop-test: wiring; the UI tests send each action.
pub fn on_actions(el: Div, cx: &mut Context<crate::workspace::Workspace>) -> Div {
    el.on_action(cx.listener(|_, _: &SendFeedback, window, cx| {
        open(window, cx);
    }))
    .on_action(cx.listener(|_, _: &MyFeedback, window, cx| {
        crate::my_feedback::open(window, cx);
    }))
}

/// Opens the sheet. Returns it, for tests to drive.
// no-prop-test: opens a dialog; `can_send`, `counter` and the engine's rules are tested.
pub fn open(window: &mut Window, cx: &mut App) -> Entity<FeedbackSheet> {
    let sheet = cx.new(|cx| FeedbackSheet::new(window, cx));
    let footer = cx.new(|cx| FeedbackFooter {
        view: sheet.clone(),
        _observing: cx.observe(&sheet, |_, _, cx| cx.notify()),
    });
    let body = sheet.clone();
    let running = sheet.clone();
    window.open_alert_dialog(cx, move |alert, _, _| {
        let running = running.clone();
        alert
            .title("Send feedback")
            .width(px(SHEET_WIDTH))
            .child(body.clone())
            .footer(footer.clone())
            // Escape closes, but not while the report is being saved.
            .on_cancel(move |_, _, cx| !running.read(cx).busy)
    });
    sheet
}

/// The feedback kind of an agent report's kind.
pub fn kind_of(kind: kanso_server::ReportKind) -> Kind {
    match kind {
        kanso_server::ReportKind::Bug => Kind::Bug,
        kanso_server::ReportKind::Idea => Kind::Idea,
        kanso_server::ReportKind::Question => Kind::Question,
    }
}

/// Opens the sheet filled in with `kind` and `text` (an agent's report,
/// issue #447). The person can edit it, and nothing is sent until they
/// press Send, as with [`open`].
// no-prop-test: opens a dialog; the report's text is property-tested in kanso-server.
pub fn open_prefilled(
    kind: Kind,
    text: String,
    window: &mut Window,
    cx: &mut App,
) -> Entity<FeedbackSheet> {
    let sheet = open(window, cx);
    sheet.update(cx, |sheet, cx| {
        sheet.set_kind(kind, window, cx);
        sheet
            .text
            .update(cx, |area, cx| area.set_value(text, window, cx));
    });
    sheet
}

/// Shows "An agent reported a problem" for `report`; its button opens the
/// feedback sheet filled in with it and takes it out of the queue.
// no-prop-test: shows a notification; open_prefilled is tested.
pub fn notify_agent_report(
    report: kanso_server::AgentReport,
    reports: kanso_server::AgentReports,
    window: &mut Window,
    cx: &mut App,
) {
    let notification =
        Notification::info("An agent reported a problem. Review it before anything is sent.")
            .autohide(false)
            .action(move |_, _, _| {
                let (report, reports) = (report.clone(), reports.clone());
                Button::new("agent-report-review")
                    .label("Review")
                    .on_click(move |_, window, cx| {
                        // Another window's notice may have opened it already.
                        if reports.remove(&report) {
                            open_prefilled(kind_of(report.kind), report.text(), window, cx);
                        }
                    })
            });
    window.push_notification(notification, cx);
}

/// Watches the agents' reports for this window and shows each new one.
// no-prop-test: wiring between the queue and notify_agent_report.
pub fn watch_agent_reports(
    window: &mut Window,
    cx: &mut Context<crate::workspace::Workspace>,
) -> Option<Task<()>> {
    let reports = cx
        .try_global::<crate::services::Services>()
        .map(|services| services.agent_tokens.agent_reports().clone())?;
    let events = reports.subscribe();
    Some(cx.spawn_in(window, async move |this, cx| {
        while let Ok(report) = events.recv().await {
            let reports = reports.clone();
            let alive = this.update_in(cx, |_, window, cx| {
                notify_agent_report(report, reports, window, cx);
            });
            if alive.is_err() {
                break;
            }
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::{
        CreditKind, MAX_SCREENSHOT, can_send, chosen_from_file, counter, credit_of, size_label,
    };
    use crate::feedback_app;
    use kanso_engine::feedback::{self, Credit};
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Oracle: an agent report the server accepts is text the feedback
        /// sheet's own rules (`feedback::clean_text`) leave exactly as it is,
        /// so the prefilled sheet can be sent unchanged.
        #[test]
        fn prop_accepted_agent_reports_pass_clean_text_unchanged(
            summary in prop_oneof![".{0,80}", "[\\PC\n\r]{0,60}", "[e\u{301}\u{e9}a ]{8,40}"],
            tool in proptest::option::of("[a-z-]{0,20}"),
            workaround in proptest::option::of("[\\PC\n]{0,60}"),
            kind in prop::sample::select(vec![
                kanso_server::ReportKind::Bug,
                kanso_server::ReportKind::Idea,
                kanso_server::ReportKind::Question,
            ]),
        ) {
            if let Ok(report) = kanso_server::AgentReport::new(
                kind, &summary, tool.as_deref(), workaround.as_deref()
            ) {
                let text = report.text();
                prop_assert_eq!(feedback::clean_text(&text), Ok(text.clone()));
                prop_assert!(can_send(text.chars().count(), None));
                prop_assert_eq!(super::kind_of(kind).wire(), kind.as_str());
            }
        }
    }

    #[test]
    fn the_counter_shows_after_3500() {
        assert_eq!(counter(3499), None);
        assert_eq!(counter(3500).as_deref(), Some("3500 / 4000"));
        assert_eq!(counter(4100).as_deref(), Some("4100 / 4000"));
    }

    #[test]
    fn send_needs_ten_characters_and_a_valid_name() {
        assert!(!can_send(9, None));
        assert!(can_send(10, None));
        assert!(can_send(4000, None));
        assert!(!can_send(4001, None));
        let name = |n: &str| Credit::Name(n.into());
        let github = |l: &str| Credit::Github(l.into());
        assert!(can_send(20, Some(&name("tendai"))));
        assert!(!can_send(20, Some(&name(""))));
        assert!(!can_send(20, Some(&name("@x"))));
        assert!(can_send(20, Some(&github("octocat"))));
        // GitHub with nobody picked yet, and a login that is not one.
        assert!(!can_send(20, Some(&github(""))));
        assert!(!can_send(20, Some(&github("@octocat"))));
        assert!(!can_send(20, Some(&github("a--b"))));
    }

    #[test]
    fn the_form_makes_the_credit_its_kind_says() {
        assert_eq!(
            credit_of(false, CreditKind::Github, "n", Some("octocat")),
            None
        );
        assert_eq!(
            credit_of(true, CreditKind::Name, "tendai", Some("octocat")),
            Some(Credit::Name("tendai".into()))
        );
        assert_eq!(
            credit_of(true, CreditKind::Github, "tendai", Some("octocat")),
            Some(Credit::Github("octocat".into()))
        );
        // Nobody picked: not sendable, whatever the name box says.
        let unpicked = credit_of(true, CreditKind::Github, "tendai", None);
        assert!(!can_send(20, unpicked.as_ref()));
    }

    #[test]
    fn a_chosen_file_must_be_a_small_png_or_jpeg() {
        let png = vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0];
        assert!(chosen_from_file(png, "a.png").is_ok());
        assert_eq!(
            chosen_from_file(b"GIF89a".to_vec(), "a.gif").unwrap_err(),
            feedback_app::NOT_AN_IMAGE
        );
        let mut big = vec![0xff, 0xd8, 0xff];
        big.resize(MAX_SCREENSHOT as usize + 1, 0);
        assert_eq!(
            chosen_from_file(big, "a.jpg").unwrap_err(),
            feedback_app::SCREENSHOT_TOO_BIG
        );
    }

    #[test]
    fn sizes_read_in_words() {
        assert_eq!(size_label(482_113), "471 KB");
        assert_eq!(size_label(1), "1 KB");
        assert_eq!(size_label(3 * 1024 * 1024), "3.0 MB");
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Oracle: with a credit, Send is on exactly when the engine builds a
        /// payload for it, so the button never promises what the payload refuses.
        #[test]
        fn prop_send_agrees_with_the_engine_about_credit(
            name in "[a-zA-Z0-9 ._@-]{0,45}",
            picked in proptest::option::of("[a-zA-Z0-9_@-]{0,42}"),
            kind in prop::sample::select(vec![CreditKind::Name, CreditKind::Github]),
            on in any::<bool>(),
        ) {
            let credit = credit_of(on, kind, &name, picked.as_deref());
            let draft = feedback::Draft {
                kind: feedback::Kind::Bug,
                text: "The export button does nothing at all".into(),
                credit: credit.clone(),
                screenshot: None,
            };
            let built = feedback::build_payload("fb_01jbx9k2m4n6p8q0r2s4t6v8w0", &draft, None).is_ok();
            prop_assert_eq!(can_send(37, credit.as_ref()), built);
            // Off is always fine; a name box never leaks into a GitHub credit.
            prop_assert_eq!(credit.is_none(), !on);
            if let (true, CreditKind::Github, Some(Credit::Github(l))) = (on, kind, &credit) {
                prop_assert_eq!(l.as_str(), picked.as_deref().unwrap_or(""));
            }
        }
    }

    proptest! {
        /// Send is on exactly when the engine would accept the text.
        #[test]
        fn prop_send_agrees_with_the_engine(text in "[ -~]{0,4100}") {
            let chars = text.chars().count();
            prop_assert_eq!(can_send(chars, None), feedback::clean_text(&text).is_ok());
        }
    }
}

//! The sheet that asks before an agent uses one of its own tools.
//!
//! An agent on this computer may ask, in the middle of an answer, to run a
//! tool of its own (read a file, run a command). Kanso can't see or limit
//! what that tool does, so it asks the person: the agent's name, the tool's
//! title and kind, and four buttons. **Reject** is the default: Enter,
//! Escape and clicking away all reject. **Allow once** allows this one
//! request. **Allow for this session** allows the same tool for the rest of
//! that agent session, held in memory only: it ends with the session or the
//! app. **Always allow** remembers the tool for that agent across launches
//! (Settings > AI lists these, each with a Remove button). A tool is the
//! agent's name plus the tool call's kind and title. Kanso never answers the
//! agent's own "always" option: it remembers itself and answers "allow once"
//! each time, so what it remembers can always be taken back. A request that
//! is already allowed this way is answered without showing the sheet.
//!
//! Requests arrive from an agent's reader thread and wait their turn here:
//! one sheet at a time, in order. A request with no window to show it in is
//! rejected at once, and one that timed out or was cancelled while it waited
//! is skipped, or closed if its sheet is up.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{WindowExt as _, h_flex, v_flex};
use gpui_kit::*;
use kanso_acp::{PermissionPolicy, PermissionReply, PermissionRequest};
use kanso_engine::agent_task::runner::Tasks;
use kanso_engine::agent_task::session::PermissionAnswer;

use crate::config::AllowedTool;
use crate::dialogs::FORM_WIDTH;
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

/// The line every sheet carries.
pub const NO_SANDBOX: &str = "Kanso can't see or limit what the agent's own tools do.";

/// What a sheet says once its request ended behind another dialog.
pub const ENDED: &str = "This request has ended. Nothing was allowed.";

/// The longest tool title shown; the agent chooses the title, so it is cut.
const TITLE_MAX: usize = 240;

/// How often a sheet looks for its request having ended behind it.
const WATCH: Duration = Duration::from_millis(250);

/// What the person answered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    Reject,
    AllowOnce,
    /// Allow this tool until the agent session ends.
    AllowSession,
    /// Allow this tool from now on, for this agent.
    AlwaysAllow,
}

/// Everything a person can do to the sheet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    ClickReject,
    ClickAllowOnce,
    ClickAllowSession,
    ClickAlwaysAllow,
    Enter,
    Escape,
    /// Clicking outside, or the dialog closing some other way.
    Dismiss,
}

/// What an input answers. Only clicking one of the three Allow buttons
/// allows; Reject is the default for every other way out.
pub fn choice(input: Input) -> Choice {
    match input {
        Input::ClickAllowOnce => Choice::AllowOnce,
        Input::ClickAllowSession => Choice::AllowSession,
        Input::ClickAlwaysAllow => Choice::AlwaysAllow,
        Input::ClickReject | Input::Enter | Input::Escape | Input::Dismiss => Choice::Reject,
    }
}

impl Choice {
    /// How the transcript records this answer.
    pub fn answer(self) -> PermissionAnswer {
        match self {
            Choice::Reject => PermissionAnswer::Rejected,
            Choice::AllowOnce => PermissionAnswer::AllowedOnce,
            Choice::AllowSession => PermissionAnswer::AllowedForSession,
            Choice::AlwaysAllow => PermissionAnswer::AllowedAlways,
        }
    }
}

/// Whether the request needs no sheet.
#[cfg(test)]
pub fn is_allowed(
    request: &PermissionRequest,
    session: &SessionGrants,
    always: &[AllowedTool],
) -> bool {
    standing(request, session, always).is_some()
}

/// Records `answer` to `request` in the transcript of the task whose agent
/// session asked, if one did.
pub fn note(tasks: &Tasks, request: &PermissionRequest, answer: PermissionAnswer) {
    let Some(id) = request
        .session
        .as_deref()
        .and_then(|session| tasks.by_session(session))
    else {
        return;
    };
    tasks.note_permission(id, &tool_line(request), request.kind.as_deref(), answer);
}

/// The sheet's title.
pub fn title(request: &PermissionRequest) -> String {
    format!("{} wants to use a tool", request.agent)
}

/// The tool's title as shown: one line, cut when very long.
pub fn tool_line(request: &PermissionRequest) -> String {
    let flat: String = request
        .title
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if flat.chars().count() <= TITLE_MAX {
        return flat;
    }
    let cut: String = flat.chars().take(TITLE_MAX).collect();
    format!("{}…", cut.trim_end())
}

/// The tool's kind as a line, if the agent gave one.
pub fn kind_line(request: &PermissionRequest) -> Option<String> {
    let kind = request.kind.as_deref()?.trim();
    (!kind.is_empty()).then(|| format!("Kind: {kind}"))
}

/// The tool a request is for, as remembered: the agent, and the tool call's
/// kind and title with its whitespace flattened.
pub fn tool_of(request: &PermissionRequest) -> AllowedTool {
    AllowedTool {
        agent: request.agent.clone(),
        kind: request
            .kind
            .as_deref()
            .map(str::trim)
            .filter(|kind| !kind.is_empty())
            .map(str::to_owned),
        title: request
            .title
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" "),
    }
}

/// The tools allowed for the rest of an agent session, by session ID. Kept
/// in memory only.
#[derive(Default)]
pub struct SessionGrants(HashSet<(String, AllowedTool)>);

impl SessionGrants {
    /// Allows the request's tool for the rest of its session. A request with
    /// no session ID can't be remembered.
    pub fn allow(&mut self, request: &PermissionRequest) {
        if let Some(session) = &request.session {
            self.0.insert((session.clone(), tool_of(request)));
        }
    }

    fn covers(&self, request: &PermissionRequest) -> bool {
        request
            .session
            .as_ref()
            .is_some_and(|session| self.0.contains(&(session.clone(), tool_of(request))))
    }
}

/// How a request needs no sheet: its tool is allowed for this session or
/// always. `None` when the person has to be asked.
pub fn standing(
    request: &PermissionRequest,
    session: &SessionGrants,
    always: &[AllowedTool],
) -> Option<PermissionAnswer> {
    if session.covers(request) {
        Some(PermissionAnswer::AllowedForSession)
    } else if always.contains(&tool_of(request)) {
        Some(PermissionAnswer::AllowedAlways)
    } else {
        None
    }
}

/// A request waiting for the person.
struct Ask {
    request: PermissionRequest,
    reply: PermissionReply,
}

/// The policy that queues each request for the sheet. If the app is gone,
/// the request goes with it and is rejected.
fn policy(queue: async_channel::Sender<(PermissionRequest, PermissionReply)>) -> PermissionPolicy {
    Arc::new(move |request, reply| {
        queue.try_send((request, reply)).ok();
    })
}

/// Starts answering agents' permission requests with the sheet. Until then,
/// and with no window, every request is rejected.
pub fn install(cx: &mut App) {
    kanso_ai::agent::set_permission_decider(Some(start(cx)));
}

/// The policy that shows each request in turn in the main window.
pub(crate) fn start(cx: &mut App) -> PermissionPolicy {
    let (queue, waiting) = async_channel::unbounded::<(PermissionRequest, PermissionReply)>();
    let grants = Arc::new(Mutex::new(SessionGrants::default()));
    cx.spawn(async move |cx| {
        while let Ok((request, reply)) = waiting.recv().await {
            // A tool already allowed is answered without the sheet, when its
            // turn comes, so one allowed while others waited covers them.
            let (standing, tasks) = cx.update(|cx| {
                let always = &cx.global::<AppState>().config.ai.agent.always_allowed;
                let session = grants.lock().unwrap_or_else(|e| e.into_inner());
                (
                    standing(&request, &session, always),
                    crate::agent_task_panel::tasks(cx),
                )
            });
            if let Some(answer) = standing {
                reply.allow_once();
                if let Some(tasks) = &tasks {
                    note(tasks, &request, answer);
                }
                continue;
            }
            present(Ask { request, reply }, grants.clone(), cx).await;
        }
    })
    .detach();
    policy(queue)
}

/// One open sheet's answer, given once.
struct Sheet {
    request: PermissionRequest,
    grants: Arc<Mutex<SessionGrants>>,
    reply: RefCell<Option<PermissionReply>>,
    /// The agent tasks, to record the answer in.
    tasks: Option<Tasks>,
    done: async_channel::Sender<()>,
    /// The dialog's own focus, taken when it first draws, so the sheet
    /// closes itself only when it is the dialog in front.
    focus: RefCell<Option<FocusHandle>>,
}

impl Sheet {
    /// Answers the request, unless something already did, and remembers the
    /// tool when the person chose to allow it for the session or for good.
    fn answer(&self, input: Input, cx: Option<&mut App>) {
        let Some(reply) = self.reply.borrow_mut().take() else {
            return;
        };
        let chosen = choice(input);
        if !reply.is_settled() {
            if let Some(tasks) = &self.tasks {
                note(tasks, &self.request, chosen.answer());
            }
            match chosen {
                Choice::AllowSession => self
                    .grants
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .allow(&self.request),
                Choice::AlwaysAllow => {
                    if let Some(cx) = cx {
                        let tool = tool_of(&self.request);
                        AppState::update_config(cx, |config| {
                            if !config.ai.agent.always_allowed.contains(&tool) {
                                config.ai.agent.always_allowed.push(tool);
                            }
                        });
                    }
                }
                Choice::AllowOnce | Choice::Reject => {}
            }
        }
        match chosen {
            Choice::AllowOnce | Choice::AllowSession | Choice::AlwaysAllow => reply.allow_once(),
            Choice::Reject => reply.reject(),
        }
        self.done.try_send(()).ok();
    }

    /// The request ended without the person (time ran out, or the turn was
    /// cancelled), or was answered.
    fn is_over(&self) -> bool {
        self.reply
            .borrow()
            .as_ref()
            .is_none_or(PermissionReply::is_settled)
    }
}

/// Shows `ask` and returns once it is answered or over.
async fn present(ask: Ask, grants: Arc<Mutex<SessionGrants>>, cx: &mut AsyncApp) {
    if ask.reply.is_settled() {
        return;
    }
    let (done, finished) = async_channel::bounded(1);
    let request = ask.request;
    let tasks = cx.update(|cx| crate::agent_task_panel::tasks(cx));
    let sheet = Rc::new(Sheet {
        request: request.clone(),
        grants,
        tasks,
        reply: RefCell::new(Some(ask.reply)),
        done,
        focus: RefCell::new(None),
    });
    let shown = cx.update(|cx| {
        let Some((window, _)) = crate::workspace::main_workspace(cx) else {
            return false;
        };
        let sheet = sheet.clone();
        window
            .update(cx, move |_, window, cx| {
                open_sheet(&request, sheet, window, cx)
            })
            .is_ok()
    });
    if !shown {
        // No window to ask in: reject.
        sheet.answer(Input::Dismiss, None);
        return;
    }
    let watched = sheet.clone();
    let watch = cx.spawn(async move |cx| {
        loop {
            cx.background_executor().timer(WATCH).await;
            if watched.reply.borrow().is_none() {
                return;
            }
            if watched.is_over() {
                let sheet = watched.clone();
                cx.update(|cx| close_sheet(&sheet, cx));
                watched.answer(Input::Dismiss, None);
                return;
            }
        }
    });
    finished.recv().await.ok();
    drop(watch);
}

/// Closes the sheet: its request ended behind it. Only when it is the
/// dialog in front: another dialog opened over it is left alone, and the
/// sheet, once back in front, says its request ended.
fn close_sheet(sheet: &Sheet, cx: &mut App) {
    if let Some((window, _)) = crate::workspace::main_workspace(cx) {
        window
            .update(cx, |_, window, cx| {
                let in_front = sheet
                    .focus
                    .borrow()
                    .as_ref()
                    .is_some_and(|focus| focus.contains_focused(window, cx));
                if in_front {
                    window.close_dialog(cx);
                } else {
                    window.refresh();
                }
            })
            .ok();
    }
}

fn open_sheet(request: &PermissionRequest, sheet: Rc<Sheet>, window: &mut Window, cx: &mut App) {
    let heading = title(request);
    let description = format!(
        "{} is asking before it uses one of its own tools.",
        request.agent
    );
    let tool = tool_line(request);
    let kind = kind_line(request);
    window.open_alert_dialog(cx, move |alert, window, cx| {
        let colors = Colors::get(cx);
        if sheet.focus.borrow().is_none() {
            *sheet.focus.borrow_mut() = window.focused(cx);
        }
        let ended = sheet.is_over();
        let reject = sheet.clone();
        let allow = sheet.clone();
        let session = sheet.clone();
        let always = sheet.clone();
        let enter = sheet.clone();
        let escape = sheet.clone();
        alert
            .title(heading.clone())
            .description(description.clone())
            .width(px(FORM_WIDTH))
            .child(
                v_flex()
                    .gap(px(6.))
                    .child(
                        div()
                            .debug_selector(|| "permission-tool".into())
                            .text_size(px(14.))
                            .line_height(px(20.))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors.text)
                            .child(tool.clone()),
                    )
                    .children(kind.clone().map(|kind| {
                        div()
                            .debug_selector(|| "permission-kind".into())
                            .text_size(px(12.5))
                            .text_color(colors.text_secondary)
                            .child(kind)
                    }))
                    .child(
                        div()
                            .text_size(px(12.5))
                            .line_height(px(17.))
                            .text_color(colors.text_secondary)
                            .child(if ended { ENDED } else { NO_SANDBOX }),
                    ),
            )
            .footer(
                h_flex()
                    .w_full()
                    .justify_end()
                    .gap(px(8.))
                    .flex_wrap()
                    .child(
                        Button::new("permission-always")
                            .label("Always allow")
                            .debug_selector(|| "permission-always".into())
                            .on_click(move |_, window, cx| {
                                window.close_dialog(cx);
                                always.answer(Input::ClickAlwaysAllow, Some(cx));
                            }),
                    )
                    .child(
                        Button::new("permission-session")
                            .label("Allow for this session")
                            .debug_selector(|| "permission-session".into())
                            .on_click(move |_, window, cx| {
                                window.close_dialog(cx);
                                session.answer(Input::ClickAllowSession, Some(cx));
                            }),
                    )
                    .child(
                        Button::new("permission-allow")
                            .label("Allow once")
                            .debug_selector(|| "permission-allow".into())
                            .on_click(move |_, window, cx| {
                                window.close_dialog(cx);
                                allow.answer(Input::ClickAllowOnce, Some(cx));
                            }),
                    )
                    .child(
                        Button::new("permission-reject")
                            .label("Reject")
                            .primary()
                            .debug_selector(|| "permission-reject".into())
                            .on_click(move |_, window, cx| {
                                window.close_dialog(cx);
                                reject.answer(Input::ClickReject, Some(cx));
                            }),
                    ),
            )
            // Enter and Escape both reject.
            .on_ok(move |_, _, _| {
                enter.answer(Input::Enter, None);
                true
            })
            .on_cancel(move |_, _, _| {
                escape.answer(Input::Escape, None);
                true
            })
    });
}

#[cfg(test)]
mod tests {
    use super::{
        Choice, Input, SessionGrants, choice, is_allowed, kind_line, title, tool_line, tool_of,
    };
    use kanso_acp::PermissionRequest;
    use proptest::prelude::*;

    fn request(title: &str, kind: Option<&str>) -> PermissionRequest {
        PermissionRequest {
            agent: "Claude Agent".to_owned(),
            title: title.to_owned(),
            kind: kind.map(str::to_owned),
            session: Some("s1".to_owned()),
        }
    }

    fn in_session(mut request: PermissionRequest, session: &str) -> PermissionRequest {
        request.session = Some(session.to_owned());
        request
    }

    #[test]
    fn reject_is_the_default_and_only_the_allow_buttons_allow() {
        assert_eq!(choice(Input::Enter), Choice::Reject);
        assert_eq!(choice(Input::Escape), Choice::Reject);
        assert_eq!(choice(Input::Dismiss), Choice::Reject);
        assert_eq!(choice(Input::ClickReject), Choice::Reject);
        assert_eq!(choice(Input::ClickAllowOnce), Choice::AllowOnce);
        assert_eq!(choice(Input::ClickAllowSession), Choice::AllowSession);
        assert_eq!(choice(Input::ClickAlwaysAllow), Choice::AlwaysAllow);
    }

    #[test]
    fn a_session_allowed_tool_is_not_asked_again_in_that_session_only() {
        let asked = request("Run cargo test", Some("execute"));
        let mut session = SessionGrants::default();
        assert!(!is_allowed(&asked, &session, &[]));
        session.allow(&asked);
        assert!(is_allowed(&asked, &session, &[]));
        // The same tool in the same session, even with other spacing.
        assert!(is_allowed(
            &request("Run   cargo\ntest", Some("execute")),
            &session,
            &[]
        ));
        // A new session asks again.
        assert!(!is_allowed(&in_session(asked.clone(), "s2"), &session, &[]));
        // Another tool, kind or agent in the same session asks too.
        assert!(!is_allowed(
            &request("Run cargo build", Some("execute")),
            &session,
            &[]
        ));
        assert!(!is_allowed(
            &request("Run cargo test", Some("read")),
            &session,
            &[]
        ));
        let mut other = asked.clone();
        other.agent = "Other Agent".to_owned();
        assert!(!is_allowed(&other, &session, &[]));
    }

    #[test]
    fn a_request_with_no_session_cannot_be_allowed_for_a_session() {
        let mut asked = request("Run cargo test", None);
        asked.session = None;
        let mut session = SessionGrants::default();
        session.allow(&asked);
        assert!(!is_allowed(&asked, &session, &[]));
    }

    #[test]
    fn an_always_allowed_tool_is_not_asked_again_in_any_session() {
        let asked = request("Run cargo test", Some("execute"));
        let always = vec![tool_of(&asked)];
        let session = SessionGrants::default();
        assert!(is_allowed(&asked, &session, &always));
        assert!(is_allowed(
            &in_session(asked.clone(), "s9"),
            &session,
            &always
        ));
        assert!(!is_allowed(
            &request("Run cargo build", Some("execute")),
            &session,
            &always
        ));
        let mut other = asked;
        other.agent = "Other Agent".to_owned();
        assert!(!is_allowed(&other, &session, &always));
    }

    #[test]
    fn the_sheet_names_the_agent_the_tool_and_its_kind() {
        let asked = request("Read ~/.ssh/id_ed25519", Some("read"));
        assert_eq!(title(&asked), "Claude Agent wants to use a tool");
        assert_eq!(tool_line(&asked), "Read ~/.ssh/id_ed25519");
        assert_eq!(kind_line(&asked).as_deref(), Some("Kind: read"));
        assert_eq!(kind_line(&request("x", None)), None);
        assert_eq!(kind_line(&request("x", Some("  "))), None);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Whatever title an agent gives, it shows as one line of bounded
        /// length.
        #[test]
        fn prop_a_tool_title_is_one_bounded_line(title in "\\PC{0,600}") {
            let shown = tool_line(&request(&title, None));
            prop_assert!(!shown.contains('\n'));
            prop_assert!(shown.chars().count() <= super::TITLE_MAX + 1);
        }

        /// No input but clicking an Allow button ever allows.
        #[test]
        fn prop_only_the_allow_buttons_allow(
            inputs in proptest::collection::vec(
                prop_oneof![
                    Just(Input::ClickReject),
                    Just(Input::ClickAllowOnce),
                    Just(Input::ClickAllowSession),
                    Just(Input::ClickAlwaysAllow),
                    Just(Input::Enter),
                    Just(Input::Escape),
                    Just(Input::Dismiss),
                ],
                1..8,
            ),
        ) {
            for input in inputs {
                let clicked_allow = matches!(
                    input,
                    Input::ClickAllowOnce | Input::ClickAllowSession | Input::ClickAlwaysAllow
                );
                prop_assert_eq!(choice(input) != Choice::Reject, clicked_allow);
            }
        }
    }
}

//! Snapshot states for the six-page Welcome Tour (issue #510) in dark and
//! light themes.

use gpui_kit::{AnyWindowHandle, Entity, HeadlessAppContext};
use kanso_store::Store;

use crate::mcp_setup::Agent;
use crate::welcome_tour;
use crate::welcome_tour_model::{DetectedAgent, SHEET_WIDTH, TourPage};
use crate::workspace::Workspace;

pub const SHOTS: &[(&str, super::Prepare, Option<f32>)] = &[
    (
        "dark-welcome-tour-1-welcome",
        page_welcome,
        Some(SHEET_WIDTH),
    ),
    (
        "light-welcome-tour-1-welcome",
        page_welcome,
        Some(SHEET_WIDTH),
    ),
    (
        "dark-welcome-tour-2-look-and-feel",
        page_look_and_feel,
        Some(SHEET_WIDTH),
    ),
    (
        "light-welcome-tour-2-look-and-feel",
        page_look_and_feel,
        Some(SHEET_WIDTH),
    ),
    (
        "dark-welcome-tour-3-writing",
        page_writing,
        Some(SHEET_WIDTH),
    ),
    (
        "light-welcome-tour-3-writing",
        page_writing,
        Some(SHEET_WIDTH),
    ),
    (
        "dark-welcome-tour-4-account",
        page_account,
        Some(SHEET_WIDTH),
    ),
    (
        "light-welcome-tour-4-account",
        page_account,
        Some(SHEET_WIDTH),
    ),
    (
        "dark-welcome-tour-5-ai-voice",
        page_ai_voice,
        Some(SHEET_WIDTH),
    ),
    (
        "light-welcome-tour-5-ai-voice",
        page_ai_voice,
        Some(SHEET_WIDTH),
    ),
    ("dark-welcome-tour-6-agents", page_agents, Some(SHEET_WIDTH)),
    (
        "light-welcome-tour-6-agents",
        page_agents,
        Some(SHEET_WIDTH),
    ),
];

fn open_page(cx: &mut HeadlessAppContext, window: AnyWindowHandle, page: TourPage) {
    cx.update(|cx| cx.set_reduce_motion(true));
    cx.update_window(window, |_, window, cx| {
        let sheet = welcome_tour::open_at(page, window, cx);
        if page == TourPage::AgentsAndMcp {
            sheet.update(cx, |s, cx| {
                s.set_detected_agents_for_test(
                    vec![
                        DetectedAgent {
                            agent: Agent::ClaudeCode,
                            detected: true,
                        },
                        DetectedAgent {
                            agent: Agent::Codex,
                            detected: true,
                        },
                        DetectedAgent {
                            agent: Agent::Cursor,
                            detected: true,
                        },
                    ],
                    cx,
                );
            });
        }
    })
    .unwrap();
}

fn page_welcome(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    open_page(cx, window, TourPage::Welcome);
}

fn page_look_and_feel(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    open_page(cx, window, TourPage::LookAndFeel);
}

fn page_writing(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    open_page(cx, window, TourPage::Writing);
}

fn page_account(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    open_page(cx, window, TourPage::AccountAndSync);
}

fn page_ai_voice(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    open_page(cx, window, TourPage::AiAndVoice);
}

fn page_agents(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    open_page(cx, window, TourPage::AgentsAndMcp);
}

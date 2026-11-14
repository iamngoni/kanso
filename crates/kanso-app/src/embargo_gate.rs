//! The embargo gate: publishing a note, making a share link to it or
//! exporting it asks first while its `embargo` is still to come.
//!
//! The rules and words are `kanso_core::embargo`'s. The Publish and Share
//! sheets ask inside the sheet; an export asks in a dialog of its own
//! ([`confirm_then`]).

use std::cell::RefCell;
use std::rc::Rc;

use gpui_kit::component::WindowExt as _;
use gpui_kit::*;
use kanso_core::embargo::{self, Ask, Gate};

use crate::clock::Clock;

/// What asking before `gate` says for a note whose text is `body`, when
/// its embargo is still in force now: `None` lets the step go ahead.
// no-prop-test: delegates to kanso_core::embargo::{check, Embargo::ask}, tested there.
pub fn ask_for(body: &str, gate: Gate, cx: &App) -> Option<Ask> {
    let now = Clock::now_ms(cx);
    let local = Clock::local_ms(cx, now);
    embargo::check(body, now, local).map(|found| found.ask(gate, now, local))
}

type Go = Box<dyn FnOnce(&mut Window, &mut App)>;

/// The step a dialog waits to run, for tests to answer it as its button
/// does.
#[derive(Default)]
struct Waiting(Option<Rc<RefCell<Option<Go>>>>);

impl Global for Waiting {}

/// Runs `go` at once when there is nothing to ask; otherwise asks `ask` in
/// a dialog first and runs it only on the go-ahead.
// no-prop-test: a dialog's wiring; the embargo tests answer it both ways.
pub fn confirm_then(
    ask: Option<Ask>,
    window: &mut Window,
    cx: &mut App,
    go: impl FnOnce(&mut Window, &mut App) + 'static,
) {
    let Some(ask) = ask else {
        go(window, cx);
        return;
    };
    let waiting: Rc<RefCell<Option<Go>>> = Rc::new(RefCell::new(Some(Box::new(go))));
    cx.default_global::<Waiting>().0 = Some(waiting.clone());
    window.open_alert_dialog(cx, move |alert, _, _| {
        let waiting = waiting.clone();
        alert
            .title(ask.title.clone())
            .description(ask.body.clone())
            .show_cancel(true)
            .ok_text(ask.confirm.clone())
            .on_ok(move |_, window, cx| {
                if let Some(go) = waiting.borrow_mut().take() {
                    // After the dialog closes, so what it opens is on top.
                    window.defer(cx, move |window, cx| go(window, cx));
                }
                true
            })
    });
}

/// Answers the open embargo dialog as its go-ahead button does, for tests.
#[cfg(test)]
pub fn go_ahead_for_test(window: &mut Window, cx: &mut App) {
    let waiting = cx.default_global::<Waiting>().0.take();
    window.close_dialog(cx);
    if let Some(go) = waiting.and_then(|w| w.borrow_mut().take()) {
        go(window, cx);
    }
}

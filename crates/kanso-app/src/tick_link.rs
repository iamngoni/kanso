//! "Tick nested tasks together" (#509): whether a tick also ticks the tasks
//! nested under the task (and the parent once its last child is done), per
//! Settings › Editor. By default Kanso asks the first time it would matter
//! in a session; the answer "Always" is kept as the setting.

use std::cell::Cell;
use std::rc::Rc;

use gpui_kit::component::WindowExt as _;
use gpui_kit::*;
use kanso_core::dates::Date;
use kanso_core::tasks::set_task_checked_linked;
use kanso_store::Store;

use crate::config::TickNested;
use crate::workspace::AppState;

/// Set once the question was shown, so it is asked once a session.
#[derive(Default)]
struct Asked(bool);

impl Global for Asked {}

/// What a tick does about nested tasks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Link {
    /// Tick the one task.
    Plain,
    /// Tick the nested tasks with it.
    Linked,
    /// Ask first.
    Ask,
}

/// What to do for a tick, given the setting, whether the question was
/// already asked this session, and whether linking would change the tick
/// (only looked at when it matters).
pub fn decide(setting: TickNested, asked: bool, cascades: impl FnOnce() -> bool) -> Link {
    match setting {
        TickNested::Always => Link::Linked,
        TickNested::Never => Link::Plain,
        TickNested::Ask if asked => Link::Plain,
        TickNested::Ask if cascades() => Link::Ask,
        TickNested::Ask => Link::Plain,
    }
}

/// [`decide`] with the app's setting and this session's answer.
pub fn link_for(cx: &App, cascades: impl FnOnce() -> bool) -> Link {
    let setting = setting(cx);
    let asked = cx.try_global::<Asked>().is_some_and(|asked| asked.0);
    decide(setting, asked, cascades)
}

/// Whether the setting is "Always": for background quick-action paths that
/// cannot put a modal dialog (a reminder notification's Mark Done).
pub fn always(cx: &App) -> bool {
    setting(cx) == TickNested::Always
}

/// The setting; "Never" where there is no app state (a bare test window
/// without settings or a dialog layer).
fn setting(cx: &App) -> TickNested {
    cx.try_global::<AppState>()
        .map_or(TickNested::Never, |state| state.config.editor.tick_nested)
}

/// Whether ticking the task with `text` on `line` of note `id` as
/// `checked` would write something different with the link than without.
// no-prop-test: glue over `set_task_checked_linked`, which has them.
pub fn store_cascades(
    store: &Store,
    id: &str,
    line: u32,
    text: &str,
    checked: bool,
    today: Date,
) -> bool {
    let Ok(Some(note)) = store.note(id) else {
        return false;
    };
    let edit = |link| set_task_checked_linked(&note.body, line, text, checked, today, link);
    edit(true) != edit(false)
}

/// Asks "Tick the tasks under it too?" in `window`. `on_choice` gets `true`
/// for "Always" (which also becomes the setting) and `false` for "Not now"
/// (a plain tick, and no more questions this session), once.
// no-prop-test: a dialog; `decide` has the properties.
pub fn ask(
    window: &mut Window,
    cx: &mut App,
    on_choice: impl Fn(bool, &mut Window, &mut App) + 'static,
) {
    cx.default_global::<Asked>().0 = true;
    let answered = Rc::new(Cell::new(false));
    let on_choice = Rc::new(on_choice);
    window.open_alert_dialog(cx, move |alert, _, _| {
        let always = on_choice.clone();
        let not_now = on_choice.clone();
        let (a, b) = (answered.clone(), answered.clone());
        alert
            .title("Tick the tasks under it too?")
            .description(
                "Ticking a task can tick the tasks nested under it, and the parent once its last one is done. You can change this in Settings › Editor.",
            )
            .show_cancel(true)
            .cancel_text("Not now")
            .ok_text("Always")
            .on_ok(move |_, window, cx| {
                if !a.replace(true) {
                    crate::settings::set_tick_nested(cx, TickNested::Always);
                    always(true, window, cx);
                }
                true
            })
            .on_cancel(move |_, window, cx| {
                if !b.replace(true) {
                    not_now(false, window, cx);
                }
                true
            })
    });
}

/// [`ask`] in the window that is in front, for views that have no window at
/// hand. `false` when there is no window in front to ask in: the caller
/// ticks plainly then.
pub fn ask_in_front(
    cx: &mut App,
    on_choice: impl Fn(bool, &mut Window, &mut App) + 'static,
) -> bool {
    let Some(handle) = cx.active_window() else {
        return false;
    };
    cx.defer(move |cx| {
        let _ = handle.update(cx, |_, window, cx| ask(window, cx, on_choice));
    });
    true
}

/// For a view's tick: `Some(link)` when the tick can go ahead now, or
/// `None` when the question was put and `apply(view, link, cx)` runs with
/// the answer (a "Not now" is `false`).
// no-prop-test: glue over `decide` and `ask_in_front`.
pub fn link_or_ask<V: 'static>(
    cx: &mut Context<V>,
    cascades: impl FnOnce() -> bool,
    apply: impl Fn(&mut V, bool, &mut Context<V>) + 'static,
) -> Option<bool> {
    match link_for(cx, cascades) {
        Link::Plain => Some(false),
        Link::Linked => Some(true),
        Link::Ask => {
            let view = cx.entity().downgrade();
            let asked = ask_in_front(cx, move |link, _, cx| {
                view.update(cx, |view, cx| apply(view, link, cx)).ok();
            });
            if asked { None } else { Some(false) }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use proptest::prelude::*;

    use super::{Link, TickNested, decide};

    fn setting() -> impl Strategy<Value = TickNested> {
        prop::sample::select(vec![TickNested::Ask, TickNested::Always, TickNested::Never])
    }

    // Always links and Never does not; Ask asks only when it was not asked
    // yet and linking would change the tick.
    fn check(s: TickNested, asked: bool, cascades: bool) -> Result<(), TestCaseError> {
        let looked = Cell::new(false);
        let link = decide(s, asked, || {
            looked.set(true);
            cascades
        });
        match s {
            TickNested::Always => prop_assert_eq!(link, Link::Linked),
            TickNested::Never => prop_assert_eq!(link, Link::Plain),
            TickNested::Ask => {
                prop_assert_eq!(link == Link::Ask, !asked && cascades);
                prop_assert_eq!(looked.get(), !asked);
            }
        }
        Ok(())
    }

    proptest! {
        #[test]
        fn prop_the_setting_decides(s in setting(), asked in any::<bool>(), cascades in any::<bool>()) {
            check(s, asked, cascades)?;
        }
    }

    struct Host;

    impl gpui_kit::Render for Host {
        fn render(
            &mut self,
            _: &mut gpui_kit::Window,
            _: &mut gpui_kit::Context<Self>,
        ) -> impl gpui_kit::IntoElement {
            use gpui_kit::Styled as _;
            gpui_kit::div().size_full()
        }
    }

    #[gpui_kit::test]
    fn escape_closes_the_question_as_not_now(cx: &mut gpui_kit::TestAppContext) {
        use gpui_kit::AppContext as _;
        use gpui_kit::component::WindowExt as _;
        use std::rc::Rc;

        cx.update(|cx| {
            gpui_kit::init(cx);
            cx.set_reduce_motion(true);
        });
        let (_, vcx) = cx.add_window_view(|window, cx| {
            let host = cx.new(|_| Host);
            gpui_kit::base::Root::new(host, window, cx)
        });
        vcx.update(|window, cx| window.draw(cx).clear(cx));

        let choices = Rc::new(std::cell::RefCell::new(Vec::new()));
        let seen = choices.clone();
        vcx.update(|window, cx| {
            super::ask(window, cx, move |link, _, _| {
                seen.borrow_mut().push(link);
            });
        });
        vcx.run_until_parked();
        vcx.update(|window, cx| window.draw(cx).clear(cx));
        vcx.update(|window, cx| assert!(window.has_active_dialog(cx)));

        vcx.simulate_keystrokes("escape");
        vcx.run_until_parked();
        vcx.update(|window, cx| window.draw(cx).clear(cx));

        vcx.update(|window, cx| assert!(!window.has_active_dialog(cx)));
        assert_eq!(*choices.borrow(), vec![false]);
    }
}

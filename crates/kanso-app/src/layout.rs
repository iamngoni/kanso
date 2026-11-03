//! What the panes share so the window reads as one surface: pane widths, a
//! top band and a bottom band of the same height in every pane, and the room
//! macOS's traffic lights take in the leftmost pane's top band.
//!
//! The sidebar and the note list keep their widths, set by dragging their
//! edges; the editor takes whatever the window has left, so a wider window
//! or a hidden sidebar gives the editor the room ([`PaneWidths`]).
//!
//! The title bar is transparent and the app moves the window itself: a press
//! in a top band that no control claims, then a move, drags the window, and
//! a double-click there does what the system setting says (zoom or
//! minimize). [`BandDrag`] decides when.

use gpui_kit::{Pixels, Point};

/// The band at the top of every pane: the title bar's row.
pub const TOP_BAND_HEIGHT: f32 = 52.;
/// The band at the bottom of every pane: counts, sync and list controls.
pub const BOTTOM_BAND_HEIGHT: f32 = 36.;

pub const SIDEBAR_WIDTH: f32 = 220.;
pub const NOTE_LIST_WIDTH: f32 = 320.;
/// How narrow and how wide dragging can make the sidebar and the list.
pub const SIDEBAR_RANGE: (f32, f32) = (180., 320.);
pub const NOTE_LIST_RANGE: (f32, f32) = (260., 480.);
/// The editor's own minimum: dragging a pane wider stops short of it.
pub const EDITOR_MIN_WIDTH: f32 = 560.;
/// The smallest a main window gets, in points. Windows scales the minimum
/// by the display scale, and half of a 1366-wide screen at 125% is 546
/// points, so 520 lets Snap's half layout through at 100% and 125%.
pub const WINDOW_MIN_SIZE: (f32, f32) = (520., 420.);
/// In a window this narrow the editor keeps only its compact minimum and
/// the list may narrow below its usual range, so the panes still fit.
pub const EDITOR_COMPACT_MIN: f32 = 260.;
/// The list's compact floor leaves room for the widest rail (macOS keeps
/// the traffic lights in it, 76 points) beside the editor's compact minimum
/// at the window's minimum width: 76 + 184 + 260 = 520.
pub const NOTE_LIST_COMPACT_MIN: f32 = 184.;
/// Below this window width the sidebar folds to the icon rail whatever the
/// user chose, so the list and the editor keep their room.
pub const SIDEBAR_FOLD_BELOW: f32 = 820.;

/// Whether a window `window` points wide is too narrow for the full sidebar.
pub fn sidebar_folds(window: f32) -> bool {
    window < SIDEBAR_FOLD_BELOW
}

/// The inspector beside the editor (outline and backlinks): its width
/// before it is first dragged, and how narrow and wide dragging makes it.
pub const INSPECTOR_WIDTH: f32 = 264.;
pub const INSPECTOR_RANGE: (f32, f32) = (220., 420.);

/// The inspector's width once its left edge is dragged to `x` points from
/// the window's left, in a window `window` wide whose panes before the
/// editor take `before_editor`: following the pointer within its range,
/// and short of leaving the editor less than its minimum.
pub fn inspector_dragged(x: f32, window: f32, before_editor: f32) -> f32 {
    let room = window - before_editor - EDITOR_MIN_WIDTH;
    (window - x)
        .min(INSPECTOR_RANGE.1.min(room))
        .max(INSPECTOR_RANGE.0)
}

/// The inspector's width as drawn, from its set `width`: as set (within its
/// range) when the editor keeps its minimum beside it, narrower when not,
/// and hidden (`None`) when even its narrowest would leave the editor short.
/// The inspector gives way before the sidebar and the list do.
pub fn inspector_fitted(width: f32, window: f32, before_editor: f32) -> Option<f32> {
    let room = window - before_editor - EDITOR_MIN_WIDTH;
    if room + 1e-3 < INSPECTOR_RANGE.0 {
        return None;
    }
    Some(
        width
            .clamp(INSPECTOR_RANGE.0, INSPECTOR_RANGE.1)
            .min(room.max(INSPECTOR_RANGE.0)),
    )
}

/// Where macOS puts the close button, from the window's top left. The three
/// buttons are about 14 points across, 20 apart.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub const TRAFFIC_LIGHTS: (f32, f32) = (16., 18.);
/// The start of the leftmost pane's top band that the traffic lights cover.
pub const TRAFFIC_LIGHT_INSET: f32 = 76.;

/// How far the pointer moves with the button down before a press in a top
/// band becomes a window drag, so a click that wobbles stays a click.
pub const DRAG_THRESHOLD: f32 = 3.;

/// Whether the traffic lights sit in the leftmost pane's top band. Only
/// macOS draws them there; elsewhere the window buttons are in Kanso's own
/// title bar, above the panes (see `title_bar`).
pub const LIGHTS_IN_BAND: bool = cfg!(target_os = "macos");

/// The room at the start of a pane's top band: the traffic lights' in the
/// leftmost pane, and none in full screen, where they hide with the menu bar.
/// The system draws the lights at the same size at any zoom, so zoomed out
/// the room stays as wide as they are.
// no-prop-test: a two-way choice; `leading_inset_examples` covers it.
pub fn leading_inset(leftmost: bool, fullscreen: bool) -> f32 {
    if LIGHTS_IN_BAND && leftmost && !fullscreen {
        TRAFFIC_LIGHT_INSET / crate::zoom::scale().min(1.)
    } else {
        0.
    }
}

/// The narrowest the icon rail gets: one column of icon buttons.
pub const RAIL_MIN_WIDTH: f32 = 52.;

/// The icon rail the sidebar collapses to. It is the leftmost pane, so it
/// is wide enough to keep the traffic lights in its top band, and narrows
/// to one column of icons in full screen, where they hide.
pub fn rail_width(fullscreen: bool) -> f32 {
    RAIL_MIN_WIDTH.max(leading_inset(true, fullscreen))
}

/// The traffic lights' room still to leave at the start of the top band of
/// a pane whose left edge is `left` points from the window's: what the
/// panes before it don't already cover.
pub fn inset_after(left: f32, fullscreen: bool) -> f32 {
    (leading_inset(true, fullscreen) - left.max(0.)).max(0.)
}

/// The pane edge a drag moves: the sidebar's right edge, the list's, or
/// the inspector's left edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaneEdge {
    Sidebar,
    NoteList,
    Inspector,
}

/// The sidebar's and the note list's widths in points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaneWidths {
    pub sidebar: f32,
    pub list: f32,
}

impl Default for PaneWidths {
    fn default() -> Self {
        PaneWidths {
            sidebar: SIDEBAR_WIDTH,
            list: NOTE_LIST_WIDTH,
        }
    }
}

impl PaneWidths {
    /// The widths once `edge` is dragged to `x` points from the window's
    /// left, in a window `window` wide, with the full sidebar (`rail` is
    /// `None`) or a rail that wide before the list. The pane follows the
    /// pointer within its range, and short of leaving the editor less than
    /// its minimum; the other pane stays as it is.
    pub fn dragged(self, edge: PaneEdge, x: f32, window: f32, rail: Option<f32>) -> PaneWidths {
        let clamp =
            |width: f32, (min, max): (f32, f32), room: f32| width.min(max.min(room)).max(min);
        match edge {
            PaneEdge::Sidebar => PaneWidths {
                sidebar: clamp(x, SIDEBAR_RANGE, window - self.list - EDITOR_MIN_WIDTH),
                ..self
            },
            PaneEdge::NoteList => {
                let left = rail.unwrap_or(self.sidebar);
                PaneWidths {
                    list: clamp(x - left, NOTE_LIST_RANGE, window - left - EDITOR_MIN_WIDTH),
                    ..self
                }
            }
            // Its width is kept apart; see `inspector_dragged`.
            PaneEdge::Inspector => self,
        }
    }

    /// The widths that fit a window `window` wide, with the full sidebar
    /// (`rail` is `None`) or a rail that wide before the list: as they are
    /// when the editor keeps its minimum, else the list gives up room first
    /// and then the sidebar, each down to the narrow end of its range. Panes
    /// never grow, so the user's widths come back once the window does.
    pub fn fitted(self, window: f32, rail: Option<f32>) -> PaneWidths {
        let left = rail.unwrap_or(self.sidebar);
        let short = EDITOR_MIN_WIDTH - (window - left - self.list);
        // Rounding leaves a fitted layout a hair short; that is room enough.
        if short <= 1e-3 {
            return self;
        }
        let list = (self.list - short).max(NOTE_LIST_RANGE.0.min(self.list));
        let short = short - (self.list - list);
        let sidebar = if rail.is_none() && short > 0. {
            (self.sidebar - short).max(SIDEBAR_RANGE.0.min(self.sidebar))
        } else {
            self.sidebar
        };
        // Still short (a window near its minimum, beside a rail): the list
        // narrows below its range, down to a compact floor, so the editor
        // keeps at least its compact minimum.
        // (Without a rail the sidebar has folded to one long before this,
        // unless opened by hand in a narrow window: then the editor is short.)
        let left = rail.unwrap_or(sidebar);
        let short = EDITOR_COMPACT_MIN - (window - left - list);
        let list = if rail.is_some() && short > 1e-3 {
            (list - short).max(NOTE_LIST_COMPACT_MIN.min(list))
        } else {
            list
        };
        PaneWidths { sidebar, list }
    }
}

/// What a pointer event in the window asks of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BandAction {
    None,
    /// Hand the drag to the system: the window follows the pointer.
    StartMove,
    /// A double-click on the title bar: zoom or minimize, as set in System
    /// Settings.
    TitlebarDoubleClick,
}

/// Turns presses and moves in the top bands into window drags. A press
/// counts when it lands in a band and no control under it claimed it.
#[derive(Clone, Copy, Debug, Default)]
pub struct BandDrag {
    /// Where an unclaimed press in a band went down, until it is released,
    /// moved far enough or handed over.
    pressed_at: Option<Point<Pixels>>,
}

impl BandDrag {
    pub fn is_armed(&self) -> bool {
        self.pressed_at.is_some()
    }

    /// The left button went down at `position`, the `click_count`th click in
    /// a row. `claimed` is whether a control under the pointer took it.
    pub fn down(
        &mut self,
        position: Point<Pixels>,
        click_count: usize,
        claimed: bool,
    ) -> BandAction {
        self.pressed_at = None;
        let y = crate::zoom::unscaled(position.y);
        if claimed || y >= TOP_BAND_HEIGHT || y < 0. {
            return BandAction::None;
        }
        if click_count == 2 {
            return BandAction::TitlebarDoubleClick;
        }
        if click_count == 1 {
            self.pressed_at = Some(position);
        }
        BandAction::None
    }

    /// The pointer moved to `position`, with the left button `pressed` or
    /// not.
    pub fn moved(&mut self, position: Point<Pixels>, pressed: bool) -> BandAction {
        let Some(from) = self.pressed_at else {
            return BandAction::None;
        };
        if !pressed {
            self.pressed_at = None;
            return BandAction::None;
        }
        let (dx, dy) = (
            f32::from(position.x - from.x),
            f32::from(position.y - from.y),
        );
        if dx.hypot(dy) >= DRAG_THRESHOLD {
            self.pressed_at = None;
            BandAction::StartMove
        } else {
            BandAction::None
        }
    }

    pub fn up(&mut self) {
        self.pressed_at = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::px;

    fn point_at(x: f32, y: f32) -> Point<Pixels> {
        gpui_kit::point(px(x), px(y))
    }
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    #[derive(Clone, Debug)]
    enum Event {
        Down {
            x: f32,
            y: f32,
            clicks: usize,
            claimed: bool,
        },
        Move {
            x: f32,
            y: f32,
            pressed: bool,
        },
        Up,
    }

    /// Heights all over the window, and right at the band's edges.
    fn height() -> impl Strategy<Value = f32> {
        prop_oneof![
            4 => -10f32..120.,
            1 => Just(TOP_BAND_HEIGHT),
            1 => Just(TOP_BAND_HEIGHT - 0.5),
            1 => Just(0.),
            1 => Just(-0.5),
        ]
    }

    fn event() -> impl Strategy<Value = Event> {
        prop_oneof![
            (0f32..400., height(), 1usize..4, any::<bool>()).prop_map(|(x, y, clicks, claimed)| {
                Event::Down {
                    x,
                    y,
                    clicks,
                    claimed,
                }
            }),
            (0f32..400., height(), any::<bool>()).prop_map(|(x, y, pressed)| Event::Move {
                x,
                y,
                pressed
            }),
            Just(Event::Up),
        ]
    }

    proptest! {
        #![proptest_config(config())]

        /// Invariants over any run of events, independent of how the drag
        /// keeps its state: a claimed press, or one outside the band (the
        /// top 52 points), never acts; a double-click acts only unclaimed in
        /// the band; a window move follows only a single unclaimed press in
        /// the band, at most once per press, never on the press itself and
        /// never once the button came up (by an Up or a move without it).
        #[test]
        fn prop_window_moves_only_follow_single_band_presses(events in prop::collection::vec(event(), 0..40)) {
            let in_band = |y: f32| (0. ..TOP_BAND_HEIGHT).contains(&y);
            let mut drag = BandDrag::default();
            // The last press: whether it may start a move, and whether a
            // move or a release has used it up.
            let mut press_may_move = false;
            let mut used_up = true;
            for event in events {
                match event {
                    Event::Down { x, y, clicks, claimed } => {
                        let action = drag.down(point_at(x, y), clicks, claimed);
                        prop_assert!(action != BandAction::StartMove);
                        if claimed || !in_band(y) {
                            prop_assert_eq!(action, BandAction::None);
                            prop_assert!(!drag.is_armed());
                        }
                        if action == BandAction::TitlebarDoubleClick {
                            prop_assert!(clicks == 2 && !claimed && in_band(y));
                        }
                        press_may_move = clicks == 1 && !claimed && in_band(y);
                        used_up = false;
                    }
                    Event::Move { x, y, pressed } => {
                        let action = drag.moved(point_at(x, y), pressed);
                        prop_assert!(action != BandAction::TitlebarDoubleClick);
                        if action == BandAction::StartMove {
                            prop_assert!(press_may_move && !used_up && pressed);
                            used_up = true;
                        }
                        if !pressed {
                            used_up = true;
                        }
                    }
                    Event::Up => {
                        drag.up();
                        prop_assert!(!drag.is_armed());
                        used_up = true;
                    }
                }
            }
        }

        /// Invariants: a dragged pane stays within its range, the other
        /// pane keeps its width, and the editor keeps its minimum whenever
        /// the window has room for it.
        #[test]
        fn prop_pane_drags_stay_in_bounds(
            sidebar in SIDEBAR_RANGE.0..=SIDEBAR_RANGE.1,
            list in NOTE_LIST_RANGE.0..=NOTE_LIST_RANGE.1,
            sidebar_edge: bool,
            shown: bool,
            rail in prop_oneof![Just(0f32), Just(RAIL_MIN_WIDTH), Just(TRAFFIC_LIGHT_INSET)],
            x in -100f32..2000.,
            window in 900f32..2400.,
        ) {
            let before = PaneWidths { sidebar, list };
            let edge = if sidebar_edge { PaneEdge::Sidebar } else { PaneEdge::NoteList };
            let rail = (!shown).then_some(rail);
            let after = before.dragged(edge, x, window, rail);
            prop_assert!((SIDEBAR_RANGE.0..=SIDEBAR_RANGE.1).contains(&after.sidebar));
            prop_assert!((NOTE_LIST_RANGE.0..=NOTE_LIST_RANGE.1).contains(&after.list));
            let left = match rail {
                Some(rail) if !sidebar_edge => rail,
                _ => after.sidebar,
            };
            match edge {
                PaneEdge::Sidebar => prop_assert_eq!(after.list, before.list),
                PaneEdge::NoteList => prop_assert_eq!(after.sidebar, before.sidebar),
                PaneEdge::Inspector => unreachable!(),
            }
            let (other, min) = match edge {
                PaneEdge::Sidebar => (after.list, SIDEBAR_RANGE.0),
                PaneEdge::NoteList | PaneEdge::Inspector => (left, NOTE_LIST_RANGE.0),
            };
            let editor = window - left - after.list;
            if window - other - min >= EDITOR_MIN_WIDTH {
                prop_assert!(editor >= EDITOR_MIN_WIDTH - 1e-3, "{:?} leaves {}", after, editor);
            }
        }

        /// Monotonicity, and the pane follows the pointer wherever the
        /// pointer asks for an allowed width.
        #[test]
        fn prop_pane_drags_follow_the_pointer(a in 0f32..900., b in 0f32..900., shown: bool) {
            let start = PaneWidths::default();
            let window = 1800.;
            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
            let rail = (!shown).then_some(rail_width(false));
            let left = rail.unwrap_or(start.sidebar);
            let at = |x| start.dragged(PaneEdge::NoteList, x, window, rail).list;
            prop_assert!(at(lo) <= at(hi));
            let wanted = lo - left;
            if (NOTE_LIST_RANGE.0..=NOTE_LIST_RANGE.1).contains(&wanted) {
                prop_assert_eq!(at(lo), wanted);
            }
        }
    }

    /// Widths anywhere in their ranges, and a little outside them, as an
    /// older config or a hand-edited file may hold.
    fn widths() -> impl Strategy<Value = PaneWidths> {
        (
            SIDEBAR_RANGE.0 - 20.0..=SIDEBAR_RANGE.1 + 20.,
            NOTE_LIST_RANGE.0 - 20.0..=NOTE_LIST_RANGE.1 + 20.,
        )
            .prop_map(|(sidebar, list)| PaneWidths { sidebar, list })
    }

    fn rail() -> impl Strategy<Value = Option<f32>> {
        prop_oneof![
            Just(None),
            Just(Some(RAIL_MIN_WIDTH)),
            Just(Some(TRAFFIC_LIGHT_INSET))
        ]
    }

    proptest! {
        #![proptest_config(config())]

        /// Invariants of fitting the panes to a window: the editor keeps its
        /// minimum whenever the narrowest panes leave it room; panes only
        /// ever narrow, never below their range unless they already were;
        /// the list gives up room before the sidebar does; nothing changes
        /// when the editor already has its room; fitting twice is fitting
        /// once; and beside the rail the sidebar's width is left alone.
        #[test]
        fn prop_fitted_panes_leave_the_editor_its_room(
            before in widths(),
            rail in rail(),
            window in 600f32..2600.,
        ) {
            let after = before.fitted(window, rail);
            let left = |w: PaneWidths| rail.unwrap_or(w.sidebar);
            let editor = |w: PaneWidths| window - left(w) - w.list;
            prop_assert!(after.sidebar <= before.sidebar && after.list <= before.list);
            prop_assert!(after.list >= before.list.min(NOTE_LIST_RANGE.0));
            prop_assert!(after.sidebar >= before.sidebar.min(SIDEBAR_RANGE.0));
            let narrowest_left = rail.unwrap_or(before.sidebar.min(SIDEBAR_RANGE.0));
            if window - narrowest_left - before.list.min(NOTE_LIST_RANGE.0) >= EDITOR_MIN_WIDTH {
                prop_assert!(editor(after) >= EDITOR_MIN_WIDTH - 1e-3, "{:?} leaves {}", after, editor(after));
            }
            if editor(before) >= EDITOR_MIN_WIDTH {
                prop_assert_eq!(after, before);
            }
            if after.sidebar < before.sidebar {
                prop_assert_eq!(after.list, before.list.min(NOTE_LIST_RANGE.0));
            }
            if rail.is_some() {
                prop_assert_eq!(after.sidebar, before.sidebar);
            }
            prop_assert_eq!(after.fitted(window, rail), after);
        }
    }

    #[test]
    fn fitted_examples() {
        let wide = PaneWidths {
            sidebar: 300.,
            list: 480.,
        };
        // Room to spare: nothing moves.
        assert_eq!(wide.fitted(1600., None), wide);
        // 1100 points leave the editor 320: the list gives up 220 points,
        // down to its 260, and the sidebar the other 20.
        assert_eq!(
            wide.fitted(1100., None),
            PaneWidths {
                sidebar: 280.,
                list: 260.
            }
        );
        // Beside the rail only the list narrows.
        assert_eq!(
            wide.fitted(1100., Some(76.)),
            PaneWidths {
                sidebar: 300.,
                list: 464.
            }
        );
        // A window too narrow for everything leaves both at their minimum.
        assert_eq!(
            wide.fitted(700., None),
            PaneWidths {
                sidebar: SIDEBAR_RANGE.0,
                list: NOTE_LIST_RANGE.0
            }
        );
    }

    proptest! {
        #![proptest_config(config())]

        /// The threshold decides: after an unclaimed press in a band, a move
        /// with the button down starts a window move exactly when it went at
        /// least the threshold away, in any direction.
        #[test]
        fn prop_threshold_decides_a_drag(
            x in 0f32..400.,
            y in 0f32..TOP_BAND_HEIGHT,
            angle in 0f32..std::f32::consts::TAU,
            distance in 0f32..2. * DRAG_THRESHOLD,
        ) {
            // Rounding makes a move right at the threshold go either way.
            prop_assume!((distance - DRAG_THRESHOLD).abs() > 1e-3);
            let mut drag = BandDrag::default();
            prop_assert_eq!(drag.down(point_at(x, y), 1, false), BandAction::None);
            let to = point_at(x + distance * angle.cos(), y + distance * angle.sin());
            let expected = if distance >= DRAG_THRESHOLD {
                BandAction::StartMove
            } else {
                BandAction::None
            };
            prop_assert_eq!(drag.moved(to, true), expected);
            prop_assert_eq!(drag.is_armed(), expected == BandAction::None);
        }
    }

    #[test]
    fn band_drag_examples() {
        let mut drag = BandDrag::default();
        // A click on a control in the band never drags.
        assert_eq!(drag.down(point_at(100., 20.), 1, true), BandAction::None);
        assert_eq!(drag.moved(point_at(140., 20.), true), BandAction::None);
        // A press on empty band, a wobble, then a real move.
        assert_eq!(drag.down(point_at(100., 20.), 1, false), BandAction::None);
        assert_eq!(drag.moved(point_at(101., 21.), true), BandAction::None);
        assert_eq!(drag.moved(point_at(102.9, 20.), true), BandAction::None);
        assert_eq!(drag.moved(point_at(103., 20.), true), BandAction::StartMove);
        assert_eq!(drag.moved(point_at(110., 20.), true), BandAction::None);
        // A press that is released doesn't drag on the next move.
        assert_eq!(drag.down(point_at(100., 20.), 1, false), BandAction::None);
        drag.up();
        assert_eq!(drag.moved(point_at(110., 20.), true), BandAction::None);
        assert_eq!(drag.down(point_at(100., 20.), 1, false), BandAction::None);
        assert_eq!(drag.moved(point_at(110., 20.), true), BandAction::StartMove);
        assert_eq!(drag.moved(point_at(130., 20.), true), BandAction::None);
        // Below the band is the panes' content.
        assert_eq!(drag.down(point_at(100., 60.), 2, false), BandAction::None);
        assert_eq!(
            drag.down(point_at(100., 51.), 2, false),
            BandAction::TitlebarDoubleClick
        );
    }

    #[test]
    fn pane_drag_examples() {
        let widths = PaneWidths::default();
        assert_eq!(
            widths,
            PaneWidths {
                sidebar: 220.,
                list: 320.
            }
        );
        // The list's right edge to x = 600 with the sidebar beside it.
        assert_eq!(
            widths.dragged(PaneEdge::NoteList, 600., 1440., None).list,
            380.
        );
        // Beside the rail the list starts at the rail's edge.
        assert_eq!(
            widths
                .dragged(PaneEdge::NoteList, 376., 1440., Some(76.))
                .list,
            300.
        );
        // Too far either way stops at the range.
        assert_eq!(
            widths.dragged(PaneEdge::Sidebar, 50., 1440., None).sidebar,
            180.
        );
        assert_eq!(
            widths.dragged(PaneEdge::Sidebar, 900., 1440., None).sidebar,
            320.
        );
        // In the smallest window the editor keeps its 560 points.
        assert_eq!(
            widths.dragged(PaneEdge::NoteList, 900., 1100., None).list,
            320.
        );
    }

    proptest! {
        #![proptest_config(config())]

        /// The room left for the traffic lights: all of it at the window's
        /// edge, none in full screen, less the further right a pane starts,
        /// and never so little that a pane's band starts under a light.
        #[test]
        fn prop_inset_after_clears_the_lights(a in -50f32..400., b in -50f32..400., fullscreen: bool) {
            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
            prop_assert!(inset_after(lo, fullscreen) >= inset_after(hi, fullscreen));
            prop_assert!(inset_after(lo, fullscreen) >= 0.);
            prop_assert_eq!(inset_after(0., fullscreen), leading_inset(true, fullscreen));
            if fullscreen || !LIGHTS_IN_BAND {
                prop_assert_eq!(inset_after(lo, fullscreen), 0.);
            } else {
                prop_assert!(lo.max(0.) + inset_after(lo, false) >= TRAFFIC_LIGHT_INSET);
            }
            // The rail covers the lights, so whatever follows it starts flush.
            prop_assert_eq!(inset_after(rail_width(fullscreen), fullscreen), 0.);
        }
    }

    proptest! {
        #![proptest_config(config())]

        /// The sidebar folds exactly below its threshold, and at the
        /// window's minimum the rail, the narrowest list and the editor's
        /// floor all fit side by side.
        #[test]
        fn folding_is_monotonic_and_the_minimum_fits(a in 0f32..3000., b in 0f32..3000.) {
            let (lo, hi) = if a < b { (a, b) } else { (b, a) };
            prop_assert!(!sidebar_folds(hi) || sidebar_folds(lo) == (lo < SIDEBAR_FOLD_BELOW));
            prop_assert!(sidebar_folds(WINDOW_MIN_SIZE.0));
            prop_assert!(rail_width(false) + NOTE_LIST_COMPACT_MIN + EDITOR_COMPACT_MIN <= WINDOW_MIN_SIZE.0);
        }
    }

    proptest! {
        #![proptest_config(config())]

        /// Invariants of the inspector's width: a drag keeps it within its
        /// range, a wider drag never makes it narrower, and fitting it to a
        /// window either hides it or leaves the editor its minimum (when the
        /// panes before it leave room for its narrowest) without going out
        /// of its range; fitting what is already fitted changes nothing.
        #[test]
        fn prop_the_inspector_leaves_the_editor_its_room(
            width in 150f32..500.,
            window in 600f32..2600.,
            before in 0f32..900.,
            a in 0f32..2600.,
            b in 0f32..2600.,
        ) {
            let dragged = inspector_dragged(a, window, before);
            prop_assert!((INSPECTOR_RANGE.0..=INSPECTOR_RANGE.1).contains(&dragged));
            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
            prop_assert!(inspector_dragged(lo, window, before) >= inspector_dragged(hi, window, before));
            match inspector_fitted(width, window, before) {
                None => prop_assert!(window - before - EDITOR_MIN_WIDTH < INSPECTOR_RANGE.0),
                Some(shown) => {
                    prop_assert!((INSPECTOR_RANGE.0..=INSPECTOR_RANGE.1).contains(&shown));
                    prop_assert!(window - before - shown >= EDITOR_MIN_WIDTH - 1e-2);
                    prop_assert_eq!(inspector_fitted(shown, window, before), Some(shown));
                }
            }
        }
    }

    #[test]
    fn inspector_examples() {
        // Room to spare: as set.
        assert_eq!(inspector_fitted(300., 1600., 540.), Some(300.));
        // 1440 wide with the sidebar and the list: 340 points spare.
        assert_eq!(inspector_fitted(400., 1440., 540.), Some(340.));
        // Too narrow: hidden, so the editor keeps its room.
        assert_eq!(inspector_fitted(264., 1200., 540.), None);
        // Dragging its edge to x = 1100 in a 1440 window: 340 wide.
        assert_eq!(inspector_dragged(1100., 1440., 540.), 340.);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn rail_examples() {
        assert_eq!(rail_width(false), TRAFFIC_LIGHT_INSET);
        assert_eq!(rail_width(true), RAIL_MIN_WIDTH);
        assert_eq!(inset_after(0., false), TRAFFIC_LIGHT_INSET);
        assert_eq!(inset_after(52., false), 24.);
        assert_eq!(inset_after(220., false), 0.);
        // A 52 point rail would leave the third light over the next pane.
        assert!(TRAFFIC_LIGHTS.0 + 2. * 20. + 14. > RAIL_MIN_WIDTH);
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn no_inset_without_lights_in_the_band() {
        assert_eq!(leading_inset(true, false), 0.);
        assert_eq!(rail_width(false), RAIL_MIN_WIDTH);
        assert_eq!(inset_after(0., false), 0.);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn leading_inset_examples() {
        assert_eq!(leading_inset(true, false), TRAFFIC_LIGHT_INSET);
        assert_eq!(leading_inset(true, true), 0.);
        assert_eq!(leading_inset(false, false), 0.);
        // The inset clears the third light with a margin.
        assert!(TRAFFIC_LIGHTS.0 + 2. * 20. + 14. < TRAFFIC_LIGHT_INSET);
        assert!(TRAFFIC_LIGHTS.1 + 14. < TOP_BAND_HEIGHT);
    }
}

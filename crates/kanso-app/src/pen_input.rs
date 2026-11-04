//! Pen pressure, tilt and proximity from the platform, for the canvas.
//!
//! GPUI's mouse events carry a position and nothing else, so the platform's
//! own tablet events are watched on the side and the canvas asks what the pen
//! is doing when a mouse event arrives ([`current`]). The shim only observes:
//! every event goes on to GPUI untouched.
//!
//! macOS reads `NSEvent`'s tablet fields (pressure, tilt, proximity, which
//! end of the pen). A pen in proximity turns the mouse off for drawing: its
//! samples are the pen's. Windows (the pointer API) and Linux (Wayland
//! `tablet-v2`, X11 XInput2) are not read yet: there, and for a trackpad or
//! mouse anywhere, pressure is simulated from speed.

use std::sync::Mutex;

/// What the pen is doing now.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PenState {
    /// 0 to 1.
    pub pressure: f64,
    /// Azimuth and altitude in radians.
    pub tilt: Option<(f64, f64)>,
    /// The eraser end is the one near the surface.
    pub eraser: bool,
    /// A pen is near the tablet: the mouse should not draw.
    pub in_proximity: bool,
}

static STATE: Mutex<Option<PenState>> = Mutex::new(None);

/// The pen's state while one is near the tablet; `None` otherwise, when the
/// canvas takes the pointer as a mouse.
pub fn current() -> Option<PenState> {
    STATE
        .lock()
        .ok()
        .and_then(|s| *s)
        .filter(|s| s.in_proximity)
}

fn update(change: impl FnOnce(&mut PenState)) {
    if let Ok(mut state) = STATE.lock() {
        let mut next = state.unwrap_or(PenState {
            pressure: 0.0,
            tilt: None,
            eraser: false,
            in_proximity: false,
        });
        change(&mut next);
        *state = Some(next);
    }
}

/// The pen near the tablet (`entering`), or gone; `eraser` for the flipped end.
pub fn proximity(entering: bool, eraser: bool) {
    update(|s| {
        s.in_proximity = entering;
        s.eraser = entering && eraser;
        if !entering {
            s.pressure = 0.0;
            s.tilt = None;
        }
    });
}

/// A mouse press that came from a mouse or trackpad, not a tablet. The pen
/// can't be near the tablet then: AppKit only tells this app it left when it
/// was in front, so a pen put down while Kanso was behind would otherwise
/// stay "near" for good, and every mouse stroke would draw at its last
/// pressure (zero, after the pen lifted).
pub fn plain_mouse_down() {
    if let Ok(mut state) = STATE.lock()
        && let Some(s) = state.as_mut()
    {
        s.in_proximity = false;
        s.eraser = false;
    }
}

/// A sample from the pen: `pressure` 0 to 1 and the tilt as the platform
/// gives it, each of x and y from -1 to 1.
pub fn sample(pressure: f64, tilt: Option<(f64, f64)>) {
    update(|s| {
        s.pressure = pressure.clamp(0.0, 1.0);
        s.tilt = tilt.map(|(x, y)| tilt_to_angles(x, y));
    });
}

/// A tilt of `(x, y)`, each from -1 to 1 (AppKit's), as azimuth and altitude
/// in radians: upright is an altitude of pi / 2, lying flat is 0. Pure.
pub fn tilt_to_angles(x: f64, y: f64) -> (f64, f64) {
    let (x, y) = (x.clamp(-1.0, 1.0), y.clamp(-1.0, 1.0));
    let lean = x.hypot(y).min(1.0);
    (y.atan2(x), std::f64::consts::FRAC_PI_2 * (1.0 - lean))
}

/// Starts watching the platform's tablet events. Call once, on the main
/// thread, at launch.
#[cfg(target_os = "macos")]
pub fn install() {
    use std::ptr::NonNull;

    use block2::RcBlock;
    use objc2_app_kit::{NSEvent, NSEventMask, NSEventSubtype, NSEventType, NSPointingDeviceType};

    let mask = NSEventMask::LeftMouseDown
        | NSEventMask::LeftMouseDragged
        | NSEventMask::LeftMouseUp
        | NSEventMask::MouseMoved
        | NSEventMask::TabletPoint
        | NSEventMask::TabletProximity;
    let handler = RcBlock::new(|event: NonNull<NSEvent>| -> *mut NSEvent {
        // SAFETY: AppKit passes a live event for the call's duration.
        let ev = unsafe { event.as_ref() };
        let kind = ev.r#type();
        if kind == NSEventType::TabletProximity {
            let eraser = ev.pointingDeviceType() == NSPointingDeviceType::Eraser;
            proximity(ev.isEnteringProximity(), eraser);
        } else if kind == NSEventType::LeftMouseDown
            && !matches!(
                ev.subtype(),
                NSEventSubtype::TabletPoint | NSEventSubtype::TabletProximity
            )
        {
            plain_mouse_down();
        } else if kind == NSEventType::TabletPoint
            || (matches!(
                kind,
                NSEventType::LeftMouseDown
                    | NSEventType::LeftMouseDragged
                    | NSEventType::LeftMouseUp
            ) && ev.subtype() == NSEventSubtype::TabletPoint)
        {
            let tilt = ev.tilt();
            let pressed = !matches!(kind, NSEventType::LeftMouseUp);
            sample(
                if pressed {
                    f64::from(ev.pressure())
                } else {
                    0.0
                },
                Some((tilt.x, tilt.y)),
            );
        }
        // Observe only: the event goes on to GPUI.
        event.as_ptr()
    });
    // SAFETY: the handler returns the event it was given, a valid pointer.
    let token = unsafe { NSEvent::addLocalMonitorForEventsMatchingMask_handler(mask, &handler) };
    // The monitor lives as long as the app.
    std::mem::forget(token);
}

#[cfg(not(target_os = "macos"))]
pub fn install() {}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn upright_is_straight_up_and_flat_is_zero() {
        let (_, alt) = tilt_to_angles(0.0, 0.0);
        assert!((alt - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
        let (az, alt) = tilt_to_angles(1.0, 0.0);
        assert!(alt.abs() < 1e-12 && az.abs() < 1e-12);
    }

    #[test]
    fn a_pen_in_proximity_is_the_current_state() {
        // The state is process-wide: this is the only test that writes it.
        proximity(false, false);
        assert_eq!(current(), None);
        proximity(true, true);
        sample(0.7, Some((0.0, 1.0)));
        let s = current().unwrap();
        assert!(s.eraser && s.in_proximity);
        assert!((s.pressure - 0.7).abs() < 1e-12);
        assert!(s.tilt.is_some());
        proximity(false, false);
        assert_eq!(current(), None);
        // A pen whose leaving was never seen: a plain mouse press ends it.
        proximity(true, false);
        sample(0.0, None);
        assert!(current().is_some());
        plain_mouse_down();
        assert_eq!(current(), None);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Invariant: whatever tilt the platform reports, the angles stay in
        /// their ranges and are finite.
        #[test]
        fn prop_angles_stay_in_range(x in -3f64..3., y in -3f64..3.) {
            let (az, alt) = tilt_to_angles(x, y);
            prop_assert!(az.is_finite() && alt.is_finite());
            prop_assert!(az.abs() <= std::f64::consts::PI + 1e-9);
            prop_assert!((0.0..=std::f64::consts::FRAC_PI_2 + 1e-12).contains(&alt));
        }
    }
}

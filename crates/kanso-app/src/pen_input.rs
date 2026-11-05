//! Pen pressure, tilt and proximity from the platform, for the canvas.
//!
//! GPUI's mouse events carry a position and nothing else, so the platform's
//! own tablet events are watched on the side and the canvas asks what the pen
//! is doing when a mouse event arrives ([`current`]). The shim only observes:
//! every event goes on to GPUI untouched.
//!
//! Every platform fills the same [`PenState`] and the canvas reads it the
//! same way ([`current`]); what differs is where the numbers come from:
//!
//! - macOS reads `NSEvent`'s tablet fields (pressure, tilt, proximity, which
//!   end of the pen) in a local event monitor.
//! - Windows (`pen_input_windows.rs`) watches the thread's message queue for
//!   `WM_POINTER*` messages of a pen and reads `GetPointerPenInfo`.
//! - Linux on X11 (`pen_input_x11.rs`) reads XInput2 raw events from the
//!   tablet's own device on a second connection to the X server (this covers
//!   XWayland too). A native Wayland window has no pen pressure: GPUI owns
//!   the only Wayland connection that receives `tablet-v2` events (they go to
//!   the focused surface) and passes none on.
//!
//! A pen in proximity turns the mouse off for drawing: its samples are the
//! pen's. For a trackpad or mouse anywhere, and for a pen with no pressure
//! axis, pressure is simulated from speed ([`NO_PRESSURE`]).
//!
//! The Windows and X11 readers decode in pure functions here ([`pointer`],
//! [`xi2`]), tested on every platform.

use std::sync::Mutex;
use std::time::{Duration, Instant};

/// The pressure written when the device reports none: the same marker the
/// renderer reads as "make the width from speed".
pub const NO_PRESSURE: f64 = 0.5;

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

/// When a pen seen by a platform with no proximity events is taken to have
/// gone, if nothing more is heard from it.
static LEASE: Mutex<Option<Instant>> = Mutex::new(None);

/// How long a pen stays near after its last event, where the platform says
/// nothing of it leaving.
pub const LEASE_FOR: Duration = Duration::from_millis(600);

/// Whether a pen whose lease ends at `lease` is still near at `now`. No lease
/// (the platform reports proximity itself) is always live. Pure.
pub fn lease_live(lease: Option<Instant>, now: Instant) -> bool {
    lease.is_none_or(|end| now <= end)
}

fn set_lease(lease: Option<Instant>) {
    if let Ok(mut l) = LEASE.lock() {
        *l = lease;
    }
}

/// The pen's state while one is near the tablet; `None` otherwise, when the
/// canvas takes the pointer as a mouse.
pub fn current() -> Option<PenState> {
    let lease = LEASE.lock().ok().and_then(|l| *l);
    if !lease_live(lease, Instant::now()) {
        return None;
    }
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
    set_lease(None);
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

/// What a platform read from the pen for one sample.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reading {
    /// The pen is near (or touching) the tablet.
    pub in_range: bool,
    /// The eraser end is the one near.
    pub eraser: bool,
    /// 0 to 1, `None` when the device has no pressure axis.
    pub pressure: Option<f64>,
    /// Azimuth and altitude in radians, when the device reports a tilt.
    pub tilt: Option<(f64, f64)>,
}

/// Takes a platform's reading as the pen's state. With `leased`, the pen is
/// taken to leave if [`LEASE_FOR`] passes with no further reading (for a
/// platform that has no proximity events).
// Used by the Windows and X11 readers only.
#[cfg_attr(
    not(any(target_os = "windows", target_os = "linux", test)),
    allow(dead_code)
)]
pub fn read(reading: Reading, leased: bool) {
    if !reading.in_range {
        proximity(false, false);
        return;
    }
    update(|s| {
        s.in_proximity = true;
        s.eraser = reading.eraser;
        s.pressure = reading.pressure.map_or(NO_PRESSURE, |p| p.clamp(0.0, 1.0));
        s.tilt = reading.tilt;
    });
    set_lease(leased.then(|| Instant::now() + LEASE_FOR));
}

/// A pen's tilt as two angles in degrees (Windows' `tiltX` and `tiltY`: the
/// pen's lean from upright in the x-z and the y-z planes, each from -90 to
/// 90) as azimuth and altitude in radians. Upright is an altitude of pi / 2,
/// lying flat 0. Pure.
pub fn tilt_planes_to_angles(tilt_x: f64, tilt_y: f64) -> (f64, f64) {
    let limit = 89.9_f64.to_radians();
    let x = tilt_x.to_radians().clamp(-limit, limit).tan();
    let y = tilt_y.to_radians().clamp(-limit, limit).tan();
    if x == 0.0 && y == 0.0 {
        return (0.0, std::f64::consts::FRAC_PI_2);
    }
    (y.atan2(x), 1f64.atan2(x.hypot(y)))
}

/// The Windows pointer API's pen fields, decoded.
#[cfg_attr(not(any(target_os = "windows", test)), allow(dead_code))]
pub mod pointer {
    use super::{Reading, tilt_planes_to_angles};

    pub const POINTER_FLAG_INRANGE: u32 = 0x0000_0002;
    pub const POINTER_FLAG_INCONTACT: u32 = 0x0000_0004;
    pub const PEN_FLAG_INVERTED: u32 = 0x0000_0002;
    pub const PEN_FLAG_ERASER: u32 = 0x0000_0004;
    pub const PEN_MASK_PRESSURE: u32 = 0x0000_0001;
    pub const PEN_MASK_TILT_X: u32 = 0x0000_0004;
    pub const PEN_MASK_TILT_Y: u32 = 0x0000_0008;
    /// `POINTER_PEN_INFO.pressure` runs 0 to 1024.
    pub const PRESSURE_MAX: f64 = 1024.0;

    /// A pen reading from `POINTER_INFO.pointerFlags` and the
    /// `POINTER_PEN_INFO` fields. Pressure is zero while the pen hovers;
    /// `pen_mask` says which of pressure and tilt the device reports. Pure.
    pub fn reading(
        pointer_flags: u32,
        pen_flags: u32,
        pen_mask: u32,
        pressure: u32,
        tilt_x: i32,
        tilt_y: i32,
    ) -> Reading {
        let contact = pointer_flags & POINTER_FLAG_INCONTACT != 0;
        Reading {
            in_range: pointer_flags & POINTER_FLAG_INRANGE != 0,
            eraser: pen_flags & (PEN_FLAG_INVERTED | PEN_FLAG_ERASER) != 0,
            pressure: (pen_mask & PEN_MASK_PRESSURE != 0).then(|| {
                if contact {
                    (f64::from(pressure) / PRESSURE_MAX).clamp(0.0, 1.0)
                } else {
                    0.0
                }
            }),
            tilt: (pen_mask & PEN_MASK_TILT_X != 0 && pen_mask & PEN_MASK_TILT_Y != 0)
                .then(|| tilt_planes_to_angles(f64::from(tilt_x), f64::from(tilt_y))),
        }
    }
}

/// XInput2 tablet axes, decoded.
#[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
pub mod xi2 {
    use super::{Reading, tilt_to_angles};

    /// One valuator axis of a device and its range.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Axis {
        pub number: u16,
        pub min: f64,
        pub max: f64,
    }

    impl Axis {
        /// `value` as a fraction of the axis' range, 0 to 1. `None` for an
        /// axis with no range. Pure.
        pub fn fraction(&self, value: f64) -> Option<f64> {
            let span = self.max - self.min;
            (span.is_finite() && span > 0.0 && value.is_finite())
                .then(|| ((value - self.min) / span).clamp(0.0, 1.0))
        }
    }

    /// A tablet device's pen axes.
    #[derive(Clone, Debug, PartialEq)]
    pub struct Tablet {
        pub pressure: Option<Axis>,
        pub tilt_x: Option<Axis>,
        pub tilt_y: Option<Axis>,
        /// The device is the pen's eraser end (xf86-input-wacom makes one
        /// device for each end).
        pub eraser: bool,
    }

    impl Tablet {
        /// A reading from one raw event's `(axis number, value)` pairs. The
        /// pen is in range: only a pen's device sends the event. A raw event
        /// lists only the axes that changed, so those it leaves out keep
        /// what `previous` (this device's last reading) had. Pure.
        pub fn reading(&self, values: &[(u16, f64)], previous: Option<&Reading>) -> Reading {
            let get = |axis: &Option<Axis>| {
                let axis = axis.as_ref()?;
                let (_, v) = values.iter().find(|(n, _)| *n == axis.number)?;
                axis.fraction(*v)
            };
            let tilt = match (get(&self.tilt_x), get(&self.tilt_y)) {
                (Some(x), Some(y)) => Some(tilt_to_angles(x * 2.0 - 1.0, y * 2.0 - 1.0)),
                _ => previous.and_then(|p| p.tilt),
            };
            Reading {
                in_range: true,
                eraser: self.eraser,
                pressure: get(&self.pressure).or_else(|| previous.and_then(|p| p.pressure)),
                tilt,
            }
        }
    }

    /// Which pen quantity an axis carries.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Kind {
        Pressure,
        TiltX,
        TiltY,
    }

    /// What an XInput2 valuator label (an `Abs ...` atom's name) measures.
    /// Pure.
    pub fn axis_kind(label: &str) -> Option<Kind> {
        match label {
            "Abs Pressure" => Some(Kind::Pressure),
            "Abs Tilt X" => Some(Kind::TiltX),
            "Abs Tilt Y" => Some(Kind::TiltY),
            _ => None,
        }
    }

    /// Whether a device is the eraser end, by its name. Pure.
    pub fn is_eraser(name: &str) -> bool {
        name.to_ascii_lowercase().contains("eraser")
    }
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

/// Starts watching the pen's `WM_POINTER*` messages. Call once, on the main
/// thread, at launch.
#[cfg(target_os = "windows")]
pub fn install() {
    pen_input_windows::install();
}

/// Starts watching the X server's tablet events, if the session has one.
#[cfg(target_os = "linux")]
pub fn install() {
    pen_input_x11::install();
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub fn install() {}

#[cfg(target_os = "windows")]
#[path = "pen_input_windows.rs"]
mod pen_input_windows;

#[cfg(target_os = "linux")]
#[path = "pen_input_x11.rs"]
mod pen_input_x11;

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
        // A platform's reading: a pressure-less pen writes the speed marker,
        // a leased pen goes when its lease runs out, and leaving ends it.
        read(
            Reading {
                in_range: true,
                eraser: false,
                pressure: None,
                tilt: None,
            },
            true,
        );
        let s = current().unwrap();
        assert!((s.pressure - NO_PRESSURE).abs() < 1e-12);
        read(
            Reading {
                in_range: true,
                eraser: true,
                pressure: Some(2.0),
                tilt: Some((0.0, 1.0)),
            },
            false,
        );
        let s = current().unwrap();
        assert!(s.eraser && (s.pressure - 1.0).abs() < 1e-12);
        read(
            Reading {
                in_range: false,
                eraser: false,
                pressure: None,
                tilt: None,
            },
            false,
        );
        assert_eq!(current(), None);
    }

    #[test]
    fn the_speed_marker_is_the_renderers() {
        assert!((NO_PRESSURE - kanso_render::canvas::pen::SIMULATED).abs() < 1e-12);
    }

    #[test]
    fn a_lease_runs_out() {
        let now = Instant::now();
        assert!(lease_live(None, now));
        assert!(lease_live(Some(now + LEASE_FOR), now));
        assert!(!lease_live(Some(now), now + Duration::from_millis(1)));
    }

    #[test]
    fn windows_pen_fields_decode() {
        use pointer::*;
        // Touching at half pressure, tilted 45 degrees along x.
        let r = reading(
            POINTER_FLAG_INRANGE | POINTER_FLAG_INCONTACT,
            0,
            PEN_MASK_PRESSURE | PEN_MASK_TILT_X | PEN_MASK_TILT_Y,
            512,
            45,
            0,
        );
        assert!(r.in_range && !r.eraser);
        assert!((r.pressure.unwrap() - 0.5).abs() < 1e-12);
        let (az, alt) = r.tilt.unwrap();
        assert!(az.abs() < 1e-9 && (alt - std::f64::consts::FRAC_PI_4).abs() < 1e-9);
        // Hovering: in range, no pressure whatever the field holds.
        let r = reading(
            POINTER_FLAG_INRANGE,
            PEN_FLAG_INVERTED,
            PEN_MASK_PRESSURE,
            900,
            0,
            0,
        );
        assert_eq!(r.pressure, Some(0.0));
        assert!(r.eraser && r.tilt.is_none());
        // A pen with no pressure axis, and one that left.
        let r = reading(POINTER_FLAG_INRANGE | POINTER_FLAG_INCONTACT, 0, 0, 0, 0, 0);
        assert!(r.pressure.is_none() && r.tilt.is_none());
        assert!(!reading(0, 0, 0, 0, 0, 0).in_range);
    }

    #[test]
    fn x11_axes_decode_and_carry_over() {
        use xi2::*;
        let tablet = Tablet {
            pressure: Some(Axis {
                number: 2,
                min: 0.0,
                max: 2000.0,
            }),
            tilt_x: Some(Axis {
                number: 3,
                min: -64.0,
                max: 63.0,
            }),
            tilt_y: Some(Axis {
                number: 4,
                min: -64.0,
                max: 63.0,
            }),
            eraser: false,
        };
        let r = tablet.reading(&[(0, 5.0), (2, 1000.0), (3, -0.5), (4, -0.5)], None);
        assert!((r.pressure.unwrap() - 0.5).abs() < 1e-12);
        assert!(r.tilt.is_some() && r.in_range && !r.eraser);
        // A motion that moved only x and y leaves pressure and tilt as they were.
        let next = tablet.reading(&[(0, 6.0), (1, 7.0)], Some(&r));
        assert_eq!(next.pressure, r.pressure);
        assert_eq!(next.tilt, r.tilt);
        // With nothing before it, there is none.
        assert!(tablet.reading(&[(0, 6.0)], None).pressure.is_none());
        assert_eq!(axis_kind("Abs Pressure"), Some(Kind::Pressure));
        assert_eq!(axis_kind("Rel X"), None);
        assert!(is_eraser("Wacom Intuos Pro M Pen eraser"));
        assert!(!is_eraser("Wacom Intuos Pro M Pen stylus"));
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

        /// Invariant: whatever plane tilts a device reports, even past its
        /// documented range, the angles are finite and in range, and a pen
        /// leaning further reads as lower.
        #[test]
        fn prop_plane_tilts_stay_in_range(x in -200f64..200., y in -200f64..200.) {
            let (az, alt) = tilt_planes_to_angles(x, y);
            prop_assert!(az.is_finite() && alt.is_finite());
            prop_assert!(az.abs() <= std::f64::consts::PI + 1e-9);
            prop_assert!((0.0..=std::f64::consts::FRAC_PI_2 + 1e-12).contains(&alt));
            let (_, more) = tilt_planes_to_angles(x * 1.5, y * 1.5);
            prop_assert!(more <= alt + 1e-9);
        }

        /// Invariant: a Windows pen's pressure is a fraction whatever the
        /// field holds, and zero unless the tip touches.
        #[test]
        fn prop_windows_pressure_is_a_fraction(flags in 0u32..8, pressure in any::<u32>(), mask in 0u32..16) {
            let r = pointer::reading(flags, 0, mask, pressure, 0, 0);
            if let Some(p) = r.pressure {
                prop_assert!((0.0..=1.0).contains(&p));
                if flags & pointer::POINTER_FLAG_INCONTACT == 0 {
                    prop_assert_eq!(p, 0.0);
                }
            }
        }

        /// Invariant: an X11 axis fraction is in 0 to 1 for any value, and
        /// grows with the value.
        #[test]
        fn prop_axis_fraction_is_monotone(min in -1000f64..0., span in 1f64..5000., a in -2000f64..8000., b in -2000f64..8000.) {
            let axis = xi2::Axis { number: 0, min, max: min + span };
            let (fa, fb) = (axis.fraction(a).unwrap(), axis.fraction(b).unwrap());
            prop_assert!((0.0..=1.0).contains(&fa));
            if a <= b { prop_assert!(fa <= fb); }
        }
    }
}

//! Pen pressure, tilt and which end of the pen, on Linux under X11 (and
//! XWayland).
//!
//! GPUI's X11 window gets the pen as a pointer and nothing more. XInput2 tells
//! more: a tablet's stylus is its own device with `Abs Pressure` and
//! `Abs Tilt X` / `Abs Tilt Y` valuators, and a client may ask for the *raw*
//! events of the master pointers on the root window, which carry every
//! valuator of the device that moved (`sourceid`) whichever window has the
//! focus. A second connection to the X server, on its own thread, selects
//! those events and keeps [`pen_input`](super) up to date. It only observes:
//! GPUI's own connection and events are untouched.
//!
//! X has no proximity event in the raw stream, so a pen is taken to be near
//! while its events keep coming and to have gone [`LEASE_FOR`](super::LEASE_FOR)
//! after the last.
//!
//! A native Wayland window gets nothing from here: Wayland delivers
//! `tablet-v2` events only to the focused surface of the connection that owns
//! it, which is GPUI's, and GPUI reads none. The canvas then simulates
//! pressure from speed.
//!
//! The decoding is `pen_input::xi2`, tested on every platform. This file is
//! the part that talks to the X server. It is checked with
//! `cargo check --target x86_64-unknown-linux-gnu` and has not been run
//! against a tablet.

use std::collections::HashMap;

use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xinput::{
    self, ConnectionExt as _, DeviceClassData, DeviceType, EventMask, Fp3232, XIEventMask,
};
use x11rb::protocol::xproto::ConnectionExt as _;
use x11rb::rust_connection::RustConnection;

use super::xi2::{Axis, Kind, Tablet, axis_kind, is_eraser};
use super::{Reading, plain_mouse_down, read};

/// XInput2's "all master devices".
const ALL_MASTER_DEVICES: xinput::DeviceId = 1;
/// XInput2's "all devices".
const ALL_DEVICES: xinput::DeviceId = 0;

pub fn install() {
    // A session with no X server (Wayland only) or no XInput 2.2 simply has
    // no pressure; nothing here is worth stopping the app for.
    if let Err(err) = std::thread::Builder::new()
        .name("kanso-pen-input".into())
        .spawn(|| {
            if let Err(why) = watch() {
                eprintln!("kanso: pen pressure is unavailable: {why}");
            }
        })
    {
        eprintln!("kanso: pen pressure is unavailable: {err}");
    }
}

fn fp(v: Fp3232) -> f64 {
    f64::from(v.integral) + f64::from(v.frac) / 4_294_967_296.0
}

fn watch() -> Result<(), String> {
    let (conn, screen) = x11rb::connect(None).map_err(|e| e.to_string())?;
    let root = conn.setup().roots[screen].root;
    conn.xinput_xi_query_version(2, 2)
        .map_err(|e| e.to_string())?
        .reply()
        .map_err(|e| e.to_string())?;
    conn.xinput_xi_select_events(
        root,
        &[EventMask {
            deviceid: ALL_MASTER_DEVICES,
            mask: vec![
                XIEventMask::RAW_MOTION
                    | XIEventMask::RAW_BUTTON_PRESS
                    | XIEventMask::RAW_BUTTON_RELEASE,
            ],
        }],
    )
    .map_err(|e| e.to_string())?;
    // Devices plugged in later: the hierarchy changes are on the root too.
    conn.xinput_xi_select_events(
        root,
        &[EventMask {
            deviceid: ALL_DEVICES,
            mask: vec![XIEventMask::HIERARCHY],
        }],
    )
    .map_err(|e| e.to_string())?;
    conn.flush().map_err(|e| e.to_string())?;

    let mut tablets = tablets(&conn);
    let mut last: HashMap<xinput::DeviceId, Reading> = HashMap::new();
    loop {
        match conn.wait_for_event().map_err(|e| e.to_string())? {
            Event::XinputHierarchy(_) => {
                tablets = self::tablets(&conn);
                last.clear();
            }
            Event::XinputRawMotion(e) | Event::XinputRawButtonPress(e) => {
                let Some(tablet) = tablets.get(&e.sourceid) else {
                    // A mouse or touchpad: only a press of it is a decision
                    // to draw with the mouse, which ends a pen's proximity.
                    if e.event_type == xinput::RAW_BUTTON_PRESS_EVENT {
                        plain_mouse_down();
                    }
                    continue;
                };
                let reading = tablet.reading(
                    &values(&e.valuator_mask, &e.axisvalues_raw),
                    last.get(&e.sourceid),
                );
                last.insert(e.sourceid, reading);
                read(reading, true);
            }
            Event::XinputRawButtonRelease(e) => {
                if let Some(tablet) = tablets.get(&e.sourceid) {
                    // The tip lifted: the pen is still near, pressing nothing.
                    let mut reading = tablet.reading(
                        &values(&e.valuator_mask, &e.axisvalues_raw),
                        last.get(&e.sourceid),
                    );
                    reading.pressure = reading.pressure.map(|_| 0.0);
                    last.insert(e.sourceid, reading);
                    read(reading, true);
                }
            }
            _ => {}
        }
    }
}

/// The `(axis number, value)` pairs of a raw event: `mask` has a bit for
/// each axis that changed, and `raw` holds those axes' values in order.
fn values(mask: &[u32], raw: &[Fp3232]) -> Vec<(u16, f64)> {
    let mut out = Vec::with_capacity(raw.len());
    let mut next = raw.iter();
    for (word, bits) in mask.iter().enumerate() {
        for bit in 0..32 {
            if bits & (1 << bit) != 0 {
                let Some(v) = next.next() else { return out };
                out.push(((word * 32 + bit) as u16, fp(*v)));
            }
        }
    }
    out
}

/// The slave devices that are pens: those with a pressure axis, by device ID.
fn tablets(conn: &RustConnection) -> HashMap<xinput::DeviceId, Tablet> {
    let mut found = HashMap::new();
    let Some(infos) = conn
        .xinput_xi_query_device(ALL_DEVICES)
        .ok()
        .and_then(|c| c.reply().ok())
        .map(|r| r.infos)
    else {
        return found;
    };
    for info in infos {
        if info.type_ != DeviceType::SLAVE_POINTER && info.type_ != DeviceType::FLOATING_SLAVE {
            continue;
        }
        let mut tablet = Tablet {
            pressure: None,
            tilt_x: None,
            tilt_y: None,
            eraser: is_eraser(&String::from_utf8_lossy(&info.name)),
        };
        for class in &info.classes {
            let DeviceClassData::Valuator(v) = &class.data else {
                continue;
            };
            let Some(label) = conn
                .get_atom_name(v.label)
                .ok()
                .and_then(|c| c.reply().ok())
                .map(|r| String::from_utf8_lossy(&r.name).into_owned())
            else {
                continue;
            };
            let axis = Some(Axis {
                number: v.number,
                min: fp(v.min),
                max: fp(v.max),
            });
            match axis_kind(&label) {
                Some(Kind::Pressure) => tablet.pressure = axis,
                Some(Kind::TiltX) => tablet.tilt_x = axis,
                Some(Kind::TiltY) => tablet.tilt_y = axis,
                None => {}
            }
        }
        if tablet.pressure.is_some() {
            found.insert(info.deviceid, tablet);
        }
    }
    found
}

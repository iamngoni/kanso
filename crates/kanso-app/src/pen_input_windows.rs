//! Pen pressure, tilt and proximity on Windows.
//!
//! GPUI's Windows window handles `WM_MOUSE*` and no pointer message, so a pen
//! reaches it as the mouse messages Windows makes from the pen's own
//! `WM_POINTER*` messages, and those carry a position and nothing else. The
//! pointer messages are posted to the thread's queue first, so a
//! `WH_GETMESSAGE` hook on the main thread sees each one as it is taken off
//! the queue and asks `GetPointerPenInfo` for the pen's fields. The hook only
//! observes: every message goes on to GPUI untouched.
//!
//! The decoding is `pen_input::pointer::reading`, tested on every platform.
//! This file is the part that talks to Windows. It is checked with
//! `cargo check --target x86_64-pc-windows-msvc` and has not been run against
//! a pen.

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::Pointer::{GetPointerPenInfo, GetPointerType, POINTER_PEN_INFO};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageExtraInfo, HC_ACTION, MSG, PM_REMOVE, PT_PEN, SetWindowsHookExW,
    WH_GETMESSAGE, WM_LBUTTONDOWN, WM_POINTERDOWN, WM_POINTERLEAVE, WM_POINTERUP, WM_POINTERUPDATE,
};

use super::{plain_mouse_down, pointer as decode, proximity, read};

/// The bits `GetMessageExtraInfo` carries for a mouse message that Windows
/// made from a pen or a finger (`MI_WP_SIGNATURE`).
const FROM_POINTER: isize = 0xFF51_5700;
const FROM_POINTER_MASK: isize = 0xFFFF_FF00;

pub fn install() {
    // SAFETY: `hook` matches HOOKPROC and only reads the MSG it is given for
    // the duration of the call; the hook lives as long as the thread.
    let hooked =
        unsafe { SetWindowsHookExW(WH_GETMESSAGE, Some(hook), None, GetCurrentThreadId()) };
    if hooked.is_err() {
        eprintln!("kanso: pen pressure is unavailable: the message hook was refused");
    }
}

unsafe extern "system" fn hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 && wparam.0 == PM_REMOVE.0 as usize && lparam.0 != 0 {
        // SAFETY: for HC_ACTION, `lparam` points to the MSG being returned.
        let msg = unsafe { &*(lparam.0 as *const MSG) };
        observe(msg);
    }
    // SAFETY: passes the hook's own arguments on.
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

fn observe(msg: &MSG) {
    match msg.message {
        WM_POINTERDOWN | WM_POINTERUPDATE | WM_POINTERUP | WM_POINTERLEAVE => {
            // GET_POINTERID_WPARAM.
            let id = (msg.wParam.0 & 0xFFFF) as u32;
            let mut kind = Default::default();
            // SAFETY: plain out-parameters.
            if unsafe { GetPointerType(id, &mut kind) }.is_err() || kind != PT_PEN {
                return;
            }
            if msg.message == WM_POINTERLEAVE {
                proximity(false, false);
                return;
            }
            let mut info = POINTER_PEN_INFO::default();
            // SAFETY: `info` is a POINTER_PEN_INFO to fill.
            if unsafe { GetPointerPenInfo(id, &mut info) }.is_err() {
                return;
            }
            read(
                decode::reading(
                    info.pointerInfo.pointerFlags.0,
                    info.penFlags,
                    info.penMask,
                    info.pressure,
                    info.tiltX,
                    info.tiltY,
                ),
                false,
            );
        }
        WM_LBUTTONDOWN => {
            // SAFETY: reads the calling thread's last message extra info.
            let extra = unsafe { GetMessageExtraInfo() }.0;
            // A press Windows made from a pen is the pen's; any other is a
            // real mouse, which ends a pen whose leaving was never seen.
            if extra & FROM_POINTER_MASK != FROM_POINTER {
                plain_mouse_down();
            }
        }
        _ => {}
    }
}

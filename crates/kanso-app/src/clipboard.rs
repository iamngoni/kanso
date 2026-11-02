//! Copying a secret (the recovery key) so it stays on this Mac and doesn't
//! stay long, nor outlast the app (docs/ACCOUNTS.md › Recovery key screen,
//! A12), and asking macOS
//! whether a folder syncs to iCloud. A few calls into AppKit and Foundation
//! through the Objective-C runtime; GPUI links both frameworks already.
//! On Windows and Linux the secret goes through GPUI's clipboard and is
//! cleared after its minute; nothing there is in iCloud.
//!
//! Also the HTML a paste can turn into Markdown, which GPUI's clipboard
//! doesn't read: `public.html` on macOS, CF_HTML on Windows.

use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use gpui_kit::*;

/// How long a copied secret stays on the clipboard.
pub const SECRET_LIFETIME: Duration = Duration::from_secs(60);

/// The clipboard's change count after the last secret copied, until it is
/// cleared or something else replaces it.
static LAST_SECRET: Mutex<Option<isize>> = Mutex::new(None);

/// Whether the clipboard still holds what was copied: nothing was copied
/// over it since (its change count is still the one after copying).
pub fn still_ours(copied: isize, now: isize) -> bool {
    copied == now
}

/// Whether to clear the clipboard now. `timer` is the copy whose minute is
/// up, or `None` at quit for whichever secret was copied last; `now` is the
/// clipboard's change count. Only the last secret is ever cleared, and only
/// while the clipboard still holds it; once its time comes it is forgotten,
/// cleared or not.
fn should_clear(last: &mut Option<isize>, timer: Option<isize>, now: Option<isize>) -> bool {
    let Some(copied) = *last else {
        return false;
    };
    if timer.is_some_and(|timer| timer != copied) {
        return false;
    }
    *last = None;
    now.is_some_and(|now| still_ours(copied, now))
}

/// Clears `timer`'s copy, or at quit (`None`) the last secret copied, if
/// the clipboard still holds it. On the main thread: AppKit's pasteboard.
fn clear_secret(timer: Option<isize>) {
    let mut last = LAST_SECRET
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    if last.is_some() && should_clear(&mut last, timer, mac::change_count()) {
        mac::clear();
    }
}

/// At quit: a recovery key copied less than a minute ago doesn't stay on
/// the clipboard after Kanso is gone.
pub fn clear_secret_on_quit() {
    clear_secret(None);
}

/// Copies `secret` to the clipboard for this Mac only (Universal Clipboard
/// doesn't send it to other devices), marked concealed and transient so
/// clipboard managers skip it, and clears it after [`SECRET_LIFETIME`]
/// unless something else was copied meanwhile. Returns false when the
/// clipboard couldn't be written.
#[cfg(target_os = "macos")]
pub fn copy_secret(secret: &str, cx: &mut App) -> bool {
    let Some(copied) = mac::copy_secret(secret) else {
        return false;
    };
    *LAST_SECRET
        .lock()
        .unwrap_or_else(|poison| poison.into_inner()) = Some(copied);
    cx.spawn(async move |cx| {
        cx.background_executor().timer(SECRET_LIFETIME).await;
        // AppKit's pasteboard, from the main thread.
        cx.update(|_| clear_secret(Some(copied)));
    })
    .detach();
    true
}

/// Copies `secret` with GPUI's clipboard, which has no concealed or
/// host-only marking off macOS, and clears it after [`SECRET_LIFETIME`]
/// while the clipboard still holds it. Only a hash of the secret waits for
/// the minute. Not cleared at quit.
#[cfg(not(target_os = "macos"))]
pub fn copy_secret(secret: &str, cx: &mut App) -> bool {
    fn fingerprint(text: &str) -> u64 {
        use std::hash::{Hash as _, Hasher as _};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        text.hash(&mut hasher);
        hasher.finish()
    }
    cx.write_to_clipboard(ClipboardItem::new_string(secret.to_owned()));
    let copied = fingerprint(secret);
    cx.spawn(async move |cx| {
        cx.background_executor().timer(SECRET_LIFETIME).await;
        cx.update(|cx| {
            let now = cx.read_from_clipboard().and_then(|item| item.text());
            if now.is_some_and(|now| still_ours(copied as isize, fingerprint(&now) as isize)) {
                cx.write_to_clipboard(ClipboardItem::new_string(String::new()));
            }
        });
    })
    .detach();
    true
}

/// Whether `folder` syncs to iCloud: macOS says it is ubiquitous (iCloud
/// Drive, or Desktop and Documents in iCloud), or it is inside iCloud
/// Drive's own folder.
pub fn syncs_to_icloud(folder: &Path, home: &Path) -> bool {
    crate::account_words::is_icloud_path(folder, home) || mac::is_ubiquitous(folder)
}

/// The HTML on the clipboard, for pasting formatted text as Markdown:
/// macOS's `public.html`, or the fragment of Windows' CF_HTML. Linux pastes
/// plain text only.
// no-prop-test: reads the system clipboard; `cf_html_fragment` has the
// properties.
#[cfg_attr(test, allow(dead_code))] // Tests set their own HTML.
pub fn read_html() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        mac::read_html()
    }
    #[cfg(target_os = "windows")]
    {
        win::read_html()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

/// The HTML fragment in a CF_HTML clipboard entry: the bytes between its
/// header's `StartFragment` and `EndFragment` offsets, or between the
/// fragment comments when the offsets don't fit.
#[cfg(any(target_os = "windows", test))]
fn cf_html_fragment(data: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(data);
    let offset = |key: &str| -> Option<usize> {
        let at = text.find(key)? + key.len();
        let digits: String = text[at..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        digits.parse().ok()
    };
    let by_offsets = match (offset("StartFragment:"), offset("EndFragment:")) {
        (Some(start), Some(end)) if start <= end && end <= data.len() => {
            Some(String::from_utf8_lossy(&data[start..end]).into_owned())
        }
        _ => None,
    };
    by_offsets
        .or_else(|| {
            let start = text.find("<!--StartFragment-->")? + "<!--StartFragment-->".len();
            let end = text[start..].find("<!--EndFragment-->")? + start;
            Some(text[start..end].to_owned())
        })
        .filter(|html| !html.trim().is_empty())
}

#[cfg(target_os = "windows")]
mod win {
    use windows_sys::Win32::System::DataExchange::{
        CloseClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard,
        RegisterClipboardFormatW,
    };
    use windows_sys::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};

    #[cfg_attr(test, allow(dead_code))]
    pub fn read_html() -> Option<String> {
        let name: Vec<u16> = "HTML Format\0".encode_utf16().collect();
        // SAFETY: a NUL-terminated UTF-16 name; registering is idempotent.
        let format = unsafe { RegisterClipboardFormatW(name.as_ptr()) };
        // SAFETY: plain queries of the clipboard, opened and closed here;
        // the locked memory is copied before it is unlocked.
        unsafe {
            if format == 0 || IsClipboardFormatAvailable(format) == 0 {
                return None;
            }
            if OpenClipboard(std::ptr::null_mut()) == 0 {
                return None;
            }
            let bytes = (|| {
                let handle = GetClipboardData(format);
                if handle.is_null() {
                    return None;
                }
                let data = GlobalLock(handle) as *const u8;
                if data.is_null() {
                    return None;
                }
                let bytes = std::slice::from_raw_parts(data, GlobalSize(handle)).to_vec();
                GlobalUnlock(handle);
                Some(bytes)
            })();
            CloseClipboard();
            let bytes = bytes?;
            // The data ends with a NUL.
            let end = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
            super::cf_html_fragment(&bytes[..end])
        }
    }
}

#[cfg(target_os = "macos")]
mod mac {
    use std::ffi::{CString, c_char, c_void};
    use std::path::Path;

    type Id = *mut c_void;
    type Sel = *mut c_void;

    #[link(name = "objc")]
    unsafe extern "C" {
        fn objc_getClass(name: *const c_char) -> Id;
        fn sel_registerName(name: *const c_char) -> Sel;
        fn objc_msgSend();
    }

    #[link(name = "AppKit", kind = "framework")]
    unsafe extern "C" {}

    #[link(name = "Foundation", kind = "framework")]
    unsafe extern "C" {
        static NSURLIsUbiquitousItemKey: Id;
    }

    /// `NSPasteboardContentsCurrentHostOnly`.
    const CURRENT_HOST_ONLY: usize = 1;
    const UTF8_TEXT: &str = "public.utf8-plain-text";
    /// nspasteboard.org's markers: don't show it, don't keep it.
    const CONCEALED: &str = "org.nspasteboard.ConcealedType";
    const TRANSIENT: &str = "org.nspasteboard.TransientType";

    fn class(name: &str) -> Option<Id> {
        let name = CString::new(name).ok()?;
        // SAFETY: a NUL-terminated name; the runtime returns nil if unknown.
        let class = unsafe { objc_getClass(name.as_ptr()) };
        (!class.is_null()).then_some(class)
    }

    fn sel(name: &str) -> Sel {
        let name = CString::new(name).expect("selector names have no NUL");
        // SAFETY: a NUL-terminated selector name; registering is idempotent.
        unsafe { sel_registerName(name.as_ptr()) }
    }

    /// `[NSString stringWithUTF8String:]`, autoreleased.
    fn ns_string(text: &str) -> Option<Id> {
        let text = CString::new(text).ok()?;
        let class = class("NSString")?;
        // SAFETY: `stringWithUTF8String:` takes a C string and returns an
        // autoreleased NSString (or nil).
        let send: unsafe extern "C" fn(Id, Sel, *const c_char) -> Id =
            unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
        let string = unsafe { send(class, sel("stringWithUTF8String:"), text.as_ptr()) };
        (!string.is_null()).then_some(string)
    }

    fn general_pasteboard() -> Option<Id> {
        let class = class("NSPasteboard")?;
        // SAFETY: `generalPasteboard` takes no arguments and returns an object.
        let send: unsafe extern "C" fn(Id, Sel) -> Id =
            unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
        let pasteboard = unsafe { send(class, sel("generalPasteboard")) };
        (!pasteboard.is_null()).then_some(pasteboard)
    }

    fn set_string(pasteboard: Id, text: &str, kind: &str) -> bool {
        let (Some(text), Some(kind)) = (ns_string(text), ns_string(kind)) else {
            return false;
        };
        // SAFETY: `setString:forType:` takes two NSStrings and returns BOOL.
        let send: unsafe extern "C" fn(Id, Sel, Id, Id) -> bool =
            unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
        unsafe { send(pasteboard, sel("setString:forType:"), text, kind) }
    }

    /// Writes the secret; returns the pasteboard's change count after it.
    pub fn copy_secret(secret: &str) -> Option<isize> {
        let pasteboard = general_pasteboard()?;
        // SAFETY: `prepareForNewContentsWithOptions:` takes an NSUInteger
        // option set and returns the new change count.
        let prepare: unsafe extern "C" fn(Id, Sel, usize) -> isize =
            unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
        unsafe {
            prepare(
                pasteboard,
                sel("prepareForNewContentsWithOptions:"),
                CURRENT_HOST_ONLY,
            )
        };
        let written = set_string(pasteboard, secret, UTF8_TEXT)
            && set_string(pasteboard, "", CONCEALED)
            && set_string(pasteboard, "", TRANSIENT);
        if !written {
            clear();
            return None;
        }
        change_count()
    }

    /// `[pasteboard stringForType:@"public.html"]`, as UTF-8.
    #[cfg_attr(test, allow(dead_code))]
    pub fn read_html() -> Option<String> {
        let pasteboard = general_pasteboard()?;
        let kind = ns_string("public.html")?;
        // SAFETY: `stringForType:` takes an NSString and returns an
        // autoreleased NSString, or nil when there is no HTML.
        let string_for: unsafe extern "C" fn(Id, Sel, Id) -> Id =
            unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
        let html = unsafe { string_for(pasteboard, sel("stringForType:"), kind) };
        if html.is_null() {
            return None;
        }
        // SAFETY: `UTF8String` returns a NUL-terminated buffer the string
        // owns, valid while it lives; it is copied at once.
        let utf8: unsafe extern "C" fn(Id, Sel) -> *const c_char =
            unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
        let bytes = unsafe { utf8(html, sel("UTF8String")) };
        if bytes.is_null() {
            return None;
        }
        let html = unsafe { std::ffi::CStr::from_ptr(bytes) };
        Some(html.to_string_lossy().into_owned())
    }

    pub fn change_count() -> Option<isize> {
        let pasteboard = general_pasteboard()?;
        // SAFETY: `changeCount` takes no arguments and returns NSInteger.
        let send: unsafe extern "C" fn(Id, Sel) -> isize =
            unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
        Some(unsafe { send(pasteboard, sel("changeCount")) })
    }

    pub fn clear() {
        if let Some(pasteboard) = general_pasteboard() {
            // SAFETY: `clearContents` takes no arguments and returns NSInteger.
            let send: unsafe extern "C" fn(Id, Sel) -> isize =
                unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
            unsafe { send(pasteboard, sel("clearContents")) };
        }
    }

    /// `NSURLIsUbiquitousItemKey` for the folder.
    pub fn is_ubiquitous(folder: &Path) -> bool {
        let (Some(class), Some(path)) = (class("NSURL"), folder.to_str().and_then(ns_string))
        else {
            return false;
        };
        // SAFETY: `fileURLWithPath:` takes an NSString and returns an
        // autoreleased NSURL.
        let file_url: unsafe extern "C" fn(Id, Sel, Id) -> Id =
            unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
        let url = unsafe { file_url(class, sel("fileURLWithPath:"), path) };
        if url.is_null() {
            return false;
        }
        let mut value: Id = std::ptr::null_mut();
        // SAFETY: `getResourceValue:forKey:error:` writes an autoreleased
        // NSNumber (or nil) through the first pointer; nil error is allowed.
        let get: unsafe extern "C" fn(Id, Sel, *mut Id, Id, *mut Id) -> bool =
            unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
        let found = unsafe {
            get(
                url,
                sel("getResourceValue:forKey:error:"),
                &mut value,
                NSURLIsUbiquitousItemKey,
                std::ptr::null_mut(),
            )
        };
        if !found || value.is_null() {
            return false;
        }
        // SAFETY: the value is an NSNumber; `boolValue` returns BOOL.
        let bool_value: unsafe extern "C" fn(Id, Sel) -> bool =
            unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
        unsafe { bool_value(value, sel("boolValue")) }
    }
}

#[cfg(not(target_os = "macos"))]
mod mac {
    use std::path::Path;

    pub fn change_count() -> Option<isize> {
        None
    }

    pub fn clear() {}

    pub fn is_ubiquitous(_: &Path) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::{cf_html_fragment, should_clear, still_ours};
    use proptest::prelude::*;

    #[test]
    fn examples() {
        assert!(still_ours(7, 7));
        assert!(!still_ours(7, 8));
    }

    /// CF_HTML as Windows writes it: a header of byte offsets, then the
    /// document with the fragment marked.
    fn cf_html(fragment: &str) -> Vec<u8> {
        let head = "Version:0.9\r\nStartHTML:0000000000\r\nEndHTML:0000000000\r\nStartFragment:0000000000\r\nEndFragment:0000000000\r\n";
        let before = "<html><body>\r\n<!--StartFragment-->";
        let after = "<!--EndFragment-->\r\n</body></html>";
        let start_html = head.len();
        let start = start_html + before.len();
        let end = start + fragment.len();
        let end_html = end + after.len();
        let head = format!(
            "Version:0.9\r\nStartHTML:{start_html:010}\r\nEndHTML:{end_html:010}\r\nStartFragment:{start:010}\r\nEndFragment:{end:010}\r\n"
        );
        format!("{head}{before}{fragment}{after}").into_bytes()
    }

    #[test]
    fn cf_html_fragments() {
        assert_eq!(
            cf_html_fragment(&cf_html("<b>bold</b> é")).as_deref(),
            Some("<b>bold</b> é")
        );
        // Offsets that don't fit: the comments mark it.
        let broken = b"StartFragment:999999\r\n<!--StartFragment--><i>x</i><!--EndFragment-->";
        assert_eq!(cf_html_fragment(broken).as_deref(), Some("<i>x</i>"));
        assert_eq!(cf_html_fragment(b"nothing here"), None);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Round trip: the fragment written into CF_HTML is read back.
        #[test]
        fn prop_cf_html_round_trips(fragment in "<p>[a-zé <>/]{1,40}</p>") {
            prop_assert_eq!(cf_html_fragment(&cf_html(&fragment)), Some(fragment));
        }
    }

    /// A12: the key copied a moment before quitting is cleared at quit, once;
    /// one replaced since, or already cleared by its minute, is left alone.
    #[test]
    fn quitting_clears_the_last_secret_copied() {
        let mut last = Some(7);
        assert!(
            should_clear(&mut last, None, Some(7)),
            "still on the clipboard"
        );
        assert_eq!(last, None);
        assert!(!should_clear(&mut last, None, Some(7)), "only once");

        let mut last = Some(7);
        assert!(!should_clear(&mut last, None, Some(8)), "copied over since");
        assert_eq!(last, None);

        // A second key copied: the first one's minute leaves it alone.
        let mut last = Some(9);
        assert!(!should_clear(&mut last, Some(7), Some(9)));
        assert_eq!(last, Some(9), "still cleared at quit");
        assert!(should_clear(&mut last, Some(9), Some(9)), "its own minute");
        assert!(
            !should_clear(&mut last, None, Some(9)),
            "then nothing at quit"
        );

        let mut last = Some(7);
        assert!(!should_clear(&mut last, None, None), "no clipboard to read");
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// The secret is cleared only while nothing else was copied over it.
        #[test]
        fn prop_only_our_copy_is_cleared(copied in any::<isize>(), later in 0isize..3) {
            let now = copied.wrapping_add(later);
            prop_assert_eq!(still_ours(copied, now), later == 0);
        }

        /// Only the last secret is cleared, only while the clipboard holds
        /// it, and at most once: its own timer or the quit forgets it, an
        /// older copy's timer leaves it.
        #[test]
        fn prop_the_last_secret_is_cleared_at_most_once(
            last in proptest::option::of(0isize..4),
            timer in proptest::option::of(0isize..4),
            now in proptest::option::of(0isize..4),
        ) {
            let mut slot = last;
            let cleared = should_clear(&mut slot, timer, now);
            let due = last.is_some() && (timer.is_none() || timer == last);
            prop_assert_eq!(cleared, due && now == last);
            prop_assert_eq!(slot, if due { None } else { last });
            prop_assert!(!should_clear(&mut slot, timer, now), "at most once");
        }
    }
}

//! Experimental phone UI for Kanso on `gpui-pre-mobile` (docs/MOBILE-SPIKE.md).
//!
//! A note list, then a note with the gpui-base editor or the Markdown preview,
//! over a `kanso-store` database that lives in the app's own container.

mod phone;

#[cfg(target_os = "ios")]
mod ios {
    use gpui::{App, WindowOptions};
    use gpui_kit::prelude::*;

    use crate::phone::Phone;

    struct NsLogLogger;

    impl log::Log for NsLogLogger {
        fn enabled(&self, _: &log::Metadata) -> bool {
            true
        }
        fn log(&self, record: &log::Record) {
            nslog(&format!("[{}] {}: {}", record.level(), record.target(), record.args()));
        }
        fn flush(&self) {}
    }

    fn nslog(msg: &str) {
        use objc2::runtime::AnyObject;
        use objc2::{class, msg_send};
        unsafe extern "C" {
            fn NSLog(fmt: *mut AnyObject, ...);
        }
        unsafe {
            let c_msg = std::ffi::CString::new(msg).unwrap_or_default();
            let ns_msg: *mut AnyObject = msg_send![class!(NSString), alloc];
            let ns_msg: *mut AnyObject = msg_send![ns_msg, initWithUTF8String: c_msg.as_ptr()];
            let c_fmt = std::ffi::CString::new("%@").unwrap_or_default();
            let ns_fmt: *mut AnyObject = msg_send![class!(NSString), alloc];
            let ns_fmt: *mut AnyObject = msg_send![ns_fmt, initWithUTF8String: c_fmt.as_ptr()];
            NSLog(ns_fmt, ns_msg);
        }
    }

    /// Called from `main.m` before the GPUI run loop starts.
    #[unsafe(no_mangle)]
    pub extern "C" fn gpui_ios_register_app() {
        let _ = log::set_logger(&NsLogLogger).map(|()| log::set_max_level(log::LevelFilter::Info));
        std::panic::set_hook(Box::new(|info| nslog(&format!("KANSO PANIC: {info}"))));
        let started = std::time::Instant::now();

        gpui_mobile::ios::ffi::set_app_callback(Box::new(move |cx: &mut App| {
            gpui_kit::init(cx);
            let dark = std::env::var("KANSO_SPIKE_DARK").is_ok();
            gpui_kit::component::Theme::change(
                if dark {
                    gpui_kit::component::ThemeMode::Dark
                } else {
                    gpui_kit::component::ThemeMode::Light
                },
                None,
                cx,
            );
            cx.open_window(WindowOptions::default(), |window, cx| {
                let phone = cx.new(|cx| Phone::new(window, cx));
                cx.new(|cx| gpui_kit::component::Root::new(phone, window, cx))
            })
            .expect("open the Kanso window");
            cx.activate(true);
            log::info!("kanso-mobile: window open {} ms after the app callback", started.elapsed().as_millis());
        }));
    }
}

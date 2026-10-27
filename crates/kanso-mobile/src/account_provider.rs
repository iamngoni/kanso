//! The phone's side of a provider sign-in (docs/ACCOUNTS.md › Sign in with
//! Apple, Google and GitHub): the engine's `Browser`, `Listener` and
//! `AppleWait` seams, filled in with events for the shell.
//!
//! The engine decides everything about the flow: the routes, the polling,
//! the deadlines. What the phone adds is where the system's pages open. The
//! shell opens them ([`ProviderEvent::OpenPage`]) in the platform's sign-in
//! session; nothing here opens a web view or touches the network itself.
//!
//! - Apple: the page ends at `kanso://auth/apple?flow=…&s=…`, which the shell
//!   hands back with `AccountHandle::provider_page_ended`.
//! - Google: Kanso listens on a loopback address of its own
//!   ([`crate::provider_loopback`]), so the shell only opens the page.
//! - GitHub: the device flow; the shell shows the code next to the page.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use kanso_engine::account::provider::{self as flow, ApplePoll, AppleWait};
use kanso_sync::provider::{GithubStart, GoogleStart, Provider};
use kanso_sync::{AccountError, AppleStart};

use crate::account_model::{ProviderEvent, ProviderKind};
use crate::account_runtime::{Out, ProviderRun};
use crate::boundary::held;
use crate::provider_loopback::{Loopback, page_url};

/// The scheme Apple's page ends at.
pub(crate) const APPLE_SCHEME: &str = "kanso";

/// Which provider the shell means.
pub(crate) fn provider_of(kind: ProviderKind) -> Provider {
    match kind {
        ProviderKind::Apple => Provider::Apple,
        ProviderKind::Google => Provider::Google,
        ProviderKind::Github => Provider::Github,
    }
}

/// The shell's name for a provider, if it knows it.
pub(crate) fn kind_of(provider: &Provider) -> Option<ProviderKind> {
    match provider {
        Provider::Apple => Some(ProviderKind::Apple),
        Provider::Google => Some(ProviderKind::Google),
        Provider::Github => Some(ProviderKind::Github),
        Provider::Other(_) => None,
    }
}

/// A flow's events on their way to the shell, and the flow's cancel flag:
/// a listener that goes away ends the flow.
pub(crate) struct Events {
    out: Out<ProviderEvent>,
    /// Whether a page is open at the shell.
    open: AtomicBool,
    pub cancel: Arc<AtomicBool>,
}

impl Events {
    pub fn new(
        sink: impl FnMut(ProviderEvent) -> bool + Send + 'static,
        cancel: Arc<AtomicBool>,
    ) -> Arc<Events> {
        Arc::new(Events {
            out: Out::new(sink),
            open: AtomicBool::new(false),
            cancel,
        })
    }

    pub fn emit(&self, event: ProviderEvent) {
        if !self.out.emit(event) {
            self.cancel.store(true, Ordering::Release);
        }
    }

    /// Asks the shell to open `url` (checked once more here) in the system's
    /// sign-in session. `scheme` is where the session ends, for Apple.
    pub fn open_page(&self, url: &str, scheme: Option<&str>) -> Result<(), ()> {
        let url = page_url(url)?;
        self.open.store(true, Ordering::Release);
        self.emit(ProviderEvent::OpenPage {
            url,
            callback_scheme: scheme.map(str::to_owned),
        });
        Ok(())
    }

    /// Tells the shell to close the page, if one is open.
    pub fn close_page(&self) {
        if self.open.swap(false, Ordering::AcqRel) {
            self.emit(ProviderEvent::ClosePage);
        }
    }
}

/// The engine's `Browser` for the phone: pages are events, and Google's
/// redirect is a loopback listener in this process.
pub(crate) struct PhoneBrowser {
    pub events: Arc<Events>,
}

impl flow::Browser for PhoneBrowser {
    fn listen(&self) -> Result<Box<dyn flow::Listener>, flow::BrowserError> {
        Ok(Box::new(PhoneListener {
            loopback: Loopback::bind()?,
            events: self.events.clone(),
        }))
    }

    fn open_google(&self, start: &GoogleStart) -> Result<(), flow::BrowserError> {
        self.events
            .open_page(&start.auth_url, None)
            .map_err(|()| flow::BrowserError::Failed)
    }

    fn open_github(&self, start: &GithubStart) -> Result<(), flow::BrowserError> {
        self.events.emit(ProviderEvent::ShowCode {
            user_code: start.user_code.clone(),
            verification_uri: start.verification_uri.clone(),
        });
        self.events
            .open_page(&start.verification_uri, None)
            .map_err(|()| flow::BrowserError::Failed)
    }
}

/// Google's loopback, closing the shell's page as soon as the code is in.
struct PhoneListener {
    loopback: Loopback,
    events: Arc<Events>,
}

impl flow::Listener for PhoneListener {
    fn uri(&self) -> String {
        self.loopback.uri()
    }

    fn wait_until(
        &self,
        state: &str,
        deadline: Instant,
        cancel: &AtomicBool,
    ) -> Result<String, flow::BrowserError> {
        let code = self.loopback.wait_until(state, deadline, cancel)?;
        self.events.close_page();
        Ok(code)
    }
}

/// Apple's callback URL as the engine's flow waits for it: it comes in
/// through `AccountHandle::provider_page_ended`.
pub(crate) struct ApplePhone(Receiver<String>);

impl AppleWait for ApplePhone {
    fn poll(&self) -> ApplePoll {
        match self.0.try_recv() {
            Ok(url) => ApplePoll::Answer(Ok(url)),
            Err(TryRecvError::Empty) => ApplePoll::Waiting,
            Err(TryRecvError::Disconnected) => ApplePoll::Closed,
        }
    }
}

/// Opens Apple's page at the shell, with the callback ready to be handed
/// back before it can arrive.
pub(crate) fn open_apple(
    events: &Events,
    run: &Mutex<Option<ProviderRun>>,
    start: &AppleStart,
) -> Result<Box<dyn AppleWait>, AccountError> {
    let (sender, receiver): (Sender<String>, Receiver<String>) = std::sync::mpsc::channel();
    if let Some(run) = held(run).as_mut() {
        run.apple = Some(sender);
    }
    events
        .open_page(&start.auth_url, Some(APPLE_SCHEME))
        .map_err(|()| AccountError::ProviderUnavailable)?;
    Ok(Box::new(ApplePhone(receiver)))
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use proptest::prelude::*;

    use super::*;

    fn collecting() -> (Arc<Mutex<Vec<ProviderEvent>>>, Arc<Events>) {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let log = seen.clone();
        let events = Events::new(
            move |event| {
                log.lock().unwrap().push(event);
                true
            },
            Arc::new(AtomicBool::new(false)),
        );
        (seen, events)
    }

    proptest! {
        /// A page opens only if it is https or this device, and is closed
        /// once however many times the flow asks.
        #[test]
        fn prop_pages_open_checked_and_close_once(
            host in "[a-z]{1,12}",
            closes in 1usize..5,
        ) {
            let (seen, events) = collecting();
            let url = format!("https://{host}.example/login");
            prop_assert!(events.open_page(&url, None).is_ok());
            let plain = format!("http://{host}.example/");
            prop_assert!(events.open_page(&plain, None).is_err());
            prop_assert!(events.open_page("kanso://auth/apple?flow=x", None).is_err());
            for _ in 0..closes {
                events.close_page();
            }
            let seen = seen.lock().unwrap();
            prop_assert_eq!(seen.len(), 2);
            prop_assert_eq!(
                &seen[0],
                &ProviderEvent::OpenPage { url, callback_scheme: None }
            );
            prop_assert_eq!(&seen[1], &ProviderEvent::ClosePage);
        }

        /// The provider names map both ways, and an unknown one has no name.
        #[test]
        fn prop_provider_names_round_trip(ix in 0usize..3, other in "[a-z]{1,10}") {
            let kind = [ProviderKind::Apple, ProviderKind::Google, ProviderKind::Github][ix];
            prop_assert_eq!(kind_of(&provider_of(kind)), Some(kind));
            prop_assert_eq!(kind_of(&Provider::Other(other)), None);
        }
    }

    // no-prop-test: a listener that leaves cancels the flow it feeds.
    #[test]
    fn a_listener_that_leaves_cancels_the_flow() {
        let cancel = Arc::new(AtomicBool::new(false));
        let events = Events::new(|_| false, cancel.clone());
        events.emit(ProviderEvent::Cancelled);
        assert!(cancel.load(Ordering::Acquire));
    }

    // no-prop-test: the callback channel's three states.
    #[test]
    fn apple_waits_until_the_shell_hands_the_callback_in() {
        let (sender, receiver) = std::sync::mpsc::channel();
        let wait = ApplePhone(receiver);
        assert!(matches!(wait.poll(), ApplePoll::Waiting));
        sender
            .send("kanso://auth/apple?flow=a&s=b".to_owned())
            .unwrap();
        assert!(matches!(
            wait.poll(),
            ApplePoll::Answer(Ok(url)) if url.starts_with("kanso://")
        ));
        drop(sender);
        assert!(matches!(wait.poll(), ApplePoll::Closed));
    }
}

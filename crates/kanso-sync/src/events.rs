//! The account event socket (docs/ACCOUNTS.md › GET /v2/account/events): a
//! WebSocket that tells a signed-in, unlocked device about approval
//! requests, so a new device's request shows without a timer.
//!
//! Events are hints, never state: a frame carries only its type and an
//! approval ID, and the app fetches the approvals list over HTTPS after each
//! one and after every connect. A frame that is late, lost or never sent is
//! caught up by that GET, which also runs at launch, when Settings › Account
//! opens and on foreground. Nothing here polls on a timer: the loop waits on
//! the socket, sends a `ping` only after a minute of silence (answered by
//! the Durable Object's auto-response, which doesn't wake it), and
//! reconnects with bounded exponential backoff.
//!
//! [`EventStream`] is the socket over `tungstenite`/`rustls`
//! ([`crate::ws`]); [`run`] is the reconnect loop, over any
//! [`EventSource`], so tests drive it with fakes and `MemoryServer`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde_json::Value;
use tungstenite::Message;
use tungstenite::protocol::WebSocket;
use tungstenite::protocol::frame::coding::CloseCode;

use crate::wire::ApiError;
use crate::ws::{Stream, WsError};

/// The path an older server's health doesn't advertise but this one does.
pub const ACCOUNT_EVENTS_PATH: &str = "/v2/account/events";

/// How long one read waits before the loop checks whether it should stop.
pub const READ_WAIT: Duration = Duration::from_secs(1);

/// How long the socket may be silent before the client sends a `ping`, so a
/// NAT or proxy doesn't drop an idle connection. The Durable Object answers
/// it with its auto-response, without waking.
pub const IDLE_PING: Duration = Duration::from_secs(60);

/// The first and the longest wait between reconnects.
pub const BACKOFF_FIRST: Duration = Duration::from_secs(1);
pub const BACKOFF_MAX: Duration = Duration::from_secs(60);

/// The close code the server sends a socket that a newer one of the same
/// device replaced.
pub const CLOSE_REPLACED: u16 = 4000;

/// An event about an approval request: its ID, nothing else.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccountEvent {
    /// A device asked to be approved.
    ApprovalCreated(String),
    /// The requester posted its key and sealed name: the code can be shown.
    ApprovalNamed(String),
    /// Either side cancelled it.
    ApprovalCancelled(String),
}

impl AccountEvent {
    /// The approval request the event is about.
    pub fn id(&self) -> &str {
        match self {
            AccountEvent::ApprovalCreated(id)
            | AccountEvent::ApprovalNamed(id)
            | AccountEvent::ApprovalCancelled(id) => id,
        }
    }
}

/// One server-to-app frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Frame {
    /// `{"type":"hello"}`, the first frame of a connection.
    Hello,
    Event(AccountEvent),
    /// A type this client doesn't know: ignored.
    Unknown,
}

/// Reads a text frame: `{"type": …, "id": …}`. `None` for a frame that isn't
/// a JSON object with a string `type`, or an approval event without a
/// 43-character ID; unknown types are [`Frame::Unknown`].
pub fn parse_frame(text: &str) -> Option<Frame> {
    let Ok(Value::Object(frame)) = serde_json::from_str::<Value>(text) else {
        return None;
    };
    let kind = frame.get("type")?.as_str()?;
    let id = || {
        frame
            .get("id")
            .and_then(Value::as_str)
            .filter(|id| crate::wire::is_approval_id(id))
            .map(str::to_owned)
    };
    Some(match kind {
        "hello" => Frame::Hello,
        "approval.created" => Frame::Event(AccountEvent::ApprovalCreated(id()?)),
        "approval.named" => Frame::Event(AccountEvent::ApprovalNamed(id()?)),
        "approval.cancelled" => Frame::Event(AccountEvent::ApprovalCancelled(id()?)),
        _ => Frame::Unknown,
    })
}

/// Why a socket ended or never opened.
#[derive(Debug)]
pub enum EventError {
    /// The server refused the upgrade with this answer.
    Refused(ApiError),
    /// A newer socket of this device took over (close code 4000): another
    /// copy of the app holds it, so this one stops instead of taking it back.
    Replaced,
    /// The connection failed or dropped; the loop reconnects.
    Dropped(String),
}

impl EventError {
    /// Whether reconnecting can't help: the session ended or was refused,
    /// the server has no event socket, or another copy took over.
    pub fn is_final(&self) -> bool {
        match self {
            EventError::Replaced => true,
            EventError::Dropped(_) => false,
            EventError::Refused(err) => matches!(
                err,
                ApiError::Unauthorized { .. }
                    | ApiError::InvalidCredentials
                    | ApiError::AccountDisabled
                    | ApiError::ForbiddenOrigin
                    | ApiError::NotFound
                    | ApiError::MethodNotAllowed
            ),
        }
    }
}

/// An open event socket.
pub trait EventSource: Send {
    /// The next frame, waiting at most `wait`; `Ok(None)` when none came.
    fn next(&mut self, wait: Duration) -> Result<Option<Frame>, EventError>;
    /// Sends the keep-alive `ping`.
    fn ping(&mut self) -> Result<(), EventError>;
}

/// The account event socket over a WebSocket.
pub struct EventStream {
    socket: WebSocket<Stream>,
}

impl EventStream {
    /// Opens the socket: the session's bearer token, the device's ID and
    /// its proof go in the upgrade's headers, which the server checks once.
    pub fn connect(
        server: &str,
        path: &str,
        authorization: &str,
        device_id: &str,
        device_proof: &str,
    ) -> Result<EventStream, EventError> {
        let socket = crate::ws::connect(
            server,
            path,
            &[
                ("authorization", authorization),
                ("x-kanso-device", device_id),
                ("x-kanso-device-proof", device_proof),
            ],
        )
        .map_err(|err| match err {
            WsError::Request(message) => EventError::Refused(ApiError::Protocol(message)),
            WsError::Transport(message) => EventError::Dropped(message),
            WsError::Refused { status, body } => {
                EventError::Refused(ApiError::from_response(status, &body))
            }
        })?;
        Ok(EventStream { socket })
    }
}

impl EventSource for EventStream {
    fn next(&mut self, wait: Duration) -> Result<Option<Frame>, EventError> {
        self.socket
            .get_ref()
            .tcp()
            .set_read_timeout(Some(wait.max(Duration::from_millis(1))))
            .ok();
        loop {
            match self.socket.read() {
                Ok(Message::Text(text)) => {
                    if let Some(frame) = parse_frame(text.as_str()) {
                        return Ok(Some(frame));
                    }
                }
                Ok(Message::Close(frame)) => {
                    return Err(match frame {
                        Some(frame) if frame.code == CloseCode::from(CLOSE_REPLACED) => {
                            EventError::Replaced
                        }
                        _ => EventError::Dropped("closed".into()),
                    });
                }
                Ok(_) => {}
                Err(tungstenite::Error::Io(err))
                    if matches!(
                        err.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    return Ok(None);
                }
                Err(err) => return Err(EventError::Dropped(err.to_string())),
            }
        }
    }

    fn ping(&mut self) -> Result<(), EventError> {
        self.socket
            .send(Message::text("ping"))
            .map_err(|err| EventError::Dropped(err.to_string()))
    }
}

impl Drop for EventStream {
    fn drop(&mut self) {
        let _ = self.socket.close(None);
        let _ = self.socket.flush();
    }
}

/// What the loop tells its owner.
#[derive(Debug)]
pub enum Signal {
    /// A socket opened: fetch the approvals list once, to catch up on
    /// anything missed while it was down.
    Connected,
    /// An event: fetch the approvals list once.
    Event(AccountEvent),
    /// The loop stopped for good ([`EventError::is_final`]).
    Ended(EventError),
}

/// Bounded exponential backoff: 1 s, 2 s, 4 s … up to a minute, back to 1 s
/// after a connection that opened.
#[derive(Clone, Copy, Debug)]
pub struct Backoff {
    next: Duration,
}

impl Default for Backoff {
    fn default() -> Backoff {
        Backoff {
            next: BACKOFF_FIRST,
        }
    }
}

impl Backoff {
    /// The wait before the next attempt; the one after is twice as long.
    pub fn next_wait(&mut self) -> Duration {
        let wait = self.next;
        self.next = (self.next * 2).min(BACKOFF_MAX);
        wait
    }

    pub fn reset(&mut self) {
        self.next = BACKOFF_FIRST;
    }
}

/// Keeps the event socket open until `stop` is set or the loop ends for good:
/// connects with `connect`, reports [`Signal::Connected`] and every event,
/// pings after [`IDLE_PING`] of silence, and after a drop waits
/// ([`Backoff`], or a 429's `retry_after` when longer) through `sleep` and
/// connects again. `sleep` is how the loop waits; it should return early
/// once `stop` is set (the app's checks the flag in short slices).
pub fn run(
    connect: &mut dyn FnMut() -> Result<Box<dyn EventSource>, EventError>,
    stop: &AtomicBool,
    on: &mut dyn FnMut(Signal),
    sleep: &dyn Fn(Duration),
) {
    let mut backoff = Backoff::default();
    while !stop.load(Ordering::SeqCst) {
        let err = match connect() {
            Ok(mut source) => {
                backoff.reset();
                on(Signal::Connected);
                let mut idle = Duration::ZERO;
                loop {
                    if stop.load(Ordering::SeqCst) {
                        return;
                    }
                    match source.next(READ_WAIT) {
                        Ok(Some(Frame::Event(event))) => {
                            idle = Duration::ZERO;
                            on(Signal::Event(event));
                        }
                        Ok(Some(Frame::Hello | Frame::Unknown)) => idle = Duration::ZERO,
                        Ok(None) => {
                            idle += READ_WAIT;
                            if idle >= IDLE_PING {
                                idle = Duration::ZERO;
                                if let Err(err) = source.ping() {
                                    break err;
                                }
                            }
                        }
                        Err(err) => break err,
                    }
                }
            }
            Err(err) => err,
        };
        if err.is_final() {
            on(Signal::Ended(err));
            return;
        }
        let mut wait = backoff.next_wait();
        if let EventError::Refused(ApiError::RateLimited { retry_after, .. }) = &err {
            wait = wait.max(Duration::from_secs(*retry_after));
        }
        if stop.load(Ordering::SeqCst) {
            return;
        }
        sleep(wait);
    }
}

/// Sleeps for `wait` in slices of at most 250 ms, returning early once
/// `stop` is set: the app's `sleep` for [`run`].
pub fn sleep_unless_stopped(stop: &AtomicBool, wait: Duration) {
    let slice = Duration::from_millis(250);
    let mut left = wait;
    while !left.is_zero() && !stop.load(Ordering::SeqCst) {
        let step = left.min(slice);
        std::thread::sleep(step);
        left -= step;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

    #[test]
    fn frames() {
        assert_eq!(parse_frame(r#"{"type":"hello"}"#), Some(Frame::Hello));
        assert_eq!(
            parse_frame(&format!(r#"{{"type":"approval.created","id":"{ID}"}}"#)),
            Some(Frame::Event(AccountEvent::ApprovalCreated(ID.into())))
        );
        assert_eq!(
            parse_frame(&format!(r#"{{"type":"approval.named","id":"{ID}","x":1}}"#)),
            Some(Frame::Event(AccountEvent::ApprovalNamed(ID.into())))
        );
        assert_eq!(
            parse_frame(&format!(r#"{{"type":"approval.cancelled","id":"{ID}"}}"#)),
            Some(Frame::Event(AccountEvent::ApprovalCancelled(ID.into())))
        );
        assert_eq!(
            parse_frame(r#"{"type":"approval.created","id":"short"}"#),
            None
        );
        assert_eq!(
            parse_frame(r#"{"type":"something.new"}"#),
            Some(Frame::Unknown)
        );
        assert_eq!(parse_frame("pong"), None);
        assert_eq!(parse_frame("[1]"), None);
    }

    #[test]
    fn backoff_doubles_to_a_minute_and_resets() {
        let mut backoff = Backoff::default();
        let waits: Vec<u64> = (0..9).map(|_| backoff.next_wait().as_secs()).collect();
        assert_eq!(waits, [1, 2, 4, 8, 16, 32, 60, 60, 60]);
        backoff.reset();
        assert_eq!(backoff.next_wait(), BACKOFF_FIRST);
    }

    struct Fake {
        frames: Vec<Option<Result<Option<Frame>, EventError>>>,
        pings: usize,
    }

    impl Fake {
        fn new(frames: Vec<Option<Result<Option<Frame>, EventError>>>) -> Fake {
            Fake { frames, pings: 0 }
        }
    }

    impl EventSource for Fake {
        fn next(&mut self, _wait: Duration) -> Result<Option<Frame>, EventError> {
            match self.frames.first_mut().map(Option::take) {
                Some(Some(result)) => {
                    self.frames.remove(0);
                    result
                }
                Some(None) => {
                    self.frames.remove(0);
                    Ok(None)
                }
                None => Ok(None),
            }
        }

        fn ping(&mut self) -> Result<(), EventError> {
            self.pings += 1;
            Ok(())
        }
    }

    #[test]
    fn a_drop_reconnects_with_backoff_and_each_connection_catches_up() {
        let stop = AtomicBool::new(false);
        let mut connects = 0;
        let mut signals = Vec::new();
        let waits = std::cell::RefCell::new(Vec::new());
        let mut connect = || {
            connects += 1;
            Ok(Box::new(Fake::new(match connects {
                1 => vec![Some(Err(EventError::Dropped("cut".into())))],
                2 => vec![
                    Some(Ok(Some(Frame::Hello))),
                    Some(Err(EventError::Dropped("cut again".into()))),
                ],
                _ => vec![Some(Ok(Some(Frame::Event(AccountEvent::ApprovalCreated(
                    ID.into(),
                )))))],
            })) as Box<dyn EventSource>)
        };
        let mut on = |signal: Signal| {
            if let Signal::Event(_) = &signal {
                stop.store(true, Ordering::SeqCst);
            }
            signals.push(signal);
        };
        let sleep = |wait: Duration| waits.borrow_mut().push(wait);
        run(&mut connect, &stop, &mut on, &sleep);

        assert_eq!(connects, 3, "two drops or backs off, then the event");
        assert!(
            matches!(signals.first(), Some(Signal::Connected)),
            "the first thing is a connect"
        );
        assert_eq!(
            signals
                .iter()
                .filter(|s| matches!(s, Signal::Connected))
                .count(),
            3,
            "every connect asks for one approvals GET"
        );
        assert!(signals.iter().any(|s| matches!(
            s,
            Signal::Event(AccountEvent::ApprovalCreated(id)) if id == ID
        )));
        assert_eq!(*waits.borrow(), [BACKOFF_FIRST, BACKOFF_FIRST]);
    }

    #[test]
    fn a_replaced_socket_stops_for_good() {
        let stop = AtomicBool::new(false);
        let mut connects = 0;
        let mut signals = Vec::new();
        let mut connect = || {
            connects += 1;
            Ok(Box::new(Fake::new(vec![Some(Err(EventError::Replaced))])) as Box<dyn EventSource>)
        };
        let mut on = |signal: Signal| signals.push(signal);
        run(&mut connect, &stop, &mut on, &|_| {});
        assert_eq!(connects, 1);
        assert!(matches!(
            signals.last(),
            Some(Signal::Ended(EventError::Replaced))
        ));
    }

    #[test]
    fn a_429_waits_at_least_its_retry_after() {
        let stop = AtomicBool::new(false);
        let mut connects = 0;
        let waits = std::cell::RefCell::new(Vec::new());
        let mut signals = Vec::new();
        let mut connect = || {
            connects += 1;
            if connects == 1 {
                Err(EventError::Refused(ApiError::RateLimited {
                    retry_after: 45,
                    limit: "session-account".into(),
                }))
            } else {
                Ok(Box::new(Fake::new(vec![])) as Box<dyn EventSource>)
            }
        };
        let mut on = |signal: Signal| {
            if matches!(signal, Signal::Connected) {
                stop.store(true, Ordering::SeqCst);
            }
            signals.push(signal);
        };
        let sleep = |wait: Duration| waits.borrow_mut().push(wait);
        run(&mut connect, &stop, &mut on, &sleep);
        assert_eq!(*waits.borrow(), [Duration::from_secs(45)]);
    }

    #[test]
    fn final_errors() {
        assert!(EventError::Replaced.is_final());
        assert!(!EventError::Dropped("x".into()).is_final());
        assert!(EventError::Refused(ApiError::InvalidCredentials).is_final());
        assert!(EventError::Refused(ApiError::NotFound).is_final());
        assert!(!EventError::Refused(ApiError::Internal).is_final());
        assert!(
            !EventError::Refused(ApiError::RateLimited {
                retry_after: 30,
                limit: "session-account".into()
            })
            .is_final()
        );
    }
}

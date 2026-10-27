//! A second device of the account, driven with `kanso_sync`'s calls for the
//! device asking to join, for the tests of approving it and of the account
//! page. And a server whose event socket the test controls.
#![allow(dead_code)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use kanso_engine::account::{Flow, Next, after_auth};
use kanso_mobile::{Library, Secrets};
use kanso_store::Store;
use kanso_sync::api::LoginDevice;
use kanso_sync::events::Frame;
use kanso_sync::{
    Api, Approval, ApprovalKeys, ApprovalRequest, Backend, DEFAULT_SERVER, DeviceIdentity,
    EventError, EventSource, KdfParams, LockedSession, Machine, MemoryServer, Password,
    RequestState, Session, SessionToken, device_identity,
};

use crate::common::*;

/// A device that has signed in to the account but not unlocked it: its own
/// library, the locked session a requester holds, and the request it made.
pub struct NewDevice {
    pub phone: Phone,
    pub name: String,
    pub device: DeviceIdentity,
    pub locked: LockedSession,
    pub keys: ApprovalKeys,
    pub request: Option<ApprovalRequest>,
    /// The code this device shows once a device has answered.
    pub approval: Option<Approval>,
    server: Arc<MemoryServer>,
}

pub fn token_of(session: &Session) -> SessionToken {
    let header = session.authorization();
    SessionToken::new(header.trim_start_matches("Bearer ").to_owned())
}

/// A locked view of `session`: the token and key ID, not the data key.
pub fn locked_from(session: &Session) -> LockedSession {
    LockedSession::new(
        session.server.clone(),
        session.origin.clone(),
        session.account_id.clone(),
        session.username.clone(),
        session.device_id.clone(),
        token_of(session),
        session.key_id(),
        "0000000000000000".to_owned(),
        None,
        None,
    )
}

fn machine(name: &str) -> Machine {
    Machine {
        hardware_uuid: None,
        computer_name: name.to_owned(),
    }
}

/// A new library that has signed in to the account by password with the
/// identity of its own library, and holds only the locked session.
pub fn new_device(server: &Arc<MemoryServer>, name: &str) -> NewDevice {
    let phone = phone(server);
    let device = {
        let store = Store::open(phone.path()).unwrap();
        device_identity(&store, &machine(name)).unwrap()
    };
    let password = Password::new(ACCOUNT_PASSWORD, ACCOUNT).unwrap();
    let signed = kanso_sync::sign_in(
        server.as_ref(),
        DEFAULT_SERVER,
        ACCOUNT,
        &password,
        &device,
        &|_| {},
    )
    .unwrap();
    let mut locked = locked_from(&signed.session).with_login_device(LoginDevice::for_identity(
        &device,
        &signed.session.origin,
        &signed.session.account_id,
    ));
    locked.set_library_id(&device.library_id);
    NewDevice {
        phone,
        name: name.to_owned(),
        device,
        locked,
        keys: ApprovalKeys::generate(),
        request: None,
        approval: None,
        server: server.clone(),
    }
}

impl NewDevice {
    /// Asks the account's other devices to approve this one.
    pub fn ask(&mut self) -> ApprovalRequest {
        let request =
            kanso_sync::request_approval(self.server.as_ref(), &self.locked, &self.keys).unwrap();
        self.request = Some(request.clone());
        request
    }

    pub fn state(&self) -> RequestState {
        let request = self.request.as_ref().unwrap();
        kanso_sync::poll_approval(self.server.as_ref(), &self.locked, request).unwrap()
    }

    /// Once a device has answered: works out the code and names this
    /// device to it. `None` while no device has answered.
    pub fn name_to_approver(&mut self) -> Option<String> {
        let request = self.request.clone().unwrap();
        let RequestState::Challenged { approver_epk, .. } = self.state() else {
            return None;
        };
        let approval =
            kanso_sync::complete_request(&self.locked, &request, &self.keys, &approver_epk)
                .unwrap();
        kanso_sync::name_device(
            self.server.as_ref(),
            &self.locked,
            &request,
            &approval,
            &self.device,
        )
        .unwrap();
        let code = approval.code();
        self.approval = Some(approval);
        Some(code)
    }

    /// After an approval: opens the data key, binds the library to the
    /// account and saves the session, as unlocking does.
    pub fn unlock(&self) {
        let RequestState::Approved(approved) = self.state() else {
            panic!("the request should be approved, got {:?}", self.state());
        };
        let approval = self.approval.as_ref().unwrap();
        let signed = kanso_sync::finish_unlock(
            self.server.as_ref(),
            &self.locked,
            approval,
            &approved,
            &self.device,
        )
        .unwrap();
        let flow = Flow {
            backend: self.server.clone(),
            secrets: self.phone.secrets.clone() as Arc<dyn Secrets>,
            store: Arc::new(Store::open(self.phone.path()).unwrap()),
            machine: Some(machine(&self.name)),
            backups: self.phone.dir.path().join("backups"),
            now_ms: kanso_core::now_ms(),
        };
        let auth = flow
            .authenticated(self.device.library_id.clone(), signed.session, None, None)
            .unwrap();
        let next = after_auth(
            kanso_sync::Flow::SignIn,
            &auth.library,
            &auth.session.username,
            false,
        );
        let Next::Bind(choice) = next else {
            panic!("a fresh library binds without a question, got {next:?}");
        };
        flow.bind_and_save(auth.session, auth.library_id, choice, None)
            .unwrap();
    }
}

/// A server whose account event socket the test drives: every socket the
/// library opens is a channel, and the test sends the frames.
pub struct Evented {
    pub server: Arc<MemoryServer>,
    sockets: Mutex<Vec<mpsc::Sender<Frame>>>,
    connects: AtomicUsize,
}

impl Evented {
    pub fn new(server: &Arc<MemoryServer>) -> Arc<Evented> {
        Arc::new(Evented {
            server: server.clone(),
            sockets: Mutex::new(Vec::new()),
            connects: AtomicUsize::new(0),
        })
    }

    /// How many sockets have been opened.
    pub fn connects(&self) -> usize {
        self.connects.load(Ordering::SeqCst)
    }

    /// Sends `frame` on every open socket. `false` when no socket is open
    /// any more: the library's thread let go of it.
    pub fn send(&self, frame: Frame) -> bool {
        let mut sockets = self.sockets.lock().unwrap();
        sockets.retain(|socket| socket.send(frame.clone()).is_ok());
        !sockets.is_empty()
    }

    /// Drops every socket, as a network that went away does.
    pub fn drop_sockets(&self) {
        self.sockets.lock().unwrap().clear();
    }

    /// Whether a socket is open now (its thread still holds the receiver).
    pub fn open(&self) -> bool {
        let mut sockets = self.sockets.lock().unwrap();
        sockets.retain(|socket| socket.send(Frame::Unknown).is_ok());
        !sockets.is_empty()
    }
}

struct Socket {
    frames: mpsc::Receiver<Frame>,
}

impl EventSource for Socket {
    fn next(&mut self, wait: Duration) -> Result<Option<Frame>, EventError> {
        match self.frames.recv_timeout(wait) {
            Ok(frame) => Ok(Some(frame)),
            Err(RecvTimeoutError::Timeout) => Ok(None),
            Err(RecvTimeoutError::Disconnected) => Err(EventError::Dropped("closed".into())),
        }
    }

    fn ping(&mut self) -> Result<(), EventError> {
        Ok(())
    }
}

impl Backend for Evented {
    fn api(&self, server: &str) -> Box<dyn Api + '_> {
        self.server.api(server)
    }

    fn kdf_floor(&self) -> KdfParams {
        self.server.kdf_floor()
    }

    fn kdf_default(&self) -> KdfParams {
        self.server.kdf_default()
    }

    fn retry_pause(&self, retry: usize) -> Duration {
        self.server.retry_pause(retry)
    }

    fn events(
        &self,
        _session: &Session,
        _device: &DeviceIdentity,
        _path: &str,
    ) -> Result<Box<dyn EventSource>, EventError> {
        let (tx, frames) = mpsc::channel();
        self.sockets.lock().unwrap().push(tx);
        self.connects.fetch_add(1, Ordering::SeqCst);
        Ok(Box::new(Socket { frames }))
    }
}

/// A library on its own folder whose server is `backend`.
pub fn phone_on(backend: Arc<Evented>) -> Phone {
    let dir = tempfile::tempdir().unwrap();
    let secrets = Arc::new(kanso_mobile::MemorySecrets::new());
    let library =
        Library::open_for_test(&dir.path().join("library.db"), secrets.clone(), backend).unwrap();
    Phone {
        dir,
        library,
        secrets,
    }
}

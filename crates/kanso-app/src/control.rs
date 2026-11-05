//! The local control channel (issue #243): how `kanso api-key create | list |
//! revoke` reaches a Kanso or `kanso serve` that is already running with the
//! data folder, so a leaked key can be revoked without stopping anything.
//!
//! The data-folder lock also guards encryption and migrations, so a second
//! process must never write the library while another holds the lock. The
//! command is therefore sent to the process that holds it, and that process
//! does the work with its own `KeyStore`: the key is refused by the very
//! cache that serves requests, and requests it already signed in are cut
//! short (`kanso-server`'s leases).
//!
//! Unix uses a mode-0600 socket in the data folder and checks peer uids at
//! both ends. Windows uses a named pipe whose DACL grants only the current
//! user's SID, and checks the connected process token SID. A key Kanso
//! already trusts cannot use either channel: neither is on the HTTP port.

use std::fmt;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use kanso_server::KeyStore;
use kanso_store::Store;

/// A library's store and the keys served from it.
type Library = (Arc<Store>, KeyStore);

/// The library the control channel works on: the one the running process
/// serves keys from. The app sets it again if the library's store changes.
#[derive(Clone, Default)]
pub struct ControlTarget(Arc<Mutex<Option<Library>>>);

impl ControlTarget {
    pub fn new(store: Arc<Store>, keys: KeyStore) -> ControlTarget {
        ControlTarget(Arc::new(Mutex::new(Some((store, keys)))))
    }

    pub fn set(&self, store: Arc<Store>, keys: KeyStore) {
        *self.0.lock().unwrap_or_else(|p| p.into_inner()) = Some((store, keys));
    }

    fn get(&self) -> Option<Library> {
        self.0.lock().unwrap_or_else(|p| p.into_inner()).clone()
    }
}

/// What a command sent to a running process comes back as.
pub enum Sent {
    /// It ran: standard output, standard error.
    Done(String, String),
    /// It ran and said no.
    Failed(String),
    /// Nothing is listening: the process is an older Kanso or has no
    /// compatible control channel.
    NoSocket,
}

impl fmt::Debug for Sent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // `out` carries the one-time token after `create`.
            Sent::Done(..) => f.write_str("Done([REDACTED])"),
            Sent::Failed(error) => f.debug_tuple("Failed").field(error).finish(),
            Sent::NoSocket => f.write_str("NoSocket"),
        }
    }
}

/// A capped writer for command output sent across the local protocol.
struct LimitedOutput {
    bytes: Vec<u8>,
    limit: usize,
}

impl LimitedOutput {
    fn new(limit: usize) -> LimitedOutput {
        LimitedOutput {
            bytes: Vec::new(),
            limit,
        }
    }

    fn into_string(self) -> String {
        String::from_utf8_lossy(&self.bytes).into_owned()
    }
}

impl Write for LimitedOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.bytes.len().saturating_add(bytes.len()) > self.limit {
            return Err(io::Error::other("the control reply is too large"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn reply_line(reply: &crate::control_model::Reply) -> io::Result<Vec<u8>> {
    use crate::control_model::Reply;

    let mut line = LimitedOutput::new(crate::control_model::MAX_LINE - 1);
    if serde_json::to_writer(&mut line, reply).is_err() {
        line = LimitedOutput::new(crate::control_model::MAX_LINE - 1);
        serde_json::to_writer(
            &mut line,
            &Reply::Failed {
                error: "Kanso's answer was too large. Narrow the request and try again.".into(),
            },
        )
        .map_err(io::Error::other)?;
    }
    line.bytes.push(b'\n');
    Ok(line.bytes)
}

fn answer(
    request: crate::control_model::Request,
    target: &ControlTarget,
) -> crate::control_model::Reply {
    use crate::control_model::Reply;

    let command = match request.command() {
        Ok(command) => command,
        Err(error) => return Reply::Failed { error },
    };
    let Some((store, keys)) = target.get() else {
        return Reply::Failed {
            error: "Kanso hasn't opened its library yet. Try again in a moment.".into(),
        };
    };
    let (mut out, mut err) = (
        LimitedOutput::new(crate::control_model::MAX_LINE / 4),
        LimitedOutput::new(crate::control_model::MAX_LINE / 4),
    );
    match crate::api_key_cli::execute(&store, &keys, command, &mut out, &mut err) {
        Ok(()) => Reply::Done {
            out: out.into_string(),
            err: err.into_string(),
        },
        Err(error) => Reply::Failed {
            error: format!("{error:#}"),
        },
    }
}

#[cfg(unix)]
mod unix {
    use std::io::{BufRead as _, BufReader, Write as _};
    use std::os::fd::{AsRawFd as _, FromRawFd as _, IntoRawFd as _, OwnedFd};
    use std::os::unix::ffi::OsStrExt as _;
    use std::os::unix::fs::{FileTypeExt as _, MetadataExt as _, PermissionsExt as _};
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    #[cfg(test)]
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    use super::{ControlTarget, Sent, answer, reply_line};
    use crate::control_model::{MAX_LINE, Reply, Request};

    /// The most a socket path may hold (`sun_path` is 104 bytes on macOS,
    /// 108 on Linux, with the end).
    const MAX_PATH: usize = 100;

    /// How long a connected client may take to send its command, and how
    /// long a command may take to answer (a revoke waits for requests the
    /// key started, a few seconds at most).
    const CLIENT_WAIT: Duration = Duration::from_secs(5);
    const ANSWER_WAIT: Duration = Duration::from_secs(30);
    /// The data-folder lock serializes separate processes. This small guard
    /// also serializes same-process callers so a just-bound socket is never
    /// mistaken for a stale, not-yet-listening socket.
    static START: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Where the socket for `data_dir` is.
    pub fn socket_path(data_dir: &Path) -> PathBuf {
        data_dir.join("control.sock")
    }

    /// Explains the one pathname-socket limitation at both startup and CLI
    /// send time, so a locked CLI can tell the user how to reach the server.
    fn path_too_long(path: &Path) -> std::io::Error {
        std::io::Error::other(format!(
            "the data folder's path is too long for a socket ({} is over {MAX_PATH} bytes); use a shorter --data-dir",
            path.as_os_str().len()
        ))
    }

    /// Resolves and checks the directory before using a pathname socket.
    /// Every ancestor must be owned by root or this user. Writable ancestors
    /// must also be sticky, so another uid cannot replace any component on
    /// the way to the socket. The data folder itself must belong to Kanso's
    /// user and be private to that user for writes.
    fn secure_data_dir(data_dir: &Path) -> std::io::Result<PathBuf> {
        let canonical = std::fs::canonicalize(data_dir)?;
        // SAFETY: geteuid has no preconditions.
        let uid = unsafe { libc::geteuid() };
        let metadata = std::fs::metadata(&canonical)?;
        if !metadata.is_dir() || metadata.uid() != uid || metadata.mode() & 0o022 != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "the data folder must be owned by this user and not writable by other users",
            ));
        }
        let mut child = canonical.as_path();
        while let Some(parent) = child.parent() {
            let parent_metadata = std::fs::metadata(parent)?;
            let trusted_owner = parent_metadata.uid() == 0 || parent_metadata.uid() == uid;
            if !trusted_owner {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "the data folder has an ancestor owned by another user",
                ));
            }
            if parent_metadata.mode() & 0o022 != 0 {
                let sticky = parent_metadata.mode() & libc::S_ISVTX as u32 != 0;
                let child_owned_by_user = std::fs::metadata(child)?.uid() == uid;
                if !sticky || !child_owned_by_user {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        "the data folder has an ancestor another user can change",
                    ));
                }
            }
            child = parent;
        }
        Ok(canonical)
    }

    /// Binds a socket without listening, makes its path owner-only, then
    /// starts listening. A different uid cannot connect during the brief
    /// mode change, and this avoids changing the process-wide umask used by
    /// unrelated file creation on other threads.
    fn bind_private_socket(path: &Path) -> std::io::Result<(UnixListener, (u64, u64))> {
        let path_bytes = path.as_os_str().as_bytes();
        let mut address: libc::sockaddr_un =
            // SAFETY: a zeroed sockaddr_un is a valid starting address.
            unsafe { std::mem::zeroed() };
        if path_bytes.len() + 1 > address.sun_path.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "the control socket path is too long",
            ));
        }
        let address_length =
            std::mem::offset_of!(libc::sockaddr_un, sun_path) + path_bytes.len() + 1;
        #[cfg(any(
            target_os = "macos",
            target_os = "ios",
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "netbsd",
            target_os = "dragonfly"
        ))]
        {
            address.sun_len = address_length as u8;
        }
        address.sun_family = libc::AF_UNIX as _;
        for (target, source) in address.sun_path.iter_mut().zip(path_bytes) {
            *target = *source as libc::c_char;
        }

        // SAFETY: `socket` has no pointer arguments and returns an owned fd.
        let raw_fd = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_STREAM, 0) };
        if raw_fd < 0 {
            return Err(std::io::Error::last_os_error());
        }
        // SAFETY: `raw_fd` is a fresh fd returned by `socket`.
        let fd = unsafe { OwnedFd::from_raw_fd(raw_fd) };
        // Keep the local control socket out of child processes.
        // SAFETY: both fcntl calls use this live fd and scalar arguments.
        let descriptor_flags = unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_GETFD) };
        if descriptor_flags < 0
            || unsafe {
                libc::fcntl(
                    fd.as_raw_fd(),
                    libc::F_SETFD,
                    descriptor_flags | libc::FD_CLOEXEC,
                )
            } < 0
        {
            return Err(std::io::Error::last_os_error());
        }
        // SAFETY: `address` contains a NUL-terminated Unix pathname and its
        // length includes the `sun_path` offset and trailing NUL.
        if unsafe {
            libc::bind(
                fd.as_raw_fd(),
                (&address as *const libc::sockaddr_un).cast(),
                address_length as libc::socklen_t,
            )
        } < 0
        {
            return Err(std::io::Error::last_os_error());
        }

        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        let path_metadata = std::fs::symlink_metadata(path)?;
        // SAFETY: geteuid has no preconditions.
        let uid = unsafe { libc::geteuid() };
        if !path_metadata.file_type().is_socket()
            || path_metadata.uid() != uid
            || path_metadata.mode() & 0o777 != 0o600
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "the control socket path is not an owner-only socket",
            ));
        }

        // Only now can another process connect to the pathname.
        // SAFETY: `fd` is a bound stream socket that has not started listening.
        if unsafe { libc::listen(fd.as_raw_fd(), 128) } < 0 {
            return Err(std::io::Error::last_os_error());
        }
        let identity = (path_metadata.dev(), path_metadata.ino());
        Ok((
            // SAFETY: ownership moves from `OwnedFd` to the listener exactly once.
            unsafe { UnixListener::from_raw_fd(fd.into_raw_fd()) },
            identity,
        ))
    }

    /// Removes only an owned stale socket. Symlinks and regular files are
    /// refused, and a live listener is never replaced if the data-folder
    /// lock contract was not followed.
    fn remove_stale_socket(path: &Path) -> std::io::Result<()> {
        let metadata = match std::fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error),
        };
        // SAFETY: geteuid has no preconditions.
        let uid = unsafe { libc::geteuid() };
        if !metadata.file_type().is_socket() || metadata.uid() != uid {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "refusing to replace a non-socket or another user's control.sock",
            ));
        }
        match UnixStream::connect(path) {
            Ok(_) => Err(std::io::Error::new(
                std::io::ErrorKind::AddrInUse,
                "a Kanso control socket is already listening",
            )),
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::ConnectionRefused | std::io::ErrorKind::NotFound
                ) =>
            {
                std::fs::remove_file(path)
            }
            Err(error) => Err(error),
        }
    }

    /// Whether the process on the other end of `stream` runs as this user.
    #[cfg(any(
        target_os = "macos",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd"
    ))]
    fn peer_is_me(stream: &UnixStream) -> bool {
        use std::os::fd::AsRawFd as _;
        let (mut uid, mut gid) = (0 as libc::uid_t, 0 as libc::gid_t);
        // SAFETY: the descriptor is open for the call, and both outputs are
        // valid for writes.
        let ok = unsafe { libc::getpeereid(stream.as_raw_fd(), &mut uid, &mut gid) } == 0;
        // SAFETY: geteuid has no preconditions.
        ok && uid == unsafe { libc::geteuid() }
    }

    #[cfg(any(target_os = "linux", target_os = "android"))]
    fn peer_is_me(stream: &UnixStream) -> bool {
        use std::os::fd::AsRawFd as _;
        let mut cred = libc::ucred {
            pid: 0,
            uid: 0,
            gid: 0,
        };
        let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
        // SAFETY: the descriptor is open, and `cred` and `len` are valid for
        // the sizes given.
        let ok = unsafe {
            libc::getsockopt(
                stream.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_PEERCRED,
                (&mut cred as *mut libc::ucred).cast(),
                &mut len,
            )
        } == 0
            && len as usize == std::mem::size_of::<libc::ucred>();
        // SAFETY: geteuid has no preconditions.
        ok && cred.uid == unsafe { libc::geteuid() }
    }

    #[cfg(not(any(
        target_os = "macos",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "linux",
        target_os = "android"
    )))]
    fn peer_is_me(_: &UnixStream) -> bool {
        // Unknown Unix platforms fail closed instead of trusting a peer
        // whose credentials this build cannot verify.
        false
    }

    /// One line from `reader`, at most [`MAX_LINE`] long.
    fn read_line(reader: impl std::io::Read) -> std::io::Result<String> {
        let mut line = Vec::new();
        BufReader::new(reader.take(MAX_LINE as u64 + 1)).read_until(b'\n', &mut line)?;
        if line.len() > MAX_LINE {
            return Err(std::io::Error::other("the line is too long"));
        }
        if line.last() != Some(&b'\n') {
            return Err(std::io::Error::other("the line is incomplete"));
        }
        String::from_utf8(line).map_err(std::io::Error::other)
    }

    /// Answers one client: reads its command, runs it on the library and
    /// writes the reply.
    fn serve_one(stream: UnixStream, target: &ControlTarget) -> std::io::Result<()> {
        // Accepted sockets can inherit the listener's nonblocking mode on
        // some Unix kernels. Framing needs a bounded blocking read instead:
        // a client may send one chunk at a time, but cannot hold this thread
        // past `CLIENT_WAIT`.
        stream.set_nonblocking(false)?;
        stream.set_read_timeout(Some(CLIENT_WAIT))?;
        stream.set_write_timeout(Some(CLIENT_WAIT))?;
        if !peer_is_me(&stream) {
            // Nothing is said to a stranger.
            return Ok(());
        }
        let reply = match serde_json::from_str::<Request>(&read_line(&stream)?) {
            Err(_) => crate::control_model::Reply::Failed {
                error: "Kanso didn't understand the command. Run the same version of `kanso api-key` as the Kanso that is running.".into(),
            },
            Ok(request) => answer(request, target),
        };
        let mut stream = stream;
        let line = reply_line(&reply)?;
        stream.write_all(&line)
    }

    /// The Unix control socket of the process holding the data-folder lock.
    /// Dropping it closes the socket and removes the file it created.
    pub struct ControlServer {
        path: PathBuf,
        identity: (u64, u64),
        stop: Arc<AtomicBool>,
        #[cfg(test)]
        accepted_clients: Arc<AtomicUsize>,
        thread: Option<std::thread::JoinHandle<()>>,
    }

    impl ControlServer {
        /// Listens at `data_dir/control.sock`. The caller holds the
        /// data-folder lock, so a stale owned socket can be replaced safely.
        pub fn start(data_dir: &Path, target: ControlTarget) -> std::io::Result<ControlServer> {
            let _starting = START
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let data_dir = secure_data_dir(data_dir)?;
            let path = socket_path(&data_dir);
            if path.as_os_str().len() > MAX_PATH {
                return Err(path_too_long(&path));
            }
            remove_stale_socket(&path)?;
            let (listener, identity) = bind_private_socket(&path)?;
            listener.set_nonblocking(true)?;
            let stop = Arc::new(AtomicBool::new(false));
            let flag = stop.clone();
            #[cfg(test)]
            let accepted_clients = Arc::new(AtomicUsize::new(0));
            #[cfg(test)]
            let accepted = accepted_clients.clone();
            let thread = std::thread::Builder::new()
                .name("kanso-control".into())
                .spawn(move || {
                    let mut clients: Vec<std::thread::JoinHandle<()>> = Vec::new();
                    loop {
                        let mut index = 0;
                        while index < clients.len() {
                            if clients[index].is_finished() {
                                let client = clients.swap_remove(index);
                                let _ = client.join();
                            } else {
                                index += 1;
                            }
                        }
                        if flag.load(Ordering::Relaxed) {
                            break;
                        }
                        let stream = match listener.accept() {
                            Ok((stream, _)) => stream,
                            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                                std::thread::sleep(Duration::from_millis(10));
                                continue;
                            }
                            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {
                                continue;
                            }
                            Err(_) => break,
                        };
                        // A thread each: a revoke may wait for requests to
                        // end, and a slow client must not hold the others.
                        let target = target.clone();
                        if let Ok(client) = std::thread::Builder::new()
                            .name("kanso-control-client".into())
                            .spawn(move || {
                                let _ = serve_one(stream, &target);
                            })
                        {
                            clients.push(client);
                            #[cfg(test)]
                            accepted.fetch_add(1, Ordering::Release);
                        }
                    }
                    // `serve_cli` releases the folder lock immediately after
                    // dropping this server. Keep this join here so no accepted
                    // command can still write after that lock is released.
                    for client in clients {
                        let _ = client.join();
                    }
                })?;
            Ok(ControlServer {
                path,
                identity,
                stop,
                #[cfg(test)]
                accepted_clients,
                thread: Some(thread),
            })
        }

        #[cfg(test)]
        pub(crate) fn accepted_clients(&self) -> usize {
            self.accepted_clients.load(Ordering::Acquire)
        }
    }

    impl Drop for ControlServer {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::Relaxed);
            if let Some(thread) = self.thread.take() {
                let _ = thread.join();
            }
            if let Ok(metadata) = std::fs::symlink_metadata(&self.path)
                && metadata.file_type().is_socket()
                && metadata.dev() == self.identity.0
                && metadata.ino() == self.identity.1
            {
                let _ = std::fs::remove_file(&self.path);
            }
        }
    }

    /// Sends `request` to the process running with `data_dir`.
    pub fn send(data_dir: &Path, request: &Request) -> std::io::Result<Sent> {
        let data_dir = secure_data_dir(data_dir)?;
        let path = socket_path(&data_dir);
        if path.as_os_str().len() > MAX_PATH {
            return Err(path_too_long(&path));
        }
        let mut line = serde_json::to_vec(request).map_err(std::io::Error::other)?;
        if line.len() + 1 > MAX_LINE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "the API key command is too large for the control socket",
            ));
        }
        line.push(b'\n');
        let stream = match UnixStream::connect(&path) {
            Ok(stream) => stream,
            Err(err)
                if matches!(
                    err.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
                ) =>
            {
                return Ok(Sent::NoSocket);
            }
            Err(err) => return Err(err),
        };
        if !peer_is_me(&stream) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "the control socket belongs to another user",
            ));
        }
        stream.set_write_timeout(Some(CLIENT_WAIT))?;
        stream.set_read_timeout(Some(ANSWER_WAIT))?;
        (&stream).write_all(&line)?;
        let reply = read_line(&stream)?;
        match serde_json::from_str::<Reply>(&reply) {
            Ok(Reply::Done { out, err }) => Ok(Sent::Done(out, err)),
            Ok(Reply::Failed { error }) => Ok(Sent::Failed(error)),
            Err(_) => Err(std::io::Error::other(
                "Kanso answered with something this `kanso` doesn't understand",
            )),
        }
    }
}

#[cfg(unix)]
pub use unix::{ControlServer, send};

#[cfg(windows)]
mod windows {
    use std::ffi::c_void;
    use std::io::{Read, Write};
    use std::os::windows::ffi::OsStrExt as _;
    use std::os::windows::io::{AsRawHandle as _, FromRawHandle as _, RawHandle};
    use std::path::Path;
    use std::ptr::{null, null_mut};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::{Duration, Instant};

    use sha2::{Digest as _, Sha256};
    use windows_sys::Win32::Foundation::{
        CloseHandle, ERROR_NO_DATA, GetLastError, HANDLE, INVALID_HANDLE_VALUE, LocalFree,
    };
    use windows_sys::Win32::Security::Authorization::{
        ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
        SDDL_REVISION_1,
    };
    use windows_sys::Win32::Security::{
        EqualSid, GetTokenInformation, SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER, TokenUser,
    };
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_FLAG_FIRST_PIPE_INSTANCE, OPEN_EXISTING,
        PIPE_ACCESS_DUPLEX,
    };
    use windows_sys::Win32::System::Pipes::{
        ConnectNamedPipe, CreateNamedPipeW, GetNamedPipeClientProcessId,
        GetNamedPipeServerProcessId, PIPE_NOWAIT, PIPE_READMODE_BYTE, PIPE_REJECT_REMOTE_CLIENTS,
        PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT, SetNamedPipeHandleState,
        WaitNamedPipeW,
    };
    use windows_sys::Win32::System::Threading::{
        GetCurrentProcess, OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    use super::{ControlTarget, Sent, answer, reply_line};
    use crate::control_model::{MAX_LINE, Reply, Request};

    const CLIENT_WAIT_MS: u32 = 5_000;
    const CLIENT_WAIT: Duration = Duration::from_secs(5);
    const ANSWER_WAIT: Duration = Duration::from_secs(30);
    const IO_RETRY: Duration = Duration::from_millis(10);

    /// An owned Win32 handle. Kernel handles can be moved between threads.
    struct OwnedHandle(HANDLE);

    // SAFETY: Win32 handles are process-wide kernel references and are safe
    // to use or close from any thread.
    unsafe impl Send for OwnedHandle {}

    impl OwnedHandle {
        fn as_raw(&self) -> HANDLE {
            self.0
        }

        fn into_file(self) -> std::fs::File {
            let handle = self.0;
            std::mem::forget(self);
            // SAFETY: this handle is an open named-pipe instance whose
            // ownership moves to `File` exactly once.
            unsafe { std::fs::File::from_raw_handle(handle as RawHandle) }
        }
    }

    impl Drop for OwnedHandle {
        fn drop(&mut self) {
            if !self.0.is_null() && self.0 != INVALID_HANDLE_VALUE {
                // SAFETY: this wrapper uniquely owns a valid kernel handle.
                unsafe { CloseHandle(self.0) };
            }
        }
    }

    fn last_error() -> std::io::Error {
        // SAFETY: GetLastError has no preconditions.
        std::io::Error::from_raw_os_error(unsafe { GetLastError() } as i32)
    }

    /// A token's user SID buffer, aligned for `TOKEN_USER` and retained for
    /// as long as a caller uses the embedded SID pointer.
    fn token_user_buffer(token: HANDLE) -> std::io::Result<Vec<usize>> {
        let mut bytes = 0u32;
        // SAFETY: the null buffer requests the required size only.
        let first = unsafe { GetTokenInformation(token, TokenUser, null_mut(), 0, &mut bytes) };
        if first != 0 || bytes == 0 {
            return Err(last_error());
        }
        // SAFETY: GetLastError has no preconditions.
        if unsafe { GetLastError() } != windows_sys::Win32::Foundation::ERROR_INSUFFICIENT_BUFFER {
            return Err(last_error());
        }
        let words = (bytes as usize).div_ceil(std::mem::size_of::<usize>());
        let mut buffer = vec![0usize; words];
        // SAFETY: `buffer` is aligned, writable, and at least `bytes` long.
        let ok = unsafe {
            GetTokenInformation(
                token,
                TokenUser,
                buffer.as_mut_ptr().cast::<c_void>(),
                bytes,
                &mut bytes,
            )
        };
        if ok == 0 {
            return Err(last_error());
        }
        Ok(buffer)
    }

    fn token_sid(buffer: &[usize]) -> windows_sys::Win32::Security::PSID {
        // SAFETY: `token_user_buffer` filled this aligned allocation with a
        // TOKEN_USER record and kept its SID data in the same allocation.
        unsafe { (*buffer.as_ptr().cast::<TOKEN_USER>()).User.Sid }
    }

    fn current_user_sid_string() -> std::io::Result<String> {
        // SAFETY: GetCurrentProcess returns a pseudo-handle valid for this call.
        let process = unsafe { GetCurrentProcess() };
        let mut token = null_mut();
        // SAFETY: `token` is a valid output pointer; process is a pseudo-handle.
        if unsafe { OpenProcessToken(process, TOKEN_QUERY, &mut token) } == 0 {
            return Err(last_error());
        }
        let token = OwnedHandle(token);
        let user = token_user_buffer(token.as_raw())?;
        let mut sid_text = null_mut();
        // SAFETY: the SID pointer is valid while `user` lives; the API writes
        // a LocalAlloc-owned UTF-16 string to `sid_text`.
        if unsafe { ConvertSidToStringSidW(token_sid(&user), &mut sid_text) } == 0 {
            return Err(last_error());
        }
        let mut len = 0usize;
        // SAFETY: the returned pointer is a NUL-terminated LocalAlloc string.
        while unsafe { *sid_text.add(len) } != 0 {
            len += 1;
        }
        // SAFETY: `sid_text` points to `len` readable UTF-16 code units.
        let sid = String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(sid_text, len) });
        // SAFETY: ConvertSidToStringSidW allocated this string with LocalAlloc.
        unsafe { LocalFree(sid_text.cast()) };
        Ok(sid)
    }

    /// The DACL explicitly grants full pipe access only to the current SID.
    fn create_pipe(name: &str, first: bool) -> std::io::Result<OwnedHandle> {
        let sid = current_user_sid_string()?;
        let sddl = format!("D:P(A;;GA;;;{sid})");
        let sddl: Vec<u16> = sddl.encode_utf16().chain(std::iter::once(0)).collect();
        let mut descriptor = null_mut();
        // SAFETY: `sddl` is NUL-terminated and `descriptor` is a valid output.
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(),
                SDDL_REVISION_1,
                &mut descriptor,
                null_mut(),
            )
        } == 0
        {
            return Err(last_error());
        }
        let attributes = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: descriptor,
            bInheritHandle: 0,
        };
        let name: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        let open_mode = PIPE_ACCESS_DUPLEX
            | if first {
                FILE_FLAG_FIRST_PIPE_INSTANCE
            } else {
                0
            };
        // SAFETY: the name and security descriptor remain valid for the call;
        // the returned pipe handle is checked before ownership is accepted.
        let handle = unsafe {
            CreateNamedPipeW(
                name.as_ptr(),
                open_mode,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
                PIPE_UNLIMITED_INSTANCES,
                MAX_LINE as u32,
                MAX_LINE as u32,
                CLIENT_WAIT_MS,
                &attributes,
            )
        };
        // SAFETY: the descriptor was allocated by the SDDL conversion above.
        unsafe { LocalFree(descriptor) };
        if handle == INVALID_HANDLE_VALUE || handle.is_null() {
            return Err(last_error());
        }
        Ok(OwnedHandle(handle))
    }

    /// A stable local pipe name for this canonical data folder. The path is
    /// hashed so neither path-length limits nor user-controlled pipe syntax
    /// can change the namespace.
    fn pipe_name(data_dir: &Path) -> std::io::Result<String> {
        let canonical = std::fs::canonicalize(data_dir)?;
        if !canonical.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "the data folder is not a directory",
            ));
        }
        let mut digest = Sha256::new();
        for unit in canonical.as_os_str().encode_wide() {
            digest.update(unit.to_le_bytes());
        }
        let hash = digest
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        Ok(format!(r"\\.\pipe\kanso-control-{hash}"))
    }

    fn peer_has_current_sid(pipe: HANDLE, peer_is_server: bool) -> bool {
        let mut pid = 0u32;
        // SAFETY: the pipe handle is connected and `pid` is writable.
        let got_pid = unsafe {
            if peer_is_server {
                GetNamedPipeServerProcessId(pipe, &mut pid)
            } else {
                GetNamedPipeClientProcessId(pipe, &mut pid)
            }
        };
        if got_pid == 0 || pid == 0 {
            return false;
        }
        // SAFETY: `pid` came from the connected pipe; the handle is checked.
        let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
        if process.is_null() {
            return false;
        }
        let process = OwnedHandle(process);
        let mut peer_token = null_mut();
        // SAFETY: `process` is an open process handle; `peer_token` is writable.
        if unsafe { OpenProcessToken(process.as_raw(), TOKEN_QUERY, &mut peer_token) } == 0 {
            return false;
        }
        let peer_token = OwnedHandle(peer_token);
        let Ok(peer_user) = token_user_buffer(peer_token.as_raw()) else {
            return false;
        };
        // SAFETY: GetCurrentProcess returns a pseudo-handle valid for this call.
        let current_process = unsafe { GetCurrentProcess() };
        let mut current_token = null_mut();
        // SAFETY: `current_token` is writable and the current process is valid.
        if unsafe { OpenProcessToken(current_process, TOKEN_QUERY, &mut current_token) } == 0 {
            return false;
        }
        let current_token = OwnedHandle(current_token);
        let Ok(current_user) = token_user_buffer(current_token.as_raw()) else {
            return false;
        };
        // SAFETY: both SID pointers are backed by live token buffers.
        unsafe { EqualSid(token_sid(&peer_user), token_sid(&current_user)) != 0 }
    }

    /// Enables nonblocking byte-mode I/O so the framing loops below can apply
    /// one deadline to the whole message instead of resetting it per byte.
    fn set_pipe_mode(pipe: HANDLE) -> std::io::Result<()> {
        let mode = PIPE_READMODE_BYTE | PIPE_NOWAIT;
        // SAFETY: `mode` is a live input word and null optional outputs are valid.
        if unsafe { SetNamedPipeHandleState(pipe, &mode, null_mut(), null_mut()) } == 0 {
            return Err(last_error());
        }
        Ok(())
    }

    fn retryable(error: &std::io::Error) -> bool {
        error.kind() == std::io::ErrorKind::WouldBlock
            || error.raw_os_error() == Some(ERROR_NO_DATA as i32)
    }

    fn read_line(mut reader: impl Read, timeout: Duration) -> std::io::Result<String> {
        let deadline = Instant::now() + timeout;
        let mut line = Vec::new();
        loop {
            let mut byte = [0u8; 1];
            match reader.read(&mut byte) {
                Ok(0) => return Err(std::io::Error::other("the line is incomplete")),
                Ok(_) if byte[0] == b'\n' => {
                    line.push(b'\n');
                    return String::from_utf8(line).map_err(std::io::Error::other);
                }
                Ok(_) => {
                    if line.len() + 1 >= MAX_LINE {
                        return Err(std::io::Error::other("the line is too long"));
                    }
                    line.push(byte[0]);
                }
                Err(error) if retryable(&error) => {
                    if Instant::now() >= deadline {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::TimedOut,
                            "timed out reading the control line",
                        ));
                    }
                    std::thread::sleep(IO_RETRY);
                }
                Err(error) => return Err(error),
            }
            if Instant::now() >= deadline {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "timed out reading the control line",
                ));
            }
        }
    }

    fn write_line(mut writer: impl Write, bytes: &[u8], timeout: Duration) -> std::io::Result<()> {
        let deadline = Instant::now() + timeout;
        let mut written = 0;
        while written < bytes.len() {
            match writer.write(&bytes[written..]) {
                Ok(0) => {
                    if Instant::now() >= deadline {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::TimedOut,
                            "timed out writing the control line",
                        ));
                    }
                    std::thread::sleep(IO_RETRY);
                }
                Ok(count) => written += count,
                Err(error) if retryable(&error) => {
                    if Instant::now() >= deadline {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::TimedOut,
                            "timed out writing the control line",
                        ));
                    }
                    std::thread::sleep(IO_RETRY);
                }
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }

    fn serve_one(mut pipe: std::fs::File, target: &ControlTarget) -> std::io::Result<()> {
        let handle = pipe.as_raw_handle() as HANDLE;
        if !peer_has_current_sid(handle, false) {
            return Ok(());
        }
        set_pipe_mode(handle)?;
        let reply = match serde_json::from_str::<Request>(&read_line(&mut pipe, CLIENT_WAIT)?) {
            Ok(request) => answer(request, target),
            Err(_) => Reply::Failed {
                error: "Kanso didn't understand the command. Run the same version of `kanso api-key` as the Kanso that is running.".into(),
            },
        };
        write_line(&mut pipe, &reply_line(&reply)?, CLIENT_WAIT)
    }

    fn connect_pipe(name: &str) -> std::io::Result<OwnedHandle> {
        let name: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        for attempt in 0..2 {
            // SAFETY: the name is NUL-terminated; no inherited security
            // attributes or template handle are requested.
            let handle = unsafe {
                CreateFileW(
                    name.as_ptr(),
                    windows_sys::Win32::Foundation::GENERIC_READ
                        | windows_sys::Win32::Foundation::GENERIC_WRITE,
                    0,
                    null(),
                    OPEN_EXISTING,
                    FILE_ATTRIBUTE_NORMAL,
                    null_mut(),
                )
            };
            if handle != INVALID_HANDLE_VALUE && !handle.is_null() {
                return Ok(OwnedHandle(handle));
            }
            let error = last_error();
            if error.raw_os_error() != Some(windows_sys::Win32::Foundation::ERROR_PIPE_BUSY as i32)
                || attempt != 0
            {
                return Err(error);
            }
            // SAFETY: the pipe name is valid and the wait is bounded to five seconds.
            if unsafe { WaitNamedPipeW(name.as_ptr(), CLIENT_WAIT_MS) } == 0 {
                return Err(last_error());
            }
        }
        Err(std::io::Error::other("the named pipe stayed busy"))
    }

    fn accept_loop(name: String, target: ControlTarget, first: OwnedHandle, stop: Arc<AtomicBool>) {
        let mut next = Some(first);
        let mut clients: Vec<std::thread::JoinHandle<()>> = Vec::new();
        loop {
            let mut index = 0;
            while index < clients.len() {
                if clients[index].is_finished() {
                    let client = clients.swap_remove(index);
                    let _ = client.join();
                } else {
                    index += 1;
                }
            }
            if stop.load(Ordering::Relaxed) {
                break;
            }
            let pipe = match next.take() {
                Some(pipe) => pipe,
                None => match create_pipe(&name, false) {
                    Ok(pipe) => pipe,
                    Err(_) => break,
                },
            };
            // Close can race with creation of the next pipe instance. If it
            // stops us before this point, do not enter a fresh blocking wait:
            // the shutdown wake-up may already have found no pipe to connect.
            if stop.load(Ordering::Relaxed) {
                break;
            }
            // SAFETY: this is a fresh, listening named-pipe instance.
            let connected = unsafe { ConnectNamedPipe(pipe.as_raw(), null_mut()) } != 0
                // SAFETY: GetLastError describes the immediately preceding failed call.
                || unsafe { GetLastError() }
                    == windows_sys::Win32::Foundation::ERROR_PIPE_CONNECTED;
            if !connected {
                continue;
            }
            if stop.load(Ordering::Relaxed) {
                break;
            }
            let target = target.clone();
            if let Ok(client) = std::thread::Builder::new()
                .name("kanso-control-client".into())
                .spawn(move || {
                    let _ = serve_one(pipe.into_file(), &target);
                })
            {
                clients.push(client);
            }
        }
        // Keep the data-folder lock held until every accepted command has
        // finished, just as the Unix transport does.
        for client in clients {
            let _ = client.join();
        }
    }

    pub struct ControlServer {
        name: String,
        stop: Arc<AtomicBool>,
        thread: Option<std::thread::JoinHandle<()>>,
    }

    impl ControlServer {
        pub fn start(data_dir: &Path, target: ControlTarget) -> std::io::Result<ControlServer> {
            let name = pipe_name(data_dir)?;
            // FIRST_PIPE_INSTANCE makes a pre-existing pipe a hard failure,
            // so a same-user process cannot squat on the control name first.
            let first = create_pipe(&name, true)?;
            let stop = Arc::new(AtomicBool::new(false));
            let flag = stop.clone();
            let server_name = name.clone();
            let thread = std::thread::Builder::new()
                .name("kanso-control".into())
                .spawn(move || accept_loop(server_name, target, first, flag))?;
            Ok(ControlServer {
                name,
                stop,
                thread: Some(thread),
            })
        }
    }

    impl Drop for ControlServer {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::Relaxed);
            // Connecting wakes the accept loop blocked in ConnectNamedPipe.
            let _ = connect_pipe(&self.name);
            if let Some(thread) = self.thread.take() {
                let _ = thread.join();
            }
        }
    }

    pub fn send(data_dir: &Path, request: &Request) -> std::io::Result<Sent> {
        let name = pipe_name(data_dir)?;
        let mut line = serde_json::to_vec(request).map_err(std::io::Error::other)?;
        if line.len() + 1 > MAX_LINE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "the API key command is too large for the control pipe",
            ));
        }
        line.push(b'\n');
        let pipe = match connect_pipe(&name) {
            Ok(pipe) => pipe,
            Err(error)
                if error.raw_os_error()
                    == Some(windows_sys::Win32::Foundation::ERROR_FILE_NOT_FOUND as i32) =>
            {
                return Ok(Sent::NoSocket);
            }
            Err(error) => return Err(error),
        };
        if !peer_has_current_sid(pipe.as_raw(), true) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "the control pipe belongs to another user",
            ));
        }
        set_pipe_mode(pipe.as_raw())?;
        let mut pipe = pipe.into_file();
        write_line(&mut pipe, &line, CLIENT_WAIT)?;
        let reply = read_line(&mut pipe, ANSWER_WAIT)?;
        match serde_json::from_str::<Reply>(&reply) {
            Ok(Reply::Done { out, err }) => Ok(Sent::Done(out, err)),
            Ok(Reply::Failed { error }) => Ok(Sent::Failed(error)),
            Err(_) => Err(std::io::Error::other(
                "Kanso answered with something this `kanso` doesn't understand",
            )),
        }
    }
}

#[cfg(windows)]
pub use windows::{ControlServer, send};

#[cfg(not(any(unix, windows)))]
pub struct ControlServer;

#[cfg(not(any(unix, windows)))]
impl ControlServer {
    pub fn start(_: &std::path::Path, _: ControlTarget) -> std::io::Result<ControlServer> {
        Err(std::io::Error::other(
            "the control channel is unavailable on this platform",
        ))
    }
}

#[cfg(not(any(unix, windows)))]
pub fn send(_: &std::path::Path, _: &crate::control_model::Request) -> std::io::Result<Sent> {
    Ok(Sent::NoSocket)
}

#[cfg(all(test, unix))]
#[path = "control_tests.rs"]
mod tests;

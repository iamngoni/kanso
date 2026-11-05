//! The control socket (issue #243): how `kanso api-key create | list |
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
//! The channel is a Unix socket in the data folder, mode 0600, that only the
//! same OS user can use: the file's permissions keep others out, and the
//! server also checks the connecting process's user id. A key Kanso already
//! trusts cannot use it: it is not on the HTTP port. Windows has no such
//! socket here, so a command there still needs the data folder to itself.

use std::sync::{Arc, Mutex};

use kanso_server::KeyStore;
use kanso_store::Store;

/// The library the control socket works on: the one the running process
/// serves keys from. The app sets it again if the library's store changes.
#[derive(Clone, Default)]
pub struct ControlTarget(Arc<Mutex<Option<(Arc<Store>, KeyStore)>>>);

impl ControlTarget {
    pub fn new(store: Arc<Store>, keys: KeyStore) -> ControlTarget {
        ControlTarget(Arc::new(Mutex::new(Some((store, keys)))))
    }

    pub fn set(&self, store: Arc<Store>, keys: KeyStore) {
        *self.0.lock().unwrap_or_else(|p| p.into_inner()) = Some((store, keys));
    }

    #[cfg(unix)]
    fn get(&self) -> Option<(Arc<Store>, KeyStore)> {
        self.0.lock().unwrap_or_else(|p| p.into_inner()).clone()
    }
}

/// What a command sent to a running process comes back as.
#[derive(Debug)]
pub enum Sent {
    /// It ran: standard output, standard error.
    Done(String, String),
    /// It ran and said no.
    Failed(String),
    /// Nothing is listening: the process is an older Kanso, runs on Windows,
    /// or has no control socket.
    NoSocket,
}

#[cfg(unix)]
mod unix {
    use std::io::{BufRead as _, BufReader, Write as _};
    use std::os::unix::fs::PermissionsExt as _;
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    use super::{ControlTarget, Sent};
    use crate::api_key_cli;
    use crate::control_model::{MAX_LINE, Reply, Request};

    /// The most a socket path may hold (`sun_path` is 104 bytes on macOS,
    /// 108 on Linux, with the end).
    const MAX_PATH: usize = 100;

    /// How long a connected client may take to send its command, and how
    /// long a command may take to answer (a revoke waits for requests the
    /// key started, a few seconds at most).
    const CLIENT_WAIT: Duration = Duration::from_secs(5);
    const ANSWER_WAIT: Duration = Duration::from_secs(30);

    /// Where the socket for `data_dir` is.
    pub fn socket_path(data_dir: &Path) -> PathBuf {
        data_dir.join("control.sock")
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
        } == 0;
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
        // The socket's 0600 mode is all there is to go on.
        true
    }

    /// One line from `reader`, at most [`MAX_LINE`] long.
    fn read_line(reader: impl std::io::Read) -> std::io::Result<String> {
        let mut line = Vec::new();
        BufReader::new(reader.take(MAX_LINE as u64 + 1)).read_until(b'\n', &mut line)?;
        if line.len() > MAX_LINE {
            return Err(std::io::Error::other("the line is too long"));
        }
        String::from_utf8(line).map_err(std::io::Error::other)
    }

    /// Answers one client: reads its command, runs it on the library and
    /// writes the reply.
    fn serve_one(stream: UnixStream, target: &ControlTarget) -> std::io::Result<()> {
        stream.set_read_timeout(Some(CLIENT_WAIT))?;
        stream.set_write_timeout(Some(CLIENT_WAIT))?;
        if !peer_is_me(&stream) {
            // Nothing is said to a stranger.
            return Ok(());
        }
        let reply = match serde_json::from_str::<Request>(&read_line(&stream)?) {
            Err(_) => Reply::Failed {
                error: "Kanso didn't understand the command. Run the same version of `kanso api-key` as the Kanso that is running.".into(),
            },
            Ok(request) => answer(request, target),
        };
        let mut stream = stream;
        let mut line = serde_json::to_vec(&reply).map_err(std::io::Error::other)?;
        line.push(b'\n');
        stream.write_all(&line)
    }

    fn answer(request: Request, target: &ControlTarget) -> Reply {
        let command = match request.command() {
            Ok(command) => command,
            Err(error) => return Reply::Failed { error },
        };
        let Some((store, keys)) = target.get() else {
            return Reply::Failed {
                error: "Kanso hasn't opened its library yet. Try again in a moment.".into(),
            };
        };
        let (mut out, mut err) = (Vec::new(), Vec::new());
        match api_key_cli::execute(&store, &keys, command, &mut out, &mut err) {
            Ok(()) => Reply::Done {
                out: String::from_utf8_lossy(&out).into_owned(),
                err: String::from_utf8_lossy(&err).into_owned(),
            },
            Err(error) => Reply::Failed {
                error: format!("{error:#}"),
            },
        }
    }

    /// The control socket of the process that holds the data-folder lock.
    /// Dropping it closes the socket and removes the file.
    pub struct ControlServer {
        path: PathBuf,
        stop: Arc<AtomicBool>,
        thread: Option<std::thread::JoinHandle<()>>,
    }

    impl ControlServer {
        /// Listens at `data_dir/control.sock`. The caller holds the
        /// data-folder lock, so any file already there is a dead process's.
        pub fn start(data_dir: &Path, target: ControlTarget) -> std::io::Result<ControlServer> {
            let path = socket_path(data_dir);
            if path.as_os_str().len() > MAX_PATH {
                return Err(std::io::Error::other(format!(
                    "the data folder's path is too long for a socket ({} is over {MAX_PATH} bytes); use a shorter --data-dir",
                    path.as_os_str().len()
                )));
            }
            let _ = std::fs::remove_file(&path);
            let listener = UnixListener::bind(&path)?;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
            let stop = Arc::new(AtomicBool::new(false));
            let flag = stop.clone();
            let thread = std::thread::Builder::new()
                .name("kanso-control".into())
                .spawn(move || {
                    for stream in listener.incoming() {
                        if flag.load(Ordering::Relaxed) {
                            break;
                        }
                        let Ok(stream) = stream else { continue };
                        // A thread each: a revoke may wait for requests to
                        // end, and a slow client must not hold the others.
                        let target = target.clone();
                        let _ = std::thread::Builder::new()
                            .name("kanso-control-client".into())
                            .spawn(move || {
                                let _ = serve_one(stream, &target);
                            });
                    }
                })?;
            Ok(ControlServer {
                path,
                stop,
                thread: Some(thread),
            })
        }
    }

    impl Drop for ControlServer {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::Relaxed);
            // Wakes the accept loop so it sees the flag.
            let _ = UnixStream::connect(&self.path);
            if let Some(thread) = self.thread.take() {
                let _ = thread.join();
            }
            let _ = std::fs::remove_file(&self.path);
        }
    }

    /// Sends `request` to the process running with `data_dir`.
    pub fn send(data_dir: &Path, request: &Request) -> std::io::Result<Sent> {
        let path = socket_path(data_dir);
        if path.as_os_str().len() > MAX_PATH {
            return Ok(Sent::NoSocket);
        }
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
        stream.set_write_timeout(Some(CLIENT_WAIT))?;
        stream.set_read_timeout(Some(ANSWER_WAIT))?;
        let mut line = serde_json::to_vec(request).map_err(std::io::Error::other)?;
        line.push(b'\n');
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

#[cfg(not(unix))]
pub struct ControlServer;

#[cfg(not(unix))]
impl ControlServer {
    pub fn start(_: &std::path::Path, _: ControlTarget) -> std::io::Result<ControlServer> {
        Err(std::io::Error::other(
            "the control socket needs a Unix system",
        ))
    }
}

#[cfg(not(unix))]
pub fn send(_: &std::path::Path, _: &crate::control_model::Request) -> std::io::Result<Sent> {
    Ok(Sent::NoSocket)
}

#[cfg(all(test, unix))]
#[path = "control_tests.rs"]
mod tests;

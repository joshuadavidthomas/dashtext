//! Single-instance coordination.
//!
//! The first Dashtext process listens on a Unix socket in the runtime
//! directory. Later invocations (`dashtext`, `dashtext capture`, or a desktop
//! shortcut bound to them) connect, hand over their request and exit, so the
//! capture window opens in the already running app within milliseconds.
//!
//! The protocol is one request line per connection, answered with `ok`.

use std::fmt;
use std::str::FromStr;

/// Something a process asks the primary instance to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    /// Show the drafts window.
    Open,
    /// Show the quick capture window.
    Capture,
    /// The library changed on disk; refresh what is shown.
    Reload,
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Open => "open",
            Self::Capture => "capture",
            Self::Reload => "reload",
        })
    }
}

impl FromStr for Request {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "open" => Ok(Self::Open),
            "capture" => Ok(Self::Capture),
            "reload" => Ok(Self::Reload),
            other => Err(format!("unknown request `{other}`")),
        }
    }
}

/// The outcome of trying to become the primary instance.
pub enum Claim {
    /// This process is the primary instance; serve requests with the listener.
    Primary(Listener),
    /// Another instance is running and accepted the request.
    Forwarded,
}

impl Claim {
    #[cfg(test)]
    fn into_primary(self) -> Option<Listener> {
        match self {
            Self::Primary(listener) => Some(listener),
            Self::Forwarded => None,
        }
    }
}

pub use platform::Listener;
pub use platform::claim;
pub use platform::notify;

#[cfg(unix)]
mod platform {
    use std::io::BufRead as _;
    use std::io::BufReader;
    use std::io::ErrorKind;
    use std::io::Write as _;
    use std::os::unix::net::UnixListener;
    use std::os::unix::net::UnixStream;
    use std::path::Path;
    use std::path::PathBuf;
    use std::time::Duration;

    use anyhow::Context as _;

    use super::Claim;
    use super::Request;
    use crate::paths::Paths;

    const TIMEOUT: Duration = Duration::from_secs(2);

    pub struct Listener {
        listener: UnixListener,
        path: PathBuf,
    }

    /// Becomes the primary instance, or forwards `request` to the running one.
    pub fn claim(paths: &Paths, request: Request) -> anyhow::Result<Claim> {
        let path = paths.socket();
        match UnixListener::bind(&path) {
            Ok(listener) => Ok(Claim::Primary(Listener { listener, path })),
            Err(error) if error.kind() == ErrorKind::AddrInUse => {
                match UnixStream::connect(&path) {
                    Ok(stream) => {
                        // Someone is listening. If it is slow to answer, it
                        // is still alive: report that rather than taking
                        // over its socket.
                        exchange(stream, request).with_context(|| {
                            format!(
                                "Dashtext is running but did not answer on {}",
                                path.display()
                            )
                        })?;
                        return Ok(Claim::Forwarded);
                    }
                    Err(error)
                        if matches!(
                            error.kind(),
                            ErrorKind::ConnectionRefused | ErrorKind::NotFound
                        ) => {}
                    Err(error) => {
                        return Err(error)
                            .with_context(|| format!("could not connect to {}", path.display()));
                    }
                }
                // Nobody is listening: a previous instance exited without cleaning up.
                std::fs::remove_file(&path)
                    .with_context(|| format!("could not remove stale socket {}", path.display()))?;
                let listener = UnixListener::bind(&path)
                    .with_context(|| format!("could not listen on {}", path.display()))?;
                Ok(Claim::Primary(Listener { listener, path }))
            }
            Err(error) => {
                Err(error).with_context(|| format!("could not listen on {}", path.display()))
            }
        }
    }

    /// Sends `request` to a running instance, if there is one.
    pub fn notify(paths: &Paths, request: Request) {
        // Failing to reach an instance only means none is running.
        if let Err(error) = send(&paths.socket(), request) {
            log::debug!("no running instance to notify: {error}");
        }
    }

    fn send(path: &Path, request: Request) -> std::io::Result<()> {
        exchange(UnixStream::connect(path)?, request)
    }

    fn exchange(mut stream: UnixStream, request: Request) -> std::io::Result<()> {
        stream.set_read_timeout(Some(TIMEOUT))?;
        stream.set_write_timeout(Some(TIMEOUT))?;
        writeln!(stream, "{request}")?;

        let mut reply = String::new();
        BufReader::new(stream).read_line(&mut reply)?;
        if reply.trim() == "ok" {
            Ok(())
        } else {
            Err(std::io::Error::other(format!(
                "unexpected reply `{}`",
                reply.trim()
            )))
        }
    }

    impl Listener {
        /// Accepts requests on a background thread and delivers them to
        /// `requests` until the receiving side is dropped. Keep the returned
        /// guard until the app quits; dropping it removes the socket.
        pub fn serve(
            self,
            requests: async_channel::Sender<Request>,
        ) -> anyhow::Result<SocketGuard> {
            let guard = SocketGuard {
                path: self.path.clone(),
            };
            let listener = self.listener;
            std::thread::Builder::new()
                .name("dashtext-instance".into())
                .spawn(move || {
                    for stream in listener.incoming() {
                        let stream = match stream {
                            Ok(stream) => stream,
                            Err(error) => {
                                log::warn!("instance socket accept failed: {error}");
                                continue;
                            }
                        };
                        match receive(&stream) {
                            Ok(request) => {
                                if requests.send_blocking(request).is_err() {
                                    break;
                                }
                            }
                            Err(error) => log::warn!("ignored instance request: {error}"),
                        }
                    }
                })
                .context("could not start the instance listener")?;
            Ok(guard)
        }
    }

    /// Removes the instance socket when dropped.
    pub struct SocketGuard {
        path: PathBuf,
    }

    fn receive(mut stream: &UnixStream) -> anyhow::Result<Request> {
        stream.set_read_timeout(Some(TIMEOUT))?;
        stream.set_write_timeout(Some(TIMEOUT))?;
        let mut line = String::new();
        BufReader::new(stream).read_line(&mut line)?;
        let request = line.trim().parse().map_err(anyhow::Error::msg)?;
        stream.write_all(b"ok\n")?;
        Ok(request)
    }

    impl Drop for SocketGuard {
        fn drop(&mut self) {
            // Best effort: a leftover socket is detected as stale next launch.
            if let Err(error) = std::fs::remove_file(&self.path) {
                log::debug!("could not remove {}: {error}", self.path.display());
            }
        }
    }
}

#[cfg(not(unix))]
mod platform {
    use super::Claim;
    use super::Request;
    use crate::paths::Paths;

    /// Without Unix sockets every process is its own instance.
    pub struct Listener;

    pub struct SocketGuard;

    pub fn claim(_: &Paths, _: Request) -> anyhow::Result<Claim> {
        Ok(Claim::Primary(Listener))
    }

    pub fn notify(_: &Paths, _: Request) {}

    impl Listener {
        pub fn serve(self, _: async_channel::Sender<Request>) -> anyhow::Result<SocketGuard> {
            Ok(SocketGuard)
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::net::UnixListener;

    use super::*;
    use crate::paths::Paths;

    fn paths(dir: &tempfile::TempDir) -> Paths {
        Paths::in_dir(dir.path())
    }

    #[test]
    fn a_leftover_socket_is_taken_over() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let paths = paths(&dir);
        // Bound and closed without unlinking, as after a crash.
        drop(UnixListener::bind(paths.socket()).expect("bind"));

        assert!(matches!(
            claim(&paths, Request::Open),
            Ok(Claim::Primary(_))
        ));
    }

    #[test]
    fn a_slow_instance_keeps_its_socket() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let paths = paths(&dir);
        // Listening but not yet answering, like an instance still starting up.
        let _busy = UnixListener::bind(paths.socket()).expect("bind");

        assert!(claim(&paths, Request::Capture).is_err());
        assert!(
            paths.socket().exists(),
            "the running instance keeps its socket"
        );
    }

    #[test]
    fn requests_reach_the_primary_instance() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let paths = paths(&dir);
        let listener = claim(&paths, Request::Open)
            .expect("claim the socket")
            .into_primary()
            .expect("the first claim becomes primary");
        let (sender, requests) = async_channel::unbounded();
        let _guard = listener.serve(sender).expect("serve");

        assert!(matches!(
            claim(&paths, Request::Capture),
            Ok(Claim::Forwarded)
        ));
        assert_eq!(requests.recv_blocking(), Ok(Request::Capture));
    }

    #[test]
    fn requests_round_trip_through_text() {
        for request in [Request::Open, Request::Capture, Request::Reload] {
            assert_eq!(request.to_string().parse(), Ok(request));
        }
        assert!("launch".parse::<Request>().is_err());
    }
}

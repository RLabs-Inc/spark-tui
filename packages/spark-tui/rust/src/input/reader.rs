//! stdin reader for raw terminal input.
//!
//! Reads raw bytes from stdin in a dedicated thread.
//! Routes to the parser for escape sequence parsing.

use std::io::{self, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::sync::mpsc::Sender;


/// Messages from stdin reader and wake watcher to the engine thread.
pub enum StdinMessage {
    /// Raw bytes from stdin.
    Data(Vec<u8>),
    /// TS wrote to SharedBuffer — wake flag detected by wake watcher.
    Wake,
    /// Terminal was resized (SIGWINCH).
    Resize(u16, u16),
    /// stdin closed or error.
    Closed,
}

/// Dedicated stdin reader thread.
///
/// Reads raw bytes and sends them through a channel.
/// Uses a small buffer and non-blocking reads with a timeout
/// for cooperative shutdown.
pub struct StdinReader {
    handle: Option<JoinHandle<()>>,
    running: Arc<AtomicBool>,
}

impl StdinReader {
    /// Spawn the stdin reader thread with an external channel sender.
    ///
    /// The engine creates the channel and passes sender clones to both
    /// StdinReader and WakeWatcher, keeping the receiver.
    pub fn spawn(tx: Sender<StdinMessage>) -> io::Result<Self> {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        let handle = thread::Builder::new()
            .name("spark-stdin".to_string())
            .spawn(move || {
                Self::read_loop(running_clone, tx);
            })?;

        Ok(Self {
            handle: Some(handle),
            running,
        })
    }

    fn read_loop(running: Arc<AtomicBool>, tx: Sender<StdinMessage>) {
        let stdin = io::stdin();
        let mut buf = [0u8; 256];

        while running.load(Ordering::SeqCst) {
            // Use a non-blocking approach: try to read with a timeout
            // On Unix, stdin.read() blocks until data is available.
            // We rely on the running flag + drop to stop the thread.
            match stdin.lock().read(&mut buf) {
                Ok(0) => {
                    // EOF
                    let _ = tx.send(StdinMessage::Closed);
                    break;
                }
                Ok(n) => {
                    let data = buf[..n].to_vec();
                    if tx.send(StdinMessage::Data(data)).is_err() {
                        break; // Channel closed
                    }
                }
                Err(e) => {
                    if e.kind() == io::ErrorKind::Interrupted {
                        continue; // Retry on interrupt
                    }
                    let _ = tx.send(StdinMessage::Closed);
                    break;
                }
            }
        }
    }

    /// Stop the reader thread.
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            // Note: the thread may be blocked on stdin.read().
            // On most systems, dropping the thread handle is sufficient.
            // The thread will exit when the process exits or stdin closes.
            let _ = handle;
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl Drop for StdinReader {
    fn drop(&mut self) {
        self.stop();
    }
}

// =============================================================================
// SIGWINCH (Terminal Resize) Watcher
// =============================================================================

/// Get current terminal size using ioctl.
#[cfg(unix)]
pub fn get_terminal_size() -> Option<(u16, u16)> {
    use libc::{ioctl, winsize, STDOUT_FILENO, TIOCGWINSZ};

    let mut ws: winsize = unsafe { std::mem::zeroed() };
    let result = unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut ws) };

    if result == 0 && ws.ws_col > 0 && ws.ws_row > 0 {
        Some((ws.ws_col, ws.ws_row))
    } else {
        None
    }
}

#[cfg(not(unix))]
pub fn get_terminal_size() -> Option<(u16, u16)> {
    None // Windows would need different handling
}

/// Module-level static for signal handler communication.
/// The signal handler writes to this pipe to notify the watcher thread.
#[cfg(unix)]
static SIGWINCH_PIPE: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(-1);

/// SIGWINCH signal handler - writes a byte to the pipe to wake the watcher thread.
#[cfg(unix)]
extern "C" fn sigwinch_handler(_: libc::c_int) {
    let fd = SIGWINCH_PIPE.load(Ordering::SeqCst);
    if fd >= 0 {
        unsafe {
            let _ = libc::write(fd, b"R".as_ptr() as *const libc::c_void, 1);
        }
    }
}

/// Watcher for terminal resize signals (SIGWINCH on Unix).
///
/// Spawns a thread that waits for SIGWINCH and sends Resize messages.
#[cfg(unix)]
pub struct ResizeWatcher {
    handle: Option<JoinHandle<()>>,
    running: Arc<AtomicBool>,
}

#[cfg(unix)]
impl ResizeWatcher {
    /// Spawn the resize watcher thread.
    ///
    /// Uses a self-pipe trick: SIGWINCH writes to a pipe, thread reads from pipe.
    pub fn spawn(tx: Sender<StdinMessage>, running: Arc<AtomicBool>) -> io::Result<Self> {
        use std::os::unix::io::FromRawFd;
        use std::fs::File;

        // Create a pipe for signal notification
        let mut fds = [0i32; 2];
        if unsafe { libc::pipe(fds.as_mut_ptr()) } != 0 {
            return Err(io::Error::last_os_error());
        }

        let read_fd = fds[0];
        let write_fd = fds[1];

        // Set write end to non-blocking
        unsafe {
            let flags = libc::fcntl(write_fd, libc::F_GETFL);
            libc::fcntl(write_fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
        }

        // Store write_fd in the module-level static for signal handler
        SIGWINCH_PIPE.store(write_fd, Ordering::SeqCst);

        // Set up SIGWINCH handler
        unsafe {
            libc::signal(libc::SIGWINCH, sigwinch_handler as *const () as usize);
        }

        // Spawn thread to read from pipe and send resize messages
        let running_clone = running.clone();
        let handle = thread::Builder::new()
            .name("spark-resize".to_string())
            .spawn(move || {
                let mut read_file = unsafe { File::from_raw_fd(read_fd) };
                let mut buf = [0u8; 1];

                while running_clone.load(Ordering::SeqCst) {
                    // Block on pipe read - signal handler writes here
                    use std::io::Read;
                    match read_file.read(&mut buf) {
                        Ok(1) => {
                            // SIGWINCH received, query new size
                            if let Some((w, h)) = get_terminal_size() {
                                let _ = tx.send(StdinMessage::Resize(w, h));
                            }
                        }
                        Ok(_) => continue,
                        Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                        Err(_) => break,
                    }
                }

                // Clean up
                SIGWINCH_PIPE.store(-1, Ordering::SeqCst);
                // Note: read_file will close read_fd on drop
                unsafe { libc::close(write_fd); }
            })?;

        Ok(Self {
            handle: Some(handle),
            running,
        })
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            let _ = handle;
        }
    }
}

#[cfg(unix)]
impl Drop for ResizeWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Dummy resize watcher for non-Unix platforms.
#[cfg(not(unix))]
pub struct ResizeWatcher;

#[cfg(not(unix))]
impl ResizeWatcher {
    pub fn spawn(_tx: Sender<StdinMessage>, _running: Arc<AtomicBool>) -> io::Result<Self> {
        Ok(Self)
    }

    pub fn stop(&mut self) {}
}

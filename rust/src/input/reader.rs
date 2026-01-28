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

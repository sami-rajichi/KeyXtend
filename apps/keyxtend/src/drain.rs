//! The kernel's thread: it drains the UI's requests until Quit ends the queue. Try again restarts a failed module;
//! Quit stops every module, then ends the UI, even when stopping panics.

use kx_kernel::Kernel;
use kx_ui::{Hub, Request};
use std::io;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::thread::{self, JoinHandle};

/// The kernel thread's name, as debuggers show it.
const THREAD: &str = "kernel";

/// Runs `drain` on the kernel's own thread, which takes the kernel.
///
/// # Errors
/// When the thread cannot start; the kernel then stops its modules as it drops.
pub fn spawn(
    kernel: Kernel,
    hub: Arc<Hub>,
    requests: Receiver<Request>,
) -> io::Result<JoinHandle<()>> {
    let run = move || drain(kernel, &hub, &requests);
    thread::Builder::new().name(THREAD.to_owned()).spawn(run)
}

/// Handles each request until the queue ends, which it does right after Quit.
pub fn drain(mut kernel: Kernel, hub: &Hub, requests: &Receiver<Request>) {
    while let Ok(request) = requests.recv() {
        match request {
            Request::Retry(module) => {
                if let Err(e) = kernel.retry(module) {
                    tracing::warn!(module = %module, error = %e, "try again was refused");
                }
            }
            Request::Quit => {
                let _end = EndUi(hub);
                kernel.stop_all();
            }
        }
    }
}

/// Stops the app after Qt's loop: asks the kernel's thread to quit, unless it was asked already, removes the tray icon
/// and waits for the thread. False when the thread ended in a panic, which is logged.
pub fn stop(hub: &Hub, kernel: JoinHandle<()>) -> bool {
    let _ = hub.quit();
    hub.tray_hide();
    let joined = kernel.join().is_ok();
    if !joined {
        tracing::error!("the kernel's thread ended in a panic");
    }
    joined
}

/// Ends the UI when it drops, so Quit ends it even if stopping the modules panics.
struct EndUi<'a>(&'a Hub);

impl Drop for EndUi<'_> {
    fn drop(&mut self) {
        self.0.end_ui();
    }
}

#[cfg(test)]
mod tests;

//! The hold engine's public side: start its hook thread, switch modes, read live values, stop.

use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use spike_core::hold::{Act, Mode};
use spike_core::inject;
use windows::Win32::Foundation::{LPARAM, RECT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{PostThreadMessageW, WM_QUIT};

use crate::hookhost::{self, Call, Cmd, WAKE};
pub use crate::hookhost::{Live, Report};
use crate::hookio::{self, SIM_TAG, Source};
use crate::win;

/// How the engine runs and whose input it acts on.
pub struct Setup {
    /// A still left hold fires after this, in ms.
    pub hold_ms: u64,
    /// Moves up to this many px count as still.
    pub still_px: i32,
    /// Whose input the engine acts on.
    pub source: Source,
    /// Our own windows: presses there are never held.
    pub own: Vec<RECT>,
    /// Longest wait for the hook thread to start or answer, in ms.
    pub reply_ms: u64,
}

/// The running engine and its hook thread.
pub struct Assist {
    tid: u32,
    tx: Sender<Call>,
    live: Arc<Mutex<Live>>,
    reply: Duration,
    thread: Option<JoinHandle<Report>>,
}

impl Assist {
    /// Starts the hook thread in `Off` mode and waits until its hooks are in.
    pub fn start(setup: Setup) -> Result<Self, String> {
        let (tx, rx) = mpsc::channel::<Call>();
        let (ready_tx, ready_rx) = mpsc::channel();
        let live = Arc::new(Mutex::new(Live::default()));
        let reply = Duration::from_millis(setup.reply_ms);
        let shared = Arc::clone(&live);
        let thread = std::thread::spawn(move || hookhost::run(setup, rx, shared, ready_tx));
        let tid = ready_rx
            .recv_timeout(reply)
            .map_err(|e| format!("assist hooks: {e}"))??;
        Ok(Self {
            tid,
            tx,
            live,
            reply,
            thread: Some(thread),
        })
    }

    /// Sends `cmd` and waits until the hook thread has applied it and injected its input.
    fn call(&self, cmd: Cmd) -> Result<(), String> {
        let (ack_tx, ack_rx) = mpsc::channel();
        self.tx
            .send((cmd, ack_tx))
            .map_err(|_| "the assist thread has ended".to_string())?;
        // SAFETY: posting to the hook thread's queue, which exists once it is ready.
        unsafe { PostThreadMessageW(self.tid, WAKE, WPARAM(0), LPARAM(0)) }
            .map_err(|e| format!("wake the assist thread: {e}"))?;
        ack_rx
            .recv_timeout(self.reply)
            .map_err(|e| format!("assist thread did not answer: {e}"))
    }

    /// Switches the engine's mode.
    pub fn set_mode(&self, mode: Mode) -> Result<(), String> {
        self.call(Cmd::Mode(mode))
    }

    /// Holds modifier `vk` down until the next left click ends.
    pub fn latch(&self, vk: u16) -> Result<(), String> {
        self.call(Cmd::Latch(vk))
    }

    /// The live values right now.
    pub fn live(&self) -> Live {
        self.live.lock().map(|l| *l).unwrap_or_default()
    }

    /// Stops the hook thread; one that does not answer is told to quit, and still lets go of everything.
    fn finish(&mut self) -> Option<Report> {
        let thread = self.thread.take()?;
        if self.call(Cmd::Stop).is_err() {
            // SAFETY: posting to the hook thread's queue; its loop ends on WM_QUIT.
            let _ = unsafe { PostThreadMessageW(self.tid, WM_QUIT, WPARAM(0), LPARAM(0)) };
        }
        thread.join().ok()
    }

    /// Lets go of everything held, removes the hooks and returns what they measured.
    pub fn stop(mut self) -> Result<Report, String> {
        self.finish()
            .ok_or_else(|| "the assist thread did not end cleanly".to_string())
    }
}

impl Drop for Assist {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

/// Plays `acts` as the simulated user, whom automated runs act on.
pub fn as_user(acts: &[Act]) -> Result<(), String> {
    inject::send(&hookio::inputs(acts, win::desktop(), SIM_TAG))
}

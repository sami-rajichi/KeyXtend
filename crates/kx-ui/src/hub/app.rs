//! Between the app and the UI: the Qt thread's waker and the keyboard's mode, both handed over to rebuilt windows, a
//! second copy's commands, Try again, and the one quit path.
//! Quit is the last request: every request is sent under the hub's lock, and the sender is dropped after Quit.

use super::{Hub, Request, Wake, Waker, json};
use kx_module_api::ports::FaceAction;
use kx_platform::ui::Command;
use serde::Serialize;
use std::sync::Arc;

/// How the user left the keyboard, which windows made again keep.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    /// It shows, once the keyboard module runs.
    #[default]
    Shown,
    /// Close put it away until Show.
    Away,
    /// Minimise put it away behind its bubble.
    Minimised,
}

/// The mode as QML's `away` and `minimised` hold it.
#[derive(Serialize)]
struct Flags {
    away: bool,
    minimised: bool,
}

impl Mode {
    /// The mode QML's `away` and `minimised` describe; a minimised keyboard is away too.
    #[must_use]
    pub fn of(away: bool, minimised: bool) -> Self {
        match (away, minimised) {
            (_, true) => Self::Minimised,
            (true, false) => Self::Away,
            (false, false) => Self::Shown,
        }
    }
}

impl Hub {
    /// Keeps how the user left the keyboard, for windows made again. Hiding it mid-glide takes Stop off the screen, so
    /// the keyboard stops the glide as Stop does; call it on the Qt thread.
    pub fn keep_mode(&self, mode: Mode) {
        let before = std::mem::replace(&mut self.lock().mode, mode);
        if before == Mode::Shown && mode != Mode::Shown && self.scrolling() {
            let _ = self.act(FaceAction::Pad { dir: None });
        }
    }

    /// How the user left the keyboard.
    #[must_use]
    pub fn mode(&self) -> Mode {
        self.lock().mode
    }

    /// The mode as JSON `{away, minimised}`, as QML holds it.
    #[must_use]
    pub fn mode_json(&self) -> String {
        let mode = self.mode();
        json(&Flags {
            away: mode != Mode::Shown,
            minimised: mode == Mode::Minimised,
        })
    }

    /// Registers the Qt thread's waker and hands it the `Show` and `End` wakes that came first, and an overlay or scroll
    /// wake QML has not read. A second waker replaces the first, which then hears nothing; that is a bug, so it warns.
    pub fn set_waker(&self, waker: Waker) {
        let (pending, replaced) = {
            let mut live = self.lock();
            let replaced = live.waker.replace(Arc::new(waker)).is_some();
            (std::mem::take(&mut live.pending), replaced)
        };
        if replaced {
            tracing::warn!("a second Ui object took the hub's wakes; make one per app");
        }
        for wake in pending {
            self.wake(wake);
        }
        self.rewake_assist();
        self.rewake_scroll();
    }

    /// Lets the waker go before the windows are made again: `Show` and `End` wait for the next `Ui` object, and an `End`
    /// already sent goes to it again, as the last one may not have run it. The new keyboard's guard run starts afresh.
    pub fn drop_waker(&self) {
        self.forget_guard_end();
        let mut live = self.lock();
        live.waker = None;
        if live.ended && !live.pending.contains(&Wake::End) {
            live.pending.push(Wake::End);
        }
    }

    /// True once the app ended the UI.
    #[must_use]
    pub fn ended(&self) -> bool {
        self.lock().ended
    }

    /// A second copy's `command`: Show goes to the UI, Quit to the app, as the user's Quit does.
    pub fn command(&self, command: Command) {
        match command {
            Command::Show => self.wake_or_wait(Wake::Show),
            Command::Quit => {
                let _ = self.quit();
            }
        }
    }

    /// The app stopped the modules after Quit: the UI ends. It waits for the waker if none is set yet.
    pub fn end_ui(&self) {
        self.wake_or_wait(Wake::End);
    }

    /// Asks the app to quit and ends its queue; false when it was asked already or no longer listens.
    /// When the app's drain thread is gone, nobody else can end the UI, so the hub ends it itself.
    pub fn quit(&self) -> bool {
        let sent = {
            let mut live = self.lock();
            let Some(requests) = live.requests.take() else {
                return false;
            };
            requests.send(Request::Quit).is_ok()
        };
        if !sent {
            self.end_ui();
        }
        sent
    }

    /// Try again on shown notice `id`: it closes, and the app restarts its module; false when it offers none.
    /// It sends under the lock, which never blocks on an unbounded queue, so no Retry lands after Quit.
    pub fn retry(&self, id: u64) -> bool {
        let mut live = self.lock();
        let Some(module) = live.notices.close(id) else {
            return false;
        };
        live.requests
            .as_ref()
            .is_some_and(|r| r.send(Request::Retry(module)).is_ok())
    }

    /// Wakes the UI with `what`, or keeps it until the waker is set; an `End` is noted under the same lock.
    fn wake_or_wait(&self, what: Wake) {
        let waiting = {
            let mut live = self.lock();
            live.ended |= what == Wake::End;
            let none = live.waker.is_none();
            if none {
                live.pending.push(what);
            }
            none
        };
        if !waiting {
            self.wake(what);
        }
    }
}

//! The Windows tray icon: a hidden top-level window per icon, whose thread adds, changes and deletes it, adds it
//! again when the taskbar restarts, hears its clicks and watches the front window. No lock is held during a call.

#[cfg(test)]
mod fake_shell;
mod front;
mod icon;
mod msgs;
#[cfg(test)]
mod tests;

use std::cell::OnceCell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use kx_platform::ui::{OnTray, Tray, TrayEvent, TrayIcon, TrayImage, UiError};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Shell::{
    NIF_ICON, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY, NIM_SETVERSION,
    NOTIFY_ICON_MESSAGE, NOTIFYICON_VERSION_4, NOTIFYICONDATAW, NOTIFYICONDATAW_0,
};
use windows::Win32::UI::WindowsAndMessaging::{WM_APP, WM_CREATE, WM_DESTROY};

use super::{TIP_UNITS, event};
use crate::hidden::{Hidden, Kind, answer};
use crate::wide;
use front::{hook, unhook};
use icon::{Icon, Shell, Taskbar, small_px};
use msgs::{allow, post, restart_msg};

/// Class of the icon's window, local to our process.
const CLASS: &str = "KeyXtend.Tray";
/// The icon's id under its window, which holds one icon.
const ICON_ID: u32 = 1;
/// The message the taskbar sends with the icon's events.
const CALLBACK_MSG: u32 = WM_APP + 1;
/// Asks the icon's thread to add the icon from its queue, so `show` never waits on the taskbar.
const ADD_MSG: u32 = WM_APP + 2;
/// Asks the icon's thread to send the owner's change, so `set` never waits on the taskbar either.
const SET_MSG: u32 = WM_APP + 3;

thread_local! {
    /// The icon's listener and state, on its window's thread only.
    static HERE: OnceCell<Here> = const { OnceCell::new() };
}

/// The Windows tray: each shown icon gets its own hidden window and thread.
#[derive(Debug)]
pub struct WinTray {
    class: Vec<u16>,
    shell: Arc<dyn Shell>,
}

impl WinTray {
    /// The tray on the real taskbar.
    #[must_use]
    pub fn new() -> Self {
        Self::with(CLASS, Arc::new(Taskbar))
    }

    /// A tray whose windows have class `class` and which calls `shell`, so a test never meets the real taskbar.
    pub(crate) fn with(class: &str, shell: Arc<dyn Shell>) -> Self {
        Self {
            class: wide(class),
            shell,
        }
    }
}

impl Default for WinTray {
    fn default() -> Self {
        Self::new()
    }
}

impl Tray for WinTray {
    fn icon_px(&self) -> u32 {
        small_px()
    }

    /// Adds the icon from its window's thread; a taskbar that is not there yet gets it when it starts.
    fn show(&self, image: &TrayImage, tip: &str, on: OnTray) -> Result<Box<dyn TrayIcon>, UiError> {
        let state = Arc::new(Mutex::new(State {
            icon: Arc::new(Icon::new(image)?),
            tip: super::tip(tip),
            window: 0,
            added: false,
            gone: false,
        }));
        let here = Here {
            on,
            state: Arc::clone(&state),
            shell: Arc::clone(&self.shell),
            restart: restart_msg(),
        };
        let setup = move || drop(HERE.with(|h| h.set(here)));
        let hidden = Hidden::start(self.class.clone(), Kind::TopLevel, Some(proc), setup)?;
        Ok(Box::new(WinIcon {
            state,
            hidden: Some(hidden),
        }))
    }
}

/// What the icon's owner and its window's thread share.
#[derive(Debug)]
struct State {
    icon: Arc<Icon>,
    tip: [u16; TIP_UNITS],
    /// The window's handle as a number, 0 until it exists.
    window: usize,
    /// The taskbar holds the icon, so a change is sent now; until then the coming add carries it.
    added: bool,
    /// Set as the owner drops the icon, so no later add brings it back.
    gone: bool,
}

impl State {
    /// The taskbar's record of the icon.
    fn data(&self) -> NOTIFYICONDATAW {
        NOTIFYICONDATAW {
            cbSize: u32::try_from(size_of::<NOTIFYICONDATAW>()).unwrap_or_default(),
            hWnd: HWND(std::ptr::without_provenance_mut(self.window)),
            uID: ICON_ID,
            uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_SHOWTIP,
            uCallbackMessage: CALLBACK_MSG,
            hIcon: self.icon.handle(),
            szTip: self.tip,
            Anonymous: NOTIFYICON_VERSION_4_DATA,
            ..NOTIFYICONDATAW::default()
        }
    }
}

/// Sends `what` once the window exists and `ok` holds. The record is taken under the lock and sent after it, as the
/// taskbar call can run this thread's other messages; `None` when nothing was sent.
fn send(
    state: &Mutex<State>,
    shell: &dyn Shell,
    what: NOTIFY_ICON_MESSAGE,
    ok: impl FnOnce(&State) -> bool,
) -> Option<bool> {
    // The icon stays alive until the taskbar has copied it, even if the owner changes it meanwhile.
    let (data, _icon) = {
        let s = lock(state);
        if s.window == 0 || !ok(&s) {
            return None;
        }
        (s.data(), Arc::clone(&s.icon))
    };
    Some(shell.notify(what, &data))
}

/// The version the icon asks for: clicks as `NIN_SELECT` with the anchor in the first value.
const NOTIFYICON_VERSION_4_DATA: NOTIFYICONDATAW_0 = NOTIFYICONDATAW_0 {
    uVersion: NOTIFYICON_VERSION_4,
};

/// Locks `state`, taking it even when a panicking thread poisoned it.
fn lock(state: &Mutex<State>) -> MutexGuard<'_, State> {
    state.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A shown icon; dropping it deletes the icon and ends its window.
#[derive(Debug)]
struct WinIcon {
    state: Arc<Mutex<State>>,
    hidden: Option<Hidden>,
}

impl TrayIcon for WinIcon {
    /// Keeps the change and queues it for the icon's thread; before the window exists, the coming add carries it.
    fn set(&self, image: &TrayImage, tip: &str) -> Result<(), UiError> {
        let icon = Arc::new(Icon::new(image)?);
        let (old, window) = {
            let mut state = lock(&self.state);
            state.tip = super::tip(tip);
            (std::mem::replace(&mut state.icon, icon), state.window)
        };
        drop(old);
        if window != 0 {
            post(HWND(std::ptr::without_provenance_mut(window)), SET_MSG);
        }
        Ok(())
    }
}

impl Drop for WinIcon {
    /// The window's thread deletes the icon as its window ends, after any add it had queued.
    fn drop(&mut self) {
        lock(&self.state).gone = true;
        if let Some(hidden) = self.hidden.take() {
            hidden.stop();
        }
    }
}

/// The icon as its window's thread sees it.
struct Here {
    on: OnTray,
    state: Arc<Mutex<State>>,
    shell: Arc<dyn Shell>,
    /// The message the taskbar broadcasts when it starts; 0 when Windows would not name it.
    restart: u32,
}

impl Here {
    /// Handles one message to the icon's window.
    fn hear(&self, hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) {
        match msg {
            WM_CREATE => self.start(hwnd),
            WM_DESTROY => {
                let _ = send(&self.state, self.shell.as_ref(), NIM_DELETE, |_| true);
                unhook();
            }
            ADD_MSG => self.add(),
            SET_MSG => {
                let held = |s: &State| s.added && !s.gone;
                let _ = send(&self.state, self.shell.as_ref(), NIM_MODIFY, held);
            }
            CALLBACK_MSG => {
                if let Some(e) = event(lparam.0, wparam.0) {
                    self.tell(e);
                }
            }
            m if m == self.restart && m != 0 => {
                lock(&self.state).added = false;
                post(hwnd, ADD_MSG);
            }
            _ => {}
        }
    }

    /// Notes the new window, lets the taskbar reach it, watches the front window and queues the add.
    fn start(&self, hwnd: HWND) {
        lock(&self.state).window = hwnd.0.addr();
        allow(hwnd, CALLBACK_MSG);
        allow(hwnd, self.restart);
        hook();
        post(hwnd, ADD_MSG);
    }

    /// Adds the icon and asks for its version; an icon the taskbar still holds is changed instead. A change the owner
    /// makes meanwhile is queued behind this add, so it follows it.
    fn add(&self) {
        let shell = self.shell.as_ref();
        let live = |s: &State| !s.gone;
        let added = send(&self.state, shell, NIM_ADD, live) == Some(true)
            || send(&self.state, shell, NIM_MODIFY, live) == Some(true);
        if added {
            let _ = send(&self.state, shell, NIM_SETVERSION, live);
            lock(&self.state).added = true;
        }
    }

    /// Calls the listener; a panic must not cross into Windows, which would end the program.
    fn tell(&self, event: TrayEvent) {
        let _ = catch_unwind(AssertUnwindSafe(|| (self.on)(event)));
    }
}

/// Hands each message to the icon; every message then gets the hidden windows' default handling.
#[allow(unsafe_code, reason = "The window procedure of the icon's window.")]
unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    HERE.with(|h| h.get().map(|here| here.hear(hwnd, msg, wparam, lparam)));
    // SAFETY: our own window's message, with its own values.
    unsafe { answer(hwnd, msg, wparam, lparam) }
}

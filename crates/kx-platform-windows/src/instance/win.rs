//! The Windows side of the one running copy: the session mutex and the message-only command window.
//! Windows keeps window messages inside one desktop, so only programs in this user session can reach the window.

use std::cell::OnceCell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use kx_platform::ui::{Claim, Command, Instance, Running, UiError};
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, HANDLE, HWND, LPARAM, LRESULT, WPARAM,
};
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::WindowsAndMessaging::{FindWindowExW, HWND_MESSAGE, PostMessageW, WM_APP};
use windows::core::PCWSTR;

use super::{Hold, class_name, code, from_code, hold_of, mutex_name};
use crate::hidden::{Hidden, Kind, answer};
use crate::{refused, wide};

/// The message a later copy posts, with the command's code as its first value.
const COMMAND_MSG: u32 = WM_APP;

/// Called with each command a later copy sends.
type OnCommand = Box<dyn Fn(Command) + Send>;
/// Where the running copy keeps its listener, shared with the command window's thread.
type Slot = Arc<Mutex<Option<OnCommand>>>;

thread_local! {
    /// The listener slot, on the command window's thread only.
    static SLOT: OnceCell<Slot> = const { OnceCell::new() };
}

/// One running copy per user session, named after the app.
#[derive(Clone, Debug)]
pub struct WinInstance {
    mutex: Vec<u16>,
    class: Vec<u16>,
}

impl WinInstance {
    /// The instance of app `app`, whose name makes the mutex and the command window's class.
    #[must_use]
    pub fn new(app: &str) -> Self {
        Self {
            mutex: wide(&mutex_name(app)),
            class: wide(&class_name(app)),
        }
    }
}

impl Instance for WinInstance {
    fn claim(&self) -> Result<Box<dyn Claim>, Running> {
        let mutex = hold(&self.mutex)?;
        Ok(Box::new(WinClaim {
            mutex,
            class: self.class.clone(),
            slot: Slot::default(),
            listener: None,
        }))
    }

    #[allow(
        unsafe_code,
        reason = "Finds the command window and posts it a number."
    )]
    fn send(&self, command: Command) -> Result<(), UiError> {
        let class = PCWSTR(self.class.as_ptr());
        // SAFETY: the class name ends with a null; only message-only windows are searched.
        let window = unsafe { FindWindowExW(Some(HWND_MESSAGE), None, class, PCWSTR::null()) }
            .map_err(|_| UiError::NoWindow)?;
        let (wparam, lparam) = (WPARAM(code(command)), LPARAM(0));
        // SAFETY: posts a plain number; nothing points into our memory.
        unsafe { PostMessageW(Some(window), COMMAND_MSG, wparam, lparam) }.map_err(|e| refused(&e))
    }
}

/// Makes the session mutex `name`: `Some` when this copy made it, `None` when Windows could not check.
#[allow(unsafe_code, reason = "Makes a named mutex that `Owned` closes.")]
fn hold(name: &[u16]) -> Result<Option<Owned>, Running> {
    // SAFETY: `name` ends with a null and outlives the call.
    let made = unsafe { CreateMutexW(None, false, PCWSTR(name.as_ptr())) };
    // SAFETY: a plain query, read right after the call it describes.
    let error = unsafe { GetLastError() }.0;
    let handle = made.ok().map(Owned);
    match hold_of(handle.is_some(), error) {
        Hold::Ours => Ok(handle),
        Hold::Taken => Err(Running),
        Hold::Unknown => Ok(None),
    }
}

/// A kernel handle we own, closed once when dropped.
#[derive(Debug)]
struct Owned(HANDLE);

// SAFETY: a kernel handle is a process-wide index; any thread may use or close it.
#[allow(unsafe_code, reason = "A kernel handle may move between threads.")]
unsafe impl Send for Owned {}

impl Drop for Owned {
    #[allow(unsafe_code, reason = "Closes the handle we own.")]
    fn drop(&mut self) {
        // SAFETY: closes a handle Windows gave us, once.
        let _ = unsafe { CloseHandle(self.0) };
    }
}

/// The running copy's claim: the session mutex and, once it listens, the command window.
struct WinClaim {
    mutex: Option<Owned>,
    class: Vec<u16>,
    slot: Slot,
    listener: Option<Hidden>,
}

impl Claim for WinClaim {
    /// The first call opens the command window on its own thread, where `on` runs; a later call replaces `on`.
    fn listen(&mut self, on: OnCommand) -> Result<(), UiError> {
        let earlier = lock(&self.slot).replace(on);
        if self.listener.is_some() {
            return Ok(());
        }
        let slot = Arc::clone(&self.slot);
        let setup = move || drop(SLOT.with(|s| s.set(slot)));
        match Hidden::start(self.class.clone(), Kind::MessageOnly, Some(proc), setup) {
            Ok(listener) => self.listener = Some(listener),
            Err(e) => {
                *lock(&self.slot) = earlier;
                return Err(e);
            }
        }
        Ok(())
    }
}

impl Drop for WinClaim {
    fn drop(&mut self) {
        if let Some(listener) = self.listener.take() {
            listener.stop();
        }
        drop(self.mutex.take());
    }
}

/// Locks `slot`, taking the listener even when a panicking one poisoned it.
fn lock(slot: &Slot) -> MutexGuard<'_, Option<OnCommand>> {
    slot.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Hands each known command to the listener; any other message gets Windows' default handling.
#[allow(unsafe_code, reason = "The window procedure of our command window.")]
unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg != COMMAND_MSG {
        // SAFETY: our own window's message, with its own values.
        return unsafe { answer(hwnd, msg, wparam, lparam) };
    }
    if let Some(command) = from_code(wparam.0) {
        SLOT.with(|s| s.get().map(|slot| heard(slot, command)));
    }
    LRESULT::default()
}

/// Calls the listener with `command` outside the lock, so it may wait for a thread that calls `listen`.
/// A panic must not cross into Windows, which would end the program; a listener set during the call wins.
fn heard(slot: &Slot, command: Command) {
    let Some(on) = lock(slot).take() else {
        return;
    };
    let _ = catch_unwind(AssertUnwindSafe(|| on(command)));
    let mut now = lock(slot);
    if now.is_none() {
        *now = Some(on);
    }
}

#[cfg(test)]
mod tests;

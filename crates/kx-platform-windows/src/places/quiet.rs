//! Windows' disk dialogs off on this thread while a place is touched, so an empty card reader never pops "There is no
//! disk in the drive" over the app in front.

#[cfg(test)]
use std::cell::Cell;
use std::marker::PhantomData;

use windows::Win32::System::Diagnostics::Debug::{
    GetThreadErrorMode, SEM_FAILCRITICALERRORS, SEM_NOOPENFILEERRORBOX, SetThreadErrorMode,
    THREAD_ERROR_MODE,
};

#[cfg(test)]
thread_local! {
    /// The error mode this thread last touched a place under, for the tests that prove each call touches it quiet.
    static TOUCHED: Cell<Option<u32>> = const { Cell::new(None) };
}

/// Keeps the disk dialogs off on this thread until it drops, then gives the thread its mode back.
pub(super) struct Quiet {
    /// The mode the thread had; `None` when Windows refused the change, so there is nothing to give back.
    old: Option<THREAD_ERROR_MODE>,
    /// Not `Send`: the error mode belongs to the thread that set it.
    _thread: PhantomData<*const ()>,
}

impl Quiet {
    /// Turns the dialogs off on this thread, keeping its other mode bits.
    #[allow(unsafe_code, reason = "Sets this thread's error mode.")]
    pub(super) fn new() -> Self {
        let off = THREAD_ERROR_MODE(mode()) | SEM_FAILCRITICALERRORS | SEM_NOOPENFILEERRORBOX;
        let mut old = THREAD_ERROR_MODE(0);
        // SAFETY: sets a flag word for this thread only; `old` is a local Windows writes once.
        let set = unsafe { SetThreadErrorMode(off, Some(&raw mut old)) };
        Self {
            old: set.ok().map(|()| old),
            _thread: PhantomData,
        }
    }
}

impl Drop for Quiet {
    #[allow(unsafe_code, reason = "Gives this thread its error mode back.")]
    fn drop(&mut self) {
        if let Some(old) = self.old {
            // SAFETY: sets back the flag word this thread had, and asks for no old mode.
            let _ = unsafe { SetThreadErrorMode(old, None) };
        }
    }
}

/// This thread's error mode.
#[allow(unsafe_code, reason = "Reads this thread's error mode.")]
pub(super) fn mode() -> u32 {
    // SAFETY: takes nothing and only returns this thread's mode.
    unsafe { GetThreadErrorMode() }
}

/// Marks a touch of a place's folder or file; a test build notes this thread's error mode then.
pub(super) fn touch() {
    #[cfg(test)]
    TOUCHED.with(|t| t.set(Some(mode())));
}

/// The error mode this thread last touched a place under, forgetting it, for tests.
#[cfg(test)]
pub(super) fn touched() -> Option<u32> {
    TOUCHED.with(Cell::take)
}

/// Sets this thread's error mode to `mode`, for tests.
#[cfg(test)]
#[allow(unsafe_code, reason = "Sets this thread's error mode.")]
pub(super) fn set_mode(mode: u32) {
    // SAFETY: sets a flag word for this thread only, and asks for no old mode.
    unsafe { SetThreadErrorMode(THREAD_ERROR_MODE(mode), None) }.unwrap();
}

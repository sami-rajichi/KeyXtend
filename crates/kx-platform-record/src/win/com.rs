//! COM on one thread, ended when dropped on the same thread.

use std::marker::PhantomData;

use windows::Win32::System::Com::{COINIT, CoInitializeEx, CoUninitialize};

/// COM on this thread until dropped; the marker keeps it on the thread that started it.
pub(crate) struct Com(PhantomData<*const ()>);

impl Com {
    /// Starts COM on this thread in `mode`.
    ///
    /// # Errors
    /// The system's error when COM would not start in that mode.
    #[allow(unsafe_code, reason = "Starts COM on this thread.")]
    pub(crate) fn start(mode: COINIT) -> windows::core::Result<Self> {
        // SAFETY: a plain start; `Drop` ends it once, on this thread, as the marker keeps it here.
        unsafe { CoInitializeEx(None, mode) }.ok()?;
        Ok(Self(PhantomData))
    }
}

impl Drop for Com {
    #[allow(unsafe_code, reason = "Ends COM on this thread.")]
    fn drop(&mut self) {
        // SAFETY: the start succeeded on this thread.
        unsafe { CoUninitialize() };
    }
}

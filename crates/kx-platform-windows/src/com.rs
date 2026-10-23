//! COM start-up for one thread, for UI Automation and the Windows Runtime.

use std::marker::PhantomData;
use std::sync::OnceLock;

use windows::Win32::Foundation::RPC_E_CHANGED_MODE;
use windows::Win32::System::Com::{
    COINIT, COINIT_APARTMENTTHREADED, COINIT_MULTITHREADED, CoIncrementMTAUsage, CoInitializeEx,
    CoUninitialize,
};

use crate::PlatformError;

/// COM on this thread; dropping it undoes our start-up, if we did one.
#[derive(Debug)]
pub struct Com {
    started: bool,
    /// Not `Send`: COM must be undone on the thread that started it.
    _thread: PhantomData<*const ()>,
}

impl Com {
    /// Starts COM multithreaded for this thread; a thread already in another mode still works.
    ///
    /// # Errors
    /// `PlatformError::Com` when Windows refuses.
    pub fn start() -> Result<Self, PlatformError> {
        Self::begin(COINIT_MULTITHREADED)
    }

    /// Starts COM apartment-threaded for this thread, as shell objects want; a thread already in another mode still
    /// works.
    ///
    /// # Errors
    /// `PlatformError::Com` when Windows refuses.
    pub fn start_apartment() -> Result<Self, PlatformError> {
        Self::begin(COINIT_APARTMENTTHREADED)
    }

    /// Starts COM for this thread in `mode`, accepting a thread already in the other one.
    #[allow(unsafe_code, reason = "COM start-up for the calling thread.")]
    fn begin(mode: COINIT) -> Result<Self, PlatformError> {
        // SAFETY: COM start-up for the calling thread; paired in `drop` when it succeeded.
        let hr = unsafe { CoInitializeEx(None, mode) };
        if hr.is_err() && hr != RPC_E_CHANGED_MODE {
            return Err(PlatformError::Com(windows::core::Error::from(hr).into()));
        }
        Ok(Self {
            started: hr.is_ok(),
            _thread: PhantomData,
        })
    }

    /// Keeps COM on for the rest of this thread's life, for objects that may outlive their owner.
    pub fn keep(self) {
        std::mem::forget(self);
    }
}

/// Keeps the process's multithreaded apartment alive for good, once, before the first Windows Runtime object is made.
/// It moves no thread into an apartment; windows-rs caches factories for the whole process, so COM must outlive
/// any thread that made one, such as Qt's GUI thread.
#[allow(unsafe_code, reason = "Keeps the process-wide apartment alive.")]
pub(crate) fn keep_mta() -> bool {
    static KEPT: OnceLock<bool> = OnceLock::new();
    // SAFETY: a process-wide count with no thread state; the cookie is kept, never handed back.
    *KEPT.get_or_init(|| unsafe { CoIncrementMTAUsage() }.is_ok())
}

impl Drop for Com {
    #[allow(unsafe_code, reason = "COM shut-down that pairs `start`.")]
    fn drop(&mut self) {
        if self.started {
            // SAFETY: pairs the successful CoInitializeEx in `start`, on the same thread.
            unsafe { CoUninitialize() };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn com_starts_twice_on_one_thread_and_each_guard_drops() {
        let first = Com::start().expect("first start");
        let second = Com::start().expect("second start");
        assert!(first.started && second.started);
        drop(second);
        drop(first);
    }

    #[test]
    #[allow(unsafe_code, reason = "asks a test thread's COM mode.")]
    fn the_apartment_start_puts_a_fresh_thread_in_an_apartment() {
        std::thread::spawn(|| {
            let com = Com::start_apartment().expect("apartment start");
            assert!(com.started);
            // SAFETY: asks for the other mode on this test's own thread, which refuses without changing it.
            let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
            assert_eq!(hr, RPC_E_CHANGED_MODE, "the thread is apartment-threaded");
            drop(com);
        })
        .join()
        .expect("the test thread ends");
    }

    #[test]
    fn an_apartment_start_on_a_multithreaded_thread_still_works_without_undoing_it() {
        std::thread::spawn(|| {
            let mta = Com::start().expect("multithreaded start");
            let com = Com::start_apartment().expect("works in the other mode");
            assert!(!com.started);
            drop(com);
            drop(mta);
        })
        .join()
        .expect("the test thread ends");
    }

    #[test]
    #[allow(unsafe_code, reason = "puts a test thread in the other COM mode.")]
    fn a_thread_in_the_other_mode_still_works_without_undoing_it() {
        std::thread::spawn(|| {
            // SAFETY: starts COM in the other mode on this test's own thread; undone below.
            let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
            assert!(hr.is_ok());
            let com = Com::start().expect("works in the other mode");
            assert!(!com.started);
            drop(com);
            // SAFETY: pairs the test's own start-up above, on the same thread.
            unsafe { CoUninitialize() };
        })
        .join()
        .expect("the test thread ends");
    }
}

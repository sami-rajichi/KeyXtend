//! COM start-up for one thread, for UI Automation and the Windows Runtime.

use std::marker::PhantomData;

use windows::Win32::Foundation::RPC_E_CHANGED_MODE;
use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize};

use crate::PlatformError;

/// COM on this thread; dropping it undoes our start-up, if we did one.
#[derive(Debug)]
pub struct Com {
    started: bool,
    /// Not `Send`: COM must be undone on the thread that started it.
    _thread: PhantomData<*const ()>,
}

impl Com {
    /// Starts COM for this thread; a thread already in another mode still works.
    ///
    /// # Errors
    /// `PlatformError::Com` when Windows refuses.
    #[allow(unsafe_code, reason = "COM start-up for the calling thread.")]
    pub fn start() -> Result<Self, PlatformError> {
        // SAFETY: COM start-up for the calling thread; paired in `drop` when it succeeded.
        let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
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
    #[allow(unsafe_code, reason = "puts a test thread in the other COM mode.")]
    fn a_thread_in_the_other_mode_still_works_without_undoing_it() {
        use windows::Win32::System::Com::COINIT_APARTMENTTHREADED;
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

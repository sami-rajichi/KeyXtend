//! COM start-up for one thread, shared by UI Automation and quick-fill.

use std::marker::PhantomData;

use windows::Win32::Foundation::RPC_E_CHANGED_MODE;
use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize};

/// COM on this thread; dropping it undoes our start-up, if we did one.
pub struct Com {
    started: bool,
    /// Not `Send`: COM must be undone on the thread that started it.
    _thread: PhantomData<*const ()>,
}

impl Com {
    /// Starts COM for this thread; a thread already in another mode still works.
    pub fn start() -> Result<Com, String> {
        // SAFETY: COM start-up for the calling thread; paired in `drop` when it succeeded.
        let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if hr.is_err() && hr != RPC_E_CHANGED_MODE {
            return Err(format!("COM start-up: {hr:?}"));
        }
        Ok(Com {
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
        drop(second);
        drop(first);
    }
}

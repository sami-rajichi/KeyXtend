//! The recording's clock: a high-resolution waitable timer wakes the run at each wait's end, so it waits on an event
//! and never sleeps.

use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::time::{Duration, Instant};

use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Threading::{
    CREATE_WAITABLE_TIMER_HIGH_RESOLUTION, CreateWaitableTimerExW, INFINITE, SetWaitableTimer,
    TIMER_ALL_ACCESS, WaitForSingleObject,
};
use windows::core::PCWSTR;

use super::refused;
use crate::record::{Clock, RecordFault};

/// Nanoseconds in a second.
const NS_S: u64 = 1_000_000_000;
/// The timer counts in steps of this many nanoseconds.
const STEP_NS: u128 = 100;
/// The longest wait in ms: one below `INFINITE`, so a wait always ends.
const LONGEST_WAIT: u32 = INFINITE - 1;

/// A clock whose waits wake by a timer of ours, each at most one slot of `fps` late.
pub(super) struct Pace {
    timer: OwnedHandle,
    slot: Duration,
}

impl Pace {
    /// A clock for frames at `fps`.
    #[allow(unsafe_code, reason = "Makes a timer of our own.")]
    pub(super) fn new(fps: u32) -> Result<Self, RecordFault> {
        let flags = CREATE_WAITABLE_TIMER_HIGH_RESOLUTION;
        // SAFETY: an unnamed timer with default security; its handle goes straight to an owner.
        let made =
            unsafe { CreateWaitableTimerExW(None, PCWSTR::null(), flags, TIMER_ALL_ACCESS.0) }
                .map_err(refused)?;
        // SAFETY: Windows just made this handle, and only the owner closes it.
        let timer = unsafe { OwnedHandle::from_raw_handle(made.0) };
        Ok(Self {
            timer,
            slot: Duration::from_nanos(NS_S / u64::from(fps.max(1))),
        })
    }
}

impl Clock for Pace {
    fn now(&self) -> Instant {
        Instant::now()
    }

    #[allow(unsafe_code, reason = "Sets and waits on our own timer.")]
    fn until(&mut self, at: Instant) -> Result<(), RecordFault> {
        let Some(left) = at.checked_duration_since(Instant::now()) else {
            return Ok(());
        };
        // A negative due time is relative to now.
        let due = -i64::try_from(left.as_nanos() / STEP_NS).unwrap_or(i64::MAX);
        let timer = HANDLE(self.timer.as_raw_handle());
        // SAFETY: our live timer and a local due time; no completion routine.
        unsafe { SetWaitableTimer(timer, &raw const due, 0, None, None, false) }
            .map_err(refused)?;
        let bound = left + self.slot;
        let ms = u32::try_from(bound.as_millis()).map_or(LONGEST_WAIT, |ms| ms.min(LONGEST_WAIT));
        // SAFETY: our live timer; the wait ends by `ms` at the latest, and a late wake only shortens the slot.
        let _ = unsafe { WaitForSingleObject(timer, ms) };
        Ok(())
    }
}

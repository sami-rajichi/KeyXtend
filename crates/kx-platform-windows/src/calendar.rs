//! The local calendar on Windows: the system clock's moment and the time zone's offset now, summer time included.
//! The tick sums are pure and tested everywhere; only `WinCalendar` calls Windows.
#![cfg_attr(
    not(windows),
    allow(
        dead_code,
        reason = "Only the Windows calendar runs these sums; tests cover them everywhere."
    )
)]

#[cfg(windows)]
pub use win::WinCalendar;

/// `FILETIME` ticks (100 ns) in one millisecond.
const TICKS_PER_MS: i64 = 10_000;
/// `FILETIME` ticks in one minute.
const TICKS_PER_MIN: i64 = 600_000_000;
/// `FILETIME` ticks from 1601-01-01, where Windows counts from, to the Unix epoch.
const UNIX_EPOCH_TICKS: u64 = 116_444_736_000_000_000;

/// The Unix milliseconds of `FILETIME` tick count `ticks`, rounded down.
fn unix_ms(ticks: u64) -> i64 {
    let since = i128::from(ticks) - i128::from(UNIX_EPOCH_TICKS);
    let ms = since.div_euclid(i128::from(TICKS_PER_MS));
    i64::try_from(ms).unwrap_or(i64::MAX)
}

/// The offset east of UTC in whole minutes between the `local` and `utc` tick counts of one moment.
fn offset_min(local: u64, utc: u64) -> i16 {
    let diff = i128::from(local) - i128::from(utc);
    let per_min = i128::from(TICKS_PER_MIN);
    let min = (diff + per_min / 2).div_euclid(per_min);
    i16::try_from(min).unwrap_or_default()
}

#[cfg(windows)]
mod win {
    use kx_module_api::ports::{Calendar, Date, LocalTime};
    use windows::Win32::Foundation::{FILETIME, SYSTEMTIME};
    use windows::Win32::System::SystemInformation::GetSystemTimeAsFileTime;
    use windows::Win32::System::Time::{
        DYNAMIC_TIME_ZONE_INFORMATION, FileTimeToSystemTime, GetDynamicTimeZoneInformation,
        SystemTimeToFileTime, SystemTimeToTzSpecificLocalTimeEx,
    };

    use super::{offset_min, unix_ms};

    /// What a clock Windows could not hold reads as: the epoch in UTC. Windows' clock never gets there.
    const LOST: LocalTime = LocalTime {
        unix_ms: 0,
        date: Date::EPOCH,
        hour: 0,
        minute: 0,
        offset_min: 0,
    };

    /// The Windows calendar: the system clock, in the time zone the user set.
    #[derive(Clone, Copy, Debug, Default)]
    pub struct WinCalendar;

    impl Calendar for WinCalendar {
        #[allow(unsafe_code, reason = "A plain query.")]
        fn now(&self) -> LocalTime {
            // SAFETY: a plain query that returns its answer.
            let utc = ticks(unsafe { GetSystemTimeAsFileTime() });
            let offset = local_ticks(utc).map_or(0, |local| offset_min(local, utc));
            LocalTime::at(unix_ms(utc), offset).unwrap_or(LOST)
        }
    }

    /// The tick count of `t`.
    fn ticks(t: FILETIME) -> u64 {
        (u64::from(t.dwHighDateTime) << u32::BITS) | u64::from(t.dwLowDateTime)
    }

    /// The local tick count of the moment `utc`, in the time zone set now with this year's summer-time rule. It loses
    /// the moment's part below a millisecond, which `offset_min` rounds away.
    #[allow(unsafe_code, reason = "Time conversions between locals.")]
    fn local_ticks(utc: u64) -> Option<u64> {
        let file = FILETIME {
            dwLowDateTime: u32::try_from(utc & u64::from(u32::MAX)).ok()?,
            dwHighDateTime: u32::try_from(utc >> u32::BITS).ok()?,
        };
        let mut zone = DYNAMIC_TIME_ZONE_INFORMATION::default();
        let (mut utc_time, mut local_time) = (SYSTEMTIME::default(), SYSTEMTIME::default());
        let mut local = FILETIME::default();
        // SAFETY: each call reads and fills locals that outlive it.
        unsafe {
            GetDynamicTimeZoneInformation(&raw mut zone);
            FileTimeToSystemTime(&raw const file, &raw mut utc_time).ok()?;
            let zone = Some(&raw const zone);
            SystemTimeToTzSpecificLocalTimeEx(zone, &raw const utc_time, &raw mut local_time)
                .ok()?;
            SystemTimeToFileTime(&raw const local_time, &raw mut local).ok()?;
        }
        Some(ticks(local))
    }
}

#[cfg(test)]
mod tests;

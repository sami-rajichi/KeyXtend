//! The tick sums everywhere, and on Windows the calendar against Windows' own local time and time zone.

use super::*;

#[test]
fn the_unix_epoch_is_ms_zero_and_a_tick_is_100_ns() {
    assert_eq!(unix_ms(UNIX_EPOCH_TICKS), 0);
    assert_eq!(unix_ms(UNIX_EPOCH_TICKS + 10_000), 1);
    assert_eq!(unix_ms(UNIX_EPOCH_TICKS + 9_999), 0);
    assert_eq!(unix_ms(UNIX_EPOCH_TICKS - 1), -1, "before 1970 rounds down");
    assert_eq!(unix_ms(0), -11_644_473_600_000, "1601-01-01");
}

#[test]
fn the_offset_is_in_whole_minutes_east_of_utc() {
    let utc = UNIX_EPOCH_TICKS + 1_234_567;
    let min = |m: u64| m * TICKS_PER_MIN.unsigned_abs();
    assert_eq!(offset_min(utc + min(60), utc), 60, "Paris in winter");
    assert_eq!(offset_min(utc - min(300), utc), -300, "New York in winter");
    assert_eq!(offset_min(utc + min(345), utc), 345, "Kathmandu");
    assert_eq!(offset_min(utc, utc), 0);
    assert_eq!(
        offset_min(utc + min(60) - 9_999, utc),
        60,
        "a cut millisecond"
    );
}

#[cfg(windows)]
mod win {
    use std::time::{SystemTime, UNIX_EPOCH};

    use kx_module_api::ports::{Calendar, LocalTime};
    use windows::Win32::Foundation::SYSTEMTIME;
    use windows::Win32::System::SystemInformation::GetLocalTime;
    use windows::Win32::System::SystemServices::{TIME_ZONE_ID_DAYLIGHT, TIME_ZONE_ID_STANDARD};
    use windows::Win32::System::Time::{GetTimeZoneInformation, TIME_ZONE_INFORMATION};

    use super::*;

    /// Windows' local time now, as year, month, day, hour and minute.
    #[allow(unsafe_code, reason = "A plain query.")]
    fn local() -> (u16, u16, u16, u16, u16) {
        // SAFETY: a plain query that returns its answer.
        let s: SYSTEMTIME = unsafe { GetLocalTime() };
        (s.wYear, s.wMonth, s.wDay, s.wHour, s.wMinute)
    }

    #[test]
    fn now_shows_the_local_time_windows_shows() {
        let before = local();
        let t = WinCalendar.now();
        let after = local();
        let seen = (
            t.date.year(),
            u16::from(t.date.month()),
            u16::from(t.date.day()),
            u16::from(t.hour),
            u16::from(t.minute),
        );
        assert!(
            seen == before || seen == after,
            "a minute may turn meanwhile"
        );
    }

    #[test]
    #[allow(unsafe_code, reason = "Fills a local struct.")]
    fn the_offset_is_the_time_zones_bias_now_summer_time_included() {
        let mut tz = TIME_ZONE_INFORMATION::default();
        // SAFETY: `tz` is a local the call fills.
        let id = unsafe { GetTimeZoneInformation(&raw mut tz) };
        let extra = match id {
            TIME_ZONE_ID_DAYLIGHT => tz.DaylightBias,
            TIME_ZONE_ID_STANDARD => tz.StandardBias,
            _ => 0,
        };
        assert_eq!(i32::from(WinCalendar.now().offset_min), -(tz.Bias + extra));
    }

    #[test]
    fn now_is_the_system_clocks_moment() {
        let since = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let ms = i64::try_from(since.as_millis()).unwrap();
        let t = WinCalendar.now();
        assert!((t.unix_ms - ms).abs() < 5_000, "within the test's run");
        assert_eq!(
            LocalTime::at(t.unix_ms, t.offset_min),
            Some(t),
            "built by the port's sums"
        );
    }
}

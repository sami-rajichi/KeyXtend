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
fn a_moments_ticks_are_the_inverse_of_its_ms_and_none_before_1601() {
    assert_eq!(ticks_of(0), Some(UNIX_EPOCH_TICKS));
    for ms in [1, -1, 1_768_478_400_123, -11_644_473_600_000] {
        assert_eq!(ticks_of(ms).map(unix_ms), Some(ms), "{ms}");
    }
    assert_eq!(ticks_of(-11_644_473_600_001), None, "before 1601");
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

    use kx_module_api::MS_PER_S;
    use kx_module_api::ports::{Calendar, Date, HOURS_PER_DAY, LocalTime, MIN_PER_HOUR, S_PER_MIN};
    use windows::Win32::Foundation::SYSTEMTIME;
    use windows::Win32::System::SystemInformation::GetLocalTime;
    use windows::Win32::System::SystemServices::{TIME_ZONE_ID_DAYLIGHT, TIME_ZONE_ID_STANDARD};
    use windows::Win32::System::Time::{
        DYNAMIC_TIME_ZONE_INFORMATION, EnumDynamicTimeZoneInformation, GetTimeZoneInformation,
        TIME_ZONE_INFORMATION,
    };

    use super::*;
    use crate::calendar::win::local_at;

    /// Paris's zone as Windows names it: an hour east of UTC, and an hour more in summer.
    const PARIS: &str = "Romance Standard Time";
    const PARIS_EAST_MIN: i16 = 60;
    const SUMMER_MIN: i16 = 60;
    /// The test moments: noon UTC on the 15th of January and of July.
    const JANUARY: u8 = 1;
    const JULY: u8 = 7;
    const NOON: i64 = 12;

    /// Paris's zone, from Windows' own list of zones.
    #[allow(unsafe_code, reason = "Fills a local struct.")]
    fn paris() -> DYNAMIC_TIME_ZONE_INFORMATION {
        let nth = |i| {
            let mut zone = DYNAMIC_TIME_ZONE_INFORMATION::default();
            // SAFETY: the call fills a local that outlives it.
            let found = unsafe { EnumDynamicTimeZoneInformation(i, &raw mut zone) } == 0;
            found.then_some(zone)
        };
        let name = |z: &DYNAMIC_TIME_ZONE_INFORMATION| String::from_utf16_lossy(&z.TimeZoneKeyName);
        (0..)
            .map_while(nth)
            .find(|z| name(z).trim_end_matches('\0') == PARIS)
            .expect("Paris")
    }

    /// Noon UTC on the 15th of `month` 2026, in Unix ms.
    fn noon_utc(month: u8) -> i64 {
        let day = Date::new(2026, month, 15).expect("a day").day_number();
        let min = (day * HOURS_PER_DAY + NOON) * MIN_PER_HOUR;
        min * S_PER_MIN * i64::from(MS_PER_S)
    }

    #[test]
    fn a_moment_shows_the_offset_its_zone_keeps_then_summer_time_included() {
        let zone = paris();
        let winter = local_at(noon_utc(JANUARY), &zone).expect("a time");
        let summer = local_at(noon_utc(JULY), &zone).expect("a time");
        let hour = |off: i16| u8::try_from(NOON + i64::from(off) / MIN_PER_HOUR).unwrap();
        let (east, later) = (PARIS_EAST_MIN, PARIS_EAST_MIN + SUMMER_MIN);
        assert_eq!((winter.offset_min, winter.hour), (east, hour(east)));
        assert_eq!((summer.offset_min, summer.hour), (later, hour(later)));
    }

    #[test]
    fn local_now_is_now() {
        let t = WinCalendar.now();
        assert_eq!(WinCalendar.local(t.unix_ms), Some(t));
    }

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

//! A clipboard row's time and meta line: today, yesterday or a weekday with the 24-hour time within a week, else the
//! date, across a year end too; the app is left out when unknown, and Arabic starts with the time.

use crate::lang;
use kx_module_api::ports::{Date, When};

/// A copy `days_ago` days before today, on `date` at `hour`:`minute`.
pub(super) fn when(days_ago: u32, (y, m, d): (u16, u8, u8), hour: u8, minute: u8) -> When {
    When {
        days_ago,
        date: Date::new(y, m, d).expect("a real day"),
        hour,
        minute,
    }
}

#[test]
fn a_rows_time_reads_today_yesterday_a_weekday_then_the_date() {
    // 2026-10-31 is a Saturday, so the 27th was a Tuesday and the 24th a Saturday a week back.
    let times = [
        when(0, (2026, 10, 31), 16, 32),
        when(1, (2026, 10, 30), 18, 21),
        when(4, (2026, 10, 27), 9, 5),
        when(7, (2026, 10, 24), 14, 12),
    ];
    let cases = [
        (
            "en",
            ["Today 16:32", "Yesterday 18:21", "Tue 09:05", "24/10/2026"],
        ),
        (
            "fr",
            [
                "Aujourd’hui 16:32",
                "Hier 18:21",
                "Mar. 09:05",
                "24/10/2026",
            ],
        ),
        (
            "ar",
            ["اليوم 16:32", "أمس 18:21", "الثلاثاء 09:05", "24/10/2026"],
        ),
    ];
    for (tag, want) in cases {
        let got = times.map(|w| lang(tag).clip_when(&w));
        assert_eq!(got, want, "{tag}");
    }
}

#[test]
fn a_time_across_a_year_end_counts_calendar_days() {
    // Seen on Saturday 2027-01-02.
    let en = lang("en");
    assert_eq!(
        en.clip_when(&when(1, (2027, 1, 1), 0, 0)),
        "Yesterday 00:00"
    );
    assert_eq!(en.clip_when(&when(3, (2026, 12, 30), 23, 59)), "Wed 23:59");
    assert_eq!(en.clip_when(&when(9, (2026, 12, 24), 8, 0)), "24/12/2026");
    assert_eq!(
        lang("fr").clip_when(&when(6, (2026, 12, 27), 7, 3)),
        "Dim. 07:03"
    );
}

#[test]
fn the_meta_line_leaves_out_an_unknown_app_and_arabic_starts_with_the_time() {
    let en = lang("en");
    let meta = en.clip_meta(Some("Chrome"), "Today 16:32", "2.41 MB");
    assert_eq!(meta, "Chrome · Today 16:32 · 2.41 MB");
    assert_eq!(
        en.clip_meta(None, "Today 16:32", "2.41 MB"),
        "Today 16:32 · 2.41 MB"
    );
    let marks = en.clip_meta(Some("\u{200f}\u{202e}"), "Today 16:32", "2.41 MB");
    assert_eq!(
        marks, "Today 16:32 · 2.41 MB",
        "an app of marks alone is unknown"
    );
    let ar = lang("ar").clip_meta(Some("Chrome"), "اليوم 16:32", "2,41 ميغابايت");
    assert_eq!(ar, "اليوم 16:32 · 2,41 ميغابايت · Chrome");
}

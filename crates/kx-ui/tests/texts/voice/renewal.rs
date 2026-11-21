//! A used allowance's renewal in each language (owner, 2026-11-20): the clock time it renews at in Western digits on a
//! 24-hour clock, and its date as the clipboard writes one when it is not today (m6).

use crate::lang;
use kx_module_api::ports::{CloudWhy, LocalTime, Renewal};

/// 09:05 UTC on 14 November 2026, a time whose hour takes a leading zero.
const RENEWS_MS: i64 = 1_794_647_100_000;

/// The used allowance's note line in language `tag`, renewing at `RENEWS_MS` today or on a later day.
fn said(tag: &str, today: bool) -> String {
    let at = LocalTime::at(RENEWS_MS, 0).expect("a time");
    let renews_at = Some(Renewal { at, today });
    lang(tag).voice_cloud(&CloudWhy::Allowance { renews_at })
}

#[test]
fn a_renewal_today_names_its_clock_time() {
    let want = [
        ("en", "The free allowance is used up. It renews at 09:05."),
        (
            "fr",
            "Le quota gratuit est épuisé. Il se renouvelle à 09:05.",
        ),
        ("ar", "نفدت الحصة المجانية، وتتجدد في الساعة 09:05."),
    ];
    for (tag, line) in want {
        assert_eq!(said(tag, true), line, "{tag}");
    }
}

#[test]
fn a_renewal_on_a_later_day_names_its_date_too() {
    let want = [
        (
            "en",
            "The free allowance is used up. It renews on 14/11/2026 at 09:05.",
        ),
        (
            "fr",
            "Le quota gratuit est épuisé. Il se renouvelle le 14/11/2026 à 09:05.",
        ),
        (
            "ar",
            "نفدت الحصة المجانية، وتتجدد يوم 14/11/2026 في الساعة 09:05.",
        ),
    ];
    for (tag, line) in want {
        assert_eq!(said(tag, false), line, "{tag}");
    }
}

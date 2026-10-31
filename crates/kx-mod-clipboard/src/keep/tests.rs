//! The limits: a copy too big, under the free floor or past caps the pins fill is refused with a note; the caps drop
//! the oldest unpinned; a new day sweeps old days and media; a clock set back keeps everything.

use std::time::Duration;

use kx_module_api::ports::{Calendar, ClipKind, LocalTime};
use kx_module_api::{Clock, Mono};
use kx_platform_fake::{FakeCalendar, FakeClock};

use super::*;
use crate::fixture::{CFG, FOLLOW, LIMITS, MB, PARIS, ROOMY, app, at, copied, date, files, text};
use crate::follow::Follow;

/// A minute.
const MINUTE: Duration = Duration::from_secs(60);

/// A text entry keeping `bytes` bytes, copied at `now`.
fn sized(bytes: u64, now: &LocalTime) -> Entry {
    let mut e = copied("x", now);
    e.stored = bytes;
    e
}

/// A video entry copied at `now`.
fn video(now: &LocalTime) -> Entry {
    Entry::new(&[files(&["C:/v/clip.mkv"])], None, now, &CFG)
}

/// The ids of `entries`, in order.
fn ids(entries: &[Entry]) -> Vec<u64> {
    entries.iter().map(|e| e.id).collect()
}

/// 2026-10-31 at 9:00 in Paris.
fn nine() -> LocalTime {
    at(date(2026, 10, 31), 9, 0)
}

#[test]
fn a_copy_over_max_entry_bytes_is_too_big() {
    let h = History::default();
    let too_big = admit(&h, &sized(32 * MB + 1, &nine()), ROOMY, &LIMITS);
    assert_eq!(too_big, Err(PanelNote::TooBig));
    assert_eq!(admit(&h, &sized(32 * MB, &nine()), ROOMY, &LIMITS), Ok(()));
    let small = Limits {
        max_bytes: MB,
        ..*LIMITS
    };
    let past_cap = admit(&h, &sized(MB + 1, &nine()), ROOMY, &small);
    assert_eq!(past_cap, Err(PanelNote::TooBig), "past the whole cap too");
}

#[test]
fn under_the_free_floor_nothing_new_is_kept_until_there_is_room() {
    let (h, e) = (History::default(), sized(MB, &nine()));
    let floor = LIMITS.min_free_bytes;
    assert_eq!(admit(&h, &e, 0, &LIMITS), Err(PanelNote::Disk));
    assert_eq!(admit(&h, &e, floor, &LIMITS), Err(PanelNote::Disk));
    assert_eq!(admit(&h, &e, floor + MB - 1, &LIMITS), Err(PanelNote::Disk));
    assert_eq!(
        admit(&h, &e, floor + MB, &LIMITS),
        Ok(()),
        "keeping resumes with room"
    );
}

#[test]
fn pins_filling_the_count_refuse_a_copy() {
    let few = Limits {
        max_entries: 2,
        ..*LIMITS
    };
    let mut h = History::default();
    let (a, b) = (h.add(copied("a", &nine())), h.add(copied("b", &nine())));
    let c = copied("c", &nine());
    assert_eq!(
        admit(&h, &c, ROOMY, &few),
        Ok(()),
        "unpinned entries make room"
    );
    h.pin(a);
    h.pin(b);
    assert_eq!(admit(&h, &c, ROOMY, &few), Err(PanelNote::PinnedFull));
}

#[test]
fn pins_filling_the_bytes_refuse_a_copy() {
    let mut h = History::default();
    let big = h.add(sized(1000 * MB, &nine()));
    assert_eq!(admit(&h, &sized(30 * MB, &nine()), ROOMY, &LIMITS), Ok(()));
    h.pin(big);
    let over = admit(&h, &sized(30 * MB, &nine()), ROOMY, &LIMITS);
    assert_eq!(over, Err(PanelNote::PinnedFull));
    assert_eq!(
        admit(&h, &sized(24 * MB, &nine()), ROOMY, &LIMITS),
        Ok(()),
        "what fits is kept"
    );
}

#[test]
fn past_max_entries_the_oldest_unpinned_leaves() {
    let few = Limits {
        max_entries: 3,
        ..*LIMITS
    };
    let mut h = History::default();
    let id: Vec<u64> = ["a", "b", "c", "d"]
        .iter()
        .map(|s| h.add(copied(s, &nine())))
        .collect();
    h.pin(id[0]);
    assert_eq!(ids(&trim(&mut h, &few)), [id[1]]);
    assert_eq!(
        ids(h.entries()),
        [id[0], id[3], id[2]],
        "the oldest is pinned, so it stays"
    );
    assert!(trim(&mut h, &few).is_empty());
}

#[test]
fn past_max_bytes_the_oldest_unpinned_leave_first() {
    let mut h = History::default();
    let old = h.add(sized(600 * MB, &nine()));
    h.add(sized(300 * MB, &nine()));
    h.add(sized(200 * MB, &nine()));
    assert_eq!(ids(&trim(&mut h, &LIMITS)), [old]);
    assert_eq!(h.bytes(), 500 * MB);
}

#[test]
fn trim_never_takes_a_pin_even_past_a_lowered_cap() {
    let mut h = History::default();
    let (a, b) = (h.add(copied("a", &nine())), h.add(copied("b", &nine())));
    let c = h.add(copied("c", &nine()));
    h.pin(a);
    h.pin(b);
    let one = Limits {
        max_entries: 1,
        ..*LIMITS
    };
    assert_eq!(ids(&trim(&mut h, &one)), [c]);
    assert_eq!(ids(h.entries()), [b, a]);
}

#[test]
fn a_new_day_sweeps_the_days_past_keep_days() {
    let today = date(2026, 11, 7);
    let mut h = History::default();
    let mut on = |d: Date| h.add(copied(&d.to_string(), &at(d, 12, 0)));
    let week = on(date(2026, 10, 31));
    let six = on(date(2026, 11, 1));
    let fresh = on(today);
    let old = on(date(2026, 9, 1));
    let pin = on(date(2026, 1, 1));
    h.pin(pin);
    assert_eq!(ids(&sweep(&mut h, today, &LIMITS)), [old, week]);
    assert_eq!(ids(h.entries()), [pin, fresh, six]);
}

#[test]
fn media_leaves_after_media_days_and_a_pin_stays() {
    let (today, yesterday) = (date(2026, 11, 7), date(2026, 11, 6));
    let mut h = History::default();
    let old_clip = h.add(video(&at(yesterday, 22, 0)));
    let pinned_clip = h.add(video(&at(yesterday, 22, 5)));
    h.pin(pinned_clip);
    let old_text = h.add(copied("note", &at(yesterday, 23, 0)));
    let new_clip = h.add(video(&at(today, 0, 5)));
    assert_eq!(h.get(old_clip).map(|e| e.kind), Some(ClipKind::Media));
    assert_eq!(ids(&sweep(&mut h, today, &LIMITS)), [old_clip]);
    assert_eq!(ids(h.entries()), [pinned_clip, new_clip, old_text]);
}

#[test]
fn midnight_sweeps_by_the_calendar_while_the_clear_window_runs_on() {
    let cal = FakeCalendar::new(date(2026, 10, 25), 10, 0, PARIS);
    let clock = FakeClock::new(Mono::default());
    let (mut h, mut follow) = (History::default(), Follow::new(*FOLLOW, Vec::new()));
    let week = h.add(copied("week", &cal.now()));
    cal.set(date(2026, 10, 31), 23, 59);
    let clip = h.add(video(&cal.now()));
    let vault = app("vault.exe");
    let secret = h.add(Entry::new(
        &[text("pw")],
        Some(vault.clone()),
        &cal.now(),
        &CFG,
    ));
    follow.kept(secret, Some(&vault), clock.now());
    assert!(
        sweep(&mut h, cal.now().date, &LIMITS).is_empty(),
        "10-25 is 6 days before"
    );
    cal.advance(MINUTE);
    clock.advance(MINUTE);
    assert_eq!(cal.now().date, date(2026, 11, 1));
    assert_eq!(ids(&sweep(&mut h, cal.now().date, &LIMITS)), [clip, week]);
    assert_eq!(follow.cleared(Some(&vault), clock.now(), &h), Some(secret));
}

#[test]
fn a_clock_set_back_sweeps_nothing_and_keeps_the_order() {
    let cal = FakeCalendar::new(date(2026, 10, 31), 9, 0, PARIS);
    let mut h = History::default();
    h.add(copied("a", &cal.now()));
    cal.advance(MINUTE);
    h.add(video(&cal.now()));
    let before = h.clone();
    cal.set(date(2026, 10, 20), 9, 0);
    assert!(sweep(&mut h, cal.now().date, &LIMITS).is_empty());
    assert_eq!(h, before);
    let fresh = h.add(copied("b", &cal.now()));
    assert_eq!(h.entries().first().map(|e| e.id), Some(fresh));
}

#[test]
fn past_counts_calendar_days() {
    let today = date(2026, 3, 1);
    assert!(past(date(2026, 2, 22), today, 7), "7 days before leaves");
    assert!(!past(date(2026, 2, 23), today, 7), "6 days before stays");
    assert!(!past(today, today, 1), "1 is today only");
    assert!(past(date(2026, 2, 28), today, 1));
    assert!(
        !past(date(2026, 3, 9), today, 7),
        "a later day never leaves"
    );
}

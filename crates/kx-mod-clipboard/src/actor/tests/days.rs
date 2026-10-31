//! Calendar days: media leaves after `media_days` and a day past `keep_days` drops whole at midnight, a clock set
//! back keeps entries and order, and the loop wakes at local midnight.

use std::time::Duration;

use kx_module_api::ports::FileFact;

use super::super::days::until_midnight;
use super::rig::{Rig, TODAY};
use crate::fixture::{at, date, files};

#[test]
fn media_leaves_after_media_days_and_a_day_past_keep_days_drops_whole() {
    let mut rig = Rig::new();
    rig.fakes.clip.set_file("C:\\clip.mp4", FileFact::Size(5));
    rig.copy(vec![files(&["C:\\clip.mp4"])], None);
    rig.text("words");
    rig.open();
    assert!(rig.page().rows[1].expires, "media goes tonight");
    rig.set_time(date(2026, 11, 1), 0, 0);
    assert_eq!(rig.titles(), ["words"], "media keeps one day");
    rig.set_time(date(2026, 11, 6), 0, 0);
    assert_eq!(rig.titles(), ["words"], "today and the 6 days before");
    assert!(!rig.names(TODAY).is_empty());
    rig.set_time(date(2026, 11, 7), 0, 0);
    assert!(rig.page().empty);
    assert!(
        rig.names(TODAY).is_empty(),
        "the day's folder dropped whole"
    );
}

#[test]
fn a_clock_set_back_keeps_entries_and_order() {
    let mut rig = Rig::new();
    rig.text("a");
    rig.text("b");
    rig.set_time(date(2026, 10, 20), 9, 0);
    rig.text("c");
    rig.open();
    assert_eq!(rig.titles(), ["c", "b", "a"]);
}

#[test]
fn the_loop_wakes_at_local_midnight_or_sooner_for_a_save() {
    let mut rig = Rig::new();
    let left = Duration::from_mins(7 * 60 + 28);
    assert_eq!(rig.wait(), left, "16:32 to midnight");
    rig.text("a");
    assert_eq!(rig.wait(), rig.settings.save());
}

#[test]
fn midnight_counts_the_seconds_into_the_minute() {
    let mut late = at(date(2026, 10, 31), 23, 59);
    assert_eq!(until_midnight(&late), Duration::from_secs(60));
    late.unix_ms += 59_999;
    assert_eq!(until_midnight(&late), Duration::from_millis(1));
}

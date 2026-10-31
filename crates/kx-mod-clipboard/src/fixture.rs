//! What the tests share: the plan's `[clipboard]` defaults, dates and wall times from the fake calendar, and copies.

use std::path::PathBuf;
use std::time::Duration;

use kx_module_api::Redacted;
use kx_module_api::ports::{AppRef, Calendar, ClipPart, Date, LocalTime};
use kx_platform_fake::FakeCalendar;

use crate::entry::{Entry, EntryConfig};
use crate::keep::Limits;

/// Bytes in a megabyte, as the settings count them.
pub(crate) const MB: u64 = 1024 * 1024;
/// Paris in winter, in minutes east of UTC.
pub(crate) const PARIS: i16 = 60;
/// The plan's `title_chars` and `search_chars`.
pub(crate) const CFG: EntryConfig = EntryConfig {
    title_chars: 300,
    search_chars: 1000,
};
/// The plan's limits: 7 days, media 1 day, 1,000 entries, 1 GB, 2 GB free, 32 MB an entry.
pub(crate) const LIMITS: Limits = Limits {
    keep_days: 7,
    media_days: 1,
    max_entries: 1000,
    max_bytes: 1024 * MB,
    min_free_bytes: 2048 * MB,
    max_entry_bytes: 32 * MB,
};
/// The plan's `clear_follow_s`.
pub(crate) const FOLLOW: Duration = Duration::from_secs(120);
/// Free space well over the floor.
pub(crate) const ROOMY: u64 = 400 * 1024 * MB;

/// A date the test knows is real.
pub(crate) fn date(year: u16, month: u8, day: u8) -> Date {
    Date::new(year, month, day).unwrap()
}

/// The wall time `hour`:`minute` on `day` in Paris, from the fake calendar.
pub(crate) fn at(day: Date, hour: u8, minute: u8) -> LocalTime {
    FakeCalendar::new(day, hour, minute, PARIS).now()
}

/// A plain text part.
pub(crate) fn text(s: &str) -> ClipPart {
    ClipPart::Text(Redacted::new(s.to_owned()))
}

/// A file list part.
pub(crate) fn files(paths: &[&str]) -> ClipPart {
    ClipPart::Files(Redacted::new(paths.iter().map(PathBuf::from).collect()))
}

/// An app whose id and name are `id`.
pub(crate) fn app(id: &str) -> AppRef {
    AppRef {
        id: id.to_owned(),
        name: id.to_owned(),
    }
}

/// A text copy of `s` made at `now`.
pub(crate) fn copied(s: &str, now: &LocalTime) -> Entry {
    Entry::new(&[text(s)], None, now, &CFG)
}

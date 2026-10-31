//! What the tests share: configs built from the `[clipboard]` defaults, dates and wall times from the fake calendar,
//! and copies.

use std::path::PathBuf;
use std::sync::LazyLock;
use std::time::Duration;

use kx_module_api::Redacted;
use kx_module_api::ports::{AppRef, Calendar, ClipPart, Date, ImageCodec, LocalTime};
use kx_platform_fake::FakeCalendar;

use crate::entry::{Entry, EntryConfig};
use crate::keep::Limits;
use crate::settings::ClipSettings;
use crate::store::StoreConfig;

/// Bytes in a megabyte, the port's one unit.
pub(crate) use kx_module_api::ports::MB;

/// Paris in winter, the fake calendar's offset.
pub(crate) use kx_platform_fake::PARIS;

/// The `[clipboard]` defaults, read once.
pub(crate) static DEFAULTS: LazyLock<ClipSettings> =
    LazyLock::new(|| ClipSettings::from_defaults().unwrap());
/// The defaults' `title_chars` and `search_chars`.
pub(crate) static CFG: LazyLock<EntryConfig> = LazyLock::new(|| DEFAULTS.entry_config());
/// The defaults' limits, in bytes and days.
pub(crate) static LIMITS: LazyLock<Limits> = LazyLock::new(|| DEFAULTS.limits());
/// The defaults' `clear_follow_s`.
pub(crate) static FOLLOW: LazyLock<Duration> = LazyLock::new(|| DEFAULTS.clear_window());
/// The defaults' `thumb_px`, and the port's blob size.
pub(crate) static STORE: LazyLock<StoreConfig> = LazyLock::new(|| DEFAULTS.store_config());
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

/// A small PNG image part.
pub(crate) fn image() -> ClipPart {
    ClipPart::Image {
        codec: ImageCodec::Png,
        bytes: Redacted::new(vec![1, 2, 3]),
    }
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

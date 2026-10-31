//! The owner's limits (spec §7 "Limits"): a new copy is kept or refused with a note, the oldest unpinned entries
//! leave past the count or size cap, and a new calendar day sweeps the days past `keep_days` and media past
//! `media_days`. Pinned entries never leave.

use kx_module_api::ports::{ClipKind, Date, PanelNote};

use crate::entry::{Entry, Place};
use crate::order::History;

/// The limits; the module fills them from `[clipboard]`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Limits {
    /// Calendar days an entry is kept: 7 is today and the 6 days before (`keep_days`).
    pub keep_days: u8,
    /// Calendar days a video or audio entry is kept (`media_days`).
    pub media_days: u8,
    /// The most entries (`max_entries`).
    pub max_entries: usize,
    /// The most bytes all entries keep (`max_mb`).
    pub max_bytes: u64,
    /// The free bytes the drive keeps; under them nothing new is kept (`min_free_mb`).
    pub min_free_bytes: u64,
    /// The most bytes one entry keeps (`max_entry_mb`).
    pub max_entry_bytes: u64,
}

/// `Ok` when `entry`, a new copy, may join `history` with `free` bytes free on the drive, else the note to show.
///
/// # Errors
/// `TooBig` past `max_entry_bytes` or `max_bytes`, `Disk` when keeping it would leave less than `min_free_bytes`,
/// `PinnedFull` when the pinned entries alone leave no room in a cap.
pub fn admit(
    history: &History,
    entry: &Entry,
    free: u64,
    limits: &Limits,
) -> Result<(), PanelNote> {
    let size = entry.stored;
    if size > limits.max_entry_bytes || size > limits.max_bytes {
        return Err(PanelNote::TooBig);
    }
    if free.saturating_sub(size) < limits.min_free_bytes {
        return Err(PanelNote::Disk);
    }
    let pins = history.entries().iter().filter(|e| e.pinned());
    let (count, bytes) = pins.fold((0, 0), |(n, b), e| {
        (n + 1, u64::saturating_add(b, e.stored))
    });
    if count >= limits.max_entries || bytes.saturating_add(size) > limits.max_bytes {
        return Err(PanelNote::PinnedFull);
    }
    Ok(())
}

/// Takes out the oldest unpinned entries until `history` is within `max_entries` and `max_bytes`; gives them.
pub fn trim(history: &mut History, limits: &Limits) -> Vec<Entry> {
    let mut bytes = history.bytes();
    let mut gone = Vec::new();
    while history.len() > limits.max_entries || bytes > limits.max_bytes {
        let Some(oldest) = history.pop_oldest() else {
            break;
        };
        bytes = bytes.saturating_sub(oldest.stored);
        gone.push(oldest);
    }
    gone
}

/// Takes out the unpinned entries whose day is past `keep_days` on `today`, and media past `media_days`; gives them.
pub fn sweep(history: &mut History, today: Date, limits: &Limits) -> Vec<Entry> {
    history.take_where(|e| leaves(e, today, limits))
}

/// True when the sweep on `today` takes `entry` out: unpinned, and past `keep_days`, or media past `media_days`.
#[must_use]
pub fn leaves(entry: &Entry, today: Date, limits: &Limits) -> bool {
    let Place::Day(day) = entry.place else {
        return false;
    };
    let media = entry.kind == ClipKind::Media && past(day, today, limits.media_days);
    media || past(day, today, limits.keep_days)
}

/// True when `day` is `days` or more calendar days before `today`, so a folder of that day leaves; a later day,
/// after a clock set back, never does.
#[must_use]
pub fn past(day: Date, today: Date, days: u8) -> bool {
    today.days_since(day) >= i64::from(days)
}

#[cfg(test)]
mod props;
#[cfg(test)]
mod tests;

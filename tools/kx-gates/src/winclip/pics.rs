//! The gate's own Copy pictures in Windows' clipboard history: at most one picture in each Copy's span goes.

use windows::ApplicationModel::DataTransfer::{
    Clipboard, ClipboardHistoryItem, StandardDataFormats,
};
use windows::core::HSTRING;

use super::{err, history, on_com};
use crate::clipkeep;

/// History time stamps count 100 ns, so a ms holds this many.
const TICKS_PER_MS: i64 = 10_000;

/// Each Copy's span, in order, with its end `pad_ms` later for a picture Windows stamps a little after the read back,
/// but short of the next span's start, so it never takes the next Copy's picture.
pub fn padded(spans: &[(i64, i64)], pad_ms: u64) -> Vec<(i64, i64)> {
    let pad = i64::try_from(pad_ms)
        .unwrap_or(i64::MAX)
        .saturating_mul(TICKS_PER_MS);
    let next = |i: usize| spans.get(i + 1).map_or(i64::MAX, |n| n.0.saturating_sub(1));
    let each = |(i, &(from, to)): (usize, &(i64, i64))| (from, to.saturating_add(pad).min(next(i)));
    spans.iter().enumerate().map(each).collect()
}

/// Deletes at most one picture stamped inside each span, the earliest not yet taken: whether each span's picture went,
/// none when history is off. A test never reaches Windows' history.
pub fn forget_pictures(spans: &[(i64, i64)]) -> Result<Option<Vec<bool>>, String> {
    clipkeep::live();
    on_com(|| forget_here(spans))
}

/// `forget_pictures` on the current COM thread.
fn forget_here(spans: &[(i64, i64)]) -> Result<Option<Vec<bool>>, String> {
    let Some(items) = history()? else {
        return Ok(None);
    };
    let bitmap = StandardDataFormats::Bitmap().map_err(err)?;
    let (pics, stamps): (Vec<_>, Vec<_>) = items
        .into_iter()
        .filter_map(|it| stamp(&it, &bitmap).map(|s| (it, s)))
        .unzip();
    let gone = |i: usize| {
        pics.get(i)
            .is_some_and(|it| Clipboard::DeleteItemFromHistory(it).unwrap_or(false))
    };
    let went = picked(&stamps, spans)
        .into_iter()
        .map(|i| i.is_some_and(gone));
    Ok(Some(went.collect()))
}

/// For each span in turn, the index of the earliest stamp inside it not yet taken, both ends included.
pub(super) fn picked(stamps: &[i64], spans: &[(i64, i64)]) -> Vec<Option<usize>> {
    let mut taken: Vec<Option<usize>> = Vec::with_capacity(spans.len());
    for &(from, to) in spans {
        let free = |&(i, s): &(usize, &i64)| (from..=to).contains(s) && !taken.contains(&Some(i));
        let first = stamps
            .iter()
            .enumerate()
            .filter(free)
            .min_by_key(|&(_, s)| *s)
            .map(|(i, _)| i);
        taken.push(first);
    }
    taken
}

/// The time stamp of history entry `item` when it holds a picture.
fn stamp(item: &ClipboardHistoryItem, bitmap: &HSTRING) -> Option<i64> {
    let content = item.Content().ok()?;
    content
        .Contains(bitmap)
        .ok()?
        .then(|| item.Timestamp().ok().map(|t| t.UniversalTime))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_copys_span_takes_its_earliest_picture_at_most_one_each() {
        // Newest first, as Windows lists its history.
        let stamps = [30, 25, 15, 5];
        assert_eq!(picked(&stamps, &[(10, 20)]), [Some(2)]);
        assert_eq!(picked(&stamps, &[(0, 40)]), [Some(3)], "the earliest");
        assert_eq!(picked(&stamps, &[(0, 40), (0, 40)]), [Some(3), Some(2)]);
        assert_eq!(
            picked(&stamps, &[(25, 30), (25, 30)]),
            [Some(1), Some(0)],
            "both ends in"
        );
        assert_eq!(
            picked(&stamps, &[(16, 24), (31, 99)]),
            [None, None],
            "none inside"
        );
        assert!(picked(&stamps, &[]).is_empty(), "no Copy, nothing goes");
    }

    #[test]
    fn a_span_ends_its_pad_later_in_history_units() {
        assert_eq!(padded(&[(100, 200)], 0), [(100, 200)]);
        assert_eq!(padded(&[(100, 200)], 2), [(100, 200 + 2 * TICKS_PER_MS)]);
        assert_eq!(
            padded(&[(0, i64::MAX)], 5),
            [(0, i64::MAX)],
            "never past the end"
        );
        assert!(padded(&[], 5).is_empty());
    }

    #[test]
    fn a_pad_stops_short_of_the_next_copys_span_so_it_never_takes_its_picture() {
        let next = 200 + TICKS_PER_MS;
        let spans = padded(&[(100, 200), (next, next + 50)], 2);
        assert_eq!(
            spans,
            [(100, next - 1), (next, next + 50 + 2 * TICKS_PER_MS)]
        );
        let overlapping = padded(&[(10, 20), (15, 30)], 1);
        assert_eq!(overlapping, [(10, 14), (15, 30 + TICKS_PER_MS)]);
        let stamps = [next + 10, 150];
        assert_eq!(picked(&stamps, &spans), [Some(1), Some(0)], "each its own");
        let lone = [next + 10];
        assert_eq!(
            picked(&lone, &spans),
            [None, Some(0)],
            "the next one's stays"
        );
    }
}

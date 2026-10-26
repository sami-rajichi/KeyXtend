//! The learner's limits (ADR-0032 decision 4, after AOSP `LatinIME`): a clean-up past 1.2 × a cap, least recently
//! used first; halving near the 16-bit count limit; the purge of entries unused too long.

/// A table is cleaned once it passes this many tenths of its cap, 1.2 ×: AOSP `LatinIME`'s value, so clean-ups stay
/// rare.
pub(super) const CLEAN_AT_TENTHS: usize = 12;
/// Tenths in a whole, for `CLEAN_AT_TENTHS`.
pub(super) const TENTHS: usize = 10;
/// Counts halve this far below the 16-bit top: room for the most uses one learn adds, `min_uses` when a forgotten
/// word comes back.
pub(super) const HEADROOM: u16 = 1 << u8::BITS;
/// The highest count a table keeps; a use past it halves every table first.
pub(super) const COUNT_TOP: u16 = u16::MAX - HEADROOM;

/// The largest cap whose peak still fits `top`: `top` / `CLEAN_AT_TENTHS` × `TENTHS`.
pub(super) const fn largest_cap(top: usize) -> usize {
    top / CLEAN_AT_TENTHS * TENTHS
}

/// The most entries a table capped at `cap` holds before its clean-up.
pub(super) fn peak(cap: usize) -> usize {
    cap.saturating_mul(CLEAN_AT_TENTHS) / TENTHS
}

/// The most entries a table capped at `cap` ever holds: its peak, plus the one a learn adds before the clean-up.
pub(super) fn most(cap: usize) -> usize {
    peak(cap).saturating_add(1)
}

/// True when adding `n` uses to `count` passes `COUNT_TOP`, so every table halves first.
pub(super) fn too_high(count: u16, n: u16) -> bool {
    over_top(u32::from(count) + u32::from(n))
}

/// True when the sum of counts `sum` passes `COUNT_TOP`.
pub(super) fn over_top(sum: u32) -> bool {
    sum > u32::from(COUNT_TOP)
}

/// `count` halved, rounded up, but kept at `floor` when it held that many: max(⌈c/2⌉, min(c, floor)). It is
/// monotone, so every order holds, and with `min_uses` as `floor` an offered word stays offered.
pub(super) fn halved(count: u16, floor: u16) -> u16 {
    count.div_ceil(2).max(count.min(floor))
}

/// True when an entry last used on `last` was unused for more than `days` by `today`.
pub(super) fn stale(last: u32, today: u32, days: u32) -> bool {
    today.saturating_sub(last) > days
}

/// The places a clean-up drops so `keep` entries stay, in order: the least recently used first, then the lower
/// count, then the lower place. Each entry is its last-used day, count and place.
pub(super) fn oldest(mut entries: Vec<(u32, u16, usize)>, keep: usize) -> Vec<usize> {
    let drop = entries.len().saturating_sub(keep);
    entries.sort_unstable();
    let mut out: Vec<usize> = entries.into_iter().take(drop).map(|e| e.2).collect();
    out.sort_unstable();
    out
}

/// Grows `vec` for `more` items by doubling, as `Vec` does, but never past `most` items, so a table at its peak
/// holds no spare half.
pub(super) fn grow<T>(vec: &mut Vec<T>, more: usize, most: usize) {
    let need = vec.len().saturating_add(more);
    if need > vec.capacity() {
        let doubled = vec.capacity().saturating_mul(2).min(most).max(need);
        vec.reserve_exact(doubled - vec.len());
    }
}

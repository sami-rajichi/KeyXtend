//! The freeze's cap (`shot_cap_mb`): the screens' raw bytes together, counted before any pixel is grabbed.

use kx_module_api::ports::{BGRX_BYTES, Part, Rect, ShotCap};

/// The raw bytes `screens` hold together, `BGRX_BYTES` a pixel, stopping at `u64::MAX`; a screen with no pixels holds
/// none.
pub fn screen_bytes(screens: impl IntoIterator<Item = Rect>) -> u64 {
    let pixel = u64::try_from(BGRX_BYTES).unwrap_or(u64::MAX);
    let one = |rect: Rect| {
        if Part::new(rect).is_none() {
            return 0;
        }
        let side = |n: i32| u64::try_from(n).unwrap_or_default();
        side(rect.w)
            .saturating_mul(side(rect.h))
            .saturating_mul(pixel)
    };
    screens.into_iter().map(one).fold(0, u64::saturating_add)
}

/// Whether `screens` together stay within `cap`.
pub fn fits(screens: impl IntoIterator<Item = Rect>, cap: ShotCap) -> bool {
    screen_bytes(screens) <= cap.bytes()
}

//! Arrays UI Automation hands back: a one-dimensional SAFEARRAY of numbers, read and freed, and line boxes from groups
//! of four doubles.

use std::ffi::c_void;

use windows::Win32::Foundation::RECT;
use windows::Win32::System::Com::SAFEARRAY;
use windows::Win32::System::Ole::{
    SafeArrayAccessData, SafeArrayDestroy, SafeArrayGetDim, SafeArrayGetElemsize,
    SafeArrayGetLBound, SafeArrayGetUBound, SafeArrayUnaccessData,
};

/// Numbers per box from `GetBoundingRectangles`: left, top, width, height.
const BOX: usize = 4;

/// A number a SAFEARRAY holds, for which every bit pattern is a value.
pub(crate) trait Number: Copy {}

impl Number for f64 {}
impl Number for i32 {}

/// Boxes from `left, top, width, height` groups; empty, non-finite and partial groups are dropped.
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    reason = "Boxes from other apps saturate at the edges of i32."
)]
pub fn rects_of(v: &[f64]) -> Vec<RECT> {
    v.as_chunks::<BOX>()
        .0
        .iter()
        .filter(|b| b.iter().all(|n| n.is_finite()) && b[2] > 0.0 && b[3] > 0.0)
        .map(|b| RECT {
            left: b[0].round() as i32,
            top: b[1].round() as i32,
            right: (b[0] + b[2]).round() as i32,
            bottom: (b[1] + b[3]).round() as i32,
        })
        .collect()
}

/// Reads and frees a one-dimensional SAFEARRAY of `T`; an array of another shape reads as empty.
///
/// # Safety
/// `sa` is null or a live array that we own, whose elements are `T` when they are `T`'s size.
#[allow(unsafe_code, reason = "Reads and frees an array Windows handed over.")]
pub(crate) unsafe fn vector<T: Number>(sa: *mut SAFEARRAY) -> windows::core::Result<Vec<T>> {
    if sa.is_null() {
        return Ok(Vec::new());
    }
    // SAFETY: the caller hands us a live array; with one dimension of `T`-sized elements, its data holds exactly `n`
    // values of `T`. It is unlocked and freed on every path.
    unsafe {
        let read = || -> windows::core::Result<Vec<T>> {
            let shaped =
                SafeArrayGetDim(sa) == 1 && SafeArrayGetElemsize(sa) as usize == size_of::<T>();
            if !shaped {
                return Ok(Vec::new());
            }
            let (lo, hi) = (SafeArrayGetLBound(sa, 1)?, SafeArrayGetUBound(sa, 1)?);
            let n = hi
                .checked_sub(lo)
                .and_then(|d| d.checked_add(1))
                .and_then(|d| usize::try_from(d).ok())
                .unwrap_or(0);
            if n == 0 {
                return Ok(Vec::new());
            }
            let mut data: *mut c_void = std::ptr::null_mut();
            SafeArrayAccessData(sa, &raw mut data)?;
            let v = std::slice::from_raw_parts(data.cast::<T>(), n).to_vec();
            SafeArrayUnaccessData(sa)?;
            Ok(v)
        };
        let got = read();
        let _ = SafeArrayDestroy(sa);
        got
    }
}

//! The PC's physical memory as Windows counts it: all of it and what is available now, which the Models page's "not
//! for this PC" reason shows.

use kx_module_api::ports::{MB, Memory};
use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

/// The PC's memory now; `None` when Windows refuses to tell.
#[allow(unsafe_code, reason = "Asks Windows to fill a status record of ours.")]
pub(super) fn now() -> Option<Memory> {
    let mut status = MEMORYSTATUSEX {
        dwLength: u32::try_from(size_of::<MEMORYSTATUSEX>()).ok()?,
        ..MEMORYSTATUSEX::default()
    };
    // SAFETY: `status` is a live local record whose `dwLength` names its own size, as the call requires.
    unsafe { GlobalMemoryStatusEx(&raw mut status) }.ok()?;
    Some(in_mb(status.ullTotalPhys, status.ullAvailPhys))
}

/// `total` and `free` bytes in whole MB, each within `u32`, free never past the total.
pub(super) fn in_mb(total: u64, free: u64) -> Memory {
    let mb = |bytes: u64| u32::try_from(bytes / MB).unwrap_or(u32::MAX);
    let total_mb = mb(total);
    Memory {
        total_mb,
        free_mb: mb(free).min(total_mb),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_read_as_whole_mb_with_free_never_past_the_total() {
        let got = in_mb(16 * 1024 * MB + MB / 2, 9 * 1024 * MB);
        assert_eq!((got.total_mb, got.free_mb), (16_384, 9_216));
        let odd = in_mb(4 * MB, 5 * MB);
        assert_eq!((odd.total_mb, odd.free_mb), (4, 4), "free is capped");
        let huge = in_mb(u64::MAX, u64::MAX);
        assert_eq!((huge.total_mb, huge.free_mb), (u32::MAX, u32::MAX));
    }

    #[test]
    fn this_pc_has_memory_and_some_of_it_free() {
        let memory = now().expect("Windows tells the memory");
        assert!(
            memory.total_mb > 0 && memory.free_mb <= memory.total_mb,
            "{memory:?}"
        );
    }
}

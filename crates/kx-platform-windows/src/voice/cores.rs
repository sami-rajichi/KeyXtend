//! The processor's real cores, as Windows lists them one record each, so `threads` 0 asks for every real core and
//! never a hybrid processor's second thread on a core.

use std::mem::offset_of;
use std::thread;

use windows::Win32::System::SystemInformation::{
    GetLogicalProcessorInformationEx, RelationProcessorCore,
    SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX,
};

/// Where a record's size sits, and the bytes before its body.
const SIZE_AT: usize = offset_of!(SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX, Size);
const HEAD: usize = SIZE_AT + size_of::<u32>();

/// The real cores, at least 1; the logical processors when Windows cannot list them.
pub(super) fn real() -> usize {
    let logical = || thread::available_parallelism().map_or(1, usize::from);
    let counted = listed().map(|buf| count(as_bytes(&buf)));
    counted.filter(|n| *n > 0).unwrap_or_else(logical)
}

/// The records Windows lists, one per core, in a buffer aligned for them.
#[allow(unsafe_code, reason = "Lets Windows fill a buffer of ours.")]
fn listed() -> Option<Vec<u64>> {
    let mut len = 0;
    // SAFETY: a size query with no buffer; it fails by design and only writes `len`.
    let _ = unsafe { GetLogicalProcessorInformationEx(RelationProcessorCore, None, &raw mut len) };
    let bytes = usize::try_from(len).ok()?;
    let mut buf = vec![0_u64; bytes.div_ceil(size_of::<u64>())];
    let at = Some(buf.as_mut_ptr().cast());
    // SAFETY: `buf` holds at least `len` writable bytes, aligned for the records, and Windows writes no more.
    unsafe { GetLogicalProcessorInformationEx(RelationProcessorCore, at, &raw mut len) }.ok()?;
    Some(buf)
}

/// `buf`'s bytes.
#[allow(unsafe_code, reason = "Views our own buffer as bytes.")]
fn as_bytes(buf: &[u64]) -> &[u8] {
    let len = size_of_val(buf);
    // SAFETY: any `u64` is plain bytes, and the slice covers exactly `buf`'s memory for its lifetime.
    unsafe { std::slice::from_raw_parts(buf.as_ptr().cast(), len) }
}

/// The records in `buf`, each with its size after its kind; one that claims less than its head or runs past the end
/// stops the count.
pub(super) fn count(buf: &[u8]) -> usize {
    let (mut at, mut n) = (0, 0);
    while let Some(size) = buf.get(at + SIZE_AT..at + HEAD) {
        let size = size.try_into().map_or(0, u32::from_ne_bytes);
        let size = usize::try_from(size).unwrap_or_default();
        if size < HEAD || at + size > buf.len() {
            break;
        }
        (at, n) = (at + size, n + 1);
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A buffer of records of `sizes`.
    fn records(sizes: &[u32]) -> Vec<u8> {
        let mut buf = Vec::new();
        for size in sizes {
            let mut record = vec![0; usize::try_from(*size).unwrap()];
            record[SIZE_AT..HEAD].copy_from_slice(&size.to_ne_bytes());
            buf.extend(record);
        }
        buf
    }

    #[test]
    fn each_record_is_one_core_and_a_broken_one_stops_the_count() {
        assert_eq!(count(&records(&[48, 48, 80])), 3);
        let mut broken = records(&[48, 48]);
        broken[48 + SIZE_AT] = 2;
        assert_eq!(count(&broken), 1, "a record smaller than its head");
        assert_eq!(count(&records(&[48])[..40]), 0, "a record past the end");
    }

    #[test]
    fn this_machine_has_at_least_one_real_core_and_no_more_than_its_threads() {
        let logical = thread::available_parallelism().map_or(1, usize::from);
        let cores = real();
        assert!((1..=logical).contains(&cores), "{cores} of {logical}");
    }
}

//! A box's mark: a stable 64-bit FNV-1a over its sealed bytes, kept for each box this computer writes, so a box found
//! later in a picked place is known as this computer's own or not. It guards no secret: the bytes are sealed already.

/// FNV-1a's 64-bit offset basis (Fowler, Noll and Vo; IETF draft-eastlake-fnv).
const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
/// FNV-1a's 64-bit prime.
const PRIME: u64 = 0x0000_0100_0000_01b3;

/// The mark of a box's `bytes`; saved blobs keep it, so it never changes.
#[must_use]
pub fn mark(bytes: &[u8]) -> u64 {
    bytes
        .iter()
        .fold(OFFSET, |h, &b| (h ^ u64::from(b)).wrapping_mul(PRIME))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_mark_matches_the_reference_vectors() {
        assert_eq!(mark(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(mark(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(mark(b"abc"), 0xe71f_a219_0541_574b);
    }

    #[test]
    fn two_seals_of_one_notebook_have_two_marks() {
        assert_ne!(mark(b"box one"), mark(b"box two"));
    }
}

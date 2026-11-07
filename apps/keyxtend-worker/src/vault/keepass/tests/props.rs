//! Properties: random bytes, with or without a KDBX version head, and a changed real file never panic a read.

use super::*;
use proptest::prelude::*;
use proptest::sample::Index;
use std::sync::OnceLock;

/// The longest junk a case reads.
const MAX_JUNK_BYTES: usize = 512;

/// The sample file, written once.
fn sample() -> &'static [u8] {
    static ONE: OnceLock<Vec<u8>> = OnceLock::new();
    ONE.get_or_init(|| Kdbx::sample().save())
}

proptest! {
    #[test]
    fn random_bytes_never_panic_a_keepass_read(junk in prop::collection::vec(any::<u8>(), 0..MAX_JUNK_BYTES)) {
        prop_assert!(read(&junk, PASSWORD).is_err());
        let headed = [&sample()[..12], &junk].concat();
        prop_assert!(read(&headed, PASSWORD).is_err());
    }

    #[test]
    fn a_changed_file_never_panics_a_keepass_read(pick in any::<Index>(), flip in 1..=u8::MAX) {
        let mut bytes = sample().to_vec();
        let at = pick.index(bytes.len());
        bytes[at] ^= flip;
        if let Ok((entries, _)) = read(&bytes, PASSWORD) {
            prop_assert_eq!(entries.len(), 5);
        }
    }
}

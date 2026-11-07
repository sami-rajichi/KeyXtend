//! Properties: any bytes, with or without a good head, never panic a box open; a changed box never opens to other
//! parts. The changed boxes are opened with the shared identity, so no case waits on scrypt.

use super::*;
use proptest::prelude::*;
use proptest::sample::Index;

/// The longest junk a case opens.
const MAX_JUNK_BYTES: usize = 512;

proptest! {
    #[test]
    fn random_bytes_never_panic_a_box_open(junk in prop::collection::vec(any::<u8>(), 0..MAX_JUNK_BYTES)) {
        prop_assert!(open(&junk, PASSWORD).is_err());
        let headed = [&BOX_MARK[..], &[BOX_VERSION], &junk].concat();
        prop_assert!(layout::read(&headed).and_then(|l| cipher::unlock(l.locked, PASSWORD)).is_err());
        prop_assert!(open_with(&headed).is_err());
    }

    #[test]
    fn a_changed_box_never_opens_to_other_parts(pick in any::<Index>(), flip in 1..=u8::MAX) {
        let mut bytes = sealed();
        let at = pick.index(bytes.len());
        bytes[at] ^= flip;
        if let Ok(parts) = open_with(&bytes) {
            prop_assert_eq!(parts, [notebook(NOTEBOOK)]);
        }
    }
}

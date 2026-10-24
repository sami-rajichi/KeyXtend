use super::*;
use proptest::prelude::*;

/// A blob name as the store's names go: lower-case letters, digits and dashes.
const NAME: &str = "[a-z0-9-]{1,32}";
/// The longest random blob a case seals.
const MAX_CASE_BYTES: usize = 4096;
/// The longest random input a case opens.
const MAX_JUNK_BYTES: usize = 256;

fn key() -> impl Strategy<Value = [u8; KEY_LEN]> {
    any::<[u8; KEY_LEN]>()
}

proptest! {
    #[test]
    fn open_never_panics_on_any_bytes(
        key in key(),
        aad in prop::collection::vec(any::<u8>(), 0..64),
        junk in prop::collection::vec(any::<u8>(), 0..MAX_JUNK_BYTES),
    ) {
        prop_assert!(open(&key, &aad, &junk).is_err());
    }

    #[test]
    fn open_never_panics_after_a_good_header(
        key in key(),
        tail in prop::collection::vec(any::<u8>(), 0..MAX_JUNK_BYTES),
    ) {
        let mut junk = MAGIC.to_vec();
        junk.push(VERSION);
        junk.extend_from_slice(&tail);
        prop_assert!(open(&key, b"", &junk).is_err());
    }

    #[test]
    fn seal_then_open_round_trips_under_its_name(
        key in key(),
        name in NAME,
        other in NAME,
        plain in prop::collection::vec(any::<u8>(), 0..MAX_CASE_BYTES),
    ) {
        let sealed = seal(&key, name.as_bytes(), &plain).unwrap();
        let opened = open(&key, name.as_bytes(), &sealed).unwrap();
        prop_assert_eq!(opened.as_slice(), plain.as_slice());
        if other != name {
            prop_assert_eq!(open(&key, other.as_bytes(), &sealed), Err(CryptoError::Refused));
        }
    }

    #[test]
    fn a_random_flipped_bit_is_refused(
        key in key(),
        plain in prop::collection::vec(any::<u8>(), 0..MAX_JUNK_BYTES),
        pick in any::<prop::sample::Index>(),
    ) {
        let mut sealed = seal(&key, b"", &plain).unwrap();
        let bit = pick.index(sealed.len() * 8);
        sealed[bit / 8] ^= 1 << (bit % 8);
        prop_assert!(open(&key, b"", &sealed).is_err());
    }
}

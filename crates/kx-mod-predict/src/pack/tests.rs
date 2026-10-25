//! The pack format's tests: refused headers and random bytes here; the round trip, the writer and every damaged
//! pack in the files below, which need the writer (`--features build`).

use std::hint::black_box;

use kx_module_api::ports::MAX_PACK_BYTES;
use proptest::prelude::*;

use super::layout::{AT_VERSION, MAGIC, VERSION};
use super::*;

#[cfg(feature = "build")]
mod damage;
#[cfg(feature = "build")]
mod trip;
#[cfg(feature = "build")]
mod writer;

/// Random packs tried by each property test.
const CASES: u32 = 512;
/// The longest random pack tried.
const MAX_RANDOM: usize = 2_048;
/// The most ids `read_all` reads, so a damaged word count cannot make a test crawl.
const MAX_READS: u32 = 64;
/// Prefixes and keys `read_all` looks up: empty, Latin, folded Arabic and unknown.
const LOOKUPS: [&str; 5] = ["", "a", "eco", "ان", "zz"];

/// Reads everything `pack` offers, so a test fails if any read panics.
pub(super) fn read_all(pack: &Pack<'_>) {
    black_box((
        pack.iso(),
        pack.word_count(),
        pack.max_chain(),
        pack.flags(),
    ));
    let ids = 0..pack.word_count().saturating_add(2).min(MAX_READS);
    for id in ids.clone() {
        black_box((pack.spelling(id), pack.freq(id)));
        black_box(pack.followers(&[id]).count());
        black_box(pack.followers(&[id, id.wrapping_add(1)]).count());
    }
    for key in LOOKUPS {
        black_box(pack.completions(key).count());
        black_box(pack.run(key));
    }
    let long: Vec<WordId> = ids.collect();
    black_box(pack.followers(&long).count());
    black_box(pack.followers(&[]).count());
    black_box(pack.followers(&[WordId::MAX]).count());
}

/// The magic and this version, the start every random pack below shares.
fn start() -> Vec<u8> {
    let mut bytes = MAGIC.to_vec();
    bytes.extend_from_slice(&VERSION.to_le_bytes());
    bytes
}

#[test]
fn bytes_without_the_magic_are_not_a_pack() {
    assert_eq!(Pack::parse(&[]).unwrap_err(), PackFormatError::BadMagic);
    assert_eq!(Pack::parse(b"KXW").unwrap_err(), PackFormatError::BadMagic);
    assert_eq!(
        Pack::parse(b"PK\x03\x04 a zip").unwrap_err(),
        PackFormatError::BadMagic
    );
}

#[test]
fn an_older_or_newer_version_is_refused_and_named() {
    // Version 1 held capital flags, which the spellings now hold.
    for other in [1, VERSION - 1, VERSION + 1] {
        let mut bytes = start();
        bytes[AT_VERSION..].copy_from_slice(&other.to_le_bytes());
        let refused = Pack::parse(&bytes).unwrap_err();
        assert_eq!(refused, PackFormatError::BadVersion(other));
        assert!(refused.to_string().contains(&other.to_string()));
    }
}

#[test]
fn a_header_cut_short_is_out_of_bounds() {
    assert_eq!(
        Pack::parse(&MAGIC).unwrap_err(),
        PackFormatError::OutOfBounds
    );
    assert_eq!(
        Pack::parse(&start()).unwrap_err(),
        PackFormatError::OutOfBounds
    );
}

#[test]
fn flags_round_trip_and_unknown_flags_are_refused() {
    for checked in [false, true] {
        for chains in [false, true] {
            let flags = Flags { checked, chains };
            assert_eq!(Flags::from_byte(flags.to_byte()), Some(flags));
        }
    }
    assert_eq!(Flags::from_byte(!layout::KNOWN_FLAGS), None);
}

#[test]
fn a_pack_fits_up_to_the_largest_size_a_platform_maps() {
    let largest = usize::try_from(MAX_PACK_BYTES).unwrap();
    assert!(layout::fits_pack(largest));
    assert!(!layout::fits_pack(largest + 1));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn random_bytes_are_refused_or_read_safely(bytes in prop::collection::vec(any::<u8>(), 0..MAX_RANDOM)) {
        if let Ok(pack) = Pack::parse(&bytes) {
            read_all(&pack);
        }
    }

    #[test]
    fn random_bytes_after_the_magic_are_refused_or_read_safely(
        tail in prop::collection::vec(any::<u8>(), 0..MAX_RANDOM),
    ) {
        let mut bytes = start();
        bytes.extend(tail);
        if let Ok(pack) = Pack::parse(&bytes) {
            read_all(&pack);
        }
    }
}

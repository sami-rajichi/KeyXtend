//! Damaged packs: every truncation, every single-byte change before parse, and changes after parse (a mapped file
//! changed under the view) are refused or read without a panic.

use std::ops::Range;

use proptest::prelude::*;

use super::super::layout::{
    AT_FLAGS, AT_FOLLOWERS, AT_ISO, AT_ISO_LEN, AT_MAX_CHAIN, AT_SECTIONS, AT_WORDS,
    CHAIN_AT_TOTAL, CHAINS, CHAINS_BASE, ENTRY_AT_LEN, ENTRY_AT_OFFSET, ENTRY_LEN, HEAD_LEN,
    KNOWN_FLAGS, LEXICON, MAX_CHAIN, MAX_ISO, MAX_WORDS, MIN_CHAIN, WORDS, u8_at, usize_at,
};
use super::super::*;
use super::trip::sample;
use super::{CASES, read_all};

/// The changes each byte gets: its low bit, its high bit, and every bit.
const FLIPS: [u8; 3] = [0x01, 0x80, 0xFF];
/// The most bytes one random case changes.
const MAX_CHANGES: usize = 8;

/// The table entry and the span of section `kind` in `bytes`.
fn section(bytes: &[u8], kind: u8) -> (usize, Range<usize>) {
    let count = usize::from(u8_at(bytes, AT_SECTIONS).unwrap());
    let mut entries = (0..count).map(|i| HEAD_LEN + i * ENTRY_LEN);
    let at = entries.find(|&at| bytes[at] == kind).unwrap();
    let start = usize_at(bytes, at + ENTRY_AT_OFFSET).unwrap();
    (
        at,
        start..start + usize_at(bytes, at + ENTRY_AT_LEN).unwrap(),
    )
}

/// `full` with the byte at `at` changed by `flip`.
fn changed(full: &[u8], at: usize, flip: u8) -> Vec<u8> {
    let mut bytes = full.to_vec();
    bytes[at] ^= flip;
    bytes
}

/// The sample with `edit` applied, parsed.
fn edited(edit: impl FnOnce(&mut Vec<u8>)) -> Result<(), PackFormatError> {
    let mut bytes = sample();
    edit(&mut bytes);
    Pack::parse(&bytes).map(|_| ())
}

#[test]
fn a_bad_header_field_is_refused() {
    let bad = Err(PackFormatError::BadHeader);
    assert_eq!(edited(|b| b[AT_FLAGS] |= !KNOWN_FLAGS), bad);
    assert_eq!(edited(|b| b[AT_FLAGS] &= !CHAINS), bad, "chains unflagged");
    assert_eq!(edited(|b| b[AT_ISO_LEN] = 0), bad);
    let long = u8::try_from(MAX_ISO + 1).unwrap();
    assert_eq!(edited(|b| b[AT_ISO_LEN] = long), bad);
    assert_eq!(edited(|b| b[AT_ISO] = b' '), bad);
    assert_eq!(edited(|b| b[AT_MAX_CHAIN] = 0), bad);
    assert_eq!(edited(|b| b[AT_MAX_CHAIN] = MAX_CHAIN + 1), bad);
    assert_eq!(edited(|b| b[AT_FOLLOWERS] = 0), bad);
    let too_many = u32::try_from(MAX_WORDS + 1).unwrap().to_le_bytes();
    assert_eq!(
        edited(|b| b[AT_WORDS..AT_MAX_CHAIN].copy_from_slice(&too_many)),
        bad
    );
}

#[test]
fn a_bad_section_table_is_refused() {
    let bad = Err(PackFormatError::BadSection);
    let lexicon = section(&sample(), LEXICON).0;
    assert_eq!(edited(|b| b[lexicon] = CHAINS_BASE), bad, "an unknown kind");
    let words = section(&sample(), WORDS).0;
    assert_eq!(edited(|b| b[words] = LEXICON), bad, "the lexicon twice");
    let unused = CHAINS_BASE + MAX_CHAIN;
    assert_eq!(
        edited(|b| b[words] = unused),
        bad,
        "a chain past the longest"
    );
    let offset = lexicon + ENTRY_AT_OFFSET..lexicon + ENTRY_AT_LEN;
    assert_eq!(
        edited(|b| b[offset.clone()].fill(0)),
        bad,
        "inside the table"
    );
    let past = Err(PackFormatError::OutOfBounds);
    assert_eq!(edited(|b| b[offset.end - 1] = u8::MAX), past);
}

#[test]
fn a_chain_section_whose_counts_do_not_add_up_is_refused() {
    let chains = section(&sample(), CHAINS_BASE + MIN_CHAIN).1;
    let bad = Err(PackFormatError::BadSection);
    assert_eq!(edited(|b| b[chains.start] += 1), bad, "one context more");
    let total = chains.start + CHAIN_AT_TOTAL;
    assert_eq!(edited(|b| b[total] += 1), bad, "one follower more");
}

#[test]
fn a_spelling_that_is_not_utf8_is_refused() {
    let words = section(&sample(), WORDS).1;
    assert_eq!(
        edited(|b| b[words.end - 1] = 0xFF),
        Err(PackFormatError::BadUtf8)
    );
}

#[test]
fn every_truncation_is_refused() {
    let full = sample();
    for len in 0..full.len() {
        assert!(Pack::parse(&full[..len]).is_err(), "cut to {len}");
    }
}

#[test]
fn every_lexicon_change_is_refused_by_its_checksum() {
    let full = sample();
    for at in section(&full, LEXICON).1 {
        for flip in FLIPS {
            let bytes = changed(&full, at, flip);
            assert_eq!(
                Pack::parse(&bytes).unwrap_err(),
                PackFormatError::BadSection,
                "byte {at}"
            );
        }
    }
}

#[test]
fn every_byte_change_is_refused_or_reads_safely() {
    let full = sample();
    for at in 0..full.len() {
        for flip in FLIPS {
            if let Ok(pack) = Pack::parse(&changed(&full, at, flip)) {
                read_all(&pack);
            }
        }
    }
}

#[test]
fn a_byte_changed_after_parse_never_panics_a_read() {
    let full = sample();
    let pack = Pack::parse(&full).unwrap();
    let lexicon = section(&full, LEXICON).1;
    for at in (0..full.len()).filter(|at| !lexicon.contains(at)) {
        for flip in FLIPS {
            let bytes = changed(&full, at, flip);
            read_all(&pack.altered(&bytes).unwrap());
        }
    }
}

/// Up to `MAX_CHANGES` changes: each a place, as a share of the pack, and a nonzero flip.
fn changes() -> impl Strategy<Value = Vec<(prop::sample::Index, u8)>> {
    prop::collection::vec((any::<prop::sample::Index>(), 1..=u8::MAX), 1..=MAX_CHANGES)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn several_changes_are_refused_or_read_safely(changes in changes()) {
        let mut bytes = sample();
        for (at, flip) in changes {
            let at = at.index(bytes.len());
            bytes[at] ^= flip;
        }
        if let Ok(pack) = Pack::parse(&bytes) {
            read_all(&pack);
        }
    }

    #[test]
    fn several_changes_after_parse_never_panic_a_read(changes in changes()) {
        let full = sample();
        let pack = Pack::parse(&full).unwrap();
        let lexicon = section(&full, LEXICON).1;
        let outside: Vec<usize> = (0..full.len()).filter(|at| !lexicon.contains(at)).collect();
        let mut bytes = full.clone();
        for (at, flip) in changes {
            bytes[*at.get(&outside)] ^= flip;
        }
        read_all(&pack.altered(&bytes).unwrap());
    }
}

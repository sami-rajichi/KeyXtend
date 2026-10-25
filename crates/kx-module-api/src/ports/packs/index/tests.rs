//! The pack index: the build's list of shipped languages, written sorted and read back, every damage an error.

use super::*;
use proptest::prelude::*;

/// Random runs per property.
const CASES: u32 = 512;

#[test]
fn the_index_lists_each_code_once_sorted_one_per_line() {
    let text = index_text(&["fr", "en", "ar", "en", "fil"]).unwrap();
    assert_eq!(text, "ar\nen\nfil\nfr\n");
    let read = read_index(text.as_bytes()).unwrap();
    assert_eq!(
        read.into_iter().collect::<Vec<_>>(),
        ["ar", "en", "fil", "fr"]
    );
}

#[test]
fn an_empty_index_ships_nothing() {
    assert_eq!(index_text(&[]).as_deref(), Ok(""));
    assert_eq!(read_index(b""), Ok(BTreeSet::new()));
}

#[test]
fn windows_line_ends_read_the_same() {
    let read = read_index(b"ar\r\nen\r\n").unwrap();
    assert_eq!(read.into_iter().collect::<Vec<_>>(), ["ar", "en"]);
}

#[test]
fn a_bad_code_or_text_is_damage_both_ways() {
    assert_eq!(index_text(&["en", "EN"]), Err(IndexError::Damaged));
    for bad in [
        &b"en\nEN\n"[..],
        b"en\n\nfr\n",
        b"../x\n",
        b"con\n",
        b"en \n",
        b"\xFF\n",
    ] {
        assert_eq!(read_index(bad), Err(IndexError::Damaged), "{bad:?}");
    }
}

#[test]
fn each_error_names_the_index() {
    for e in [
        IndexError::Missing,
        IndexError::Unreadable,
        IndexError::Damaged,
    ] {
        assert!(e.to_string().contains("index"), "{e}");
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn any_bytes_are_read_or_refused_without_a_panic(bytes in prop::collection::vec(any::<u8>(), 0..64)) {
        let _ = read_index(&bytes);
    }

    #[test]
    fn any_codes_round_trip(codes in prop::collection::vec("[a-z]{2,3}", 0..8)) {
        let codes: Vec<&str> = codes.iter().map(String::as_str).filter(|c| fits_iso(c)).collect();
        let text = index_text(&codes).unwrap();
        let want: BTreeSet<String> = codes.iter().map(|c| (*c).to_owned()).collect();
        prop_assert_eq!(read_index(text.as_bytes()), Ok(want));
    }
}

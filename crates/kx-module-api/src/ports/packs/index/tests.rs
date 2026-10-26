//! The pack index: the build's list of shipped languages and the layout codes each serves, written sorted and read
//! back, every damage an error.

use super::*;
use proptest::prelude::*;

/// Random runs per property.
const CASES: u32 = 512;

/// No layout code served besides the packs' own.
fn none() -> BTreeMap<String, Vec<String>> {
    BTreeMap::new()
}

/// Wordfreq's Serbo-Croatian pack serving the Croatian and Bosnian layouts.
fn sh() -> BTreeMap<String, Vec<String>> {
    BTreeMap::from([("sh".to_owned(), vec!["hr".to_owned(), "bs".to_owned()])])
}

/// The packs `index` lists.
fn packs(index: &PackIndex) -> Vec<&str> {
    index.packs().collect()
}

#[test]
fn the_index_lists_each_code_once_sorted_one_per_line() {
    let index = PackIndex::new(&["fr", "en", "ar", "en", "fil"], &none()).unwrap();
    assert_eq!(index.text(), "ar\nen\nfil\nfr\n");
    let read = PackIndex::read(index.text().as_bytes()).unwrap();
    assert_eq!(packs(&read), ["ar", "en", "fil", "fr"]);
}

#[test]
fn a_pack_serves_its_own_code_and_the_layout_codes_listed_after_it() {
    let index = PackIndex::new(&["en", "sh"], &sh()).unwrap();
    assert_eq!(index.text(), "en\nsh bs hr\n");
    let read = PackIndex::read(index.text().as_bytes()).unwrap();
    assert_eq!(read, index);
    for (iso, pack) in [
        ("en", Some("en")),
        ("sh", Some("sh")),
        ("hr", Some("sh")),
        ("bs", Some("sh")),
    ] {
        assert_eq!(read.pack_of(iso), pack, "{iso}");
    }
    assert_eq!(
        read.pack_of("sr"),
        None,
        "Serbian has no Latin tag to go by"
    );
    assert_eq!(packs(&read), ["en", "sh"], "an alias is no pack");
}

#[test]
fn aliases_of_a_pack_not_built_are_left_out() {
    let index = PackIndex::new(&["en"], &sh()).unwrap();
    assert_eq!(index.text(), "en\n");
    assert_eq!(index.pack_of("hr"), None);
}

#[test]
fn an_index_that_lists_no_pack_is_refused_both_ways() {
    assert_eq!(PackIndex::new(&[], &none()), Err(IndexError::Empty));
    assert_eq!(PackIndex::new(&[], &sh()), Err(IndexError::Empty));
    assert_eq!(PackIndex::read(b""), Err(IndexError::Empty));
    assert_eq!(PackIndex::default().pack_of("en"), None);
}

#[test]
fn an_index_past_its_cap_is_unreadable() {
    let cap = usize::try_from(MAX_INDEX_BYTES).unwrap();
    let line = "en\n";
    let fits = line.repeat(cap / line.len());
    assert!(PackIndex::read(fits.as_bytes()).is_ok());
    let over = format!("{fits}{line}");
    assert!(over.len() > cap);
    assert_eq!(
        PackIndex::read(over.as_bytes()),
        Err(IndexError::Unreadable)
    );
}

#[test]
fn a_layout_code_and_its_pack_clash_in_either_order() {
    for bad in [&b"sh en\nen\n"[..], b"en\nsh en\n", b"sh hr\nhr\n"] {
        assert_eq!(PackIndex::read(bad), Err(IndexError::Damaged), "{bad:?}");
    }
}

#[test]
fn windows_line_ends_read_the_same() {
    let read = PackIndex::read(b"ar\r\nsh hr\r\n").unwrap();
    assert_eq!(packs(&read), ["ar", "sh"]);
    assert_eq!(read.pack_of("hr"), Some("sh"));
}

#[test]
fn a_bad_code_or_text_is_damage_both_ways() {
    assert_eq!(
        PackIndex::new(&["en", "EN"], &none()),
        Err(IndexError::Damaged)
    );
    let bad_alias = BTreeMap::from([("en".to_owned(), vec!["HR".to_owned()])]);
    assert_eq!(
        PackIndex::new(&["en"], &bad_alias),
        Err(IndexError::Damaged)
    );
    for bad in [
        &b"en\nEN\n"[..],
        b"en\n\nfr\n",
        b"../x\n",
        b"con\n",
        b"en \n",
        b"sh  hr\n",
        b"sh con\n",
        b"\xFF\n",
    ] {
        assert_eq!(PackIndex::read(bad), Err(IndexError::Damaged), "{bad:?}");
    }
}

#[test]
fn a_layout_code_that_is_a_pack_or_served_twice_is_damage() {
    let pack_too = BTreeMap::from([("sh".to_owned(), vec!["en".to_owned()])]);
    assert_eq!(
        PackIndex::new(&["en", "sh"], &pack_too),
        Err(IndexError::Damaged)
    );
    for bad in [&b"en\nsh en\n"[..], b"sh hr\nsl hr\n"] {
        assert_eq!(PackIndex::read(bad), Err(IndexError::Damaged), "{bad:?}");
    }
}

#[test]
fn merging_two_folders_keeps_every_pack_and_alias() {
    let mut first = PackIndex::read(b"en\n").unwrap();
    first
        .merge(&PackIndex::read(b"fr\nsh hr\n").unwrap())
        .unwrap();
    assert_eq!(packs(&first), ["en", "fr", "sh"]);
    assert_eq!(first.pack_of("hr"), Some("sh"));
    let same = first.clone();
    first.merge(&same).unwrap();
    assert_eq!(first, same, "the same index twice changes nothing");
}

#[test]
fn merging_folders_that_clash_is_refused_and_keeps_the_first() {
    for (first, other) in [
        (&b"sh hr\n"[..], &b"sl hr\n"[..]),
        (b"sh hr\n", b"hr\n"),
        (b"hr\n", b"sh hr\n"),
    ] {
        let mut index = PackIndex::read(first).unwrap();
        let kept = index.clone();
        let other = PackIndex::read(other).unwrap();
        assert_eq!(index.merge(&other), Err(IndexError::Damaged), "{first:?}");
        assert_eq!(index, kept, "{first:?}");
    }
}

#[test]
fn each_error_names_the_index() {
    for e in [
        IndexError::Missing,
        IndexError::Unreadable,
        IndexError::Damaged,
        IndexError::Empty,
    ] {
        assert!(e.to_string().contains("index"), "{e}");
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn any_bytes_are_read_or_refused_without_a_panic(bytes in prop::collection::vec(any::<u8>(), 0..64)) {
        let _ = PackIndex::read(&bytes);
    }

    #[test]
    fn any_codes_round_trip(codes in prop::collection::vec("[a-z]{2,3}", 1..8)) {
        let codes: Vec<&str> = codes.iter().map(String::as_str).filter(|c| fits_iso(c)).collect();
        prop_assume!(!codes.is_empty());
        let index = PackIndex::new(&codes, &none()).unwrap();
        prop_assert_eq!(PackIndex::read(index.text().as_bytes()), Ok(index));
    }
}

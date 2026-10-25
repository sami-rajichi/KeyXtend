//! The saved emails' byte form: the round trip, and every damage refused as an error, never a panic.

use proptest::prelude::*;

use super::*;

/// The most emails the tests keep.
const MAX: usize = 50;

/// Known emails from `list`, most recent first.
fn emails(list: &[&str]) -> Emails {
    let list: Vec<String> = list.iter().map(|e| (*e).to_owned()).collect();
    Emails::new(&list, MAX)
}

#[test]
fn the_form_reads_back_equal_in_the_same_order() {
    let known = emails(&["sami@x.org", "نور@مثال.عرب", "a.b+c@mail.co.uk"]);
    let bytes = known.to_bytes();
    assert_eq!(bytes.get(..MAGIC.len()), Some(&MAGIC[..]));
    assert_eq!(Emails::from_bytes(&bytes, MAX), Ok(known));
    let none = Emails::default();
    assert_eq!(Emails::from_bytes(&none.to_bytes(), MAX), Ok(none));
}

#[test]
fn a_lowered_cap_keeps_the_most_recent() {
    let known = emails(&["a@x.org", "b@x.org", "c@x.org"]);
    let kept = Emails::from_bytes(&known.to_bytes(), 2).unwrap();
    assert_eq!(kept.list(), ["a@x.org", "b@x.org"]);
}

#[test]
fn a_wrong_magic_or_version_is_refused() {
    let mut bytes = emails(&["a@x.org"]).to_bytes();
    let mut magic = bytes.clone();
    magic[0] ^= 1;
    assert_eq!(
        Emails::from_bytes(&magic, MAX),
        Err(EmailFormatError::BadMagic)
    );
    let next = VERSION + 1;
    bytes.splice(MAGIC.len()..MAGIC.len() + 2, next.to_le_bytes());
    assert_eq!(
        Emails::from_bytes(&bytes, MAX),
        Err(EmailFormatError::BadVersion(next))
    );
}

#[test]
fn every_cut_is_truncated_and_extra_bytes_are_refused() {
    let bytes = emails(&["sami@x.org", "b@y.com"]).to_bytes();
    for end in 0..bytes.len() {
        let got = Emails::from_bytes(&bytes[..end], MAX);
        assert_eq!(got, Err(EmailFormatError::Truncated), "cut at {end}");
    }
    let mut long = bytes;
    long.push(0);
    assert_eq!(
        Emails::from_bytes(&long, MAX),
        Err(EmailFormatError::BadEntry)
    );
}

#[test]
fn an_entry_that_is_no_email_or_no_utf8_is_refused() {
    let one = emails(&["ab@x.org"]).to_bytes();
    let at = one.len() - "ab@x.org".len();
    let mut word = one.clone();
    word.splice(at.., b"abcxorg".iter().chain(b"!").copied());
    assert_eq!(
        Emails::from_bytes(&word, MAX),
        Err(EmailFormatError::BadEntry)
    );
    let mut broken = one;
    broken[at] = 0xFF;
    assert_eq!(
        Emails::from_bytes(&broken, MAX),
        Err(EmailFormatError::BadEntry)
    );
}

#[test]
fn debug_of_the_error_names_no_email() {
    let shown = format!("{:?}", EmailFormatError::BadEntry);
    assert!(!shown.contains('@'), "{shown}");
}

proptest! {
    #[test]
    fn any_bytes_are_read_or_refused_without_a_panic(bytes in proptest::collection::vec(any::<u8>(), 0..64)) {
        let _ = Emails::from_bytes(&bytes, MAX);
    }

    #[test]
    fn any_change_to_a_form_is_read_or_refused_without_a_panic(at in 0usize..64, flip in 1u8..=255) {
        let mut bytes = emails(&["sami@x.org", "b@y.com"]).to_bytes();
        let at = at % bytes.len();
        bytes[at] ^= flip;
        if let Ok(read) = Emails::from_bytes(&bytes, MAX) {
            prop_assert!(read.list().iter().all(|e| crate::email::looks_like(e)));
        }
    }

    #[test]
    fn any_known_list_round_trips(locals in proptest::collection::vec("[a-z]{1,8}", 0..12)) {
        let list: Vec<String> = locals.iter().map(|l| format!("{l}@x.org")).collect();
        let known = Emails::new(&list, MAX);
        prop_assert_eq!(Emails::from_bytes(&known.to_bytes(), MAX), Ok(known));
    }
}

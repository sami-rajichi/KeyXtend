//! The byte form: its layout, the round trip, and every damage refused as an error, never a panic.

use super::super::bytes::MAX_TEXT;
use super::*;

/// One word as the byte form lists it: its id, spelling, count and day.
pub(super) type W<'a> = (u16, &'a str, u16, u32);
/// One chain: its ids, count and day.
pub(super) type C<'a> = (&'a [u16], u16, u32);
/// One blocked word: its spelling, uses and day.
type B<'a> = (&'a str, u16, u32);

/// Settings keeping one chain length.
pub(super) const ONE: LearnConfig = LearnConfig {
    context_words: 1,
    ..CFG
};

/// Appends `text` with its one-byte length.
fn text(out: &mut Vec<u8>, text: &str) {
    out.push(u8::try_from(text.len()).unwrap());
    out.extend(text.as_bytes());
}

/// The byte form of `iso` holding `words`, the chain `tables` and `blocked`, written by hand.
pub(super) fn encode(
    iso: &str,
    words: &[W<'_>],
    tables: &[&[C<'_>]],
    blocked: &[B<'_>],
) -> Vec<u8> {
    let len = |n: usize| u32::try_from(n).unwrap().to_le_bytes();
    let mut out = MAGIC.to_vec();
    out.extend(VERSION.to_le_bytes());
    text(&mut out, iso);
    out.extend(len(words.len()));
    for (id, spelling, count, day) in words {
        out.extend(id.to_le_bytes());
        text(&mut out, spelling);
        out.extend(count.to_le_bytes());
        out.extend(day.to_le_bytes());
    }
    out.push(u8::try_from(tables.len()).unwrap());
    for table in tables {
        out.extend(len(table.len()));
        for (key, count, day) in *table {
            key.iter().for_each(|id| out.extend(id.to_le_bytes()));
            out.extend(count.to_le_bytes());
            out.extend(day.to_le_bytes());
        }
    }
    out.extend(len(blocked.len()));
    for (spelling, uses, day) in blocked {
        text(&mut out, spelling);
        out.extend(uses.to_le_bytes());
        out.extend(day.to_le_bytes());
    }
    out
}

/// Two words and the chain between them.
fn small() -> Vec<u8> {
    let words = [(0, "a", 1, DAY), (1, "b", 1, DAY)];
    encode("en", &words, &[&[(&[0, 1], 1, DAY)]], &[])
}

#[test]
fn the_byte_form_is_laid_out_as_documented() {
    let mut l = Learner::new("en", ONE).unwrap();
    teach(&mut l, None, "a b", DAY);
    assert_eq!(l.to_bytes(), small());
    assert_eq!(Learner::from_bytes(&small(), ONE), Ok(l));
}

#[test]
fn the_byte_form_round_trips_to_an_equal_learner() {
    let cfg = LearnConfig {
        context_words: 4,
        ..CFG
    };
    let mut l = Learner::new("ar", cfg).unwrap();
    assert_eq!(Learner::from_bytes(&l.to_bytes(), cfg), Ok(l.clone()));
    teach(&mut l, None, "انا في المدرسة و انا في البيت", DAY);
    teach(&mut l, None, "في البيت انا", DAY + 1);
    assert!(l.forget(None, "و", DAY + 1), "a hole among the ids");
    let back = Learner::from_bytes(&l.to_bytes(), cfg).unwrap();
    assert_eq!(back, l);
    assert_eq!(back.to_bytes(), l.to_bytes());
}

#[cfg(feature = "build")]
#[test]
fn the_blocklist_round_trips() {
    let bytes = super::seed::english();
    let en = Pack::parse(&bytes).unwrap();
    let mut l = learner("en");
    teach(&mut l, Some(&en), "in the theory", DAY);
    assert!(l.forget(Some(&en), "the", DAY));
    assert!(l.forget(Some(&en), "London", DAY));
    l.learn(Some(&en), "the", &[], DAY + 1);
    assert_eq!(Learner::from_bytes(&l.to_bytes(), CFG), Ok(l));
}

#[test]
fn a_bad_header_is_refused() {
    let good = small();
    let mut magic = good.clone();
    magic[0] ^= 1;
    assert_eq!(
        Learner::from_bytes(&magic, ONE),
        Err(LearnFormatError::BadMagic)
    );
    let mut version = good.clone();
    version[MAGIC.len()] += 1;
    let next = Err(LearnFormatError::BadVersion(VERSION + 1));
    assert_eq!(Learner::from_bytes(&version, ONE), next);
    let iso = encode("EN", &[], &[], &[]);
    assert_eq!(
        Learner::from_bytes(&iso, ONE),
        Err(LearnFormatError::BadIso)
    );
}

#[test]
fn every_truncation_and_a_trailing_byte_are_refused() {
    let good = small();
    for end in 0..good.len() {
        assert!(Learner::from_bytes(&good[..end], ONE).is_err(), "{end}");
    }
    let mut long = good.clone();
    long.push(0);
    assert_eq!(
        Learner::from_bytes(&long, ONE),
        Err(LearnFormatError::BadEntry)
    );
}

/// True when `bytes` are refused as holding a bad entry.
fn refused(bytes: &[u8]) -> bool {
    Learner::from_bytes(bytes, CFG) == Err(LearnFormatError::BadEntry)
}

#[test]
fn damaged_words_are_refused() {
    let a = (0, "a", 1, DAY);
    assert!(
        refused(&encode("en", &[a, (0, "b", 1, DAY)], &[], &[])),
        "an id twice"
    );
    let far = (u16::MAX, "a", 1, DAY);
    assert!(
        refused(&encode("en", &[far, (u16::MAX, "b", 1, DAY)], &[], &[])),
        "an id twice among ids renumbered"
    );
    assert!(
        refused(&encode("en", &[a, (1, "a", 1, DAY)], &[], &[])),
        "a spelling twice"
    );
    assert!(
        refused(&encode("en", &[(0, "a", 0, DAY)], &[], &[])),
        "no uses"
    );
    assert!(
        refused(&encode("en", &[(0, "", 1, DAY)], &[], &[])),
        "an empty spelling"
    );
    let mut utf8 = encode("en", &[a], &[], &[]);
    let at = utf8.iter().rposition(|&b| b == b'a').unwrap();
    utf8[at] = u8::MAX;
    assert!(refused(&utf8), "not UTF-8");
}

#[test]
fn damaged_chains_are_refused() {
    let (a, b) = ((0, "a", 1, DAY), (1, "b", 1, DAY));
    let chain: C<'_> = (&[0, 1], 1, DAY);
    assert!(
        refused(&encode("en", &[a], &[&[chain]], &[])),
        "a missing word"
    );
    let swapped = [(&[1, 0][..], 1, DAY), chain];
    assert!(
        refused(&encode("en", &[a, b], &[&swapped], &[])),
        "out of order"
    );
    assert!(
        refused(&encode("en", &[a, b], &[&[chain, chain]], &[])),
        "a chain twice"
    );
    let unused: C<'_> = (&[0, 1], 0, DAY);
    assert!(
        refused(&encode("en", &[a, b], &[&[unused]], &[])),
        "no uses"
    );
    let five: Vec<&[C<'_>]> = vec![&[]; usize::from(MAX_CHAIN)];
    assert!(refused(&encode("en", &[], &five, &[])), "too many tables");
}

#[test]
fn a_damaged_blocklist_is_refused() {
    let a = (0, "a", 1, DAY);
    assert!(
        refused(&encode("en", &[a], &[], &[("a", 0, DAY)])),
        "a blocked word learned"
    );
    let swapped = [("b", 0, DAY), ("a", 0, DAY)];
    assert!(refused(&encode("en", &[], &[], &swapped)), "out of order");
    assert!(
        refused(&encode("en", &[], &[], &[("", 0, DAY)])),
        "an empty spelling"
    );
}

#[test]
fn a_word_longer_than_the_byte_form_allows_is_not_learned() {
    let mut l = learner("en");
    let long = "a".repeat(MAX_TEXT + 1);
    l.learn(None, &long, &[], DAY);
    l.learn(None, &long, &[], DAY);
    assert_eq!((l.words.len(), l.view(None).count(&long)), (0, 0));
    assert_eq!(Learner::from_bytes(&l.to_bytes(), CFG), Ok(l));
}

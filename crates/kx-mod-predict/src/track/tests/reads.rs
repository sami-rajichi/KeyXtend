//! The tracker's reads: they fill the context, keep the last word open and learn nothing.

use super::*;

#[test]
fn a_read_fills_the_context_and_its_last_word_stays_open() {
    let mut t = tracker(3);
    t.reset();
    t.read(Some("je vais à l'éc"), &knows);
    let show = t.show(&knows);
    assert!(show.read);
    let split = (
        owned(&["vais", "à", "l'"]),
        "l'".to_owned(),
        "éc".to_owned(),
    );
    assert!(readings(&show).contains(&split));
    assert_eq!(
        show.words.map(|q| q.word.expose().clone()),
        Some("l'éc".to_owned())
    );
    assert_eq!(t.context(), Some(owned(&["je", "vais", "à"]).as_slice()));
}

#[test]
fn a_read_word_is_learned_only_when_some_of_it_was_typed() {
    let mut t = tracker(2);
    t.read(Some("hel"), &knows);
    assert!(learned(&type_text(&mut t, " ")).is_empty());
    t.read(Some("hel"), &knows);
    let told = type_text(&mut t, "lo ");
    assert_eq!(learned(&told), [("hello".to_owned(), owned(&[]))]);
}

#[test]
fn a_read_learns_nothing_and_an_empty_read_knows_the_box_is_empty() {
    let mut t = tracker(2);
    t.read(Some("hello world "), &knows);
    assert_eq!(t.context(), Some(owned(&["hello", "world"]).as_slice()));
    assert!(!t.show(&knows).read);
    t.reset();
    t.read(Some(""), &knows);
    assert_eq!(t.context(), Some(&[][..]));
    t.reset();
    t.read(None, &knows);
    assert_eq!(t.context(), None);
}

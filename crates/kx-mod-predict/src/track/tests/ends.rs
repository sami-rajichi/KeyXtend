//! Word ends: a space, marks then a space, Enter and Tab learn the word; glued forms and breaks learn nothing.

use super::*;

#[test]
fn a_no_break_space_ends_the_word_as_a_space_does() {
    let mut t = tracker(2);
    let told = type_text(&mut t, "oui\u{00A0}!");
    assert_eq!(learned(&told), [("oui".to_owned(), owned(&[]))]);
}

#[test]
fn marks_then_a_space_learn_the_word_before_and_clear_the_context() {
    let mut t = tracker(2);
    let told = type_text(&mut t, "say hello, (world) ");
    let expected = [
        ("say".to_owned(), owned(&[])),
        ("hello".to_owned(), owned(&["say"])),
        ("world".to_owned(), owned(&[])),
    ];
    assert_eq!(learned(&told), expected);
    assert_eq!(t.context(), Some(&[][..]));
}

#[test]
fn an_arabic_word_before_the_arabic_comma_is_learned_and_the_comma_clears_the_context() {
    let mut t = tracker(2);
    let told = type_text(&mut t, "قال مرحبا\u{060C} ");
    let expected = [
        ("قال".to_owned(), owned(&[])),
        ("مرحبا".to_owned(), owned(&["قال"])),
    ];
    assert_eq!(learned(&told), expected);
    assert_eq!(t.context(), Some(&[][..]));
}

#[test]
fn glued_forms_are_never_learned_and_clear_the_context() {
    let mut t = tracker(2);
    let told = type_text(&mut t, "one a.b site.com/path ");
    assert_eq!(learned(&told), [("one".to_owned(), owned(&[]))]);
    assert_eq!(t.context(), Some(&[][..]));
}

#[test]
fn enter_and_tab_end_the_word_learned_then_clear_the_context() {
    let mut t = tracker(2);
    type_text(&mut t, "one two");
    let told = t.typed(&Piece::Advance, &knows);
    assert_eq!(learned(&told), [("two".to_owned(), owned(&["one"]))]);
    assert!(!told.contains(&Told::Read));
    assert_eq!(t.context(), Some(&[][..]));
    assert!(
        learned(&type_text(&mut t, "three "))
            .iter()
            .all(|l| l.1.is_empty())
    );
}

#[test]
fn right_after_a_mark_nothing_shows_and_an_erase_brings_the_word_back() {
    let mut t = tracker(2);
    type_text(&mut t, "say hello,");
    assert_eq!(t.show(&knows).words, None);
    t.typed(&Piece::Erase, &knows);
    assert_eq!(
        readings(&t.show(&knows)),
        [(owned(&["say"]), String::new(), "hello".to_owned())]
    );
}

#[test]
fn a_word_after_a_leading_mark_is_learned_without_context() {
    let mut t = tracker(2);
    let told = type_text(&mut t, "say \"hello ");
    assert_eq!(
        learned(&told).last(),
        Some(&("hello".to_owned(), owned(&[])))
    );
    assert_eq!(t.context(), Some(owned(&["hello"]).as_slice()));
}

#[test]
fn a_word_of_a_script_without_spaces_is_never_learned_and_clears_the_context() {
    let mut t = tracker(2);
    let told = type_text(&mut t, "one 你好 two ");
    let expected = [
        ("one".to_owned(), owned(&[])),
        ("two".to_owned(), owned(&[])),
    ];
    assert_eq!(learned(&told), expected);
}

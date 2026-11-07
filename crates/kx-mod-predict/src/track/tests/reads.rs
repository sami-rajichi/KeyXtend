//! The tracker's reads: they fill the context, keep the last word open and learn nothing.

use super::*;

#[test]
fn a_read_fills_the_context_and_its_last_word_stays_open() {
    let mut t = tracker(3);
    t.reset();
    t.read(Some("je vais à l'éc"), &knows);
    let show = t.show(&knows);
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
fn a_lines_at_at_lists_emails_while_a_boxes_never_does() {
    let mut t = tracker(2);
    t.read(Some("@@"), &knows);
    assert_eq!(t.show(&knows).emails, None);
    t.read_line("find @@", &knows);
    let all = EmailAsk::All(Redacted::new("@@".to_owned()));
    assert_eq!(t.show(&knows).emails, Some(all));
    assert_eq!(t.context(), Some(owned(&["find"]).as_slice()));
}

#[test]
fn a_line_is_read_afresh_each_time_and_hands_nothing_out() {
    let mut t = tracker(2);
    t.read_line("hello world", &knows);
    t.read_line("other wo", &knows);
    assert_eq!(t.context(), Some(owned(&["other"]).as_slice()));
    assert_eq!(t.email(), None, "no at sign");
    t.read_line("", &knows);
    assert_eq!(t.context(), Some(owned(&[]).as_slice()));
}

#[test]
fn noting_a_box_keeps_what_the_tracker_follows() {
    let mut t = tracker(2);
    t.read_line("hello wo", &knows);
    let change = FieldChange {
        kind: FieldKind::Password,
        box_id: 7,
    };
    t.note(change);
    assert_eq!(t.context(), Some(owned(&["hello"]).as_slice()));
    assert!(!t.leaves(change));
    assert_eq!(
        t.focus(change),
        Vec::new(),
        "the same box when the aim leaves"
    );
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
    t.reset();
    t.read(Some(""), &knows);
    assert_eq!(t.context(), Some(&[][..]));
    t.reset();
    t.read(None, &knows);
    assert_eq!(t.context(), None);
}

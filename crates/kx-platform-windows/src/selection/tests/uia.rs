//! The selection watch's readers on hidden edit boxes, through a `Mine` that is not this process: a selection's line
//! box and a caret's box with no text call, a password box or our own asked nothing, `still` once the selection
//! clears, a read cut at its cap in Latin and Arabic, and a look from a child with no text up to its box.

use super::super::uia::{READS, read, spot, still, text_of};
use super::super::win::Mine;
use crate::field::testedit::{LONG_MS, TestEdit, far, started};
use crate::field::{TEXT_CALLS, box_id};
use crate::scroll::testwin::one_client;
use kx_module_api::ports::SelKind;
use std::cell::Cell;
use std::time::{Duration, Instant};
use windows::Win32::UI::WindowsAndMessaging::ES_PASSWORD;

/// Latin words.
const LATIN: &str = "alpha beta gamma delta";
/// Arabic words: "I love reading a lot".
const ARABIC: &str = "أنا أحب القراءة كثيرا";
/// A process that is never ours, the idle process, so our own test boxes read as another app's.
const NOT_US: u32 = 0;
/// Where the first word of `LATIN` ends, in UTF-16 units.
const ALPHA: usize = 5;

/// The end of `text` in UTF-16 units, the edit box's own.
fn end(text: &str) -> usize {
    text.encode_utf16().count()
}

/// How many selection texts this thread asked for.
fn reads() -> usize {
    READS.with(Cell::get)
}

#[test]
fn a_selection_gives_its_line_box_and_a_caret_its_box_with_no_text_call() {
    let _one = one_client();
    let (_com, auto) = started();
    let edit = TestEdit::start(LATIN, 0);
    let el = edit.element(&auto);
    let (id, mine, before) = (box_id(&el), Mine::of(NOT_US), reads());
    edit.select(0, ALPHA);
    let sel = spot(&el, id, mine, far()).unwrap();
    assert_eq!(
        (sel.box_id, sel.kind, sel.selected),
        (id, SelKind::Editable, true)
    );
    assert!(sel.first.is_some() && sel.first == sel.last, "one line");
    assert_eq!(sel.caret, None);
    edit.select(end(LATIN), end(LATIN));
    let caret = spot(&el, id, mine, far()).unwrap();
    assert!(!caret.selected && caret.first.is_none() && caret.last.is_none());
    assert!(caret.caret.is_some(), "the caret's box");
    assert_eq!(reads(), before, "no text call");
}

#[test]
fn a_password_box_or_our_own_is_closed_and_asked_no_caret() {
    let _one = one_client();
    let (_com, auto) = started();
    let (secret, plain) = (
        TestEdit::start(LATIN, ES_PASSWORD),
        TestEdit::start(LATIN, 0),
    );
    let looks = [
        (secret.element(&auto), Mine::of(NOT_US)),
        (plain.element(&auto), Mine::new()),
    ];
    secret.select(0, ALPHA);
    plain.select(0, ALPHA);
    for (el, mine) in looks {
        let calls = TEXT_CALLS.with(Cell::get);
        let got = spot(&el, box_id(&el), mine, far()).unwrap();
        assert_eq!(
            (got.kind, got.selected, got.caret),
            (SelKind::Closed, false, None)
        );
        assert_eq!(TEXT_CALLS.with(Cell::get), calls, "no caret call");
    }
}

#[test]
fn still_holds_only_while_the_same_box_holds_a_selection() {
    let _one = one_client();
    let (_com, auto) = started();
    let edit = TestEdit::start(LATIN, 0);
    let el = edit.element(&auto);
    let id = box_id(&el);
    edit.select(0, ALPHA);
    assert!(still(&el, id, far()));
    assert!(!still(&el, id.wrapping_add(1), far()), "another box");
    assert!(!still(&el, 0, far()), "a box with no id");
    edit.select(end(LATIN), end(LATIN));
    assert!(!still(&el, id, far()), "the selection cleared");
}

#[test]
fn a_look_from_a_child_with_no_text_reads_its_box_and_a_box_with_text_stays() {
    let _one = one_client();
    let (_com, auto) = started();
    let edit = TestEdit::holding(LATIN, 0);
    let (el, mine) = (edit.element(&auto), Mine::of(NOT_US));
    let id = box_id(&el);
    let up = text_of(edit.child(&auto), &auto, mine, far());
    assert_eq!(box_id(&up), id, "the box above");
    assert_eq!(box_id(&text_of(el, &auto, mine, far())), id, "its own text");
    edit.select(0, ALPHA);
    let sel = spot(&up, id, mine, far()).unwrap();
    assert_eq!((sel.kind, sel.selected), (SelKind::Editable, true));
    assert!(still(&up, id, far()));
    let own = text_of(edit.child(&auto), &auto, Mine::new(), far());
    assert_ne!(box_id(&own), id, "our own never climbs");
    let ago = Duration::from_millis(u64::from(LONG_MS));
    let past = Instant::now().checked_sub(ago).unwrap();
    let late = text_of(edit.child(&auto), &auto, mine, past);
    assert_ne!(box_id(&late), id, "past the deadline it never climbs");
}

#[test]
fn a_child_of_a_password_box_is_closed_and_asks_no_caret() {
    let _one = one_client();
    let (_com, auto) = started();
    let secret = TestEdit::holding(LATIN, ES_PASSWORD);
    secret.select(0, ALPHA);
    let up = text_of(secret.child(&auto), &auto, Mine::of(NOT_US), far());
    let calls = TEXT_CALLS.with(Cell::get);
    let got = spot(&up, box_id(&up), Mine::of(NOT_US), far()).unwrap();
    assert_eq!((got.kind, got.selected), (SelKind::Closed, false));
    assert_eq!(TEXT_CALLS.with(Cell::get), calls, "no caret call");
}

#[test]
fn a_read_cuts_at_its_cap_in_latin_and_arabic_and_a_caret_reads_nothing() {
    let _one = one_client();
    let (_com, auto) = started();
    let cases = [
        (LATIN, 5, "alpha"),
        (ARABIC, 3, "أنا"),
        (ARABIC, 7, "أنا أحب"),
    ];
    for (text, cap, want) in cases {
        let edit = TestEdit::start(text, 0);
        let el = edit.element(&auto);
        edit.select(0, end(text));
        let before = reads();
        let got = read(&el, cap, far()).unwrap().into_inner();
        assert_eq!((got.as_str(), reads()), (want, before + 1), "{cap}");
        edit.select(end(text), end(text));
        let none = read(&el, cap, far()).unwrap().into_inner();
        assert_eq!((none.as_str(), reads()), ("", before + 1), "a caret");
    }
}

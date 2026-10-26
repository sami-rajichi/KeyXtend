//! A selection in a hidden edit box, made either way round with `EM_SETSEL`: a check never confirms an end while it
//! holds text, a caret still does, and a read stops at the selection's start.

use super::super::{before_cursor, ends_with};
use super::{LATIN, chars, text};
use crate::field::testedit::{TestEdit, far, started};
use crate::scroll::testwin::one_client;
use kx_module_api::ports::{FieldEnds, FieldKind};

/// Where `delta`, the last word of `LATIN`, starts and ends, in UTF-16 units.
fn delta() -> (usize, usize) {
    let end = LATIN.encode_utf16().count();
    (end - "delta".len(), end)
}

/// A check of a text box that answered `ends`.
fn text_box(ends: Option<bool>) -> FieldEnds {
    FieldEnds {
        kind: FieldKind::Text,
        ends,
    }
}

#[test]
fn a_forward_or_backward_selection_never_ends_with_a_word_and_a_caret_still_does() {
    let _one = one_client();
    let (_com, auto) = started();
    let edit = TestEdit::start(LATIN, 0);
    let el = edit.element(&auto);
    let (start, end) = delta();
    for (from, to) in [(start, end), (end, start)] {
        edit.select(from, to);
        for word in ["delta", "gamma "] {
            let checked = ends_with(&el, word, far(), chars());
            assert_eq!(checked, Ok(text_box(Some(false))), "{from}..{to}: {word:?}");
        }
    }
    edit.select(end, end);
    let checked = ends_with(&el, "delta", far(), chars());
    assert_eq!(
        checked,
        Ok(text_box(Some(true))),
        "a caret answers as before"
    );
}

#[test]
fn a_read_with_a_selection_ends_at_its_start_either_way_round() {
    let _one = one_client();
    let (_com, auto) = started();
    let edit = TestEdit::start(LATIN, 0);
    let el = edit.element(&auto);
    let (start, end) = delta();
    for (from, to) in [(start, end), (end, start)] {
        edit.select(from, to);
        let read = text(before_cursor(&el, 2, far(), chars()));
        assert_eq!(read, Ok(Some("beta gamma ".to_owned())), "{from}..{to}");
    }
}

//! What a `send` key tells word prediction when it goes out alone: Backspace an erase, Enter and Tab an advance, the
//! space bar its space, and every other key a break.

use super::*;
use kx_module_api::Redacted;
use kx_module_api::ports::Piece;

/// The keys that tell more than a break, and what each tells.
fn telling() -> Vec<(&'static str, Tell)> {
    vec![
        ("bksp", Tell::Erase),
        ("tab", Tell::Advance),
        ("enter", Tell::Advance),
        ("space", Tell::Text(" ".into())),
    ]
}

#[test]
fn backspace_erases_enter_and_tab_advance_and_the_space_bar_types_its_space() {
    let s = spec();
    let want = telling();
    for (id, tell) in &want {
        assert_eq!(named(&s, id).tell.as_ref(), Some(tell), "{id}");
    }
    let keys = s.rows.iter().flatten().filter_map(|i| match i {
        RowItem::Key(k) => Some(k),
        RowItem::Chars(_) => None,
    });
    for k in keys.filter(|k| !want.iter().any(|(id, _)| *id == k.id)) {
        assert_eq!(k.tell, None, "{} tells a break", k.id);
    }
    assert_eq!(s.tells().len(), want.len());
}

#[test]
fn backspace_is_the_key_a_picked_word_erases_with() {
    assert_eq!(spec().erase_key(), Some(Scan::new(0x0E)));
}

#[test]
fn only_a_key_that_sends_its_code_may_tell() {
    for id in ["shift", "lang", "caps"] {
        let mut s = spec();
        named_mut(&mut s, id).tell = Some(Tell::Erase);
        assert_eq!(s.check(), Err(BoardError::Role(id.into())), "{id}");
    }
}

#[test]
fn a_tell_reads_from_the_text() {
    let text = BOARD.replace("id = \"esc\",", "id = \"esc\", tell = { text = \"x\" },");
    let esc = BoardSpec::read(&text).unwrap();
    assert_eq!(named(&esc, "esc").tell, Some(Tell::Text("x".into())));
    assert_eq!(esc.tells()["esc"], Tell::Text("x".into()));
}

#[test]
fn each_tell_names_its_piece() {
    assert_eq!(Tell::Erase.piece(), Piece::Erase);
    assert_eq!(Tell::Advance.piece(), Piece::Advance);
    let space = Piece::Text(Redacted::new(" ".into()));
    assert_eq!(Tell::Text(" ".into()).piece(), space);
}

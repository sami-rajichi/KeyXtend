//! Which keys repeat while held: the owner's named keys and every character key, and only keys that send.

use super::*;

/// The named keys the owner chose to repeat (2026-10-11).
const REPEATING: [&str; 8] = [
    "bksp", "del", "enter", "space", "up", "left", "down", "right",
];

#[test]
fn the_board_marks_exactly_the_owners_repeating_keys() {
    let s = spec();
    let named: Vec<String> = (0..s.rows.len())
        .flat_map(|r| s.row_keys(r))
        .filter_map(|k| match k {
            RowKey::Named(n) if n.repeat => Some(n.id.clone()),
            _ => None,
        })
        .collect();
    let mut want: Vec<String> = REPEATING.map(String::from).to_vec();
    let mut got = named;
    want.sort();
    got.sort();
    assert_eq!(got, want);
}

#[test]
fn every_character_key_repeats() {
    let s = spec();
    let repeating = s.repeating();
    for sc in s.char_scans() {
        assert!(repeating.contains(&char_id(sc)), "{}", char_id(sc));
    }
    assert_eq!(repeating.len(), s.char_scans().len() + REPEATING.len());
}

#[test]
fn a_key_without_the_flag_never_repeats() {
    let s = spec();
    let repeating = s.repeating();
    let still = [
        "esc", "tab", "caps", "shift", "rshift", "menu", "ctrl", "win", "alt", "lang", "altgr",
    ];
    for id in still {
        assert!(!named(&s, id).repeat, "{id}");
        assert!(!repeating.contains(id), "{id}");
    }
    assert!(
        s.side.iter().all(|k| !repeating.contains(&k.id)),
        "side keys"
    );
}

#[test]
fn only_a_key_that_sends_its_code_may_repeat() {
    for id in ["shift", "lang", "caps"] {
        let mut s = spec();
        named_mut(&mut s, id).repeat = true;
        assert_eq!(s.check(), Err(BoardError::Role(id.into())), "{id}");
    }
    let text = BOARD.replace("id = \"tab\",", "id = \"tab\", repeat = true,");
    let tab = BoardSpec::read(&text).unwrap();
    assert!(
        tab.repeating().contains("tab"),
        "the flag reads from the text"
    );
}

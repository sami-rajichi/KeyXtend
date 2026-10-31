//! The clipboard-history chord over the injector's fake OS (ADR-0034 decision 7), and the paste the keyboard sends
//! when the clipboard module asks.

use super::*;
use kx_module_api::ports::PASTE_ID;

/// The keyboard's Win latch and V, which the fake's AZERTY keeps at the QWERTY place.
const WIN: Scan = Scan::new(0xE05B);
const V: Scan = Scan::new(0x2F);

#[test]
fn the_history_chord_is_win_with_the_key_that_gives_v_and_nothing_more() {
    let fake = Fake::default();
    assert!(history(&fake, table(), &[WIN], V));
    assert!(!history(&fake, table(), &[], V), "V alone types v");
    assert!(!history(&fake, table(), &[CTRL], V), "Ctrl+V pastes");
    assert!(!history(&fake, table(), &[WIN, SHIFT], V), "another chord");
    assert!(!history(&fake, table(), &[WIN], Scan::new(C)), "Win+C");
    assert!(fake.sends.borrow().is_empty(), "an ask sends nothing");
}

#[test]
fn the_history_key_is_found_in_the_app_layout_else_the_keyboards_own() {
    let names = "win = 0x5B\na = 0x41";
    let text = "history = [\"win\", \"a\"]\nshortcuts = [{ id = \"x\", steps = [[\"a\"]] }]";
    let t = Table::parse(text, names).unwrap();
    let front = Fake::default();
    assert!(history(&front, &t, &[WIN], Scan::new(A)), "AZERTY's A");
    assert!(!history(&front, &t, &[WIN], Scan::new(OWN_A)));
    let away = Fake {
        away: true,
        ..Fake::default()
    };
    assert!(history(&away, &t, &[WIN], Scan::new(OWN_A)), "QWERTY's A");
}

#[test]
fn a_table_without_a_history_chord_never_takes_one_and_a_broken_one_is_refused() {
    let names = "win = 0x5B\nv = 0x56";
    let entry = "shortcuts = [{ id = \"x\", steps = [[\"v\"]] }]";
    let plain = Table::parse(entry, names).unwrap();
    assert!(!history(&Fake::default(), &plain, &[WIN], V));
    let with = |chord: &str| Table::parse(&format!("history = {chord}\n{entry}"), names);
    assert_eq!(with("[]").err(), Some(TableError::Empty));
    assert_eq!(
        with("[\"win\", \"q\"]").err(),
        Some(TableError::UnknownName)
    );
    assert!(matches!(with("[\"Win\"]"), Err(TableError::Parse(_))));
}

#[test]
fn the_table_holds_the_paste_the_keyboard_sends_on_request() {
    assert!(table().find(&id(PASTE_ID)).is_some());
}

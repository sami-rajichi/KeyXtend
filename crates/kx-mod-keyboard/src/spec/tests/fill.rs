//! Quick-fill on the board: the `fill` side key opens its panel, and `[fill]` names the key a fill's Tab part sends.

use super::*;

#[test]
fn the_fill_key_opens_quick_fill_and_sends_no_code() {
    let s = spec();
    let fill = s.side.iter().find(|k| k.id == "fill").unwrap();
    assert_eq!(fill.role, Role::Fill);
    assert!(!Role::Fill.sends());
}

#[test]
fn a_fills_tab_sends_the_boards_tab_key() {
    let s = spec();
    assert_eq!(s.fill.tab, "tab");
    assert_eq!(s.fill_tab(), Some(Scan::new(0x0F)));
}

#[test]
fn the_fill_key_is_a_side_key() {
    let mut s = spec();
    let menu = named_mut(&mut s, "menu");
    (menu.role, menu.sc) = (Role::Fill, None);
    assert_eq!(s.check(), Err(BoardError::Role("menu".into())));
}

#[test]
fn a_tab_part_must_name_a_send_key_of_the_main_rows() {
    let others = [
        "nothing".to_owned(),
        char_id(Scan::new(0x10)),
        "shift".into(),
        "lang".into(),
        "fill".into(),
    ];
    for id in others {
        let mut s = spec();
        s.fill.tab.clone_from(&id);
        assert_eq!(s.check(), Err(BoardError::Fill(id.clone())), "{id}");
    }
    let mut s = spec();
    s.fill.tab = "del".into();
    assert_eq!(s.check(), Ok(()), "any send key may stand in");
}

#[test]
fn an_unknown_key_in_the_fill_table_is_refused() {
    let line = "tab = \"tab\"\n";
    assert_eq!(BOARD.matches(line).count(), 1, "the fill table's one line");
    let text = BOARD.replace(line, "tab = \"tab\"\nwait_ms = 1\n");
    assert!(matches!(BoardSpec::read(&text), Err(BoardError::Parse(_))));
}

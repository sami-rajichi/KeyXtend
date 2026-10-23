//! Key ids and roles: character ids follow scan codes, and every key is listed once with its role.

use super::*;

#[test]
fn character_ids_name_their_scan_code() {
    assert_eq!(char_id(Scan::new(0x2C)), "c-0x002C");
    assert_eq!(char_id(Scan::new(0xE035)), "c-0xE035");
    assert!(Role::Char.sends() && Role::Latch(LatchRole::of(Latch::Win)).sends());
    assert!(!Role::Lang.sends() && !Role::Later.sends());
    assert!(!Role::Assist(Assist::RClick).sends());
}

#[test]
fn side_keys_flip_mouse_assist_show_the_layer_ask_for_power_or_come_later() {
    let s = BoardSpec::shipped().unwrap();
    let role = |id: &str| s.side.iter().find(|k| k.id == id).unwrap().role;
    assert_eq!(role("rclick"), Role::Assist(Assist::RClick));
    assert_eq!(role("grab"), Role::Assist(Assist::Grab));
    assert_eq!(
        (role("shortcuts"), role("power")),
        (Role::Layer, Role::Power)
    );
    let now = |r: Role| matches!(r, Role::Assist(_) | Role::Layer | Role::Power);
    let rest: Vec<_> = s.side.iter().filter(|k| !now(k.role)).collect();
    assert_eq!(rest.len(), 5);
    assert!(rest.iter().all(|k| k.role == Role::Later));
    assert_eq!(Toggle::from(Assist::RClick), Toggle::RightClick);
    assert_eq!(Toggle::from(Assist::Grab), Toggle::Grab);
}

#[test]
fn an_unknown_mouse_assist_mode_is_refused() {
    let text = BOARD.replace("assist = \"grab\"", "assist = \"scroll\"");
    assert!(matches!(BoardSpec::read(&text), Err(BoardError::Parse(_))));
}

/// Each character key's id, by its scan code.
fn char_ids(s: &BoardSpec) -> std::collections::HashMap<Scan, String> {
    let keys = (0..s.rows.len()).flat_map(|r| s.row_keys(r));
    let chars = keys.filter_map(|k| match k {
        RowKey::Char { id, sc } => Some((sc, id)),
        RowKey::Named(_) => None,
    });
    chars.collect()
}

#[test]
fn a_key_keeps_its_id_when_a_key_is_added_before_it() {
    let before = spec();
    let mut after = spec();
    let Some(RowItem::Chars(run)) = after.rows[3].get_mut(1) else {
        panic!("row 4 starts with Shift, then its character run");
    };
    run.chars.insert(0, Scan::new(0x59));
    let moved = char_ids(&after);
    for (sc, id) in char_ids(&before) {
        assert_eq!(moved[&sc], id, "{sc}");
    }
}

#[test]
fn roles_list_every_key_once_with_its_role_and_code() {
    let s = spec();
    let roles = s.roles();
    let find = |id: &str| roles.iter().find(|r| r.0 == id).map(|r| (r.1, r.2));
    assert_eq!(roles.len(), crate::geom::board(&s, 0.0, 1.0).keys.len());
    let q = char_id(Scan::new(0x10));
    assert_eq!(find(&q), Some((Role::Char, Some(Scan::new(0x10)))));
    assert_eq!(find("esc"), Some((Role::Send, Some(Scan::new(0x01)))));
    assert_eq!(find("lang"), Some((Role::Lang, None)));
    assert_eq!(find("mic"), Some((Role::Later, None)));
    let chars = roles.iter().filter(|r| r.1 == Role::Char).count();
    assert_eq!(chars, s.char_scans().len());
}

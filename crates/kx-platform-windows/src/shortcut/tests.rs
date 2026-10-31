//! The tables and plans over the injector's fake OS: French AZERTY in front, US QWERTY as the keyboard's own.

use std::collections::{BTreeMap, BTreeSet};

use kx_module_api::ports::{KeyName, Step};

use super::*;
use crate::inject::tests::fake::Fake;
use crate::strokes::Key;

mod history;
mod props;

const CTRL: Scan = Scan::new(0x1D);
const SHIFT: Scan = Scan::new(0x2A);
/// Scan codes in the fake's AZERTY: Ctrl, Shift, A (at the QWERTY Q place), C, T, and Delete as it goes, extended.
const CTRL_KEY: u16 = 0x1D;
const SHIFT_KEY: u16 = 0x2A;
const A: u16 = 0x10;
const C: u16 = 0x2E;
const T: u16 = 0x14;
const DEL: u16 = 0xE053;
/// A in the keyboard's own US QWERTY.
const OWN_A: u16 = 0x1E;
/// Shortcuts the shipped table holds.
const ENTRIES: usize = 37;

fn id(text: &str) -> ShortcutId {
    ShortcutId::new(text).unwrap()
}

fn name(text: &str) -> KeyName {
    KeyName::new(text).unwrap()
}

/// The events `id` plans with `held` latched; any other plan fails the test.
fn keys(fake: &Fake, id_text: &str, held: &[Scan]) -> Vec<KeyEvent> {
    match plan(fake, table(), &id(id_text), held) {
        Ok(Plan::Keys(events)) => events,
        other => panic!("{id_text}: {other:?}"),
    }
}

/// A key's scan code with its prefix, as it goes to Windows.
fn code(key: Key) -> u16 {
    match key {
        Key::Scan(s) => s.code(),
        Key::Vk { scan, extended, .. } => {
            u16::from_be_bytes([if extended { 0xE0 } else { 0 }, scan])
        }
        Key::Unit(u) => u,
    }
}

/// Each event as (code, up).
fn shown(events: &[KeyEvent]) -> Vec<(u16, bool)> {
    events.iter().map(|e| (code(e.key), e.up)).collect()
}

fn down_up(codes: &[u16]) -> Vec<(u16, bool)> {
    let down = codes.iter().map(|&c| (c, false));
    down.chain(codes.iter().rev().map(|&c| (c, true))).collect()
}

#[test]
fn the_shipped_tables_parse_into_every_shortcut_once() {
    let known = Table::parse(SHORTCUTS, KEYNAMES).unwrap().known();
    assert_eq!(known.len(), ENTRIES);
    assert_eq!(
        known,
        table().known(),
        "the table in use is the shipped one"
    );
    let ids: BTreeSet<&ShortcutId> = known.iter().map(|i| &i.id).collect();
    assert_eq!(ids.len(), ENTRIES);
    assert_eq!(
        (&known[0].id, &known[ENTRIES - 1].id),
        (&id("copy"), &id("f12"))
    );
}

#[test]
fn every_key_name_the_shortcuts_use_has_a_virtual_key_and_every_mapped_name_is_used() {
    let names: BTreeMap<KeyName, u16> = toml::from_str(KEYNAMES).unwrap();
    let used: BTreeSet<KeyName> = table()
        .known()
        .into_iter()
        .flat_map(|i| i.steps)
        .flat_map(|s| s.held.into_iter().chain([s.key]))
        .collect();
    assert_eq!(used, names.into_keys().collect());
}

#[test]
fn known_splits_each_step_into_the_keys_held_and_the_key_tapped() {
    let known = table().known();
    let find = |text: &str| {
        known
            .iter()
            .find(|i| i.id == id(text))
            .unwrap()
            .steps
            .clone()
    };
    let step = |held: &[&str], key: &str| Step {
        held: held.iter().map(|h| name(h)).collect(),
        key: name(key),
    };
    assert_eq!(find("reopen-tab"), [step(&["ctrl", "shift"], "t")]);
    assert_eq!(find("delete-all"), [step(&["ctrl"], "a"), step(&[], "del")]);
    assert_eq!(
        find("lock-pc"),
        [step(&["win"], "l")],
        "the tip still names Win+L"
    );
}

#[test]
fn select_all_on_french_azerty_sends_the_key_at_the_qwerty_q_place() {
    let fake = Fake::default();
    assert_eq!(
        shown(&keys(&fake, "select-all", &[])),
        down_up(&[CTRL_KEY, A])
    );
}

#[test]
fn with_no_app_in_front_a_shortcut_maps_its_keys_in_the_keyboards_own_layout() {
    let fake = Fake {
        away: true,
        ..Fake::default()
    };
    assert_eq!(
        shown(&keys(&fake, "select-all", &[])),
        down_up(&[CTRL_KEY, OWN_A])
    );
}

#[test]
fn a_latched_ctrl_and_copys_ctrl_are_one_press() {
    let fake = Fake::default();
    let got = keys(&fake, "copy", &[CTRL]);
    assert_eq!(shown(&got), down_up(&[CTRL_KEY, C]));
    assert_eq!(
        got[0].key,
        Key::Scan(CTRL),
        "the latch goes first, as its scan code"
    );
}

#[test]
fn a_latched_shift_and_new_tab_hold_shift_and_ctrl_around_t() {
    let fake = Fake::default();
    let got = keys(&fake, "new-tab", &[SHIFT]);
    assert_eq!(shown(&got), down_up(&[SHIFT_KEY, CTRL_KEY, T]));
}

#[test]
fn a_latched_shift_and_reopen_tabs_shift_are_one_press() {
    let fake = Fake::default();
    let got = keys(&fake, "reopen-tab", &[SHIFT]);
    assert_eq!(shown(&got), down_up(&[SHIFT_KEY, CTRL_KEY, T]));
}

#[test]
fn copy_all_sends_two_ctrl_combos_in_order_reading_the_layout_once() {
    let fake = Fake::default();
    let got = keys(&fake, "copy-all", &[]);
    assert_eq!(
        shown(&got),
        [down_up(&[CTRL_KEY, A]), down_up(&[CTRL_KEY, C])].concat()
    );
    assert_eq!(fake.layouts.get(), 1);
}

#[test]
fn a_latch_joins_every_step_of_a_chain() {
    let fake = Fake::default();
    let got = keys(&fake, "cut-all", &[SHIFT]);
    let x = 0x2D;
    let want = [
        down_up(&[SHIFT_KEY, CTRL_KEY, A]),
        down_up(&[SHIFT_KEY, CTRL_KEY, x]),
    ];
    assert_eq!(shown(&got), want.concat());
}

#[test]
fn delete_alls_delete_goes_as_an_extended_key() {
    let fake = Fake::default();
    let got = keys(&fake, "delete-all", &[]);
    assert_eq!(
        shown(&got),
        [down_up(&[CTRL_KEY, A]), down_up(&[DEL])].concat()
    );
    assert!(got[4..].iter().all(|e| e.key.extended()));
}

#[test]
fn a_refused_send_lets_go_of_every_held_key_and_says_how_many_went() {
    // Copy all with Shift latched is 12 events in one batch; the OS takes 8, inside the second step.
    let fake = Fake::taking(&[8]);
    let events = keys(&fake, "copy-all", &[SHIFT]);
    let got = deliver_keys(&fake, &events);
    assert_eq!(got, Err(ShortcutError::Refused { sent: 8, total: 12 }));
    let sends = fake.sends.borrow();
    assert_eq!(sends.len(), 2, "one batch, then the release");
    assert_eq!(shown(&sends[1]), [(CTRL_KEY, true), (SHIFT_KEY, true)]);
}

#[test]
fn a_whole_send_goes_in_one_batch() {
    let fake = Fake::default();
    let events = keys(&fake, "copy-all", &[]);
    assert_eq!(deliver_keys(&fake, &events), Ok(()));
    assert_eq!(*fake.sends.borrow(), [events]);
}

#[test]
fn lock_pc_plans_the_os_call_and_never_keys() {
    let fake = Fake::default();
    let got = plan(&fake, table(), &id("lock-pc"), &[CTRL]);
    assert_eq!(got, Ok(Plan::Call(Call::Lock)));
}

#[test]
fn an_unknown_id_is_refused() {
    let fake = Fake::default();
    let got = plan(&fake, table(), &id("no-such-shortcut"), &[CTRL]);
    assert_eq!(got, Err(ShortcutError::Unknown));
}

#[test]
fn a_broken_shortcut_table_is_an_error_not_a_panic() {
    let names = "ctrl = 0x11\nc = 0x43";
    let one = |entry: &str| format!("shortcuts = [{entry}]");
    let cases = [
        (
            one(r#"{ id = "copy", steps = [["ctrl", "v"]] }"#),
            TableError::UnknownName,
        ),
        (one(r#"{ id = "copy", steps = [[]] }"#), TableError::Empty),
        (one(r#"{ id = "copy", steps = [] }"#), TableError::Empty),
        (
            one(r#"{ id = "copy", steps = [["c"]] }, { id = "copy", steps = [["c"]] }"#),
            TableError::Duplicate,
        ),
    ];
    for (text, want) in cases {
        assert_eq!(Table::parse(&text, names).err(), Some(want), "{text}");
    }
    let refused = [
        String::new(),
        one(r#"{ id = "copy", steps = [["ctrl", "c"]], more = 1 }"#),
        one(r#"{ id = "Copy", steps = [["ctrl", "c"]] }"#),
        one(r#"{ id = "copy", steps = [["Ctrl", "c"]] }"#),
        one(r#"{ id = "copy", steps = [["c"]], call = "reboot" }"#),
        format!("{}\nmore = 1", one(r#"{ id = "copy", steps = [["c"]] }"#)),
    ];
    for text in refused {
        let got = Table::parse(&text, names);
        assert!(matches!(got, Err(TableError::Parse(_))), "{text}");
    }
}

#[test]
fn a_broken_key_name_table_is_an_error_not_a_panic() {
    let shortcuts = r#"shortcuts = [{ id = "copy", steps = [["c"]] }]"#;
    assert_eq!(
        Table::parse(shortcuts, "c = 0x43").map(|t| t.known().len()),
        Ok(1)
    );
    for names in ["C = 0x43", "c = -1", "c = 0x10000", "c = \"0x43\"", "[c]"] {
        let got = Table::parse(shortcuts, names);
        assert!(matches!(got, Err(TableError::Parse(_))), "{names}");
    }
}

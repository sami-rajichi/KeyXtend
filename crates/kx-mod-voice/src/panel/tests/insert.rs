//! Insert the text on the Options page (owner, 2026-11-17): two chips write `insert`, and while Paste at once is
//! chosen its hint takes the row below, which acts on nothing.

use kx_module_api::ports::{OptionRow, OptionValue, VoiceClick, VoiceSheet, VoiceTab};

use super::super::rig::Rig;
use super::Do;
use crate::INSERTS;

/// The rig on the Options page whose rows hold Insert the text, and that page's rows.
fn insert_page(rig: &mut Rig) -> Vec<OptionRow> {
    rig.click(&VoiceClick::Open(VoiceTab::Options));
    loop {
        let rows = match rig.view().unwrap().sheet {
            VoiceSheet::Options(rows) => rows,
            other => panic!("not Options: {other:?}"),
        };
        if rows.iter().any(|r| r.key == "insert") {
            return rows;
        }
        rig.click(&VoiceClick::Page { back: false });
    }
}

/// The place of the Insert row on `rows`.
fn at(rows: &[OptionRow]) -> u8 {
    let at = rows.iter().position(|r| r.key == "insert").unwrap();
    u8::try_from(at).unwrap()
}

#[test]
fn typing_shows_no_hint_and_its_second_chip_picks_paste() {
    let mut rig = Rig::new();
    let rows = insert_page(&mut rig);
    let row = at(&rows);
    assert!(!rows.iter().any(|r| r.value == OptionValue::Hint));
    let paste = toml::Value::String(INSERTS[1].0.into());
    let out = rig.click(&VoiceClick::Step { row, up: true });
    assert_eq!(out, [Do::Set("insert", paste)]);
    let again = rig.click(&VoiceClick::Step { row, up: false });
    assert!(again.is_empty(), "typing is already chosen");
}

#[test]
fn paste_at_once_shows_its_hint_on_the_row_below_which_acts_on_nothing() {
    let mut rig = Rig::new();
    rig.settings.insert = INSERTS[1].0.into();
    let rows = insert_page(&mut rig);
    let row = at(&rows);
    let hint = &rows[usize::from(row) + 1];
    assert_eq!(
        (hint.key, hint.value, hint.live),
        ("insert", OptionValue::Hint, true)
    );
    let below = row + 1;
    for click in [
        VoiceClick::Row(below),
        VoiceClick::Flip(below),
        VoiceClick::Step {
            row: below,
            up: false,
        },
    ] {
        assert!(rig.click(&click).is_empty(), "{click:?}");
    }
    let typed = toml::Value::String(INSERTS[0].0.into());
    let out = rig.click(&VoiceClick::Step { row, up: false });
    assert_eq!(out, [Do::Set("insert", typed)]);
}

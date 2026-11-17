//! Insert the text on the Options page (owner, 2026-11-17): two chips write `insert`, and while Paste at once is
//! chosen its hint takes the row below, on the same page, and acts on nothing. A chip click keeps its row on screen.

use kx_module_api::ports::{OptionRow, OptionValue, VoiceClick, VoiceSheet, VoiceTab};

use super::super::rig::Rig;
use super::Do;
use crate::INSERTS;

/// The shown page's Options rows.
fn shown(rig: &mut Rig) -> Vec<OptionRow> {
    match rig.view().unwrap().sheet {
        VoiceSheet::Options(rows) => rows,
        other => panic!("not Options: {other:?}"),
    }
}

/// The rig on the Options page whose rows hold Insert the text, and that page's rows.
fn insert_page(rig: &mut Rig) -> Vec<OptionRow> {
    rig.click(&VoiceClick::Open(VoiceTab::Options));
    loop {
        let rows = shown(rig);
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

/// Every page of the Options list at `per` rows a page, the first first.
fn option_pages(settings: &str, per: u8) -> Vec<Vec<OptionRow>> {
    let mut rig = Rig::new();
    rig.settings.insert = settings.into();
    rig.click(&VoiceClick::Rows(per));
    rig.click(&VoiceClick::Open(VoiceTab::Options));
    let mut pages = Vec::new();
    loop {
        let view = rig.view().unwrap();
        let VoiceSheet::Options(rows) = view.sheet else {
            panic!("not Options: {:?}", view.sheet);
        };
        pages.push(rows);
        if view.pages.at + 1 >= view.pages.count {
            return pages;
        }
        rig.click(&VoiceClick::Page { back: false });
    }
}

#[test]
fn the_hint_stays_on_its_chips_page_even_when_insert_ends_a_page() {
    let typed = option_pages(INSERTS[0].0, u8::MAX).concat();
    let last = (at(&typed) + 1).max(2);
    for per in (2..=8).chain([last]) {
        let pages = option_pages(INSERTS[1].0, per);
        let page = pages
            .iter()
            .find(|p| p.iter().any(|r| r.key == "insert"))
            .unwrap();
        let row = usize::from(at(page));
        assert!(
            matches!(page[row].value, OptionValue::Pair { .. }),
            "{per} a page"
        );
        assert_eq!(
            page.get(row + 1).map(|r| r.value),
            Some(OptionValue::Hint),
            "{per} a page"
        );
        assert!(
            pages.iter().all(|p| p.len() <= usize::from(per)),
            "{per} a page"
        );
        assert_eq!(
            pages.concat().len(),
            typed.len() + 1,
            "no row lost at {per} a page"
        );
    }
}

#[test]
fn a_chip_click_keeps_the_insert_row_on_screen_both_ways() {
    for per in 2..=8 {
        let mut rig = Rig::new();
        rig.click(&VoiceClick::Rows(per));
        let mut row = at(&insert_page(&mut rig));
        for (up, insert) in [(true, INSERTS[1].0), (false, INSERTS[0].0)] {
            rig.click(&VoiceClick::Step { row, up });
            rig.settings.insert = insert.into();
            let rows = shown(&mut rig);
            assert!(
                rows.iter().any(|r| r.key == "insert"),
                "{per} a page, up {up}"
            );
            row = at(&rows);
        }
    }
}

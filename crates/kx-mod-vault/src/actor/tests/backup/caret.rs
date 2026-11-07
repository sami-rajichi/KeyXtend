//! The password page's caret over the actor: Left and Right move it, letters, Backspace and Delete act at it, and a
//! click places it; `carets` in `entry::line` tests the line itself.

use kx_module_api::Redacted;
use kx_module_api::ports::{FillButton, FillClick, PanelKey, Sheet};

use super::super::rig::Rig;
use super::{CLOUD_ROW, pick, press};
use crate::actor::Msg;

/// Beh with a fatha, then teh, which reads right to left.
const BEH_FATHA_TEH: &str = "\u{628}\u{64e}\u{62a}";

/// The password page of Setup, nothing typed yet.
fn on_password() -> Rig {
    let mut rig = Rig::new();
    rig.open();
    press(&mut rig, FillButton::Backup);
    rig.jobs();
    pick(&mut rig, CLOUD_ROW);
    rig.jobs();
    assert!(matches!(rig.page().sheet, Sheet::Password(_)));
    rig
}

/// The password page's text and caret on the last view.
fn typed(rig: &Rig) -> (String, u32) {
    let Sheet::Password(sheet) = rig.page().sheet else {
        panic!("not the password page");
    };
    (sheet.text.expose().to_string(), *sheet.caret.expose())
}

/// A key from our keyboard.
fn key(rig: &mut Rig, key: PanelKey) {
    rig.send(Msg::Key(key));
}

#[test]
fn the_arrows_move_the_caret_and_stop_at_the_ends() {
    let mut rig = on_password();
    rig.text("abc");
    assert_eq!(typed(&rig), ("abc".to_owned(), 3));
    key(&mut rig, PanelKey::Right);
    assert_eq!(typed(&rig), ("abc".to_owned(), 3), "already at the end");
    key(&mut rig, PanelKey::Left);
    assert_eq!(typed(&rig), ("abc".to_owned(), 2));
    key(&mut rig, PanelKey::Right);
    assert_eq!(typed(&rig), ("abc".to_owned(), 3));
}

#[test]
fn a_right_to_left_password_takes_left_as_on_and_its_harakat_as_one() {
    let mut rig = on_password();
    rig.text(BEH_FATHA_TEH);
    key(&mut rig, PanelKey::Right);
    assert_eq!(typed(&rig).1, 2, "back before the teh");
    key(&mut rig, PanelKey::Right);
    assert_eq!(typed(&rig).1, 0, "over the beh and its fatha as one");
    key(&mut rig, PanelKey::Left);
    assert_eq!(typed(&rig).1, 2);
}

#[test]
fn letters_backspace_space_and_delete_act_at_the_caret() {
    let mut rig = on_password();
    rig.text("ac");
    key(&mut rig, PanelKey::Left);
    rig.text("b");
    assert_eq!(typed(&rig), ("abc".to_owned(), 2));
    key(&mut rig, PanelKey::Tick);
    assert_eq!(typed(&rig), ("ab c".to_owned(), 3));
    key(&mut rig, PanelKey::Erase);
    assert_eq!(typed(&rig), ("abc".to_owned(), 2));
    key(&mut rig, PanelKey::Delete);
    assert_eq!(typed(&rig), ("ab".to_owned(), 2));
    key(&mut rig, PanelKey::Delete);
    assert_eq!(typed(&rig), ("ab".to_owned(), 2), "nothing after the caret");
}

#[test]
fn a_click_places_the_caret_clamped_and_inside_no_grapheme() {
    let mut rig = on_password();
    rig.text(BEH_FATHA_TEH);
    rig.click(FillClick::Caret { at: 0.into() });
    assert_eq!(typed(&rig).1, 0);
    rig.click(FillClick::Caret { at: 1.into() });
    assert_eq!(typed(&rig).1, 0, "inside the beh's grapheme");
    rig.click(FillClick::Caret { at: 99.into() });
    assert_eq!(typed(&rig).1, 3);
}

#[test]
fn the_password_page_ignores_a_chip_wherever_its_caret_is() {
    let mut rig = on_password();
    rig.text("ab");
    key(&mut rig, PanelKey::Left);
    key(
        &mut rig,
        PanelKey::Word {
            erase: 1,
            text: Redacted::new("x".to_owned()),
        },
    );
    assert_eq!(typed(&rig), ("ab".to_owned(), 1));
}

#[test]
fn other_backup_pages_take_neither_the_keys_nor_the_click() {
    let mut rig = Rig::new();
    rig.open();
    press(&mut rig, FillButton::Backup);
    rig.jobs();
    let before = rig.page().sheet;
    for k in [PanelKey::Left, PanelKey::Right, PanelKey::Delete] {
        key(&mut rig, k);
    }
    rig.click(FillClick::Caret { at: 1.into() });
    assert_eq!(rig.page().sheet, before);
}

#[test]
fn the_page_shows_no_caret_in_its_debug() {
    let mut rig = on_password();
    rig.text("abc");
    let shown = format!("{:?}", rig.page());
    assert!(!shown.contains("caret: 3"), "{shown}");
}

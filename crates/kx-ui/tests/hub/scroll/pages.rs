//! Our open list turns a page when the pad scrolls it: the bus's request waits for the Qt thread, which sends the
//! list's own page button with `[ui] page_rows`, never past either end, and nothing with no list open.

use super::listened;
use crate::{FakeFace, Rig};
use kx_module_api::ports::{FaceAction, FaceReply, KeyId, ScrollPage};
use kx_test_support::host::FakeHost;
use kx_ui::Wake;
use serde_json::json;
use std::sync::Arc;

/// The board keys that turn the list back and on, as the face names them.
const BACK: &str = "left";
const ON: &str = "right";

/// A face state with the list open: `len` rows with row `hi` highlighted, and the page keys `back` and `on`.
fn listing(len: usize, hi: usize, back: Option<&str>, on: Option<&str>) -> String {
    let list = json!({ "rows": vec![json!({}); len], "hi": hi, "keys": [BACK, ON], "page_back": back, "page_on": on });
    json!({ "lang": {}, "list": list }).to_string()
}

/// The list's page button `key`, as a click on it sends.
fn button(key: &str, rows: usize) -> FaceAction {
    FaceAction::PanelKey {
        key: KeyId::new(key),
        rows,
    }
}

/// `[ui] page_rows` by default.
fn rows() -> usize {
    kx_ui::UiSettings::from_defaults().unwrap().page_rows
}

/// A rig with a face whose list shows `len` rows with row `hi` highlighted, and the host behind its settings.
fn open(len: usize, hi: usize) -> (Rig, Arc<FakeFace>, FakeHost) {
    let (rig, face, host) = Rig::with_face();
    face.set_state(listing(len, hi, Some(BACK), Some(ON)));
    (rig, face, host)
}

#[test]
fn a_page_turn_waits_for_the_qt_thread() {
    let (rig, face, _host) = Rig::with_face();
    let (bus, _subs) = listened(&rig);
    for back in [true, false] {
        bus.publish(&ScrollPage { back }).unwrap();
        assert_eq!(rig.wakes(), [Wake::Page { back }]);
    }
    assert!(
        face.acts().is_empty(),
        "the face is called on the Qt thread only"
    );
}

#[test]
fn a_page_turn_sends_the_lists_own_page_button_with_the_page_rows() {
    let rows = rows();
    let (rig, face, _host) = open(2 * rows + 1, rows);
    rig.hub.turn_page(true);
    rig.hub.turn_page(false);
    assert_eq!(face.acts(), [button(BACK, rows), button(ON, rows)]);
}

/// Page on at the last page would move the highlight, so neither end turns.
#[test]
fn no_page_turns_past_either_end() {
    let rows = rows();
    let len = 2 * rows + 1;
    for (hi, back) in [
        (0, true),
        (rows - 1, true),
        (len - 1, false),
        (2 * rows, false),
    ] {
        let (rig, face, _host) = open(len, hi);
        rig.hub.turn_page(back);
        assert!(face.acts().is_empty(), "row {hi}, back {back}");
    }
    let (rig, face, _host) = open(rows, 0);
    rig.hub.turn_page(true);
    rig.hub.turn_page(false);
    assert!(face.acts().is_empty(), "one page");
}

#[test]
fn a_turned_page_wakes_the_keyboard_to_read_its_state() {
    let (rig, face, _host) = open(2 * rows(), 0);
    face.answer(Ok(FaceReply {
        state: true,
        ..FaceReply::default()
    }));
    rig.hub.turn_page(false);
    assert_eq!(rig.wakes(), [Wake::Keys]);
    face.answer(Ok(FaceReply::default()));
    rig.hub.turn_page(true);
    assert!(rig.wakes().is_empty(), "nothing changed");
}

#[test]
fn with_no_list_open_no_page_key_or_no_face_nothing_turns() {
    let rows = rows();
    Rig::new().hub.turn_page(false);
    let (rig, face, _host) = Rig::with_face();
    rig.hub.turn_page(false);
    face.set_state(json!({ "list": null }).to_string());
    rig.hub.turn_page(false);
    face.set_state(listing(2 * rows, 0, Some(BACK), None));
    rig.hub.turn_page(false);
    face.set_state(listing(2 * rows, rows, None, Some(ON)));
    rig.hub.turn_page(true);
    face.set_state("not json".to_owned());
    rig.hub.turn_page(false);
    assert!(face.acts().is_empty(), "{:?}", face.acts());
}

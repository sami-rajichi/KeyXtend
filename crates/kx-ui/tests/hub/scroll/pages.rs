//! Our open list turns a page when the pad scrolls it: the bus's request waits for the Qt thread, which sends the
//! list's own page button with `[ui] page_rows` only the ways the face's page says it turns, and nothing with no list
//! open; the hub does no page maths.

use super::listened;
use crate::{FakeFace, Rig, provide};
use kx_module_api::ports::{FaceAction, FaceReply, KeyId, ScrollPage};
use kx_module_api::{Module, ModuleCx};
use kx_test_support::host::FakeHost;
use kx_ui::{MANIFEST, UiModule, Wake};
use serde_json::json;
use std::sync::Arc;

/// The board keys that turn the list back and on, as the face names them.
const BACK: &str = "left";
const ON: &str = "right";
/// Both page keys, as the shipped board has them.
const KEYS: [Option<&str>; 2] = [Some(BACK), Some(ON)];

/// A face state with the list open, whose page turns `back` and `on` as the face says, and the page keys `keys`.
fn listing(back: bool, on: bool, keys: [Option<&str>; 2]) -> String {
    let page = json!({ "at": 0, "count": 1, "back": back, "on": on });
    let [page_back, page_on] = keys;
    let list = json!({ "rows": [], "hi": 0, "page": page, "keys": [BACK, ON], "page_back": page_back, "page_on": page_on });
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

/// A rig with a face whose list turns `back` and `on`, and the host behind its settings.
fn open(back: bool, on: bool) -> (Rig, Arc<FakeFace>, FakeHost) {
    let (rig, face, host) = Rig::with_face();
    face.set_state(listing(back, on, KEYS));
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
    let (rig, face, _host) = open(true, true);
    rig.hub.turn_page(true);
    rig.hub.turn_page(false);
    assert_eq!(face.acts(), [button(BACK, rows), button(ON, rows)]);
}

/// A page size other than the default, set in `[ui]`, pages the state the hub reads and goes with the key.
#[test]
fn the_page_size_comes_from_the_ui_settings() {
    let rows = rows() + 1;
    let (rig, face) = (Rig::new(), Arc::new(FakeFace::default()));
    let mut host = FakeHost::new(&MANIFEST);
    host.put("page_rows", i64::try_from(rows).unwrap().into());
    provide(&mut host, Arc::clone(&face));
    let mut module = UiModule::new(Arc::clone(&rig.hub));
    module.start(&mut ModuleCx::new(&mut host)).unwrap();
    face.set_state(listing(false, true, KEYS));
    rig.hub.turn_page(false);
    assert_eq!(face.rows(), Some(rows), "the page turn reads the state");
    assert_eq!(face.acts(), [button(ON, rows)]);
    let _ = rig.hub.state_json();
    assert_eq!(face.rows(), Some(rows), "QML reads it at that size too");
}

/// The page buttons dim at either end, as the face's page says, so neither end turns.
#[test]
fn a_page_turns_only_the_ways_the_face_says() {
    let (rig, face, _host) = open(false, false);
    rig.hub.turn_page(true);
    rig.hub.turn_page(false);
    assert!(face.acts().is_empty(), "one page");
    let (rig, face, _host) = open(true, false);
    rig.hub.turn_page(false);
    assert!(face.acts().is_empty(), "the last page");
    rig.hub.turn_page(true);
    assert_eq!(face.acts(), [button(BACK, rows())]);
    let (rig, face, _host) = open(false, true);
    rig.hub.turn_page(true);
    assert!(face.acts().is_empty(), "the first page");
}

#[test]
fn a_turned_page_wakes_the_keyboard_to_read_its_state() {
    let (rig, face, _host) = open(false, true);
    face.answer(Ok(FaceReply {
        state: true,
        ..FaceReply::default()
    }));
    rig.hub.turn_page(false);
    assert_eq!(rig.wakes(), [Wake::Keys]);
    face.answer(Ok(FaceReply::default()));
    rig.hub.turn_page(false);
    assert_eq!(face.acts().len(), 2, "the fake's page still turns on");
    assert!(rig.wakes().is_empty(), "nothing changed");
}

#[test]
fn with_no_list_open_no_page_key_or_no_face_nothing_turns() {
    Rig::new().hub.turn_page(false);
    let (rig, face, _host) = Rig::with_face();
    rig.hub.turn_page(false);
    face.set_state(json!({ "list": null }).to_string());
    rig.hub.turn_page(false);
    face.set_state(listing(true, true, [Some(BACK), None]));
    rig.hub.turn_page(false);
    face.set_state(listing(true, true, [None, Some(ON)]));
    rig.hub.turn_page(true);
    face.set_state(json!({ "list": { "page_back": BACK, "page_on": ON } }).to_string());
    rig.hub.turn_page(true);
    face.set_state("not json".to_owned());
    rig.hub.turn_page(false);
    assert!(face.acts().is_empty(), "{:?}", face.acts());
}

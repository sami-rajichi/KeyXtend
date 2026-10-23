//! The shortcuts layer from the keyboard's side (spec §4.2): the Shortcuts key shows and hides it, Next turns its
//! pages, a settings change keeps it, and showing it stops the pad; its cells' sends are in `sends`, and the keys it
//! hides in `hides`.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

mod hides;
mod sends;

use crate::assist::{Bench, Said, ear};
use crate::host::{Fakes, GOLDEN};
use crate::known::{self, sid};
use crate::rig::{BOTH, Rig, STATE};
use kx_mod_keyboard::KeyboardSettings;
use kx_mod_keyboard::spec::{NEXT_ID, shortcut_key_id};
use kx_mod_keyboard::view::NO_SHORTCUTS_TIP;
use kx_module_api::ports::{Dir, FaceError, FaceReply, GrabState, ScrollChanged, ScrollStop};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

/// The Shortcuts key.
pub const KEY: &str = "shortcuts";

/// The ids of the side keys `r`'s board draws, in board order.
pub fn side(r: &Rig) -> Vec<String> {
    let board = r.board();
    let keys = board["keys"].as_array().unwrap().iter();
    let side = keys.filter(|k| k["side"] == true);
    side.map(|k| k["id"].as_str().unwrap().to_owned()).collect()
}

/// The side keys while page `page` (from 0) of the owner's pages shows: the Shortcuts key, the page's cells, then
/// Next.
pub fn page_keys(page: usize) -> Vec<String> {
    let pages = KeyboardSettings::from_defaults().unwrap().shortcut_pages;
    let cells = pages[page].iter().map(shortcut_key_id);
    let keys = [KEY.to_owned()].into_iter().chain(cells);
    keys.chain([NEXT_ID.to_owned()]).collect()
}

/// The layer's part of `r`'s state.
fn layer(r: &Rig) -> Value {
    r.state()["layer"].clone()
}

#[test]
fn the_shortcuts_key_shows_and_hides_the_layer_and_redraws_both() {
    let r = Rig::new(&GOLDEN, None);
    let usual = side(&r);
    assert_eq!(r.board()["layer"], false);
    assert_eq!(r.tap(KEY), Ok(BOTH));
    assert_eq!(side(&r), page_keys(0));
    assert_eq!(r.board()["layer"], true, "the pad hides");
    assert_eq!(layer(&r), json!({ "shown": true, "page": 0, "pages": 3 }));
    let key = r.state()["keys"][KEY].clone();
    assert_eq!(
        (&key["on"], &key["paint"]["fill"]),
        (&json!(true), &json!("on"))
    );
    assert_eq!(r.tap(KEY), Ok(BOTH));
    assert_eq!(side(&r), usual, "a second click brings the side keys back");
    assert_eq!(layer(&r)["shown"], false);
    assert_eq!(r.state()["keys"][KEY]["on"], false);
    assert!(r.fakes.injector.strokes().is_empty());
    assert!(r.fakes.shortcuts.sent().is_empty());
}

/// Plan edge case 1: Select all, then Copy, needs no trip back.
#[test]
fn the_layer_stays_after_a_shortcut() {
    let r = Rig::new(&GOLDEN, None);
    r.tap(KEY).unwrap();
    assert_eq!(r.tap("sc-select-all"), Ok(STATE));
    assert_eq!(r.tap("sc-copy"), Ok(STATE));
    assert_eq!(layer(&r)["shown"], true);
    let want = [(sid("select-all"), Vec::new()), (sid("copy"), Vec::new())];
    assert_eq!(r.fakes.shortcuts.sent(), want);
}

#[test]
fn next_turns_the_page_and_wraps_from_page_3_to_page_1() {
    let r = Rig::new(&GOLDEN, None);
    r.tap(KEY).unwrap();
    for page in [1, 2, 0] {
        assert_eq!(r.tap(NEXT_ID), Ok(BOTH), "to page {page}");
        assert_eq!(side(&r), page_keys(page));
        assert_eq!(layer(&r)["page"], page);
    }
    assert!(r.fakes.shortcuts.sent().is_empty(), "Next sends nothing");
}

/// Plan edge case 3: nothing to show dims the Shortcuts key with its tip, and a click does nothing.
#[test]
fn with_nothing_to_show_the_key_dims_says_so_and_does_nothing() {
    let fakes = Fakes::golden(&GOLDEN);
    fakes.shortcuts.set_known(Vec::new());
    let unknown = Rig::on(fakes, None);
    let none = toml::Value::Array(Vec::new());
    let no_pages = Rig::new(&GOLDEN, Some(("shortcut_pages", none)));
    for r in [unknown, no_pages] {
        let key = r.state()["keys"][KEY].clone();
        let dim = json!({ "dim": true, "tip": NO_SHORTCUTS_TIP, "on": false, "fill": "dim" });
        let got = json!({ "dim": key["dim"], "tip": key["tip"], "on": key["on"], "fill": key["paint"]["fill"] });
        assert_eq!(got, dim);
        assert_eq!(r.tap(KEY), Ok(FaceReply::default()));
        assert_eq!(layer(&r), json!({ "shown": false, "page": 0, "pages": 0 }));
        assert_eq!(r.board()["layer"], false);
    }
}

/// Plan edge case 3: ids the platform lacks are skipped, and a page they leave empty goes.
#[test]
fn ids_the_platform_lacks_and_pages_they_empty_are_skipped() {
    let fakes = Fakes::golden(&GOLDEN);
    fakes
        .shortcuts
        .set_known(known::only(&["copy", "paste", "f1"]));
    let r = Rig::on(fakes, None);
    r.tap(KEY).unwrap();
    let want = [KEY, "sc-copy", "sc-paste", NEXT_ID];
    assert_eq!(side(&r), want);
    assert_eq!(layer(&r)["pages"], 2, "page 2 had none of them");
    r.tap(NEXT_ID).unwrap();
    assert_eq!(side(&r), [KEY, "sc-f1", NEXT_ID]);
}

/// Plan edge case 13: a settings change keeps the layer shown and brings its page back in range.
#[test]
fn a_settings_change_keeps_the_layer_shown_and_brings_the_page_back_in_range() {
    let mut r = Rig::new(&GOLDEN, None);
    r.tap(KEY).unwrap();
    r.tap(NEXT_ID).unwrap();
    r.tap(NEXT_ID).unwrap();
    let pages = toml::Value::try_from([["copy"], ["paste"]]).unwrap();
    r.restart(Some(("shortcut_pages", pages)));
    assert_eq!(layer(&r), json!({ "shown": true, "page": 1, "pages": 2 }));
    assert_eq!(side(&r), [KEY, "sc-paste", NEXT_ID]);
}

/// Plan edge case 1: the layer is not saved, so a new keyboard opens on the usual side keys.
#[test]
fn a_new_keyboard_opens_on_the_usual_side_keys() {
    let r = Rig::new(&GOLDEN, None);
    let usual = side(&r);
    r.tap(KEY).unwrap();
    let again = Rig::on(r.fakes.clone(), None);
    assert_eq!(side(&again), usual);
    assert_eq!(layer(&again)["shown"], false);
}

#[test]
fn a_cell_the_layer_does_not_draw_now_is_no_key() {
    let r = Rig::new(&GOLDEN, None);
    for id in ["sc-copy", NEXT_ID] {
        assert_eq!(r.tap(id), Err(FaceError::UnknownKey), "hidden: {id}");
    }
    r.tap(KEY).unwrap();
    assert_eq!(r.tap("sc-f1"), Err(FaceError::UnknownKey), "page 3");
    assert_eq!(r.press("sc-f1"), Err(FaceError::UnknownKey));
    assert!(r.fakes.shortcuts.sent().is_empty());
}

/// Plan edge case 11: showing the layer while the pad scrolls stops it, as Stop leaves the screen; Right-click and Grab
/// keep their modes.
#[test]
fn showing_the_layer_while_the_pad_scrolls_stops_it_and_mouse_assist_keeps_its_modes() {
    let stops = Arc::new(Mutex::new(Vec::new()));
    let heard = Arc::clone(&stops);
    let b = Bench::after(None, move |bus| vec![ear(bus, &heard, |_: &ScrollStop| ())]);
    let scroll = |dir| ScrollChanged {
        live: true,
        dir,
        fast: false,
    };
    b.mouse(true, true, GrabState::Armed);
    b.send(&scroll(Some(Dir::Down)));
    b.said();
    let stopped = || std::mem::take(&mut *stops.lock().unwrap()).len();
    assert_eq!(b.rig.tap(KEY), Ok(BOTH));
    assert_eq!(stopped(), 1, "showing stops the pad");
    b.rig.tap(KEY).unwrap();
    assert_eq!(stopped(), 0, "hiding never stops");
    b.send(&scroll(None));
    b.rig.tap(KEY).unwrap();
    assert_eq!(stopped(), 0, "a still pad needs no stop");
    let toggled = b.said().into_iter().any(|s| matches!(s, Said::Toggle(_)));
    assert!(!toggled, "the mouse modes stay");
    b.rig.tap(KEY).unwrap();
    assert_eq!(
        (b.key("rclick")["on"].clone(), b.key("grab")["on"].clone()),
        (json!(true), json!(true))
    );
}

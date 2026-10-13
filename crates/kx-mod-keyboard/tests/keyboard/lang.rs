//! The language key's three designs as the mock-up moves them, and the List driven by the board's keys.

use crate::host::{GOLDEN, id};
use crate::rig::{Rig, STATE};
use kx_mod_keyboard::view::LONE_TIP;
use kx_module_api::keys;
use kx_module_api::ports::{FaceAction, FaceError, FaceReply, Scan, SwitchError};
use kx_platform_fake::{Ask, Stroke};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

/// Space's scan code, as the board sends it.
const SPACE: u16 = 0x39;

/// The short names of the Carousel: before, now and after.
fn carousel(r: &Rig) -> [Value; 3] {
    let lang = &r.state()["lang"];
    ["prev", "cur", "next"].map(|at| lang[at]["short"].clone())
}

#[test]
fn the_carousel_steps_back_on_its_left_part_and_on_elsewhere() {
    let r = Rig::new(&GOLDEN, None);
    assert_eq!(carousel(&r), [json!("ع"), json!("EN"), json!("FR")]);
    assert_eq!(
        r.click("lang", false),
        Ok(FaceReply::default()),
        "it only asks"
    );
    assert_eq!(r.click("lang", true), Ok(FaceReply::default()));
    assert_eq!(r.fakes.layouts.asked(), [Ask::Step(1), Ask::Step(-1)]);
}

#[test]
fn the_carousel_names_turn_the_way_it_stepped_once_the_app_switches() {
    let r = Rig::new(&GOLDEN, None);
    r.tap("lang").unwrap();
    assert_eq!(r.switch("fr-fr"), STATE);
    assert_eq!(carousel(&r), [json!("EN"), json!("FR"), json!("ع")]);
    assert_eq!(r.state()["lang"]["turn"], 1, "from the right");
    r.click("lang", true).unwrap();
    r.switch("en-us");
    assert_eq!(r.state()["lang"]["turn"], -1, "from the left");
    r.switch("ar-tn");
    assert_eq!(r.state()["lang"]["turn"], 0, "a switch made another way");
}

#[test]
fn arabic_takes_the_arabic_font_and_reads_right_to_left() {
    let r = Rig::new(&GOLDEN, None);
    let prev = &r.state()["lang"]["prev"];
    assert_eq!(
        (&prev["rtl"], &prev["arabic"]),
        (&json!(true), &json!(true))
    );
    r.switch("ar-tn");
    let lang = &r.state()["lang"];
    assert_eq!(lang["cur"]["short"], "ع");
    assert_eq!(lang["space"], json!(["العربية", "Arabic (102) AZERTY"]));
}

#[test]
fn the_face_names_the_language_of_the_layout_shown() {
    let r = Rig::new(&GOLDEN, None);
    assert_eq!(r.face.lang(), "en");
    for (name, iso) in [("fr-fr", "fr"), ("ar-tn", "ar"), ("en-us", "en")] {
        r.switch(name);
        assert_eq!(r.face.lang(), iso, "{name}");
    }
}

#[test]
fn the_cycle_steps_on_wherever_it_is_clicked_and_lights_one_dot() {
    let r = Rig::new(&GOLDEN, Some(("lang_key", "cycle".into())));
    r.click("lang", true).unwrap();
    r.click("lang", false).unwrap();
    assert_eq!(r.fakes.layouts.asked(), [Ask::Step(1), Ask::Step(1)]);
    r.switch("fr-fr");
    let lang = &r.state()["lang"];
    assert_eq!((&lang["at"], &lang["count"]), (&json!(1), &json!(3)));
    assert_eq!(lang["turn"], 0, "the Cycle does not slide");
    assert_eq!(
        r.board()["keys"]
            .as_array()
            .unwrap()
            .iter()
            .find(|k| k["id"] == "lang")
            .unwrap()["icon"],
        "lang.cycle"
    );
}

#[test]
fn with_one_layout_the_language_key_stays_still_and_says_so() {
    assert!(Rig::new(&GOLDEN, None).state()["keys"]["lang"]["tip"].is_null());
    for design in ["carousel", "cycle", "list"] {
        let r = Rig::new(&["ar-tn"], Some(("lang_key", design.into())));
        assert_eq!(r.state()["keys"]["lang"]["tip"], LONE_TIP, "{design}");
        assert_eq!(r.tap("lang"), Ok(FaceReply::default()), "{design}");
        assert!(r.state()["list"].is_null(), "{design}: no list opens");
        assert!(
            r.fakes.layouts.asked().is_empty(),
            "{design}: nothing asked"
        );
    }
}

#[test]
fn a_switch_the_app_refuses_shows_a_notice() {
    let r = Rig::new(&GOLDEN, None);
    r.fakes.layouts.set_error(Some(SwitchError::NoTarget));
    let want = FaceReply {
        notice: Some(keys::LANG_NOT_SWITCHED),
        ..FaceReply::default()
    };
    assert_eq!(r.tap("lang"), Ok(want));
}

/// A List-design face on the golden layouts.
fn list() -> Rig {
    Rig::new(&GOLDEN, Some(("lang_key", "list".into())))
}

#[test]
fn an_open_list_closes_from_the_key_once_one_layout_is_left() {
    let r = list();
    assert_eq!(r.tap("lang"), Ok(STATE));
    let gone = ["fr-fr", "ar-tn"].map(|name| r.fakes.layouts.remove(id(name)));
    assert_eq!(gone, [true, true]);
    assert_eq!(r.tap("lang"), Ok(STATE), "the key closes it");
    assert!(r.state()["list"].is_null());
    assert_eq!(r.state()["keys"]["lang"]["tip"], LONE_TIP, "then says so");
    assert_eq!(r.tap("lang"), Ok(FaceReply::default()), "and stays still");
    assert!(r.state()["list"].is_null() && r.fakes.layouts.asked().is_empty());
}

#[test]
fn the_list_opens_on_the_current_row_and_ticks_it() {
    let r = list();
    r.switch("fr-fr");
    assert_eq!(r.tap("lang"), Ok(STATE));
    let open = &r.state()["list"];
    assert_eq!(open["hi"], 1);
    assert_eq!(open["rows"][1]["icon"], "lang.current");
    assert_eq!(open["rows"][2]["short"], "ع");
    assert_eq!(
        open["keys"],
        json!(["esc", "enter", "up", "left", "down", "right"])
    );
    assert_eq!(r.tap("lang"), Ok(STATE), "a second click closes it");
    assert!(r.state()["list"].is_null());
}

#[test]
fn while_the_list_is_open_the_arrows_enter_and_esc_drive_it() {
    let r = list();
    r.tap("lang").unwrap();
    assert_eq!(r.panel("down", 5), Ok(STATE));
    assert_eq!(r.panel("down", 5), Ok(STATE));
    assert_eq!(r.panel("down", 5), Ok(FaceReply::default()), "the last row");
    assert_eq!(r.panel("up", 5), Ok(STATE));
    assert_eq!(r.panel("enter", 5), Ok(STATE));
    assert_eq!(r.fakes.layouts.asked(), [Ask::Pick(id("fr-fr"))]);
    assert!(r.state()["list"].is_null(), "a pick closes it");
    r.tap("lang").unwrap();
    assert_eq!(r.panel("esc", 5), Ok(STATE));
    assert!(r.state()["list"].is_null());
    assert_eq!(r.fakes.layouts.asked().len(), 1, "Esc picks nothing");
}

#[test]
fn paging_follows_the_page_size_the_ui_gives() {
    let r = list();
    r.tap("lang").unwrap();
    assert_eq!(r.panel("right", 2), Ok(STATE));
    assert_eq!(
        r.state()["list"]["hi"],
        2,
        "two-row pages: page 2 starts at row 3"
    );
    assert_eq!(
        r.panel("right", 2),
        Ok(FaceReply::default()),
        "the last page"
    );
    r.panel("left", 2).unwrap();
    assert_eq!(r.state()["list"]["hi"], 0);
    r.panel("right", 5).unwrap();
    assert_eq!(r.state()["list"]["hi"], 0, "five-row pages hold all three");
}

#[test]
fn a_panel_key_clicked_as_a_tap_never_reaches_the_app() {
    let r = list();
    r.tap("lang").unwrap();
    assert_eq!(r.tap("enter"), Ok(FaceReply::default()));
    assert!(r.fakes.injector.strokes().is_empty());
    assert_eq!(r.tap("c-0x0010"), Ok(STATE), "other keys still type");
    assert_eq!(r.tap("space"), Ok(STATE), "the list does not use Space");
    let strokes = r.fakes.injector.strokes();
    assert_eq!(strokes.len(), 2);
    assert_eq!(
        strokes[1],
        Stroke::Tap(Scan::new(SPACE)),
        "Space reaches the app"
    );
    assert!(!r.state()["list"].is_null(), "still open");
}

#[test]
fn a_pick_the_app_refuses_closes_the_list_and_shows_a_notice() {
    let r = list();
    r.tap("lang").unwrap();
    r.fakes.layouts.set_error(Some(SwitchError::NoTarget));
    let want = FaceReply {
        notice: Some(keys::LANG_NOT_SWITCHED),
        ..STATE
    };
    assert_eq!(r.act(FaceAction::LangPick { index: 2 }), Ok(want));
    assert!(r.state()["list"].is_null());
    assert_eq!(r.shown(), "EN", "nothing switched");
}

#[test]
fn a_layout_change_listener_reads_the_new_name_with_its_turn() {
    let r = Rig::new(&GOLDEN, None);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let (face, log) = (Arc::downgrade(&r.face), Arc::clone(&seen));
    r.fakes.layouts.on_change(move || {
        let json = face.upgrade().map(|f| f.state_json()).unwrap_or_default();
        let lang = serde_json::from_str::<Value>(&json).unwrap()["lang"].clone();
        log.lock()
            .unwrap()
            .push((lang["cur"]["short"].clone(), lang["turn"].clone()));
    });
    r.tap("lang").unwrap();
    r.switch("fr-fr");
    assert_eq!(*seen.lock().unwrap(), [(json!("FR"), json!(1))]);
}

#[test]
fn a_panel_key_with_no_list_open_does_nothing() {
    let r = list();
    assert_eq!(r.panel("down", 5), Ok(FaceReply::default()));
    assert_eq!(
        r.panel("c-0x0010", 5),
        Ok(FaceReply::default()),
        "not a panel key"
    );
}

#[test]
fn a_row_can_be_picked_by_its_place_and_a_missing_one_is_refused() {
    let r = list();
    r.tap("lang").unwrap();
    assert_eq!(r.act(FaceAction::LangPick { index: 2 }), Ok(STATE));
    assert_eq!(r.fakes.layouts.asked(), [Ask::Pick(id("ar-tn"))]);
    assert!(r.state()["list"].is_null());
    let missing = r.act(FaceAction::LangPick { index: 3 });
    assert_eq!(missing, Err(FaceError::NoChoice(3)));
}

#[test]
fn a_new_design_closes_an_open_list() {
    let mut r = list();
    r.tap("lang").unwrap();
    r.restart(Some(("lang_key", "list".into())));
    assert!(!r.state()["list"].is_null(), "the same design keeps it");
    r.restart(None);
    assert!(r.state()["list"].is_null());
}

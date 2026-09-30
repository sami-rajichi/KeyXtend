//! The face's JSON pinned by golden files in `tests/golden/`; the QML test fixtures are made from them.
//! Run with `KX_BLESS=1` to rewrite them after a deliberate change, and review the diff.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

use crate::host::GOLDEN;
use crate::rig::Rig;
use kx_module_api::ports::{BASE_DPI, FaceAction, Rect, Screen};
use kx_test_support::golden::assert_golden;
use serde_json::Value;
use std::path::PathBuf;

/// `json` laid out one value per line, keys sorted, so a change shows as a small diff.
fn pretty(json: &str) -> String {
    let value: Value = serde_json::from_str(json).unwrap();
    serde_json::to_string_pretty(&value).unwrap() + "\n"
}

fn golden(name: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "tests", "golden", name]
        .iter()
        .collect()
}

/// The face on the golden layouts, showing `name`.
fn showing(name: &str, change: Option<(&str, toml::Value)>) -> Rig {
    let r = Rig::new(&GOLDEN, change);
    if name != GOLDEN[0] {
        r.switch(name);
    }
    r
}

#[test]
fn the_board_matches_its_golden_file() {
    let r = showing("en-us", None);
    assert_golden(golden("board.json"), &pretty(&r.face.board_json()));
}

#[test]
fn the_english_state_matches_its_golden_file() {
    let r = showing("en-us", None);
    assert_golden(golden("state-en.json"), &pretty(&r.face.state_json()));
}

#[test]
fn the_french_state_matches_its_golden_file() {
    let r = showing("fr-fr", None);
    r.tap("shift").unwrap();
    let state = r.state();
    assert_eq!(
        state["keys"]["c-0x0056"]["main"], ">",
        "the ISO key, shifted"
    );
    assert_golden(golden("state-fr.json"), &pretty(&r.face.state_json()));
}

#[test]
fn the_arabic_state_matches_its_golden_file() {
    let r = showing("ar-tn", None);
    let state = r.state();
    let cur = &state["lang"]["cur"];
    assert_eq!(cur["short"], "ع");
    assert_eq!(
        (&cur["rtl"], &cur["arabic"]),
        (&Value::Bool(true), &Value::Bool(true))
    );
    assert!(
        state["keys"]["c-0x0056"]["main"].is_string(),
        "the ISO key is on the board"
    );
    assert_golden(golden("state-ar.json"), &pretty(&r.face.state_json()));
}

#[test]
fn the_open_list_matches_its_golden_file() {
    let r = showing("fr-fr", Some(("lang_key", "list".into())));
    let rect = Rect {
        x: 0,
        y: 0,
        w: 1920,
        h: 1040,
    };
    let screens = vec![Screen {
        rect,
        dpi: BASE_DPI,
        primary: true,
    }];
    r.act(FaceAction::Placed { rect, screens }).unwrap();
    r.tap("lang").unwrap();
    r.panel("down", 5).unwrap();
    assert_golden(golden("state-list.json"), &pretty(&r.face.state_json()));
}

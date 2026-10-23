//! The layer's shortcut cells (spec §4.2): each sends its id with the latched keys held, then the latches let go as
//! after any key; a refused send shows the notice, a dead key's wait ends, and a held cell never repeats.

use super::KEY;
use crate::host::GOLDEN;
use crate::known::sid;
use crate::rig::{BOTH, Rig, STATE};
use kx_mod_keyboard::spec::{NEXT_ID, char_id};
use kx_module_api::Redacted;
use kx_module_api::keys::KEYS_NOT_SENT;
use kx_module_api::ports::{FaceReply, Scan, ShortcutError, TextRoute};
use kx_platform_fake::Stroke;
use serde_json::Value;

const SHIFT: u16 = 0x2A;
const CTRL: u16 = 0x1D;
const WIN: u16 = 0xE05B;
/// The French dead `^`, and `e`.
const HAT: u16 = 0x1A;
const E: u16 = 0x12;

/// The keyboard on the golden layouts with `change`, the layer shown on its first page.
fn shown(change: Option<(&str, toml::Value)>) -> Rig {
    let r = Rig::new(&GOLDEN, change);
    r.tap(KEY).unwrap();
    r
}

/// The scan codes `codes`.
fn held(codes: &[u16]) -> Vec<Scan> {
    codes.iter().map(|&c| Scan::new(c)).collect()
}

/// True while key `id` shows latched.
fn on(r: &Rig, id: &str) -> bool {
    r.state()["keys"][id]["on"] == true
}

/// The modifiers setting as `value`.
fn modifiers(value: &str) -> (&'static str, toml::Value) {
    ("modifiers", value.into())
}

/// Plan edge case 5: latched keys join the shortcut, once each; the platform merges a key both hold.
#[test]
fn a_latched_ctrl_joins_copy_and_a_latched_shift_joins_new_tab() {
    let r = shown(None);
    r.tap("ctrl").unwrap();
    assert_eq!(r.tap("sc-copy"), Ok(STATE));
    r.tap(NEXT_ID).unwrap();
    r.tap("shift").unwrap();
    r.tap("sc-new-tab").unwrap();
    let want = [
        (sid("copy"), held(&[CTRL])),
        (sid("new-tab"), held(&[SHIFT])),
    ];
    assert_eq!(r.fakes.shortcuts.sent(), want);
    assert!(!on(&r, "ctrl") && !on(&r, "shift"), "they let go after");
    assert!(
        r.fakes.injector.strokes().is_empty(),
        "nothing typed beside"
    );
}

/// Plan edge case 5: "Stay held" keeps the latch for the next shortcut.
#[test]
fn stay_held_keeps_the_latches_after_a_shortcut() {
    let r = shown(Some(modifiers("stay")));
    r.tap("ctrl").unwrap();
    r.tap("sc-copy").unwrap();
    r.tap("sc-paste").unwrap();
    let want = [(sid("copy"), held(&[CTRL])), (sid("paste"), held(&[CTRL]))];
    assert_eq!(r.fakes.shortcuts.sent(), want);
    assert!(on(&r, "ctrl"));
}

/// Plan edge case 6: a shortcut uses a lone-ready Win, so Win clicked again never opens Start.
#[test]
fn a_lone_ready_win_then_a_shortcut_gives_no_lone_press() {
    for mode in ["release", "stay"] {
        let r = shown(Some(modifiers(mode)));
        r.tap("win").unwrap();
        r.tap("sc-copy").unwrap();
        r.tap("win").unwrap();
        assert_eq!(
            r.fakes.shortcuts.sent(),
            [(sid("copy"), held(&[WIN]))],
            "{mode}"
        );
        assert!(r.fakes.injector.strokes().is_empty(), "{mode}: no lone Win");
    }
}

/// Plan edge case 7: a chain is one send; the platform sends its steps in order.
#[test]
fn a_chain_goes_out_as_one_send() {
    let r = shown(None);
    r.tap("sc-copy-all").unwrap();
    assert_eq!(r.fakes.shortcuts.sent(), [(sid("copy-all"), Vec::new())]);
}

/// Plan edge case 7: a refused send shows the input notice; the latches let go only if some of it went out, as for a
/// refused chord.
#[test]
fn a_refused_send_shows_the_notice_and_lets_the_latches_go_as_a_refused_chord_does() {
    let notice = FaceReply {
        notice: Some(KEYS_NOT_SENT),
        ..STATE
    };
    let cases = [
        (ShortcutError::Refused { sent: 0, total: 4 }, true),
        (ShortcutError::Refused { sent: 2, total: 4 }, false),
        (ShortcutError::Failed, true),
    ];
    for (error, kept) in cases {
        let r = shown(None);
        r.tap("shift").unwrap();
        r.fakes.shortcuts.set_error(Some(error));
        assert_eq!(r.tap("sc-copy"), Ok(notice), "{error:?}");
        assert_eq!(on(&r, "shift"), kept, "{error:?}");
        assert_eq!(r.state()["layer"]["shown"], true);
    }
}

/// Plan edge case 9: a dead key waits, then a shortcut ends the wait, so the next letter types as text.
#[test]
fn a_dead_key_waits_then_a_shortcut_ends_the_wait() {
    let r = Rig::new(&["fr-fr"], None);
    r.tap(&char_id(Scan::new(HAT))).unwrap();
    r.tap(KEY).unwrap();
    r.tap("sc-copy").unwrap();
    r.tap(KEY).unwrap();
    r.tap(&char_id(Scan::new(E))).unwrap();
    let text = Stroke::Text(Redacted::new("e".into()), TextRoute::Unicode);
    assert_eq!(
        r.fakes.injector.strokes(),
        [Stroke::Tap(Scan::new(HAT)), text]
    );
}

/// Plan edge case 10: a held shortcut cell sends once and never repeats.
#[test]
fn holding_a_shortcut_cell_never_repeats() {
    let r = shown(None);
    let board = r.board();
    let keys = board["keys"].as_array().unwrap();
    let cells = keys
        .iter()
        .filter(|k| k["id"].as_str().is_some_and(|id| id.starts_with("sc-")));
    assert!(cells.clone().count() > 0 && cells.into_iter().all(|k| k["repeat"] == false));
    assert_eq!(
        r.press("sc-copy"),
        Ok(STATE),
        "it types at the press and holds nothing"
    );
    assert_eq!(r.tick_in(10_000), FaceReply::default());
    assert_eq!(r.let_go("sc-copy"), Ok(FaceReply::default()));
    assert_eq!(r.fakes.shortcuts.sent(), [(sid("copy"), Vec::new())]);
}

/// Plan edge case 10: with the language list open, a shortcut cell still goes to the app.
#[test]
fn with_the_language_list_open_a_shortcut_cell_still_sends() {
    let r = Rig::new(&GOLDEN, Some(("lang_key", "list".into())));
    r.tap("lang").unwrap();
    assert_eq!(r.tap(KEY), Ok(BOTH));
    assert_eq!(r.tap("sc-copy"), Ok(STATE));
    assert_eq!(r.fakes.shortcuts.sent(), [(sid("copy"), Vec::new())]);
    assert_ne!(r.state()["list"], Value::Null, "the list stays open");
}

//! Letters follow the last click (ADR-0040): the open panel takes the keys only while aimed at, and every other key
//! reaches the app as if no panel were open.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

mod chords;
mod forms;
mod picks;
mod presses;
mod props;
mod state;
mod told;

use crate::clip::{self, Clip};
use crate::fill::{APP, Heard, Quick, WINDOW, gap_ms, line, login, text, typed, unicode};
use crate::rig::{HELD, STATE};
use kx_module_api::ports::{
    ClipPanel, FaceAction, FaceReply, PanelKey, PasteAsk, PointerPressed, Scan,
};
use kx_platform_fake::Stroke;

/// The board's Q key.
const LETTER: &str = "c-0x0010";
/// A key of each kind: a letter, an arrow, Delete, Enter and Esc.
const KEYS: [&str; 5] = [LETTER, "up", "del", "enter", "esc"];
/// The board's Up arrow, as it reaches the app.
const UP: u16 = 0xE048;
/// A window other than the one in front while the tests aim.
const ELSEWHERE: u64 = 2;

impl Quick {
    /// A click outside our windows.
    fn away(&self) {
        self.b.send(&PointerPressed { ours: false });
    }

    /// A press on the open panel.
    fn aim(&self) -> FaceReply {
        self.b.rig.act(FaceAction::AimPanel).unwrap()
    }

    /// The state's `aimed`.
    fn aimed(&self) -> bool {
        self.b.rig.state()["aimed"] == true
    }

    /// Clicks the letter: true when the panel took it, false when the app did.
    fn letter_to_panel(&self) -> bool {
        self.b.rig.tap(LETTER).unwrap();
        let (heard, strokes) = (self.heard(), self.strokes());
        match (heard.as_slice(), strokes.as_slice()) {
            ([key], []) if *key == text("q") => true,
            ([], [stroke]) if *stroke == unicode("q") => false,
            other => panic!("the letter went nowhere or twice: {other:?}"),
        }
    }
}

#[test]
fn a_panel_opening_takes_a_letter_and_tells_prediction_nothing() {
    let q = Quick::opened(None);
    q.pieces();
    assert!(q.letter_to_panel());
    assert!(q.pieces().is_empty(), "panel keys never publish Typed");
}

#[test]
fn after_an_outside_click_every_key_reaches_the_app() {
    let q = Quick::opened(None);
    q.away();
    for id in KEYS {
        q.b.rig.tap(id).unwrap();
        assert_eq!(q.strokes().len(), 1, "{id} reaches the app");
    }
    assert!(q.heard().is_empty(), "no FillKey goes out");
    assert!(!q.pieces().is_empty(), "prediction hears them as usual");
}

#[test]
fn after_an_outside_click_the_clipboard_panels_keys_reach_the_app_too() {
    let c = Clip::quiet(None);
    c.open(true);
    c.b.send(&PointerPressed { ours: false });
    for id in KEYS {
        c.b.rig.tap(id).unwrap();
        assert_eq!(c.strokes().len(), 1, "{id} reaches the app");
    }
    assert!(c.heard().is_empty(), "no ClipKey goes out");
    c.b.rig.act(FaceAction::AimPanel).unwrap();
    c.b.rig.tap(LETTER).unwrap();
    assert_eq!(c.heard(), [clip::text("q")]);
}

#[test]
fn unaimed_a_held_key_starts_its_hold_as_usual() {
    let q = Quick::opened(None);
    q.away();
    assert_eq!(q.b.rig.press(LETTER), Ok(HELD));
    assert_eq!(q.strokes(), [unicode("q")]);
    assert!(q.heard().is_empty());
}

#[test]
fn unaimed_shift_latches_for_the_app() {
    let q = Quick::opened(None);
    q.away();
    q.b.rig.tap("shift").unwrap();
    assert!(q.on("shift"));
    q.b.rig.tap(LETTER).unwrap();
    assert_eq!(q.strokes(), [unicode("Q")]);
}

#[test]
fn a_click_on_our_own_windows_changes_nothing() {
    let q = Quick::opened(None);
    q.b.send(&PointerPressed { ours: true });
    assert!(q.letter_to_panel());
    q.away();
    q.b.send(&PointerPressed { ours: true });
    assert!(!q.letter_to_panel());
}

#[test]
fn the_aim_action_aims_again_and_asks_a_redraw() {
    let q = Quick::opened(None);
    q.away();
    assert!(!q.letter_to_panel());
    assert_eq!(q.aim(), STATE);
    assert!(q.letter_to_panel());
}

#[test]
fn with_no_panel_open_the_aim_action_does_nothing() {
    let q = Quick::quiet(None);
    assert_eq!(q.aim(), FaceReply::default());
    assert!(!q.aimed());
    assert!(!q.letter_to_panel());
}

#[test]
fn an_outside_click_after_the_aim_takes_it_again() {
    let q = Quick::opened(None);
    q.away();
    q.aim();
    assert!(q.letter_to_panel());
    q.away();
    assert!(!q.letter_to_panel());
}

#[test]
fn the_front_window_changing_takes_the_aim_at_the_next_key_which_reaches_the_app() {
    let q = Quick::opened(None);
    q.window(Some(ELSEWHERE));
    assert!(!q.letter_to_panel());
    assert!(!q.letter_to_panel());
    q.aim();
    assert!(q.letter_to_panel(), "aimed in the new front window");
}

#[test]
fn two_windows_of_one_program_count_as_a_change() {
    let q = Quick::opened(None);
    q.front(Some(APP));
    q.window(Some(ELSEWHERE));
    assert!(!q.letter_to_panel(), "same program, another window");
}

#[test]
fn the_programs_name_alone_never_takes_the_aim() {
    let q = Quick::opened(None);
    q.front(Some("mail.exe"));
    assert!(q.letter_to_panel(), "the window stayed");
}

#[test]
fn a_front_window_the_system_cannot_name_changes_nothing() {
    let q = Quick::opened(None);
    q.window(None);
    assert!(q.letter_to_panel());
    q.window(Some(WINDOW));
    assert!(q.letter_to_panel());
    q.window(Some(ELSEWHERE));
    assert!(!q.letter_to_panel());
}

#[test]
fn opened_with_no_named_front_the_first_name_is_where_it_stays() {
    let q = Quick::quiet(None);
    q.window(None);
    q.open(true);
    q.heard();
    q.window(Some(ELSEWHERE));
    assert!(q.letter_to_panel());
    q.window(Some(WINDOW));
    assert!(!q.letter_to_panel());
}

#[test]
fn a_language_switch_keeps_the_aim() {
    let q = Quick::opened(None);
    q.b.rig.switch("ar-tn");
    q.b.rig.tap(LETTER).unwrap();
    assert_eq!(q.heard(), [text("ض")], "the search survives it");
    assert!(q.strokes().is_empty());
}

#[test]
fn a_fill_takes_the_aim() {
    let q = Quick::opened(None);
    q.fill(1, vec![line("a")]);
    assert_eq!(q.heard(), [typed(1, true)]);
    q.strokes();
    assert!(!q.letter_to_panel());
}

#[test]
fn a_refused_fill_leaves_the_aim() {
    let q = Quick::quiet(None);
    q.b.rig.switch("fr-fr");
    q.b.rig.press("c-0x001A").unwrap();
    q.strokes();
    q.open(true);
    q.heard();
    q.fill(2, vec![line("a")]);
    assert_eq!(q.heard(), [typed(2, false)]);
    assert!(q.aimed(), "nothing typed, so the letters stay");
}

#[test]
fn a_paste_takes_the_aim() {
    let c = Clip::quiet(None);
    c.open(true);
    c.b.send(&PasteAsk { round: 1 });
    c.heard();
    c.b.rig.tap(LETTER).unwrap();
    assert!(c.heard().is_empty());
    assert_eq!(c.strokes().len(), 1, "the letter reaches the app");
}

#[test]
fn unaimed_with_a_fill_waiting_a_key_does_nothing() {
    let q = Quick::opened(None);
    q.fill(3, login());
    q.strokes();
    for id in KEYS {
        q.b.rig.tap(id).unwrap();
    }
    q.b.rig.press("enter").unwrap();
    assert!(q.strokes().is_empty() && q.heard().is_empty());
    q.relabel_in(gap_ms());
    assert_eq!(q.strokes(), [unicode("pw")]);
    assert_eq!(q.heard(), [typed(3, true)]);
}

#[test]
fn a_panel_closed_then_opened_aims_afresh() {
    let q = Quick::opened(None);
    q.away();
    q.open(false);
    assert!(!q.aimed());
    q.open(true);
    q.heard();
    assert!(q.aimed());
    assert!(q.letter_to_panel());
}

#[test]
fn a_repeated_open_word_changes_nothing() {
    let q = Quick::opened(None);
    q.away();
    assert!(!q.letter_to_panel());
    q.open(true);
    q.heard();
    assert!(!q.letter_to_panel(), "the word repeats, nothing was opened");
}

#[test]
fn the_other_panel_opening_aims_at_it() {
    let q = Quick::opened(None);
    q.away();
    assert!(!q.letter_to_panel());
    q.b.send(&ClipPanel { open: true });
    assert!(q.aimed());
    q.b.rig.tap(LETTER).unwrap();
    assert!(q.strokes().is_empty(), "the clipboard panel took it");
}

#[test]
fn closing_the_panel_not_open_leaves_the_aim() {
    let q = Quick::opened(None);
    q.b.send(&ClipPanel { open: false });
    assert!(q.aimed());
}

#[test]
fn the_vault_stopping_ends_the_aim() {
    let q = Quick::opened(None);
    q.live(false);
    assert!(!q.aimed());
    assert!(!q.letter_to_panel());
}

#[test]
fn a_restart_forgets_the_aim_until_a_panel_opens_again() {
    let mut q = Quick::opened(None);
    q.b.rig.restart(Some(("lang_key", "cycle".into())));
    assert!(!q.aimed());
    q.live(true);
    q.open(true);
    assert!(q.aimed());
    q.heard();
    q.away();
    assert!(!q.letter_to_panel());
}

#[test]
fn unaimed_an_arrow_held_reaches_the_app_and_aimed_it_moves_the_panel() {
    let q = Quick::opened(None);
    q.away();
    q.b.rig.press("up").unwrap();
    assert_eq!(q.strokes(), [Stroke::Tap(Scan::new(UP))]);
    q.b.rig.let_go("up").unwrap();
    q.aim();
    q.b.rig.tap("up").unwrap();
    assert_eq!(q.heard(), [Heard::Key(PanelKey::Up)]);
}

//! Voice typing from the keyboard's side, over the bus (ADR-0042 decision 7): the Dictation key asks the voice module
//! to start or stop, its light follows `VoiceLit`, and a `TextAsk` erases, then types, as the keyboard stays the only
//! typist.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

mod during;
mod pieces;
mod text;

use crate::assist::{Bench, ear};
use crate::rig::STATE;
use kx_mod_keyboard::{BoardSpec, KeyboardSettings, Role};
use kx_module_api::ports::{
    FaceAction, FaceReply, Piece, TextAsk, TextDone, Typed, VoiceAsk, VoiceKey, VoiceLit,
};
use kx_module_api::{Secret, Zeroizing};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

/// The Dictation key.
const MIC: &str = "mic";

/// What the voice module and word prediction heard from the keyboard, in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Heard {
    /// The Dictation key was clicked.
    Key,
    /// The keyboard asked for the light.
    Ask,
    /// A piece told to word prediction.
    Piece(Piece),
    /// A text's answer.
    Done(TextDone),
}

/// The module on a bus the test listens to, with what the voice module and prediction heard.
pub struct Mic {
    /// The module and its bus.
    pub b: Bench,
    heard: Arc<Mutex<Vec<Heard>>>,
}

impl Mic {
    /// The module with `change` over the defaults, heard from before it starts.
    pub fn new(change: Option<(&str, toml::Value)>) -> Self {
        let heard = Arc::default();
        let h = Arc::clone(&heard);
        let b = Bench::after(change, move |bus| {
            vec![
                ear(bus, &h, |_: &VoiceKey| Heard::Key),
                ear(bus, &h, |_: &VoiceAsk| Heard::Ask),
                ear(bus, &h, |e: &Typed| Heard::Piece(e.piece.clone())),
                ear(bus, &h, |e: &TextDone| Heard::Done(*e)),
            ]
        });
        Self { b, heard }
    }

    /// The module with `change`, with what it said at start already taken.
    pub fn quiet(change: Option<(&str, toml::Value)>) -> Self {
        let m = Self::new(change);
        m.heard();
        m.b.said();
        m
    }

    /// What was heard since the last look.
    pub fn heard(&self) -> Vec<Heard> {
        let mut heard = self.heard.lock().unwrap_or_else(PoisonError::into_inner);
        std::mem::take(&mut *heard)
    }

    /// The voice module says the microphone is open, or not.
    pub fn lit(&self, rec: bool) {
        self.b.send(&VoiceLit { rec });
    }

    /// The voice module asks for `erase` erased, then `t` typed with `gap_ms` between pieces, in round `round`.
    pub fn ask(&self, round: u32, erase: u32, t: &str, gap_ms: u16) {
        self.b.send(&ask(round, erase, t, gap_ms));
    }

    /// The Dictation key's light: its flags and paint.
    fn light(&self) -> Value {
        let k = self.b.key(MIC);
        let p = &k["paint"];
        json!({ "rec": k["rec"], "on": k["on"], "dim": k["dim"], "paint": [p["fill"], p["ink"], p["skirt"]] })
    }
}

/// A request for `erase` erased, then `t` typed with `gap_ms` between pieces, in round `round`.
pub fn ask(round: u32, erase: u32, t: &str, gap_ms: u16) -> TextAsk {
    let text = Secret::new(Zeroizing::new(t.to_owned()));
    TextAsk {
        round,
        erase,
        text,
        gap_ms,
    }
}

/// The answer to round `round`: `count` characters typed, and whether all of it went.
pub fn done(round: u32, count: u32, ok: bool) -> Heard {
    Heard::Done(TextDone { round, count, ok })
}

/// The one break word prediction hears for a text.
pub const BREAK: Heard = Heard::Piece(Piece::Break);

/// `ms` milliseconds.
pub fn ms(ms: u64) -> Duration {
    Duration::from_millis(ms)
}

/// The keyboard's own pause after each Backspace (`erase_gap_ms`), by default.
pub fn erase_gap() -> Duration {
    ms(KeyboardSettings::from_defaults().unwrap().erase_gap_ms)
}

/// The key unlit: a side key's own paint.
fn unlit() -> Value {
    json!({ "rec": false, "on": false, "dim": false, "paint": ["key_act", "legend_act", "key_mod"] })
}

/// The key lit while the microphone is open: the theme's recording tokens.
fn lit() -> Value {
    json!({ "rec": true, "on": true, "dim": false, "paint": ["rec", "rec_ink", "rec"] })
}

#[test]
fn the_mic_key_has_the_voice_role_and_sends_no_scan_code() {
    let spec = BoardSpec::shipped().unwrap();
    let mic = spec.side.iter().find(|k| k.id == MIC).unwrap();
    assert_eq!(mic.role, Role::Voice);
    assert!(!Role::Voice.sends());
}

#[test]
fn the_board_draws_the_mic_key_live_and_checkable() {
    let m = Mic::quiet(None);
    let board = m.b.rig.board();
    let keys = board["keys"].as_array().unwrap();
    let mic = keys.iter().find(|k| k["id"] == MIC).unwrap();
    let got = json!([mic["dim"], mic["checkable"], mic["tip"], mic["repeat"]]);
    assert_eq!(got, json!([false, true, null, false]));
}

#[test]
fn a_click_sends_voice_key_and_keeps_every_latch() {
    let m = Mic::quiet(None);
    m.b.rig.tap("shift").unwrap();
    m.b.said();
    assert_eq!(m.b.rig.tap(MIC), Ok(STATE));
    assert_eq!(m.heard(), [Heard::Key]);
    assert!(m.b.rig.fakes.injector.strokes().is_empty(), "nothing typed");
    assert!(m.b.said().is_empty(), "the latched set never changed");
    assert_eq!(m.b.key("shift")["on"], true);
}

#[test]
fn the_keyboard_starts_unlit_and_asks_the_voice_module_for_the_light() {
    let m = Mic::new(None);
    assert_eq!(m.heard(), [Heard::Ask]);
    assert_eq!(m.light(), unlit());
}

#[test]
fn voice_lit_sets_the_rec_flag_and_paint_and_redraws_at_the_next_relabel() {
    let m = Mic::quiet(None);
    m.b.rig.state();
    m.lit(true);
    assert_eq!(m.b.rig.act(FaceAction::Relabel), Ok(STATE));
    assert_eq!(m.light(), lit());
    m.lit(true);
    let again = m.b.rig.act(FaceAction::Relabel);
    assert_eq!(again, Ok(FaceReply::default()), "no change, no redraw");
    m.lit(false);
    assert_eq!(m.b.rig.act(FaceAction::Relabel), Ok(STATE));
    assert_eq!(m.light(), unlit());
}

#[test]
fn a_restart_forgets_the_light_and_asks_again() {
    let mut m = Mic::quiet(None);
    m.lit(true);
    m.b.rig.restart(None);
    assert_eq!(m.heard(), [Heard::Ask]);
    assert_eq!(m.light(), unlit());
}

#[test]
fn only_the_mic_key_lights_red() {
    let m = Mic::quiet(None);
    m.lit(true);
    let state = m.b.rig.state();
    let keys = state["keys"].as_object().unwrap();
    let red: Vec<&String> = keys
        .iter()
        .filter(|(_, k)| k["rec"] == true)
        .map(|(id, _)| id)
        .collect();
    assert_eq!(red, [MIC]);
}

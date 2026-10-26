//! Word prediction from the keyboard's side, over the bus (ADR-0033): each typed piece goes out once and in order,
//! the typing mode at start and on each change, and a picked word types through the keyboard's own route.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

mod caps;
mod during;
mod gap;
mod mode;
mod threads;

use crate::assist::ear;
use crate::held::{A, hold};
use crate::host::{Fakes, GOLDEN};
use crate::rig::{Rig, STATE};
use crate::told::{BKSP, text};
use crate::typing::{tap, unicode};
use kx_mod_keyboard::KeyboardSettings;
use kx_module_api::ports::{FaceAction, InjectError, Piece, TypeWord, Typed, TypingMode};
use kx_module_api::{Bus, BusCore, Module, Redacted, Subscription};
use kx_test_support::host::FakeCore;
use std::sync::{Arc, Mutex, PoisonError};

/// The module on a bus the test hears the pieces and modes on, and sends picked words on.
pub struct Desk {
    /// The started module.
    pub rig: Rig,
    /// The bus, with no limit.
    pub bus: Bus,
    pieces: Arc<Mutex<Vec<Piece>>>,
    modes: Arc<Mutex<Vec<TypingMode>>>,
    _ears: Vec<Subscription>,
}

/// The mode with nothing latched or waiting, at the default `text_cap`.
pub fn plain() -> TypingMode {
    let cap = KeyboardSettings::from_defaults().unwrap().text_cap;
    TypingMode {
        cap: Some(cap),
        ..TypingMode::default()
    }
}

/// Takes what `said` holds.
fn take<T>(said: &Mutex<Vec<T>>) -> Vec<T> {
    std::mem::take(&mut *said.lock().unwrap_or_else(PoisonError::into_inner))
}

impl Desk {
    /// The module on the golden layouts with `change` over the defaults, heard from before it starts.
    pub fn new(change: Option<(&str, toml::Value)>) -> Self {
        Self::on(Fakes::golden(&GOLDEN), change)
    }

    /// The module over `fakes` with `change` over the defaults, heard from before it starts.
    pub fn on(fakes: Fakes, change: Option<(&str, toml::Value)>) -> Self {
        let core: Arc<dyn BusCore> = Arc::new(FakeCore::default());
        let bus = Bus::new(Arc::clone(&core));
        let (pieces, modes) = (Arc::default(), Arc::default());
        let ears = vec![
            ear(&bus, &pieces, |e: &Typed| e.piece.clone()),
            ear(&bus, &modes, |e: &TypingMode| *e),
        ];
        let rig = Rig::on_bus(fakes, change, core);
        Self {
            rig,
            bus,
            pieces,
            modes,
            _ears: ears,
        }
    }

    /// The pieces told since the last look.
    pub fn pieces(&self) -> Vec<Piece> {
        take(&self.pieces)
    }

    /// The modes told since the last look.
    pub fn modes(&self) -> Vec<TypingMode> {
        take(&self.modes)
    }

    /// The prediction module asks for `erase` Backspaces, then `t`.
    pub fn word(&self, erase: u16, t: &str) {
        let text = Redacted::new(t.into());
        let word = TypeWord {
            round: 0,
            erase,
            text,
        };
        self.bus.publish(&word).unwrap();
    }
}

#[test]
fn typing_through_the_face_tells_each_piece_once_in_order() {
    let d = Desk::new(None);
    d.rig.tap(A).unwrap();
    for id in ["space", "bksp", "enter", "esc"] {
        d.rig.tap(id).unwrap();
    }
    let want = [
        text("a"),
        text(" "),
        Piece::Erase,
        Piece::Advance,
        Piece::Break,
    ];
    assert_eq!(d.pieces(), want);
    d.rig.act(FaceAction::Relabel).unwrap();
    assert!(d.pieces().is_empty(), "nothing twice");
}

#[test]
fn a_refused_send_tells_nothing() {
    let d = Desk::new(None);
    let refused = InjectError::Refused { sent: 0, total: 2 };
    d.rig.fakes.injector.set_error(Some(refused));
    d.rig.tap(A).unwrap();
    assert!(d.pieces().is_empty());
}

#[test]
fn a_held_backspace_tells_one_erase_per_repeat() {
    let d = Desk::new(None);
    hold(&d.rig, "bksp", 2);
    assert_eq!(
        d.pieces(),
        vec![Piece::Erase; 4],
        "the press, the delay, two beats"
    );
}

#[test]
fn a_language_step_or_pick_breaks_and_opening_the_list_tells_nothing() {
    let d = Desk::new(Some(("lang_key", "cycle".into())));
    d.rig.tap("lang").unwrap();
    assert_eq!(d.pieces(), [Piece::Break], "a step");
    let d = Desk::new(Some(("lang_key", "list".into())));
    d.rig.tap("lang").unwrap();
    assert!(d.pieces().is_empty(), "the list opens");
    d.rig.act(FaceAction::LangPick { index: 1 }).unwrap();
    assert_eq!(d.pieces(), [Piece::Break], "a pick");
}

#[test]
fn a_picked_word_types_through_the_bus_and_is_told_like_keys() {
    let d = Desk::new(None);
    d.word(3, "hello ");
    let want = [tap(BKSP), tap(BKSP), tap(BKSP), unicode("hello ")];
    assert_eq!(d.rig.fakes.injector.strokes(), want);
    let told = [Piece::Erase, Piece::Erase, Piece::Erase, text("hello ")];
    assert_eq!(d.pieces(), told);
}

#[test]
fn a_picked_word_types_nothing_while_ctrl_is_latched() {
    let d = Desk::new(None);
    d.rig.tap("ctrl").unwrap();
    d.word(1, "hello");
    assert!(d.rig.fakes.injector.strokes().is_empty());
    assert!(d.pieces().is_empty());
}

#[test]
fn a_one_shot_shift_used_by_a_word_redraws_at_the_next_relabel() {
    let d = Desk::new(None);
    d.rig.tap("shift").unwrap();
    d.word(0, "Hello");
    assert_eq!(d.rig.act(FaceAction::Relabel), Ok(STATE));
    assert_eq!(d.rig.state()["keys"]["shift"]["on"], false);
}

#[test]
fn a_picked_word_types_nothing_while_the_keyboard_is_down() {
    let mut d = Desk::new(None);
    // The kernel drops the handlers before the stop, but one may still be running on the sender's thread.
    d.rig.module.stop();
    d.word(1, "hello");
    assert!(d.rig.fakes.injector.strokes().is_empty());
    d.rig.restart(None);
    d.word(0, "hi");
    assert_eq!(d.rig.fakes.injector.strokes(), [unicode("hi")]);
}

#[test]
fn a_new_text_cap_holds_after_a_restart() {
    let mut d = Desk::new(None);
    let cap = *kx_mod_keyboard::TEXT_CAP.start();
    d.rig.restart(Some(("text_cap", i64::from(cap).into())));
    d.word(0, &"a".repeat(usize::from(cap) + 1));
    assert!(d.rig.fakes.injector.strokes().is_empty());
    d.word(cap, "");
    assert_eq!(d.pieces(), vec![Piece::Erase; usize::from(cap)]);
}

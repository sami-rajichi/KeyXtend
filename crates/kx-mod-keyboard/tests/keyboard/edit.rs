//! The edit keys the selection chips ask for (ADR-0041 decision 4): the keyboard sends exactly Copy, Cut or Paste, or
//! a terminal's own copy and paste keys, with nothing held, and every latch stays as it was; each round is answered.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

use crate::assist::latches::stay;
use crate::assist::{Bench, Said, ear};
use crate::fill::{GAP, login};
use crate::held::A;
use crate::known::sid;
use crate::rig::{HELD, STATE};
use crate::told::BKSP;
use crate::typing::{tap, unicode};
use crate::words::Desk;
use kx_module_api::Module;
use kx_module_api::ports::{
    COPY_ID, COPY_TERM_ID, CUT_ID, ClipPanel, Edit, EditAsk, EditDone, FaceAction, FaceReply,
    FillType, PASTE_ID, PASTE_TERM_ID, Piece, Scan, ShortcutError, ShortcutId, Typed,
};
use kx_platform_fake::Stroke;
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;

/// The French dead `^`.
const CIRCUMFLEX: &str = "c-0x001A";
/// The Win key's scan code.
const WIN: u16 = 0xE05B;

/// The keyboard on a bus the test listens to, with its answers to the selection module and the pieces it told.
struct Chips {
    b: Bench,
    done: Arc<Mutex<Vec<EditDone>>>,
    pieces: Arc<Mutex<Vec<Piece>>>,
}

/// Takes what `said` holds.
fn take<T>(said: &Mutex<Vec<T>>) -> Vec<T> {
    std::mem::take(&mut *said.lock().unwrap_or_else(PoisonError::into_inner))
}

/// The request for `edit` in round `round`, in a terminal when `terminal`.
fn ask(round: u32, edit: Edit, terminal: bool) -> EditAsk {
    EditAsk {
        round,
        edit,
        terminal,
    }
}

/// The answer to round `round`.
fn done(round: u32, ok: bool) -> EditDone {
    EditDone { round, ok }
}

/// Shortcut `id` as the port took it, with nothing held.
fn alone(id: &str) -> (ShortcutId, Vec<Scan>) {
    (sid(id), Vec::new())
}

impl Chips {
    /// The keyboard started on the defaults, with what it said at start already taken.
    fn quiet() -> Self {
        let (done, pieces) = (Arc::default(), Arc::default());
        let (d, p) = (Arc::clone(&done), Arc::clone(&pieces));
        let b = Bench::after(None, move |bus| {
            vec![
                ear(bus, &d, |e: &EditDone| *e),
                ear(bus, &p, |e: &Typed| e.piece.clone()),
            ]
        });
        b.said();
        Self { b, done, pieces }
    }

    /// The selection module asks for `edit` in round `round`, in a terminal when `terminal`.
    fn ask(&self, round: u32, edit: Edit, terminal: bool) {
        self.b.send(&ask(round, edit, terminal));
    }

    /// The answers since the last look.
    fn done(&self) -> Vec<EditDone> {
        take(&self.done)
    }

    /// The pieces told to word prediction since the last look.
    fn pieces(&self) -> Vec<Piece> {
        take(&self.pieces)
    }

    /// The shortcuts sent since the last look, each with its held keys.
    fn sent(&self) -> Vec<(ShortcutId, Vec<Scan>)> {
        let shortcuts = &self.b.rig.fakes.shortcuts;
        let sent = shortcuts.sent();
        shortcuts.clear();
        sent
    }

    /// Whether key `id` is lit.
    fn on(&self, id: &str) -> bool {
        self.b.key(id)["on"] == true
    }
}

#[test]
fn an_edit_ask_sends_its_shortcut_alone_and_keeps_ctrl_and_shift_latched() {
    for (edit, id) in [
        (Edit::Copy, COPY_ID),
        (Edit::Cut, CUT_ID),
        (Edit::Paste, PASTE_ID),
    ] {
        let c = Chips::quiet();
        c.b.rig.tap("ctrl").unwrap();
        c.b.rig.tap("shift").unwrap();
        c.b.said();
        c.ask(4, edit, false);
        assert_eq!(c.done(), [done(4, true)], "{id}");
        assert_eq!(c.sent(), [alone(id)], "{id}: nothing held");
        assert!(c.b.rig.fakes.injector.strokes().is_empty(), "{id}");
        assert!(c.b.said().is_empty(), "{id}: the latched set never changed");
        let relabel = c.b.rig.act(FaceAction::Relabel);
        assert_eq!(relabel, Ok(FaceReply::default()), "{id}: nothing to redraw");
        assert!(c.on("ctrl") && c.on("shift"), "{id}: both stay latched");
    }
}

#[test]
fn a_one_shot_shift_latched_before_an_edit_still_shifts_the_next_letter() {
    let c = Chips::quiet();
    c.b.rig.tap("shift").unwrap();
    c.ask(1, Edit::Copy, false);
    c.b.rig.tap(A).unwrap();
    assert_eq!(c.b.rig.fakes.injector.strokes(), [unicode("A")]);
    assert!(!c.on("shift"), "the letter spent it");
}

#[test]
fn in_stay_mode_an_edit_keeps_ctrl_on_and_leaves_a_lone_win_unused() {
    let b = stay();
    b.rig.tap("ctrl").unwrap();
    b.rig.tap("win").unwrap();
    b.said();
    b.send(&ask(1, Edit::Copy, false));
    assert_eq!(b.rig.fakes.shortcuts.sent(), [alone(COPY_ID)]);
    assert!(b.said().is_empty(), "nothing else was told");
    assert_eq!(b.key("ctrl")["on"], true);
    b.rig.tap("win").unwrap();
    let start = [Stroke::Tap(Scan::new(WIN))];
    assert_eq!(b.rig.fakes.injector.strokes(), start, "Start opens");
}

#[test]
fn in_a_terminal_copy_and_paste_send_its_own_keys_and_cut_sends_nothing() {
    let c = Chips::quiet();
    c.b.rig.tap("ctrl").unwrap();
    c.ask(1, Edit::Copy, true);
    c.ask(2, Edit::Paste, true);
    c.ask(3, Edit::Cut, true);
    assert_eq!(c.sent(), [alone(COPY_TERM_ID), alone(PASTE_TERM_ID)]);
    assert_eq!(c.done(), [done(1, true), done(2, true), done(3, false)]);
    assert_eq!(
        c.pieces(),
        [Piece::Break, Piece::Break],
        "the Cut told nothing"
    );
    assert!(c.on("ctrl"), "Ctrl stays latched");
}

#[test]
fn prediction_hears_one_break_for_each_edit_that_reached_the_app() {
    let c = Chips::quiet();
    c.ask(1, Edit::Paste, false);
    assert_eq!(c.pieces(), [Piece::Break]);
    let shortcuts = &c.b.rig.fakes.shortcuts;
    shortcuts.set_error(Some(ShortcutError::Refused { sent: 0, total: 4 }));
    c.ask(2, Edit::Copy, false);
    assert!(c.pieces().is_empty(), "refused whole");
    shortcuts.set_error(Some(ShortcutError::Refused { sent: 2, total: 4 }));
    c.ask(3, Edit::Cut, false);
    assert_eq!(c.pieces(), [Piece::Break], "taken in part");
}

#[test]
fn a_refused_or_failed_edit_answers_not_ok_with_its_round() {
    let c = Chips::quiet();
    let errors = [
        ShortcutError::Refused { sent: 2, total: 4 },
        ShortcutError::Refused { sent: 0, total: 4 },
        ShortcutError::Failed,
        ShortcutError::Unknown,
    ];
    for (round, error) in (3..).zip(errors) {
        c.b.rig.fakes.shortcuts.set_error(Some(error));
        c.ask(round, Edit::Copy, false);
        assert_eq!(c.done(), [done(round, false)], "{error:?}");
    }
}

#[test]
fn an_edit_ask_heard_after_the_stop_sends_nothing_and_answers_not_ok() {
    let mut c = Chips::quiet();
    c.b.rig.tap("ctrl").unwrap();
    // The kernel drops the handlers before the stop, but one may still be running on the sender's thread.
    c.b.rig.module.stop();
    c.ask(5, Edit::Paste, false);
    assert!(c.sent().is_empty(), "no paste");
    assert_eq!(c.done(), [done(5, false)]);
    assert!(c.pieces().is_empty());
    assert!(c.on("ctrl"), "the latches stay as they were");
}

#[test]
fn while_a_fill_waits_its_gap_an_edit_sends_nothing_and_answers_not_ok() {
    let c = Chips::quiet();
    c.b.send(&FillType {
        round: 1,
        parts: login(),
        tab_gap: GAP,
    });
    c.ask(6, Edit::Paste, false);
    assert!(
        c.sent().is_empty(),
        "nothing lands between the fill's parts"
    );
    assert_eq!(c.done(), [done(6, false)]);
}

#[test]
fn an_edit_ask_ends_a_held_key_as_its_let_go_would() {
    let c = Chips::quiet();
    c.b.rig.tap("shift").unwrap();
    assert_eq!(c.b.rig.press(A), Ok(HELD));
    c.b.said();
    c.ask(2, Edit::Copy, false);
    assert_eq!(c.sent(), [alone(COPY_ID)]);
    c.b.rig.fakes.injector.clear();
    assert_eq!(c.b.rig.tick_in(1_000), FaceReply::default(), "no beat");
    let strokes = c.b.rig.fakes.injector.strokes();
    assert!(strokes.is_empty(), "the hold ended with the edit");
    let spent = [Said::Latches(Vec::new())];
    assert_eq!(
        c.b.said(),
        spent,
        "the A it typed spent Shift, as at a let-go"
    );
    assert!(!c.on("shift"));
}

#[test]
fn an_edit_takes_the_aim_from_an_open_panel_as_its_keys_went_to_the_app() {
    let c = Chips::quiet();
    c.b.send(&ClipPanel { open: true });
    assert_eq!(c.b.rig.state()["aimed"], true);
    c.ask(1, Edit::Paste, false);
    assert_eq!(c.b.rig.act(FaceAction::Relabel), Ok(STATE));
    assert_eq!(c.b.rig.state()["aimed"], false);
}

#[test]
fn an_edit_ends_a_dead_keys_wait_as_any_shortcut_does() {
    let d = Desk::new(None);
    d.rig.switch("fr-fr");
    d.rig.tap(CIRCUMFLEX).unwrap();
    assert_eq!(d.modes().last().map(|m| m.dead), Some(true));
    d.bus.publish(&ask(1, Edit::Copy, false)).unwrap();
    let wait = d.modes().last().map(|m| m.dead);
    assert_eq!(wait, Some(false), "the copy ended it");
}

#[test]
fn an_edit_during_a_picked_words_pause_goes_out_and_stops_the_rest_of_the_word() {
    let d = Desk::new(None);
    let bus = d.bus.clone();
    let mut once = Some(ask(1, Edit::Paste, false));
    d.rig.fakes.pause.during(move || {
        if let Some(ask) = once.take() {
            let bus = bus.clone();
            let told = thread::spawn(move || bus.publish(&ask).is_ok());
            assert!(told.join().unwrap(), "the edit was asked");
        }
    });
    d.word(3, "hello ");
    let strokes = d.rig.fakes.injector.strokes();
    assert_eq!(
        strokes,
        [tap(BKSP)],
        "no Backspace lands on the pasted text"
    );
    assert_eq!(d.rig.fakes.shortcuts.sent(), [alone(PASTE_ID)]);
    assert_eq!(
        d.pieces(),
        [Piece::Erase, Piece::Break],
        "what went is told"
    );
}

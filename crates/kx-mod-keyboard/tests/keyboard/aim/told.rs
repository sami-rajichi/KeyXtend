//! What prediction hears of the line (ADR-0040 decisions 6 and 7): the open panel's line while aimed, else none, once
//! per change, in order with the pieces, and again when it asks. Panel keys still tell it nothing.

use super::forms::search;
use super::{ELSEWHERE, LETTER, Quick};
use crate::assist::ear;
use crate::clip::Clip;
use kx_module_api::ports::{
    AimedLine, ClipLine, ClipPanel, FaceAction, PanelLine, Piece, Typed, TypingModeAsk,
};
use kx_module_api::{Bus, Module, Subscription};
use std::sync::{Arc, Mutex, PoisonError};

/// One thing prediction heard.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Said {
    /// A typed piece.
    Piece(Piece),
    /// The line it follows.
    Line(PanelLine),
}

/// What prediction hears, in order.
#[derive(Clone, Default)]
pub(super) struct Ears(Arc<Mutex<Vec<Said>>>);

impl Ears {
    /// Hears both events on `bus`.
    pub(super) fn on(&self, bus: &Bus) -> Vec<Subscription> {
        vec![
            ear(bus, &self.0, |e: &Typed| Said::Piece(e.piece.clone())),
            ear(bus, &self.0, |e: &AimedLine| Said::Line(e.line.clone())),
        ]
    }

    /// What it heard since the last look.
    pub(super) fn take(&self) -> Vec<Said> {
        std::mem::take(&mut *self.0.lock().unwrap_or_else(PoisonError::into_inner))
    }

    /// The lines it heard since the last look, with no piece among them.
    fn lines(&self) -> Vec<PanelLine> {
        let told = self.take().into_iter().map(|said| match said {
            Said::Line(line) => line,
            Said::Piece(piece) => panic!("a piece went out: {piece:?}"),
        });
        told.collect()
    }
}

/// The keyboard with the vault module running, and what prediction hears from here on.
pub(super) fn quiet() -> (Quick, Ears) {
    let ears = Ears::default();
    let on = ears.clone();
    let q = Quick::after(None, move |bus| on.on(bus));
    q.live(true);
    q.heard();
    ears.take();
    (q, ears)
}

/// The same with Quick-fill open on an empty search.
pub(super) fn searching() -> (Quick, Ears) {
    let (q, ears) = quiet();
    q.open(true);
    q.line(search(""));
    q.heard();
    ears.take();
    (q, ears)
}

#[test]
fn a_panel_opening_tells_its_search_and_each_new_text_follows() {
    let (q, ears) = quiet();
    q.open(true);
    assert!(ears.take().is_empty(), "no line told yet");
    q.line(search(""));
    assert_eq!(ears.lines(), [search("")]);
    q.line(search("a"));
    q.line(search("ab"));
    assert_eq!(ears.lines(), [search("a"), search("ab")]);
    q.line(search("ab"));
    assert!(ears.take().is_empty(), "nothing changed");
}

#[test]
fn an_outside_click_tells_no_line_at_the_next_relabel_and_a_click_on_the_line_tells_it_again() {
    let (q, ears) = searching();
    q.line(search("a"));
    ears.take();
    q.away();
    assert!(ears.take().is_empty(), "the press takes no lock");
    q.b.rig.act(FaceAction::Relabel).unwrap();
    assert_eq!(ears.lines(), [PanelLine::None]);
    q.aim();
    assert_eq!(ears.lines(), [search("a")]);
}

#[test]
fn a_state_read_tells_a_lost_aim_too() {
    let (q, ears) = searching();
    q.away();
    q.b.rig.state();
    assert_eq!(ears.lines(), [PanelLine::None]);
}

#[test]
fn another_window_in_front_tells_no_line() {
    let (q, ears) = searching();
    q.window(Some(ELSEWHERE));
    q.b.rig.act(FaceAction::Relabel).unwrap();
    assert_eq!(ears.lines(), [PanelLine::None]);
}

#[test]
fn a_search_changing_unaimed_tells_nothing_until_it_aims() {
    let (q, ears) = searching();
    q.away();
    q.b.rig.act(FaceAction::Relabel).unwrap();
    ears.take();
    q.line(search("x"));
    assert!(ears.take().is_empty());
    q.aim();
    assert_eq!(ears.lines(), [search("x")]);
}

#[test]
fn a_private_line_tells_only_its_kind_and_leaving_it_tells_the_next() {
    let (q, ears) = searching();
    q.line(PanelLine::Private);
    assert_eq!(ears.lines(), [PanelLine::Private]);
    q.line(PanelLine::None);
    assert_eq!(ears.lines(), [PanelLine::None]);
}

#[test]
fn a_close_tells_no_line_and_the_other_panel_tells_its_own() {
    let (q, ears) = searching();
    q.line(search("a"));
    ears.take();
    q.b.send(&ClipPanel { open: true });
    assert_eq!(ears.lines(), [PanelLine::None], "not the old panel's text");
    q.line(search("late"));
    assert!(ears.take().is_empty(), "the panel not open says nothing");
    q.b.send(&ClipLine(search("b")));
    assert_eq!(ears.lines(), [search("b")]);
    q.b.send(&ClipPanel { open: false });
    assert_eq!(ears.lines(), [PanelLine::None]);
}

#[test]
fn panel_keys_never_tell_a_piece_on_a_search_line_or_a_private_one() {
    for line in [search("a"), PanelLine::Private] {
        let (q, ears) = searching();
        q.line(line);
        ears.take();
        for id in [LETTER, "space", "bksp", "enter", "up", "esc"] {
            q.b.rig.tap(id).unwrap();
        }
        assert!(ears.take().is_empty(), "no Typed, no line change");
        assert!(q.strokes().is_empty());
        assert_eq!(q.heard().len(), 6);
    }
}

#[test]
fn a_key_the_panel_leaves_to_the_app_is_told_as_usual() {
    let (q, ears) = searching();
    q.b.rig.tap("ctrl").unwrap();
    q.b.rig.tap(LETTER).unwrap();
    assert!(!q.strokes().is_empty(), "Ctrl acts on a letter in the app");
    assert_eq!(ears.take(), [Said::Piece(Piece::Break)]);
}

#[test]
fn a_piece_typed_before_the_aim_goes_first() {
    let (q, ears) = searching();
    q.away();
    q.b.rig.act(FaceAction::Relabel).unwrap();
    ears.take();
    q.b.rig.tap("bksp").unwrap();
    q.aim();
    let want = [Said::Piece(Piece::Erase), Said::Line(search(""))];
    assert_eq!(ears.take(), want);
}

#[test]
fn a_lost_aim_goes_before_the_key_that_found_it_lost() {
    let (q, ears) = searching();
    q.away();
    q.b.rig.tap("bksp").unwrap();
    let want = [Said::Line(PanelLine::None), Said::Piece(Piece::Erase)];
    assert_eq!(ears.take(), want);
}

#[test]
fn an_ask_tells_the_line_again_so_a_late_module_hears_it() {
    let (q, ears) = searching();
    q.b.send(&TypingModeAsk);
    assert_eq!(ears.lines(), [search("")]);
    q.away();
    q.b.rig.act(FaceAction::Relabel).unwrap();
    ears.take();
    q.b.send(&TypingModeAsk);
    assert_eq!(ears.lines(), [PanelLine::None], "no line, said again");
}

#[test]
fn the_vault_stopping_tells_no_line() {
    let (q, ears) = searching();
    q.live(false);
    assert_eq!(ears.lines(), [PanelLine::None]);
}

#[test]
fn a_stop_tells_no_line_and_a_panel_after_the_restart_tells_its_own() {
    let (mut q, ears) = searching();
    q.b.rig.module.stop();
    assert_eq!(ears.lines(), [PanelLine::None]);
    q.b.rig.restart(None);
    assert!(ears.take().is_empty(), "the panel counts as shut");
    q.live(true);
    q.open(true);
    q.line(search("a"));
    assert_eq!(ears.lines(), [search("a")]);
}

#[test]
fn the_clipboards_search_is_told_the_same_way() {
    let ears = Ears::default();
    let on = ears.clone();
    let c = Clip::after(None, move |bus| on.on(bus));
    c.heard();
    ears.take();
    c.open(true);
    c.b.send(&ClipLine(search("a")));
    assert_eq!(ears.lines(), [search("a")]);
    c.b.rig.tap(LETTER).unwrap();
    assert!(ears.take().is_empty(), "its key tells nothing");
}

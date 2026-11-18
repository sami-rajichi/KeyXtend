//! The Voice panel's lines from the keyboard's side (ADR-0049): one panel shows at a time, the keys go to the voice
//! module only while its page shows a line and that line is aimed at, Esc always reaches the app, and prediction hears
//! a private line, so its lane stays empty.

use crate::assist::{Bench, ear};
use kx_module_api::Redacted;
use kx_module_api::ports::{
    AimedLine, ClipAct, ClipPanel, FaceAction, FillClose, FillPanel, PanelKey, PanelLine, Piece,
    PointerPressed, Scan, TextRoute, Typed, VoiceClose, VoiceLine, VoiceLineKey,
};
use kx_platform_fake::Stroke;
use std::sync::{Arc, Mutex, PoisonError};

/// The board's Q key.
const LETTER: &str = "c-0x0010";
/// The board's Esc key, as it reaches the app.
const ESC: u16 = 0x01;
/// The window in front while the tests aim.
const WINDOW: u64 = 1;
/// Another window.
const ELSEWHERE: u64 = 2;

/// What the other modules heard from the keyboard, in order.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Said {
    /// A key for the Voice panel's line.
    Key(PanelKey),
    /// The Voice panel was asked to close.
    Close,
    /// Quick-fill was asked to close.
    FillClose,
    /// The clipboard panel was asked to close.
    ClipClose,
    /// The line prediction follows.
    Line(PanelLine),
    /// A piece told to prediction.
    Piece(Piece),
}

/// The keyboard on a bus the test listens to.
struct Lines {
    b: Bench,
    said: Arc<Mutex<Vec<Said>>>,
}

impl Lines {
    /// The keyboard started, with what it said at start already taken.
    fn new() -> Self {
        let said = Arc::default();
        let s = Arc::clone(&said);
        let b = Bench::after(None, move |bus| {
            vec![
                ear(bus, &s, |e: &VoiceLineKey| Said::Key(e.0.clone())),
                ear(bus, &s, |_: &VoiceClose| Said::Close),
                ear(bus, &s, |_: &FillClose| Said::FillClose),
                ear(bus, &s, |_: &ClipAct| Said::ClipClose),
                ear(bus, &s, |e: &AimedLine| Said::Line(e.line.clone())),
                ear(bus, &s, |e: &Typed| Said::Piece(e.piece.clone())),
            ]
        });
        b.rig.fakes.injector.set_window(Some(WINDOW));
        let l = Self { b, said };
        l.said();
        l
    }

    /// The keyboard with the Voice panel open on a key page, with what that said already taken.
    fn on_key_page() -> Self {
        let l = Self::new();
        l.line(Some(PanelLine::Private));
        l.said();
        l.strokes();
        l
    }

    /// The voice module says which line its panel shows.
    fn line(&self, line: Option<PanelLine>) {
        self.b.send(&VoiceLine(line));
    }

    /// What the keyboard said since the last look.
    fn said(&self) -> Vec<Said> {
        std::mem::take(&mut *self.said.lock().unwrap_or_else(PoisonError::into_inner))
    }

    /// What reached the app since the last look.
    fn strokes(&self) -> Vec<Stroke> {
        let injector = &self.b.rig.fakes.injector;
        let strokes = injector.strokes();
        injector.clear();
        strokes
    }

    /// Clicks the letter: true when the voice line took it, false when the app did.
    fn letter_to_line(&self) -> bool {
        self.b.rig.tap(LETTER).unwrap();
        let keys: Vec<_> = (self.said().into_iter())
            .filter(|s| matches!(s, Said::Key(_) | Said::Piece(_)))
            .collect();
        match (keys.as_slice(), self.strokes().as_slice()) {
            ([Said::Key(key)], []) if *key == PanelKey::Text(Redacted::new("q".into())) => true,
            ([Said::Piece(_)], [stroke]) if *stroke == q() => false,
            other => panic!("the letter went nowhere or twice: {other:?}"),
        }
    }
}

/// The letter as it reaches the app.
fn q() -> Stroke {
    Stroke::Text(Redacted::new("q".into()), TextRoute::Unicode)
}

#[test]
fn the_voice_panel_opening_closes_the_other_two_and_either_of_them_closes_it() {
    let l = Lines::new();
    l.line(Some(PanelLine::None));
    assert_eq!(l.said(), [Said::FillClose, Said::ClipClose]);
    l.b.send(&ClipPanel { open: true });
    assert_eq!(l.said(), [Said::FillClose, Said::Close]);
    l.b.send(&FillPanel { open: true });
    assert_eq!(l.said(), [Said::ClipClose, Said::Close]);
}

#[test]
fn letters_go_to_the_key_line_and_prediction_hears_a_private_line() {
    let l = Lines::new();
    l.line(Some(PanelLine::Private));
    assert!(l.said().contains(&Said::Line(PanelLine::Private)));
    assert!(l.letter_to_line());
    l.b.rig.tap("enter").unwrap();
    assert_eq!(l.said(), [Said::Key(PanelKey::Enter)]);
    assert!(l.strokes().is_empty());
}

#[test]
fn with_no_line_on_its_page_every_key_reaches_the_app() {
    let l = Lines::on_key_page();
    l.line(Some(PanelLine::None));
    assert!(l.said().contains(&Said::Line(PanelLine::None)));
    assert!(!l.letter_to_line());
    l.b.rig.tap("enter").unwrap();
    assert_eq!(l.strokes().len(), 1);
}

#[test]
fn esc_reaches_the_app_while_the_line_is_aimed() {
    let l = Lines::on_key_page();
    l.b.rig.tap("esc").unwrap();
    assert_eq!(l.strokes(), [Stroke::Tap(Scan::new(ESC))]);
    assert!(!l.said().iter().any(|s| matches!(s, Said::Key(_))));
    assert!(l.letter_to_line(), "the line keeps its aim");
}

#[test]
fn an_outside_click_takes_the_aim_and_a_press_on_the_panel_brings_it_back() {
    let l = Lines::on_key_page();
    l.b.send(&PointerPressed { ours: false });
    assert!(!l.letter_to_line());
    l.b.rig.act(FaceAction::AimPanel).unwrap();
    assert!(l.letter_to_line());
}

#[test]
fn a_window_taking_the_front_never_takes_the_keys_letters() {
    let l = Lines::on_key_page();
    l.b.rig.fakes.injector.set_window(Some(ELSEWHERE));
    assert!(l.letter_to_line(), "a key's letters stay in its line");
}

#[test]
fn the_panel_closing_or_a_restart_gives_the_keys_back() {
    let mut l = Lines::on_key_page();
    l.line(None);
    assert!(!l.letter_to_line());
    l.line(Some(PanelLine::Private));
    assert!(l.letter_to_line());
    l.b.rig.restart(None);
    l.said();
    assert!(!l.letter_to_line(), "unheard since, so shut");
}

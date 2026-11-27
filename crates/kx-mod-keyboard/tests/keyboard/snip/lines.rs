//! Snip's windows from the keyboard's side (ADR-0040, ADR-0057): their keys go to the snip module as `SnipLineKey`
//! only while its `SnipLine` shows and is aimed at. Only the user's press on the Snip key or on the windows aims, so a
//! line that shows on its own takes no key the user aims elsewhere, and an Esc during a recording reaches the app.

use super::SNIP;
use crate::assist::{Bench, ear};
use crate::rig::STATE;
use kx_module_api::Redacted;
use kx_module_api::ports::{
    AimedLine, ClipAct, ClipKey, ClipPanel, FaceAction, FaceReply, FillClose, PanelKey, PanelLine,
    PointerPressed, Scan, SnipLine, SnipLineKey, SnipLit, TextRoute, VoiceClose,
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
pub(super) enum Said {
    /// A key for Snip's windows.
    Key(PanelKey),
    /// A key for the clipboard panel.
    Clip(PanelKey),
    /// Quick-fill was asked to close.
    FillClose,
    /// The clipboard panel was asked to close.
    ClipClose,
    /// The Voice panel was asked to close.
    VoiceClose,
    /// The line prediction follows.
    Line(PanelLine),
}

/// Every close the other panels heard.
const CLOSES: [Said; 3] = [Said::FillClose, Said::ClipClose, Said::VoiceClose];

/// The keyboard on a bus the test listens to.
pub(super) struct Lines {
    pub b: Bench,
    said: Arc<Mutex<Vec<Said>>>,
}

impl Lines {
    /// The keyboard started, with what it said at start already taken.
    pub fn new() -> Self {
        let said = Arc::default();
        let s = Arc::clone(&said);
        let b = Bench::after(None, move |bus| {
            vec![
                ear(bus, &s, |e: &SnipLineKey| Said::Key(e.0.clone())),
                ear(bus, &s, |e: &ClipKey| Said::Clip(e.0.clone())),
                ear(bus, &s, |_: &FillClose| Said::FillClose),
                ear(bus, &s, |_: &ClipAct| Said::ClipClose),
                ear(bus, &s, |_: &VoiceClose| Said::VoiceClose),
                ear(bus, &s, |e: &AimedLine| Said::Line(e.line.clone())),
            ]
        });
        b.rig.fakes.injector.set_window(Some(WINDOW));
        let l = Self { b, said };
        l.said();
        l
    }

    /// The keyboard with the clipboard panel open and aimed at, with what that said already taken.
    pub fn with_clip() -> Self {
        let l = Self::new();
        l.b.send(&ClipPanel { open: true });
        l.said();
        l
    }

    /// The snip module says which line its windows show.
    pub fn line(&self, line: Option<PanelLine>) {
        self.b.send(&SnipLine(line));
    }

    /// A press on Snip's windows.
    pub fn press(&self) -> FaceReply {
        self.b.rig.act(FaceAction::AimSnip).unwrap()
    }

    /// What the keyboard said since the last look.
    pub fn said(&self) -> Vec<Said> {
        std::mem::take(&mut *self.said.lock().unwrap_or_else(PoisonError::into_inner))
    }

    /// The closes the other panels heard since the last look.
    pub fn closes(&self) -> Vec<Said> {
        let said = self.said().into_iter();
        said.filter(|s| CLOSES.contains(s)).collect()
    }

    /// Clicks key `id`: the keys Snip's windows took, and what reached the app.
    fn tap(&self, id: &str) -> (Vec<PanelKey>, Vec<Stroke>) {
        self.b.rig.tap(id).unwrap();
        let keys = self.said().into_iter().filter_map(|s| match s {
            Said::Key(key) => Some(key),
            _ => None,
        });
        let injector = &self.b.rig.fakes.injector;
        let strokes = injector.strokes();
        injector.clear();
        (keys.collect(), strokes)
    }

    /// Clicks Esc: true when Snip's windows took it, false when the app did.
    pub fn esc_to_snip(&self) -> bool {
        match self.tap("esc") {
            (keys, strokes) if keys == [PanelKey::Esc] && strokes.is_empty() => true,
            (keys, strokes) if keys.is_empty() && strokes == [Stroke::Tap(Scan::new(ESC))] => false,
            other => panic!("Esc went nowhere or twice: {other:?}"),
        }
    }

    /// Clicks the letter: true when Snip's windows took it, false when the app did.
    fn letter_to_snip(&self) -> bool {
        match self.tap(LETTER) {
            (keys, strokes) if keys == [text("q")] && strokes.is_empty() => true,
            (keys, strokes) if keys.is_empty() && strokes == [typed("q")] => false,
            other => panic!("the letter went nowhere or twice: {other:?}"),
        }
    }
}

/// `t` as a key for a line.
fn text(t: &str) -> PanelKey {
    PanelKey::Text(Redacted::new(t.into()))
}

/// `t` as it reaches the app.
fn typed(t: &str) -> Stroke {
    Stroke::Text(Redacted::new(t.into()), TextRoute::Unicode)
}

#[test]
fn the_snip_key_aims_at_the_capture_so_esc_and_enter_go_to_it() {
    let l = Lines::new();
    l.b.rig.tap(SNIP).unwrap();
    l.line(Some(PanelLine::None));
    assert!(l.esc_to_snip());
    assert_eq!(l.tap("enter"), (vec![PanelKey::Enter], vec![]));
}

#[test]
fn the_editors_text_takes_the_letters_and_arabic_as_the_layout_gives_it() {
    let l = Lines::new();
    l.line(Some(PanelLine::None));
    l.press();
    l.line(Some(PanelLine::Private));
    let said = l.said();
    assert!(said.contains(&Said::Line(PanelLine::Private)), "{said:?}");
    assert!(l.letter_to_snip());
    l.b.rig.switch("ar-tn");
    assert_eq!(l.tap(LETTER), (vec![text("ض")], vec![]));
    l.b.rig.tap("altgr").unwrap();
    let marks = text("\u{651}\u{64E}");
    assert_eq!(l.tap(LETTER), (vec![marks], vec![]), "shadda, then fatha");
    assert_eq!(l.tap("space"), (vec![PanelKey::Tick], vec![]));
    assert_eq!(l.tap("bksp"), (vec![PanelKey::Erase], vec![]));
    assert_eq!(l.tap("enter"), (vec![PanelKey::Enter], vec![]));
}

#[test]
fn no_key_goes_to_snip_without_a_line_so_esc_reaches_the_recorded_app() {
    let l = Lines::new();
    l.b.rig.tap(SNIP).unwrap();
    l.line(Some(PanelLine::None));
    assert!(l.esc_to_snip(), "while picking");
    l.line(None);
    assert!(!l.esc_to_snip(), "the recorded app's Esc");
    assert!(!l.letter_to_snip());
}

#[test]
fn a_line_that_shows_on_its_own_takes_keys_only_after_a_press_on_it() {
    let l = Lines::new();
    l.line(Some(PanelLine::None));
    assert!(!l.esc_to_snip(), "a strip shown by a recording's end");
    assert!(!l.letter_to_snip());
    let other = l.b.rig.act(FaceAction::AimPanel);
    assert_eq!(other, Ok(FaceReply::default()), "no panel is open");
    assert!(!l.esc_to_snip());
    assert_eq!(l.press(), STATE);
    assert!(l.esc_to_snip());
}

#[test]
fn a_text_that_shows_on_its_own_never_takes_the_letters() {
    let l = Lines::new();
    l.line(Some(PanelLine::Private));
    assert!(!l.letter_to_snip(), "only a press aims");
    l.press();
    assert!(l.letter_to_snip());
}

#[test]
fn the_aim_holds_after_a_text_ends_with_enter_whichever_came_first() {
    for press_first in [true, false] {
        let l = Lines::new();
        l.line(Some(PanelLine::None));
        if press_first {
            l.press();
        }
        l.line(Some(PanelLine::Private));
        if !press_first {
            l.press();
        }
        assert!(l.letter_to_snip());
        assert_eq!(l.tap("enter"), (vec![PanelKey::Enter], vec![]));
        l.line(Some(PanelLine::None));
        assert!(l.esc_to_snip(), "the editor keeps the keys");
    }
}

#[test]
fn a_press_with_no_line_aims_nothing() {
    let l = Lines::new();
    assert_eq!(l.press(), FaceReply::default());
    l.line(Some(PanelLine::None));
    assert!(!l.esc_to_snip(), "the press was not on this line");
}

#[test]
fn a_line_after_the_snip_key_takes_the_keys_unless_an_outside_click_came_between() {
    let l = Lines::new();
    l.b.rig.tap(SNIP).unwrap();
    l.line(Some(PanelLine::None));
    assert!(l.esc_to_snip());
    l.line(None);
    l.b.rig.tap(SNIP).unwrap();
    l.b.send(&PointerPressed { ours: false });
    l.line(Some(PanelLine::None));
    assert!(!l.esc_to_snip(), "the click went elsewhere since");
}

#[test]
fn a_cancelled_countdown_or_a_refused_freeze_leaves_no_click_waiting() {
    let l = Lines::new();
    l.b.send(&SnipLit::On);
    l.b.rig.tap(SNIP).unwrap();
    l.b.send(&SnipLit::Off);
    l.line(Some(PanelLine::None));
    assert!(!l.esc_to_snip(), "the panel the tray opened later");
    l.line(None);
    l.b.rig.tap(SNIP).unwrap();
    l.b.send(&SnipLit::On);
    l.b.send(&SnipLit::Off);
    l.line(Some(PanelLine::None));
    assert!(!l.esc_to_snip(), "after a freeze that was refused");
    assert_eq!(l.closes(), []);
}

#[test]
fn an_outside_click_takes_the_aim_and_a_press_brings_it_back() {
    let l = Lines::new();
    l.line(Some(PanelLine::Private));
    l.press();
    assert!(l.letter_to_snip());
    l.b.send(&PointerPressed { ours: false });
    assert!(!l.letter_to_snip());
    l.press();
    assert!(l.letter_to_snip());
}

#[test]
fn a_window_taking_the_front_never_takes_the_texts_letters() {
    let l = Lines::new();
    l.line(Some(PanelLine::Private));
    l.press();
    l.b.rig.fakes.injector.set_window(Some(ELSEWHERE));
    assert!(l.letter_to_snip(), "a text's letters stay in it");
}

#[test]
fn the_line_ending_or_a_restart_gives_the_keys_back() {
    let mut l = Lines::new();
    l.line(Some(PanelLine::Private));
    l.press();
    l.line(None);
    assert!(!l.letter_to_snip());
    l.line(Some(PanelLine::Private));
    assert!(!l.letter_to_snip(), "a new line waits for a press");
    l.press();
    assert!(l.letter_to_snip());
    l.b.rig.restart(None);
    l.said();
    assert!(!l.letter_to_snip(), "unheard since, so shut");
}

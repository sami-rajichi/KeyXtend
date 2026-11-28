//! Snip's windows beside another of our panels (ADR-0057): both may show, neither closes the other, and the keys go to
//! the last one pressed, which the state names, so only its line draws a caret.

use super::SNIP;
use super::lines::{Lines, Said};
use kx_module_api::Redacted;
use kx_module_api::ports::{ClipLine, ClipPanel, FaceAction, PanelKey, PanelLine, PointerPressed};
use serde_json::{Value, json};

impl Lines {
    /// The panel the state names as taking the keys, and whether our open panel's line draws its caret.
    fn aim(&self) -> Value {
        let state = self.b.rig.state();
        json!([state["aim"], state["aimed"]])
    }

    /// Clicks Esc: true when the clipboard panel alone took it; the lines prediction heard meanwhile are passed over.
    fn esc_to_clip(&self) -> bool {
        self.b.rig.tap("esc").unwrap();
        let injector = &self.b.rig.fakes.injector;
        let to_app = !injector.strokes().is_empty();
        injector.clear();
        let said = self.said().into_iter();
        let heard: Vec<Said> = said.filter(|s| !matches!(s, Said::Line(_))).collect();
        !to_app && heard == [Said::Clip(PanelKey::Esc)]
    }

    /// A press on the open panel.
    fn press_panel(&self) {
        self.b.rig.act(FaceAction::AimPanel).unwrap();
    }
}

#[test]
fn a_line_that_shows_on_its_own_never_takes_an_open_panels_keys_or_closes_it() {
    let l = Lines::with_clip();
    l.line(Some(PanelLine::None));
    assert_eq!(l.closes(), []);
    assert!(l.esc_to_clip());
    l.b.send(&ClipPanel { open: false });
    assert!(!l.esc_to_snip(), "it takes the free place unaimed");
    l.press();
    assert!(l.esc_to_snip());
}

#[test]
fn a_press_on_snip_takes_the_keys_and_the_open_panel_stays() {
    let l = Lines::with_clip();
    l.line(Some(PanelLine::None));
    l.press();
    assert_eq!(l.closes(), []);
    assert!(l.esc_to_snip());
    assert_eq!(
        l.b.key("clip")["on"],
        true,
        "the clipboard panel still shows"
    );
    l.press_panel();
    assert!(l.esc_to_clip(), "a press on the panel takes them back");
    l.press();
    assert!(l.esc_to_snip());
}

#[test]
fn the_state_names_the_panel_that_takes_the_keys() {
    let l = Lines::new();
    assert_eq!(l.aim(), json!([null, false]));
    l.b.send(&ClipPanel { open: true });
    assert_eq!(l.aim(), json!(["clip", true]));
    l.line(Some(PanelLine::None));
    assert_eq!(
        l.aim(),
        json!(["clip", true]),
        "a line that shows on its own"
    );
    l.press();
    assert_eq!(
        l.aim(),
        json!(["snip", false]),
        "the clipboard's line draws no caret"
    );
    l.b.send(&PointerPressed { ours: false });
    assert_eq!(l.aim(), json!([null, false]));
    l.press_panel();
    assert_eq!(l.aim(), json!(["clip", true]));
}

#[test]
fn the_snip_key_closes_no_panel_whether_it_starts_or_ends_a_capture() {
    let l = Lines::with_clip();
    l.b.rig.tap(SNIP).unwrap();
    l.line(Some(PanelLine::None));
    assert_eq!(l.closes(), [], "a capture starts");
    assert!(l.esc_to_snip(), "the click aimed at it");
    l.b.rig.tap(SNIP).unwrap();
    l.line(None);
    assert_eq!(l.closes(), [], "the capture ends");
    l.line(Some(PanelLine::None));
    l.press_panel();
    l.b.rig.tap(SNIP).unwrap();
    assert_eq!(l.closes(), [], "the strip that showed on its own ends");
}

#[test]
fn a_panel_opening_takes_the_keys_from_snip_and_never_closes_it() {
    let l = Lines::new();
    l.line(Some(PanelLine::None));
    l.press();
    l.b.send(&ClipPanel { open: true });
    assert_eq!(
        l.closes(),
        [Said::FillClose, Said::VoiceClose],
        "never Snip's"
    );
    assert!(l.esc_to_clip());
    l.b.send(&ClipPanel { open: false });
    assert!(!l.esc_to_snip(), "Snip takes the free place back, unaimed");
    l.press();
    assert!(l.esc_to_snip());
}

#[test]
fn snip_ending_while_it_holds_the_keys_gives_them_to_no_panel() {
    let l = Lines::with_clip();
    l.line(Some(PanelLine::None));
    l.press();
    l.line(None);
    assert!(!l.esc_to_snip(), "to the app, not to the clipboard panel");
    assert_eq!(l.b.key("clip")["on"], true);
    l.press_panel();
    assert!(l.esc_to_clip());
}

#[test]
fn a_form_that_ends_behind_snip_never_drops_the_panels_aim_later() {
    let l = Lines::with_clip();
    l.b.send(&PointerPressed { ours: false });
    l.b.send(&ClipLine(PanelLine::Private));
    l.line(Some(PanelLine::None));
    l.press();
    l.b.send(&ClipLine(PanelLine::None));
    l.press_panel();
    l.b.send(&ClipLine(PanelLine::Search(Redacted::new("q".into()))));
    assert!(l.esc_to_clip(), "the press on the panel holds");
}

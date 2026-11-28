//! The keyboard's keys reach a snip only through its line: the line says when a snip window is aimed at, Esc acts
//! only then (never while counting down or recording), and Esc closes the open panel first (ADR-0040). At the question
//! over frozen screens Esc means Don't save (ADR-0059); the editor's keeps Back (`editor.rs`).

use kx_module_api::Redacted;
use kx_module_api::ports::{AskKey, BarKey, PanelKey, PanelLine, SnipClick, SnipStage, StripKey};

use super::rig::{FAR, NEAR, Rig};

#[test]
fn the_line_shows_while_a_snip_window_takes_keys() {
    let mut rig = Rig::new();
    assert_eq!(rig.told_line(), None, "nothing at rest");
    rig.key();
    assert_eq!(rig.told_line(), Some(PanelLine::None), "the bar");
    rig.act(SnipClick::Spot(NEAR));
    rig.act(SnipClick::Spot(FAR));
    assert_eq!(rig.told_line(), Some(PanelLine::None), "the strip");
    rig.act(SnipClick::Strip(StripKey::Close));
    assert_eq!(rig.told_line(), Some(PanelLine::None), "the question");
    rig.act(SnipClick::Ask(AskKey::Drop));
    assert_eq!(rig.told_line(), None, "gone with the snip");
}

#[test]
fn no_line_shows_while_counting_down_or_recording_so_esc_never_ends_them() {
    let mut rig = Rig::new();
    rig.take(BarKey::VideoRegion);
    rig.act(SnipClick::Strip(StripKey::Record));
    assert!(matches!(rig.stage(), SnipStage::Count(_)));
    assert_eq!(rig.told_line(), None);
    rig.line(PanelKey::Esc);
    assert!(
        matches!(rig.stage(), SnipStage::Count(_)),
        "Esc went to the app"
    );
    let mut rig = Rig::new();
    rig.record();
    assert_eq!(rig.told_line(), None);
    rig.line(PanelKey::Esc);
    assert!(matches!(rig.stage(), SnipStage::Record(_)));
}

#[test]
fn esc_ends_a_snip_being_picked_and_a_second_esc_drops_at_the_question_over_frozen_screens() {
    let mut rig = Rig::new();
    rig.key();
    rig.line(PanelKey::Esc);
    assert_eq!(rig.stage(), SnipStage::Idle);
    rig.snap();
    rig.line(PanelKey::Esc);
    assert!(rig.page().ask.is_some(), "Esc is Close, which asks first");
    rig.line(PanelKey::Esc);
    assert_eq!(
        rig.stage(),
        SnipStage::Idle,
        "Don't save over frozen screens (ADR-0059)"
    );
}

#[test]
fn letters_with_no_text_being_typed_do_nothing() {
    let mut rig = Rig::new();
    rig.snap();
    let told = rig.views().len();
    rig.line(PanelKey::Text(Redacted::new("a".into())));
    assert_eq!(rig.views().len(), told);
}

#[test]
fn the_panel_shows_a_line_and_esc_closes_it_first() {
    let mut rig = Rig::new();
    rig.act(SnipClick::Open);
    assert_eq!(rig.told_line(), Some(PanelLine::None));
    rig.line(PanelKey::Esc);
    assert_eq!(rig.page().panel, None);
    assert_eq!(rig.told_line(), None);
    rig.key();
    rig.act(SnipClick::Open);
    rig.line(PanelKey::Esc);
    assert!(
        matches!(rig.stage(), SnipStage::Pick(_)),
        "the panel closed, the snip stays"
    );
    rig.line(PanelKey::Esc);
    assert_eq!(rig.stage(), SnipStage::Idle);
}

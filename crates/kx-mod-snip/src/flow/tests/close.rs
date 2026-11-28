//! The close question (edge 7): Close asks only when the snip was not kept since its last change; Don't save deletes;
//! Back returns, as Esc does with no frozen screen, while over frozen screens Esc means Don't save; Save saves first;
//! nothing else acts while it shows.

use super::*;

#[test]
fn edge_7_close_asks_when_nothing_was_kept() {
    for input in [In::Strip(StripKey::Close), In::Esc, In::Key] {
        let mut rig = Rig::new();
        rig.picture();
        assert_eq!(rig.on(input.clone()), [], "{input:?}");
        assert!(rig.flow.asking(), "{input:?}");
        assert!(matches!(rig.stage(), Stage::Strip(_)));
    }
}

#[test]
fn edge_7_back_returns_to_the_strip() {
    let mut rig = Rig::new();
    rig.picture();
    rig.on(In::Strip(StripKey::Close));
    assert_eq!(rig.on(In::Ask(AskKey::Back)), []);
    assert!(!rig.flow.asking());
    assert!(matches!(rig.stage(), Stage::Strip(_)));
}

/// ADR-0059: over frozen screens a stray Esc only asks and a second always ends the snip, keeping nothing.
#[test]
fn esc_at_the_question_means_dont_save_over_frozen_screens() {
    let mut rig = Rig::new();
    rig.picture();
    assert_eq!(rig.on(In::Esc), [], "the first asks");
    let outs = rig.on(In::Esc);
    assert!(ended(&outs) && !kept_any(&outs), "{outs:?}");
    assert_eq!(rig.stage(), &Stage::Idle);
}

/// A recorded video's strip and the editor show no frozen screen, so Esc at their question means Back.
#[test]
fn esc_at_the_question_means_back_with_no_frozen_screen() {
    let mut video = Rig::new();
    video.stopped();
    let mut editor = Rig::new();
    editor.editor();
    for mut rig in [video, editor] {
        rig.on(In::Esc);
        assert!(rig.flow.asking() && rig.flow.frozen().is_none());
        assert_eq!(rig.on(In::Esc), []);
        assert!(!rig.flow.asking());
        assert!(!matches!(rig.stage(), Stage::Idle), "{:?}", rig.stage());
    }
}

#[test]
fn edge_7_dont_save_ends_keeping_nothing() {
    let mut rig = Rig::new();
    rig.picture();
    rig.on(In::Strip(StripKey::Close));
    let outs = rig.on(In::Ask(AskKey::Drop));
    assert!(ended(&outs));
    assert!(!kept_any(&outs));
    assert!(!rig.flow.asking());
    assert_eq!(rig.stage(), &Stage::Idle);
}

#[test]
fn edge_7_dont_save_deletes_a_recorded_video() {
    let mut rig = Rig::new();
    let video = rig.stopped();
    rig.on(In::Strip(StripKey::Close));
    let outs = rig.on(In::Ask(AskKey::Drop));
    assert!(outs.contains(&Out::Delete(video)));
    assert!(ended(&outs));
}

#[test]
fn edge_7_the_questions_save_saves_then_ends() {
    let mut rig = Rig::new();
    rig.picture();
    rig.on(In::Strip(StripKey::Close));
    let save = Out::Keep {
        keep: Keep::Save,
        what: What::Picture(shot()),
    };
    assert_eq!(rig.on(In::Ask(AskKey::Save)), [save]);
    assert!(!rig.flow.asking());
    rig.on(In::Kept(Outcome::Done));
    assert!(ended(&rig.wait(rig.shown())));
}

#[test]
fn edge_7_a_cancelled_dialog_from_the_question_comes_back() {
    let mut rig = Rig::new();
    rig.picture();
    rig.on(In::Strip(StripKey::Close));
    rig.on(In::Ask(AskKey::Save));
    assert_eq!(rig.on(In::Kept(Outcome::Cancelled)), []);
    assert!(!rig.flow.asking());
    assert!(matches!(rig.stage(), Stage::Strip(_)));
}

#[test]
fn nothing_else_acts_while_the_question_shows() {
    let mut rig = Rig::new();
    rig.picture();
    rig.on(In::Strip(StripKey::Close));
    for input in [
        In::Strip(StripKey::Copy),
        In::Strip(StripKey::Edit),
        In::Strip(StripKey::Close),
        In::Key,
    ] {
        assert_eq!(rig.on(input.clone()), [], "{input:?}");
        assert!(rig.flow.asking());
    }
}

#[test]
fn a_ready_video_closes_without_asking() {
    let mut rig = Rig::new();
    rig.ready();
    assert!(ended(&rig.on(In::Strip(StripKey::Close))));
    assert!(!rig.flow.asking());
}

#[test]
fn a_recorded_video_asks() {
    let mut rig = Rig::new();
    rig.stopped();
    assert_eq!(rig.on(In::Strip(StripKey::Close)), []);
    assert!(rig.flow.asking());
}

#[test]
fn edge_7_the_editor_asks_until_kept() {
    let mut rig = Rig::new();
    rig.editor();
    rig.on(In::Edit(EditKey::Close));
    assert!(rig.flow.asking());
    rig.on(In::Ask(AskKey::Back));
    rig.flat_kept(Keep::Copy);
    assert_eq!(rig.on(In::Edit(EditKey::Close)), [Out::End]);
    assert!(!rig.flow.asking());
    assert_eq!(rig.stage(), &Stage::Idle);
}

#[test]
fn edge_7_a_change_after_a_copy_asks_again() {
    let mut rig = Rig::new();
    rig.editor();
    rig.flat_kept(Keep::Copy);
    rig.on(In::Changed);
    assert_eq!(rig.on(In::Edit(EditKey::Close)), []);
    assert!(rig.flow.asking());
}

#[test]
fn edge_7_a_change_during_a_save_leaves_it_unkept() {
    let mut rig = Rig::new();
    rig.editor();
    rig.on(In::Flat {
        keep: Keep::Save,
        shot: flat(),
    });
    rig.on(In::Changed);
    rig.on(In::Kept(Outcome::Done));
    rig.on(In::Edit(EditKey::Close));
    assert!(rig.flow.asking());
}

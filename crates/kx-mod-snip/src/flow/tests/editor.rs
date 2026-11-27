//! The picture editor: Copy and Save keep it open with a short tick, and the Snip key with unkept work asks first, then
//! starts (edge 22).

use super::*;

fn edit(done: Option<Keep>) -> Stage {
    Stage::Edit(Edit { done })
}

#[test]
fn a_flat_copy_or_save_keeps_the_editor_open_with_a_tick() {
    for keep in Keep::ALL {
        let mut rig = Rig::new();
        rig.editor();
        let asked = Out::Keep {
            keep,
            what: What::Picture(flat()),
        };
        assert_eq!(rig.on(In::Flat { keep, shot: flat() }), [asked]);
        assert_eq!(rig.on(In::Kept(Outcome::Done)), []);
        assert_eq!(rig.stage(), &edit(Some(keep)));
        assert_eq!(rig.wait(rig.shown()), []);
        assert_eq!(rig.stage(), &edit(None));
    }
}

#[test]
fn a_change_clears_the_tick() {
    let mut rig = Rig::new();
    rig.editor();
    rig.flat_kept(Keep::Copy);
    rig.on(In::Changed);
    assert_eq!(rig.stage(), &edit(None));
    assert_eq!(rig.flow.deadline(), None);
}

#[test]
fn the_questions_save_waits_for_the_flat_picture() {
    let mut rig = Rig::new();
    rig.editor();
    rig.on(In::Edit(EditKey::Close));
    assert_eq!(rig.on(In::Ask(AskKey::Save)), []);
    assert!(rig.flow.asking());
    let copy = In::Flat {
        keep: Keep::Copy,
        shot: flat(),
    };
    assert_eq!(rig.on(copy), []);
    let save = In::Flat {
        keep: Keep::Save,
        shot: flat(),
    };
    assert_eq!(rig.on(save).len(), 1);
    assert!(!rig.flow.asking());
}

#[test]
fn the_questions_save_ticks_saved_then_ends() {
    let mut rig = Rig::new();
    rig.editor();
    rig.on(In::Edit(EditKey::Close));
    rig.flat_kept(Keep::Save);
    assert_eq!(rig.stage(), &edit(Some(Keep::Save)));
    assert_eq!(rig.wait(rig.shown().saturating_sub(MS)), []);
    assert_eq!(rig.wait(MS), [Out::End]);
}

#[test]
fn a_change_during_the_questions_save_comes_back_unkept() {
    let mut rig = Rig::new();
    rig.editor();
    rig.on(In::Edit(EditKey::Close));
    rig.on(In::Flat {
        keep: Keep::Save,
        shot: flat(),
    });
    rig.on(In::Changed);
    assert_eq!(rig.on(In::Kept(Outcome::Done)), []);
    assert_eq!(rig.wait(rig.shown()), []);
    assert!(matches!(rig.stage(), Stage::Edit(_)));
    assert_eq!(rig.on(In::Edit(EditKey::Close)), []);
    assert!(rig.flow.asking());
}

#[test]
fn edge_22_a_change_during_the_save_first_asks_again_and_starts_nothing() {
    let mut rig = Rig::new();
    rig.editor();
    rig.on(In::Key);
    rig.on(In::Flat {
        keep: Keep::Save,
        shot: flat(),
    });
    rig.on(In::Changed);
    let outs = rig.on(In::Kept(Outcome::Done));
    let moved = outs.contains(&Out::End) || outs.contains(&Out::Freeze);
    assert!(!moved, "{outs:?}");
    assert_eq!(rig.wait(rig.shown()), []);
    rig.on(In::Key);
    assert!(rig.flow.asking());
}

#[test]
fn undo_and_redo_are_the_editors_own() {
    let mut rig = Rig::new();
    rig.editor();
    assert_eq!(rig.on(In::Edit(EditKey::Undo)), []);
    assert_eq!(rig.on(In::Edit(EditKey::Redo)), []);
    assert_eq!(rig.stage(), &edit(None));
}

#[test]
fn esc_in_the_editor_closes_as_close_does() {
    let mut rig = Rig::new();
    rig.editor();
    assert_eq!(rig.on(In::Esc), []);
    assert!(rig.flow.asking());
}

/// A new snip starts after the old snip's end: the key lights and the keyboard steps aside, which the freeze waits for.
fn restarted(outs: &[Out]) -> bool {
    let end = at(outs, &Out::End);
    let start = [Out::Lit(SnipLit::On), Out::Aside(SnipAside::Bubble)];
    end.is_some() && start.iter().all(|step| at(outs, step) > end)
}

#[test]
fn edge_22_the_key_with_unkept_work_asks_then_starts() {
    let mut rig = Rig::new();
    rig.editor();
    assert_eq!(rig.on(In::Key), []);
    assert!(rig.flow.asking());
    let outs = rig.on(In::Ask(AskKey::Drop));
    assert!(restarted(&outs), "{outs:?}");
    assert_eq!(rig.stage(), &Stage::Freezing);
}

#[test]
fn edge_22_save_first_then_start() {
    let mut rig = Rig::new();
    rig.editor();
    rig.on(In::Key);
    rig.on(In::Flat {
        keep: Keep::Save,
        shot: flat(),
    });
    assert!(restarted(&rig.on(In::Kept(Outcome::Done))));
}

#[test]
fn edge_22_kept_work_starts_at_once() {
    let mut rig = Rig::new();
    rig.editor();
    rig.flat_kept(Keep::Copy);
    assert!(restarted(&rig.on(In::Key)));
}

#[test]
fn edge_22_back_stays_in_the_editor() {
    let mut rig = Rig::new();
    rig.editor();
    rig.on(In::Key);
    assert_eq!(rig.on(In::Ask(AskKey::Back)), []);
    assert_eq!(rig.stage(), &edit(None));
}

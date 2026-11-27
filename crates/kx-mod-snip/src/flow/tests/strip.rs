//! The picture strip: Copy and Save tick and end the snip, a cancelled dialog comes back, a failure notes and stays,
//! and Edit opens the editor with the keyboard back.

use super::*;

/// The strip's picture, with the tick of `done`.
fn strip(done: Option<Keep>) -> Stage {
    Stage::Strip(Strip {
        part: part(),
        home: home(),
        held: Held::Picture(shot()),
        done,
        frame: Some(region_frame()),
    })
}

fn picture(keep: Keep) -> Out {
    Out::Keep {
        keep,
        what: What::Picture(shot()),
    }
}

#[test]
fn copy_and_save_ask_for_the_picture() {
    for (key, keep) in [(StripKey::Copy, Keep::Copy), (StripKey::Save, Keep::Save)] {
        let mut rig = Rig::new();
        rig.picture();
        assert_eq!(rig.on(In::Strip(key)), [picture(keep)]);
    }
}

#[test]
fn a_kept_picture_ticks_for_shown_ms_then_the_snip_ends() {
    for (key, keep) in [(StripKey::Copy, Keep::Copy), (StripKey::Save, Keep::Save)] {
        let mut rig = Rig::with(|s| s.shown_ms = 2_000);
        rig.picture();
        rig.on(In::Strip(key));
        assert_eq!(rig.on(In::Kept(Outcome::Done)), []);
        assert_eq!(rig.stage(), &strip(Some(keep)));
        assert_eq!(
            rig.flow.deadline(),
            Some(rig.now().saturating_add(rig.shown()))
        );
        assert_eq!(rig.wait(rig.shown().saturating_sub(MS)), []);
        assert!(ended(&rig.wait(MS)));
        assert_eq!(rig.stage(), &Stage::Idle);
    }
}

#[test]
fn the_key_during_the_tick_ends_at_once() {
    let mut rig = Rig::new();
    rig.picture();
    rig.on(In::Strip(StripKey::Copy));
    rig.on(In::Kept(Outcome::Done));
    assert_eq!(rig.on(In::Strip(StripKey::Edit)), []);
    assert!(ended(&rig.on(In::Key)));
    assert_eq!(rig.flow.deadline(), None);
}

#[test]
fn clicks_wait_while_a_copy_runs() {
    let mut rig = Rig::new();
    rig.picture();
    rig.on(In::Strip(StripKey::Copy));
    for input in [
        In::Strip(StripKey::Save),
        In::Strip(StripKey::Close),
        In::Strip(StripKey::Edit),
        In::Key,
        In::Esc,
    ] {
        assert_eq!(rig.on(input.clone()), [], "{input:?}");
    }
    assert_eq!(rig.stage(), &strip(None));
}

#[test]
fn a_busy_clipboard_notes_and_the_strip_stays() {
    let mut rig = Rig::new();
    rig.picture();
    rig.on(In::Strip(StripKey::Copy));
    let busy = Outcome::Failed(SnipNote::ClipBusy);
    assert_eq!(rig.on(In::Kept(busy)), [Out::Note(SnipNote::ClipBusy)]);
    assert_eq!(rig.stage(), &strip(None));
    assert_eq!(rig.on(In::Strip(StripKey::Copy)), [picture(Keep::Copy)]);
}

#[test]
fn a_cancelled_dialog_comes_back_to_the_strip() {
    let mut rig = Rig::new();
    rig.picture();
    rig.on(In::Strip(StripKey::Save));
    assert_eq!(rig.on(In::Kept(Outcome::Cancelled)), []);
    assert_eq!(rig.stage(), &strip(None));
    assert!(!rig.flow.asking());
    assert_eq!(rig.on(In::Strip(StripKey::Save)), [picture(Keep::Save)]);
}

#[test]
fn a_failed_save_notes_and_the_strip_stays() {
    let mut rig = Rig::new();
    rig.picture();
    rig.on(In::Strip(StripKey::Save));
    let failed = Outcome::Failed(SnipNote::SaveFailed);
    assert_eq!(rig.on(In::Kept(failed)), [Out::Note(SnipNote::SaveFailed)]);
    assert_eq!(rig.stage(), &strip(None));
}

#[test]
fn edit_opens_the_editor_and_brings_the_keyboard_back() {
    let mut rig = Rig::new();
    rig.picture();
    let outs = rig.on(In::Strip(StripKey::Edit));
    assert!(outs.contains(&Out::Edit(shot())));
    assert!(outs.contains(&Out::Lit(SnipLit::Off)));
    assert!(outs.contains(&Out::Aside(SnipAside::Back)));
    assert!(!outs.contains(&Out::End));
    let edit = Edit { done: None };
    assert_eq!(rig.stage(), &Stage::Edit(edit));
    assert_eq!(rig.flow.frozen(), None);
}

#[test]
fn video_keys_do_nothing_on_a_picture() {
    let mut rig = Rig::new();
    rig.picture();
    for key in [StripKey::Record, StripKey::Mic, StripKey::Sound] {
        assert_eq!(rig.on(In::Strip(key)), [], "{key:?}");
    }
    assert_eq!(rig.on(In::Record(RecKey::Stop)), []);
    assert_eq!(rig.stage(), &strip(None));
}

#[test]
fn an_answer_nobody_asked_for_is_dropped() {
    let mut rig = Rig::new();
    rig.picture();
    assert_eq!(rig.on(In::Kept(Outcome::Done)), []);
    rig.on(In::Strip(StripKey::Close));
    assert!(rig.flow.asking());
}

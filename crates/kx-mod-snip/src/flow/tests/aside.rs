//! The keyboard's step aside before the freeze: with an aside the screens freeze only once the ui answers that the
//! keyboard stepped aside, or at `aside_ms` without an answer; the key or Esc meanwhile ends the snip.

use super::*;

/// The wait for the ui's answer, as the settings the flow started with set it.
fn aside(rig: &Rig) -> Duration {
    Duration::from_millis(u64::from(rig.settings.aside_ms))
}

#[test]
fn with_an_aside_the_freeze_waits_for_the_uis_answer() {
    for (keyboard, way) in [
        (SnipKeyboard::Bubble, SnipAside::Bubble),
        (SnipKeyboard::Hidden, SnipAside::Hidden),
    ] {
        let mut rig = Rig::with(|s| s.keyboard = keyboard);
        assert_eq!(rig.on(In::Key), [Out::Lit(SnipLit::On), Out::Aside(way)]);
        assert_eq!(rig.stage(), &Stage::Freezing);
        assert_eq!(rig.on(In::Aside), [Out::Freeze], "{keyboard:?}");
        assert_eq!(rig.on(In::Aside), [], "a second answer does nothing");
        assert_eq!(rig.wait(aside(&rig)), [], "nor does the deadline");
    }
}

#[test]
fn with_no_answer_the_deadline_freezes_anyway() {
    let mut rig = Rig::new();
    rig.on(In::Key);
    let due = rig.now().saturating_add(aside(&rig));
    assert_eq!(rig.flow.deadline(), Some(due));
    assert_eq!(rig.wait(aside(&rig).saturating_sub(MS)), []);
    assert_eq!(rig.wait(MS), [Out::Freeze]);
    assert_eq!(rig.flow.deadline(), None);
    assert_eq!(rig.on(In::Aside), [], "a late answer does nothing");
    rig.on(In::Frozen(Ok(frozen())));
    assert!(matches!(rig.stage(), Stage::Picking(_)));
}

#[test]
fn the_key_or_esc_during_the_wait_ends_the_snip_and_nothing_freezes() {
    for input in [In::Key, In::Esc] {
        let mut rig = Rig::new();
        rig.on(In::Key);
        assert!(ended(&rig.on(input.clone())), "{input:?}");
        assert_eq!(rig.flow.deadline(), None);
        assert_eq!(rig.on(In::Aside), [], "{input:?}");
        assert_eq!(rig.wait(aside(&rig)), [], "{input:?}");
        assert_eq!(rig.stage(), &Stage::Idle);
    }
}

#[test]
fn a_freeze_answer_while_waiting_is_dropped() {
    let mut rig = Rig::new();
    rig.on(In::Key);
    assert_eq!(rig.on(In::Frozen(Ok(frozen()))), []);
    assert_eq!(rig.stage(), &Stage::Freezing);
    assert_eq!(rig.flow.frozen(), None);
    assert_eq!(rig.on(In::Aside), [Out::Freeze]);
}

#[test]
fn shown_freezes_at_once_and_ignores_an_answer() {
    let mut rig = Rig::with(|s| s.keyboard = SnipKeyboard::Shown);
    assert_eq!(rig.on(In::Key), [Out::Lit(SnipLit::On), Out::Freeze]);
    assert_eq!(rig.flow.deadline(), None);
    assert_eq!(rig.on(In::Aside), []);
}

#[test]
fn a_snip_started_from_the_editor_waits_for_the_ui_in_every_mode() {
    for (keyboard, way) in [
        (SnipKeyboard::Bubble, SnipAside::Bubble),
        (SnipKeyboard::Hidden, SnipAside::Hidden),
        (SnipKeyboard::Shown, SnipAside::Stay),
    ] {
        let mut rig = Rig::with(|s| s.keyboard = keyboard);
        rig.editor();
        rig.on(In::Key);
        let outs = rig.on(In::Ask(AskKey::Drop));
        assert!(outs.contains(&Out::Aside(way)), "{keyboard:?}: {outs:?}");
        assert!(
            !outs.contains(&Out::Freeze),
            "{keyboard:?}: the editor may still show"
        );
        assert_eq!(rig.on(In::Aside), [Out::Freeze], "{keyboard:?}");
    }
}

#[test]
fn shown_from_the_editor_freezes_at_aside_ms_and_never_brings_the_keyboard_back() {
    let mut rig = Rig::with(|s| s.keyboard = SnipKeyboard::Shown);
    rig.editor();
    rig.flat_kept(Keep::Copy);
    let outs = rig.on(In::Key);
    assert!(outs.contains(&Out::Aside(SnipAside::Stay)), "{outs:?}");
    assert_eq!(rig.wait(aside(&rig).saturating_sub(MS)), []);
    assert_eq!(rig.wait(MS), [Out::Freeze]);
    rig.on(In::Frozen(Ok(frozen())));
    let ends = rig.on(In::Key);
    let stepped = ends.iter().any(|o| matches!(o, Out::Aside(_)));
    assert!(!stepped, "it never stepped aside: {ends:?}");
}

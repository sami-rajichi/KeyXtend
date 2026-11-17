//! The fake's looks: each finds the box in front, one focus moved to unheard too, keeps and tells nothing, fails on cue
//! and is counted.

use super::*;

#[test]
fn a_look_finds_a_box_focus_moved_to_unheard_and_keeps_and_tells_nothing() {
    let fake = FakeFieldWatch::new();
    let (mut run, heard) = watched(&fake);
    fake.focus(FieldKind::None);
    let box_id = fake.box_id();
    fake.move_unheard(FieldKind::Text);
    let found = FieldChange {
        kind: FieldKind::Text,
        box_id,
    };
    assert_eq!(run.look(), Ok(Some(found)));
    assert_eq!(run.kind(), FieldKind::None, "the box told last stays");
    assert_eq!(heard.lock().unwrap().len(), 1, "nothing more told");
    assert_eq!((fake.looks(), fake.before_calls()), (1, 0), "no read");
}

#[test]
fn a_look_fails_as_reads_fail_until_focus_moves() {
    let fake = FakeFieldWatch::new();
    let (mut run, _) = watched(&fake);
    fake.focus(FieldKind::None);
    fake.fail_reads(FieldError::Refused);
    assert_eq!(run.look(), Err(FieldError::Refused));
    fake.focus(FieldKind::Text);
    assert!(
        run.look()
            .is_ok_and(|c| c.is_some_and(|c| c.kind == FieldKind::Text))
    );
    assert_eq!(fake.looks(), 2, "failed ones too");
}

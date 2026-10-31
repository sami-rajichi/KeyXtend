//! The paste the clipboard module asks for (ADR-0034 decision 2): every latch lets go first, then one plain paste goes
//! through the shortcuts to the app in front, and the answer names the request (plan edge cases 5 and 13).

use super::{Clip, Heard};
use crate::assist::Said;
use crate::known::sid;
use crate::rig::{HELD, STATE};
use kx_module_api::ports::{
    FaceAction, FaceReply, PASTE_ID, PasteAsk, Pasted, Scan, ShortcutError, ShortcutId,
};

/// The paste as the shortcuts port took it: the id alone, nothing held, and no window named.
fn one_paste() -> Vec<(ShortcutId, Vec<Scan>)> {
    vec![(sid(PASTE_ID), Vec::new())]
}

#[test]
fn a_paste_ask_lets_ctrl_and_shift_go_and_sends_one_plain_paste() {
    let c = Clip::quiet(None);
    c.b.rig.tap("ctrl").unwrap();
    c.b.rig.tap("shift").unwrap();
    c.b.said();
    c.b.send(&PasteAsk { round: 7 });
    assert_eq!(c.heard(), [Heard::Pasted(Pasted { round: 7, ok: true })]);
    assert_eq!(c.b.rig.fakes.shortcuts.sent(), one_paste());
    assert!(c.strokes().is_empty(), "nothing else reaches the app");
    assert_eq!(
        c.b.said(),
        [Said::Latches(Vec::new())],
        "the bus hears they let go"
    );
    assert_eq!(c.b.rig.act(FaceAction::Relabel), Ok(STATE));
    assert!(!c.on("ctrl") && !c.on("shift"));
}

#[test]
fn in_stay_mode_a_paste_ask_lets_the_latches_go_too() {
    let c = Clip::quiet(Some(("modifiers", "stay".into())));
    c.b.rig.tap("ctrl").unwrap();
    c.b.send(&PasteAsk { round: 1 });
    assert_eq!(c.b.rig.fakes.shortcuts.sent(), one_paste());
    assert!(!c.on("ctrl"));
}

#[test]
fn a_refused_or_failed_paste_answers_not_ok_with_its_round() {
    let c = Clip::quiet(None);
    let errors = [
        ShortcutError::Refused { sent: 2, total: 4 },
        ShortcutError::Failed,
    ];
    for (round, error) in (3..).zip(errors) {
        c.b.rig.fakes.shortcuts.set_error(Some(error));
        c.b.send(&PasteAsk { round });
        assert_eq!(c.heard(), [Heard::Pasted(Pasted { round, ok: false })]);
    }
}

#[test]
fn a_paste_ask_ends_a_held_key_so_it_never_repeats_after() {
    let c = Clip::quiet(None);
    c.b.rig.tap("shift").unwrap();
    assert_eq!(c.b.rig.press("c-0x0010"), Ok(HELD));
    c.b.send(&PasteAsk { round: 2 });
    c.strokes();
    assert_eq!(c.b.rig.tick_in(1_000), FaceReply::default());
    assert!(c.strokes().is_empty(), "the hold ended with the paste");
    assert!(!c.on("shift"));
}

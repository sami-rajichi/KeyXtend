//! A fill waiting its gap (spec §8.1 "Filling"): it is the only fill, our clicks reach only a panel aimed at again, a
//! picked word or a paste waits its turn by being refused, and the stop ends it with a no.

use super::{Heard, Quick, gap_ms, line, login, tab, text, typed, unicode};
use crate::assist::ear;
use kx_module_api::ports::{FaceAction, PanelKey, PasteAsk, Pasted, TypeWord};
use kx_module_api::{Module, Redacted};
use std::sync::{Arc, Mutex};

#[test]
fn a_second_fill_during_the_gap_answers_no_at_once_and_the_first_carries_on() {
    let q = Quick::quiet(None);
    q.fill(1, login());
    q.fill(2, vec![line("other")]);
    assert_eq!(q.heard(), [typed(2, false)]);
    assert_eq!(
        q.strokes(),
        [unicode("alex"), tab()],
        "the second sent nothing"
    );
    q.relabel_in(gap_ms());
    assert_eq!(q.strokes(), [unicode("pw")]);
    assert_eq!(q.heard(), [typed(1, true)]);
    q.fill(3, vec![line("next")]);
    assert_eq!(q.heard(), [typed(3, true)], "once done, the next goes");
}

#[test]
fn clicks_during_the_gap_go_to_a_panel_aimed_at_and_never_to_the_app() {
    let q = Quick::opened(None);
    q.fill(4, login());
    q.strokes();
    q.b.rig.act(FaceAction::AimPanel).unwrap();
    q.b.rig.tap("c-0x0010").unwrap();
    assert_eq!(q.heard(), [text("q")], "the panel takes its keys as usual");
    for id in ["tab", "del", "ctrl", "clip", "c-0x0011"] {
        q.b.rig.tap(id).unwrap();
    }
    q.b.rig.press("enter").unwrap();
    assert!(q.strokes().is_empty(), "nothing reaches the app");
    assert!(!q.on("ctrl"), "nothing latches either");
    q.relabel_in(gap_ms());
    assert_eq!(q.strokes(), [unicode("pw")]);
    let delete = Heard::Key(PanelKey::Delete);
    let enter = Heard::Key(PanelKey::Enter);
    assert_eq!(q.heard(), [delete, text("w"), enter, typed(4, true)]);
}

#[test]
fn with_the_panel_shut_during_the_gap_no_key_reaches_the_app() {
    let q = Quick::opened(None);
    q.fill(5, login());
    q.strokes();
    q.open(false);
    for id in ["space", "c-0x0010", "bksp"] {
        q.b.rig.tap(id).unwrap();
    }
    assert!(q.strokes().is_empty() && q.heard().is_empty());
    q.relabel_in(gap_ms());
    assert_eq!(q.strokes(), [unicode("pw")]);
    q.b.rig.tap("space").unwrap();
    assert_eq!(q.strokes().len(), 1, "after the fill, keys type again");
}

#[test]
fn a_picked_word_or_a_paste_during_the_gap_sends_nothing() {
    let pasted = Arc::new(Mutex::new(Vec::new()));
    let p = Arc::clone(&pasted);
    let q = Quick::after(None, move |bus| vec![ear(bus, &p, |e: &Pasted| *e)]);
    q.heard();
    q.fill(6, login());
    q.strokes();
    let text = Redacted::new("word".into());
    q.b.send(&TypeWord {
        round: 1,
        erase: 1,
        text,
        line: false,
    });
    q.b.send(&PasteAsk { round: 2 });
    assert!(q.strokes().is_empty());
    assert!(q.b.rig.fakes.shortcuts.sent().is_empty(), "no paste");
    let no = Pasted {
        round: 2,
        ok: false,
    };
    assert_eq!(*pasted.lock().unwrap(), [no]);
    q.relabel_in(gap_ms());
    assert_eq!(q.heard(), [typed(6, true)]);
}

#[test]
fn a_fill_heard_after_the_stop_sends_nothing_and_answers_no() {
    let mut q = Quick::quiet(None);
    q.b.rig.tap("ctrl").unwrap();
    // The kernel drops the handlers before the stop, but one may still be running on the sender's thread.
    q.b.rig.module.stop();
    q.fill(7, vec![line("a")]);
    assert!(q.strokes().is_empty());
    assert_eq!(q.heard(), [typed(7, false)]);
    assert!(q.on("ctrl"), "the latches stay as they were");
}

#[test]
fn the_stop_ends_a_waiting_fill_with_no_and_nothing_more_goes_out() {
    let mut q = Quick::quiet(None);
    q.fill(8, login());
    q.strokes();
    q.b.rig.module.stop();
    assert_eq!(q.heard(), [typed(8, false)]);
    q.relabel_in(1_000);
    assert!(q.strokes().is_empty() && q.heard().is_empty());
    q.b.rig.restart(None);
    q.relabel_in(gap_ms());
    assert!(q.strokes().is_empty(), "a restart brings none of it back");
}

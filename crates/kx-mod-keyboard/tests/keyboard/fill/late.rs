//! A fill never finishes late (design "Filling"): past `fill_late_ms` after its part was due, or after a click outside
//! our windows, the rest is dropped with a no, so a password never lands in a window the user moved to.

use super::{GAP, Quick, late, line, login, ms, tab, typed, unicode};
use kx_module_api::ports::{FaceAction, PointerPressed};
use std::time::Duration;

#[test]
fn a_call_past_the_bound_sends_nothing_more_and_answers_no_once() {
    let q = Quick::quiet(None);
    q.fill(1, login());
    assert_eq!(q.strokes(), [unicode("alex"), tab()]);
    q.relabel_in(ms(GAP + late()) + 1);
    assert!(q.strokes().is_empty(), "the password never goes late");
    assert_eq!(q.heard(), [typed(1, false)]);
    q.relabel_in(ms(GAP));
    assert!(q.strokes().is_empty() && q.heard().is_empty());
}

#[test]
fn a_call_within_the_bound_still_sends_the_rest() {
    let q = Quick::quiet(None);
    q.fill(2, login());
    q.strokes();
    q.relabel_in(ms(GAP + late()));
    assert_eq!(q.strokes(), [unicode("pw")]);
    assert_eq!(q.heard(), [typed(2, true)]);
}

#[test]
fn a_new_fill_goes_out_as_usual_after_one_dropped() {
    let q = Quick::quiet(None);
    q.fill(3, login());
    q.relabel_in(ms(GAP + late()) + 1);
    q.strokes();
    assert_eq!(q.heard(), [typed(3, false)]);
    q.fill(4, login());
    assert_eq!(q.strokes(), [unicode("alex"), tab()]);
    q.relabel_in(ms(GAP));
    assert_eq!(q.strokes(), [unicode("pw")]);
    assert_eq!(q.heard(), [typed(4, true)]);
}

#[test]
fn a_late_fill_no_call_reached_is_dropped_when_the_next_one_comes() {
    let q = Quick::quiet(None);
    q.fill(5, login());
    q.strokes();
    q.b.rig
        .fakes
        .clock
        .advance(GAP + late() + Duration::from_millis(1));
    q.fill(6, vec![line("next")]);
    assert_eq!(
        q.strokes(),
        [unicode("next")],
        "the old password stays unsent"
    );
    assert_eq!(q.heard(), [typed(5, false), typed(6, true)]);
}

#[test]
fn a_fill_waiting_inside_the_bound_refuses_the_next_and_is_left_alone() {
    let q = Quick::quiet(None);
    q.fill(7, login());
    q.strokes();
    q.b.rig.fakes.clock.advance(GAP);
    q.fill(8, vec![line("next")]);
    assert!(
        q.strokes().is_empty(),
        "neither types from the vault's thread"
    );
    assert_eq!(q.heard(), [typed(8, false)]);
    q.relabel_in(0);
    assert_eq!(
        q.strokes(),
        [unicode("pw")],
        "the waiting one goes at the UI's call"
    );
    assert_eq!(q.heard(), [typed(7, true)]);
}

#[test]
fn a_fill_clicked_away_is_dropped_when_the_next_one_comes() {
    let q = Quick::quiet(None);
    q.fill(12, login());
    q.strokes();
    q.b.send(&PointerPressed { ours: false });
    q.fill(13, vec![line("next")]);
    assert_eq!(q.strokes(), [unicode("next")]);
    assert_eq!(q.heard(), [typed(12, false), typed(13, true)]);
}

/// The design's edge case: the keyboard hides mid-fill, its timer rests, and it shows again minutes later.
#[test]
fn a_fill_while_the_keyboard_is_minimised_sends_nothing_once_past_the_bound() {
    let q = Quick::quiet(None);
    q.fill(9, login());
    q.strokes();
    q.b.rig.act(FaceAction::Leave).unwrap();
    q.b.rig.fakes.clock.advance(Duration::from_secs(300));
    q.b.rig.act(FaceAction::Relabel).unwrap();
    assert!(
        q.strokes().is_empty(),
        "nothing lands in the window in front now"
    );
    assert_eq!(q.heard(), [typed(9, false)]);
}

#[test]
fn a_click_outside_our_windows_during_the_gap_drops_the_rest_at_the_next_call() {
    let q = Quick::quiet(None);
    q.fill(10, login());
    q.strokes();
    q.b.send(&PointerPressed { ours: false });
    q.relabel_in(0);
    assert_eq!(q.heard(), [typed(10, false)], "even before the gap ends");
    q.relabel_in(ms(GAP));
    assert!(q.strokes().is_empty(), "the caret may have moved");
    q.fill(11, login());
    q.strokes();
    q.b.send(&PointerPressed { ours: true });
    q.relabel_in(ms(GAP));
    assert_eq!(
        q.strokes(),
        [unicode("pw")],
        "a click on our own windows keeps it"
    );
    assert_eq!(q.heard(), [typed(11, true)]);
}

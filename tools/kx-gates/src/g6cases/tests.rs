//! Tests of G6's cases: the ratio from both slopes, sizing faults, Stop, and the skipped press-stop.

use super::*;

/// A phase whose percent climbs `per_s` a second, read every 200 ms.
fn phase(per_s: f64) -> Phase {
    let reads = (0..10_u32)
        .map(|i| (u64::from(i) * 200, 10.0 + per_s * f64::from(i) / 5.0))
        .collect();
    let look = Look {
        share: None,
        before: None,
        after: None,
    };
    Phase { look, reads }
}

#[test]
fn the_ratio_is_judged_from_both_slopes_and_left_unjudged_without_one() {
    let g = crate::config::load().expect("loads").g6;
    let want = 10.0 / 3.0;
    let near = ratio(&phase(3.0), &phase(10.0), want, &g);
    assert_eq!(near.ok, Some(true), "{}", near.note);
    assert_eq!(ratio(&phase(3.0), &phase(3.0), want, &g).ok, Some(false));
    let few = Phase {
        reads: vec![(0, 10.0)],
        ..phase(3.0)
    };
    let unread = ratio(&few, &phase(10.0), want, &g);
    assert_eq!(unread.ok, None);
    assert_eq!(unread.json()["case"], "ratio");
}

#[test]
fn a_case_the_pad_met_the_end_in_fails_as_a_sizing_fault() {
    let min = crate::config::load().expect("loads").g6.min_changed;
    let cases = [
        Case::stilled("third_tap", None, min),
        Case::unjudged("sideways", "no sideways overflow"),
        Case::too_short("press_stop"),
    ];
    assert_eq!(cases[0].ok, Some(false), "never passes unjudged");
    assert_eq!(cases[2].ok, Some(false));
    assert_eq!(sizing_faults(&cases), ["third_tap", "press_stop"]);
}

#[test]
fn a_stop_case_is_judged_on_its_look_and_fails_without_one() {
    let min = crate::config::load().expect("loads").g6.min_changed;
    let still = Look {
        share: Some(0.0),
        before: None,
        after: None,
    };
    assert_eq!(Case::stilled("third_tap", Some(still), min).ok, Some(true));
    let moved = Look {
        share: Some(0.5),
        ..still
    };
    assert_eq!(Case::stilled("third_tap", Some(moved), min).ok, Some(false));
    let gone = Case::stilled("third_tap", None, min);
    assert_eq!(gone.ok, Some(false));
    assert_eq!(gone.note[SIZING], STOPPED_ITSELF);
    assert_eq!(Case::stopped(Some(still), min).ok, Some(true));
}

#[test]
fn a_pad_that_stopped_itself_on_the_way_up_fails_stop_with_its_own_note() {
    let min = crate::config::load().expect("loads").g6.min_changed;
    let early = Case::stopped(None, min);
    assert_eq!((early.name, early.ok), ("stop", Some(false)));
    assert_eq!(early.note[EARLY], UP_FIRST);
    assert!(sizing_faults(&[early]).is_empty(), "no sizing fault");
}

#[test]
fn a_press_stop_without_a_safe_spot_is_skipped_with_a_note_and_never_passes() {
    let skipped = Case::unjudged(PRESS, NO_SPOT);
    assert_eq!(skipped.ok, None, "not a pass");
    assert_eq!(skipped.json()["note"]["unjudged"], NO_SPOT);
    assert!(sizing_faults(&[skipped]).is_empty());
}

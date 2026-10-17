//! Tests of G6's cases: the ratio from both slopes, sizing faults, Stop, the press-stop and its skip, and the notes.

use windows::Win32::UI::Accessibility::UIA_PaneControlTypeId;

use super::*;
use crate::g6spot::{Part, Tried};

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
fn a_press_stop_without_a_safe_spot_is_skipped_with_each_try_and_never_passes() {
    let tried = Tried {
        part: Part::Header,
        kind: Some(UIA_PaneControlTypeId),
        class: "DirectUIHWND".to_string(),
        owner: "[CabinetWClass] of explorer.exe".to_string(),
    };
    let skipped = Case::unspotted(std::slice::from_ref(&tried));
    assert_eq!((skipped.name, skipped.ok), (PRESS, None), "not a pass");
    let note = skipped.json()["note"].clone();
    assert_eq!(note["unjudged"], NO_SPOT);
    assert_eq!(note[TRIED], json!([tried.json()]));
    assert!(sizing_faults(&[skipped]).is_empty());
}

/// A look whose picture stood still, its percents `before` and `after`.
fn still(before: (f64, f64), after: (f64, f64)) -> Look {
    Look {
        share: Some(0.0),
        before: Some(before),
        after: Some(after),
    }
}

#[test]
fn each_still_case_notes_both_looks_percents() {
    let min = crate::config::load().expect("loads").g6.min_changed;
    let late = still((0.0, 40.0), (0.0, 40.5));
    let third = Case::stilled("third_tap", Some(late), min);
    assert_eq!(third.ok, Some(false), "a late percent still moved");
    assert_eq!(third.note[PAIRS], json!([[0.0, 40.0], [0.0, 40.5]]));
    assert_eq!(
        Case::stopped(Some(late), min).note[PAIRS],
        third.note[PAIRS]
    );
}

#[test]
fn a_press_stop_is_judged_on_its_run_and_its_still_look_with_their_notes() {
    let min = crate::config::load().expect("loads").g6.min_changed;
    let going = Look {
        share: Some(0.3),
        before: Some((0.0, 10.0)),
        after: Some((0.0, 14.0)),
    };
    let after = still((0.0, 14.0), (0.0, 14.0));
    let case = Case::pressed(&going, &after, min);
    assert_eq!((case.name, case.ok), (PRESS, Some(true)));
    assert_eq!(case.note["going"], 0.3);
    assert_eq!(case.note[PAIRS], json!([[0.0, 14.0], [0.0, 14.0]]));
    let on = still((0.0, 14.0), (0.0, 15.0));
    assert_eq!(
        Case::pressed(&going, &on, min).ok,
        Some(false),
        "it kept going"
    );
}

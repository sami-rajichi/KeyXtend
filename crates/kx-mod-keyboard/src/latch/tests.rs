use super::*;
use proptest::prelude::*;
use std::collections::BTreeMap;

const SHIFT: Scan = Scan::new(0x2A);
const RSHIFT: Scan = Scan::new(0x36);
const CTRL: Scan = Scan::new(0x1D);
const ALTGR: Scan = Scan::new(0xE038);
const WIN: Scan = Scan::new(0xE05B);
const ALL: [Latch; 5] = [
    Latch::Shift,
    Latch::Ctrl,
    Latch::Alt,
    Latch::AltGr,
    Latch::Win,
];

#[test]
fn a_click_latches_and_a_second_click_lets_go() {
    let mut m = Latches::new(Modifiers::Release);
    m.toggle(Latch::Shift, SHIFT);
    assert!(m.is_on(Latch::Shift));
    m.toggle(Latch::Shift, RSHIFT);
    assert!(!m.is_on(Latch::Shift), "either Shift key lets it go");
}

#[test]
fn a_new_mode_keeps_what_is_latched_and_applies_from_the_next_key() {
    let mut m = Latches::new(Modifiers::Stay);
    m.toggle(Latch::Shift, SHIFT);
    m.set_mode(Modifiers::Release);
    assert!(m.is_on(Latch::Shift));
    m.used();
    assert!(!m.is_on(Latch::Shift));
}

#[test]
fn latches_show_neither_a_scan_code_nor_which_are_on() {
    let mut m = Latches::new(Modifiers::Release);
    m.toggle(Latch::AltGr, ALTGR);
    m.toggle(Latch::Shift, RSHIFT);
    let shown = format!("{m:?}");
    for leak in ["54", "36", "57400", "E038", "Scan", "Shift", "AltGr", "on"] {
        assert!(!shown.contains(leak), "{shown}");
    }
    assert_eq!(shown, "Latches { mode: Release, .. }");
    m.spend();
    assert_eq!(format!("{m:?}"), shown, "nor which were used");
}

#[test]
fn the_next_key_holds_every_latch_once_in_release_mode() {
    let mut m = Latches::new(Modifiers::Release);
    m.toggle(Latch::Ctrl, CTRL);
    m.toggle(Latch::Shift, SHIFT);
    assert_eq!(m.held(&[]), [SHIFT, CTRL], "in a fixed order");
    m.used();
    assert!(m.held(&[]).is_empty(), "let go after one key");
    assert!(!m.is_on(Latch::Ctrl));
}

#[test]
fn a_word_that_started_under_some_latches_uses_or_spends_only_those() {
    let mut m = Latches::new(Modifiers::Release);
    m.toggle(Latch::Shift, SHIFT);
    let then = m.clone();
    m.toggle(Latch::Ctrl, CTRL);
    assert!(m.used_of(&then));
    assert!(
        !m.is_on(Latch::Shift) && m.unused(Latch::Ctrl),
        "Ctrl came later"
    );
    let mut s = Latches::new(Modifiers::Stay);
    let then = s.clone();
    s.toggle(Latch::Shift, SHIFT);
    assert!(!s.used_of(&then), "nothing was on at the start");
    s.spend_of(&then);
    assert!(s.unused(Latch::Shift));
    let then = s.clone();
    s.spend_of(&then);
    assert!(s.is_on(Latch::Shift) && !s.unused(Latch::Shift));
}

#[test]
fn latches_stay_after_keys_in_stay_mode_until_clicked_again() {
    let mut m = Latches::new(Modifiers::Stay);
    m.toggle(Latch::Shift, SHIFT);
    m.used();
    m.used();
    assert_eq!((m.held(&[]), m.mode()), (vec![SHIFT], Modifiers::Stay));
    m.toggle(Latch::Shift, SHIFT);
    assert!(!m.is_on(Latch::Shift));
}

#[test]
fn a_skipped_latch_is_not_held_and_still_lets_go() {
    let mut m = Latches::new(Modifiers::Release);
    m.toggle(Latch::AltGr, ALTGR);
    m.toggle(Latch::Shift, SHIFT);
    assert_eq!(m.held(&[Latch::AltGr]), [SHIFT]);
    m.used();
    assert!(!m.is_on(Latch::AltGr), "let go too");
}

#[test]
fn a_latch_used_in_stay_mode_stays_on_but_is_no_longer_unused() {
    let mut m = Latches::new(Modifiers::Stay);
    assert!(!m.unused(Latch::Win), "not latched");
    m.toggle(Latch::Win, WIN);
    assert!(m.unused(Latch::Win));
    assert!(!m.used(), "nothing let go");
    assert!(m.is_on(Latch::Win) && !m.unused(Latch::Win));
    m.toggle(Latch::Win, WIN);
    m.toggle(Latch::Win, WIN);
    assert!(m.unused(Latch::Win), "latched again, it starts unused");
}

#[test]
fn used_in_release_mode_says_whether_any_let_go() {
    let mut m = Latches::new(Modifiers::Release);
    assert!(!m.used(), "none latched");
    m.toggle(Latch::Ctrl, CTRL);
    assert!(m.used());
    assert!(!m.is_on(Latch::Ctrl));
}

#[test]
fn a_latch_latched_while_one_is_on_uses_it_but_one_latched_before_does_not() {
    let mut m = Latches::new(Modifiers::Release);
    m.toggle(Latch::Win, WIN);
    m.toggle(Latch::Shift, SHIFT);
    assert!(!m.unused(Latch::Win), "Win+Shift");
    assert!(m.unused(Latch::Shift), "nothing came after Shift");
    m.toggle(Latch::Shift, SHIFT);
    assert!(m.is_on(Latch::Win), "latching never lets another go");
    assert!(!m.unused(Latch::Win), "letting Shift go undoes nothing");
}

#[test]
fn a_spent_latch_stays_on_in_both_modes() {
    for mode in [Modifiers::Release, Modifiers::Stay] {
        let mut m = Latches::new(mode);
        m.toggle(Latch::Win, WIN);
        m.spend();
        assert!(m.is_on(Latch::Win), "{mode:?}");
        assert!(!m.unused(Latch::Win), "{mode:?}");
    }
}

/// One step of a run: a click on a modifier key, a key that holds the latches but `skip`, or a Caps Lock tap.
#[derive(Clone, Debug)]
enum Step {
    Click(Latch, Scan),
    Key(Vec<Latch>),
    Caps,
}

fn step() -> impl Strategy<Value = Step> {
    let latch = prop::sample::select(ALL.to_vec());
    let codes = vec![0x2A, 0x36, 0x1D, 0x38, 0xE038, 0xE05B];
    let key = prop::sample::select(codes).prop_map(Scan::new);
    prop_oneof![
        (latch.clone(), key).prop_map(|(l, k)| Step::Click(l, k)),
        prop::collection::vec(latch, 0..3).prop_map(Step::Key),
        Just(Step::Caps),
    ]
}

/// The latches on, each with the key of its last click and whether anything used it since.
type Model = BTreeMap<Latch, (Scan, bool)>;

/// What the model holds for a key: each latch on but `skip`, by its key.
fn want(model: &Model, skip: &[Latch]) -> Vec<Scan> {
    let on = model.iter().filter(|(l, _)| !skip.contains(l));
    on.map(|(_, &(key, _))| key).collect()
}

/// Marks every latch on in the model as used.
fn spend(model: &mut Model) {
    for (_, used) in model.values_mut() {
        *used = true;
    }
}

/// One step on the model; `release` lets every latch go after a key.
fn model_step(model: &mut Model, s: &Step, release: bool) {
    match s {
        Step::Click(l, key) => {
            if model.remove(l).is_none() {
                spend(model);
                model.insert(*l, (*key, false));
            }
        }
        Step::Key(_) | Step::Caps => {
            spend(model);
            if release && matches!(s, Step::Key(_)) {
                model.clear();
            }
        }
    }
}

/// Runs `steps` in `mode`, checking every key's held codes, what let go, and every latch after each step.
fn run(mode: Modifiers, steps: Vec<Step>) -> Result<(), TestCaseError> {
    let release = mode == Modifiers::Release;
    let mut m = Latches::new(mode);
    let mut model = Model::new();
    for s in steps {
        match &s {
            Step::Click(l, key) => m.toggle(*l, *key),
            Step::Key(skip) => {
                prop_assert_eq!(m.held(skip), want(&model, skip));
                prop_assert_eq!(m.used(), release && !model.is_empty());
            }
            Step::Caps => m.spend(),
        }
        model_step(&mut model, &s, release);
        for l in ALL {
            let on = model.get(&l);
            prop_assert_eq!(m.is_on(l), on.is_some(), "{:?}", l);
            let unused = on.is_some_and(|&(_, used)| !used);
            prop_assert_eq!(m.unused(l), unused, "{:?} unused", l);
        }
    }
    Ok(())
}

proptest! {
    #[test]
    fn release_lets_every_latch_go_after_the_next_key(steps in prop::collection::vec(step(), 0..40)) {
        run(Modifiers::Release, steps)?;
    }

    #[test]
    fn stay_keeps_every_latch_until_it_is_clicked_again(steps in prop::collection::vec(step(), 0..40)) {
        run(Modifiers::Stay, steps)?;
    }
}

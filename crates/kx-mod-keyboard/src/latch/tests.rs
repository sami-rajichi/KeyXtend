use super::*;
use proptest::prelude::*;
use std::collections::BTreeMap;

const SHIFT: Scan = Scan::new(0x2A);
const RSHIFT: Scan = Scan::new(0x36);
const CTRL: Scan = Scan::new(0x1D);
const ALTGR: Scan = Scan::new(0xE038);
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
fn latches_never_show_a_scan_code() {
    let mut m = Latches::new(Modifiers::Release);
    m.toggle(Latch::AltGr, ALTGR);
    m.toggle(Latch::Shift, RSHIFT);
    let shown = format!("{m:?}");
    for leak in ["54", "36", "57400", "E038", "Scan"] {
        assert!(!shown.contains(leak), "{shown}");
    }
    assert!(shown.contains("Shift") && shown.contains("AltGr") && shown.contains("Release"));
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

/// One step of a run: a click on a modifier key, or a key that holds the latches but `skip`.
#[derive(Clone, Debug)]
enum Step {
    Click(Latch, Scan),
    Key(Vec<Latch>),
}

fn step() -> impl Strategy<Value = Step> {
    let latch = prop::sample::select(ALL.to_vec());
    let codes = vec![0x2A, 0x36, 0x1D, 0x38, 0xE038, 0xE05B];
    let key = prop::sample::select(codes).prop_map(Scan::new);
    prop_oneof![
        (latch.clone(), key).prop_map(|(l, k)| Step::Click(l, k)),
        prop::collection::vec(latch, 0..3).prop_map(Step::Key),
    ]
}

/// Clicks per latch since it last let go, and the key of the last click.
type Model = BTreeMap<Latch, (usize, Scan)>;

/// What the model holds for a key: each latch clicked an odd number of times, by its last key.
fn want(model: &Model, skip: &[Latch]) -> Vec<Scan> {
    let on = model
        .iter()
        .filter(|(l, (n, _))| n % 2 == 1 && !skip.contains(l));
    on.map(|(_, &(_, key))| key).collect()
}

/// Runs `steps` in `mode`, checking every key's held codes and every latch after each step.
fn run(mode: Modifiers, steps: Vec<Step>) -> Result<(), TestCaseError> {
    let mut m = Latches::new(mode);
    let mut model = Model::new();
    for s in steps {
        match s {
            Step::Click(l, key) => {
                let entry = model.entry(l).or_insert((0, key));
                *entry = (entry.0 + 1, key);
                m.toggle(l, key);
            }
            Step::Key(skip) => {
                prop_assert_eq!(m.held(&skip), want(&model, &skip));
                m.used();
                if mode == Modifiers::Release {
                    model.clear();
                }
            }
        }
        for l in ALL {
            let clicks = model.get(&l).map_or(0, |&(n, _)| n);
            prop_assert_eq!(m.is_on(l), clicks % 2 == 1, "{:?}", l);
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

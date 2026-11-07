//! Any run of panel opens and closes, stale panel words, keys, fills, waits, late calls, outside clicks and restarts
//! keeps at most one panel open, the keys lit as the modules say until a stale word, and answers every fill once.

use super::one::{Two, clip_said, fill_said};
use super::{GAP, Heard, line};
use crate::assist::scans;
use kx_module_api::ports::{FillOpen, FillPart, PointerPressed, Scan};
use proptest::collection::vec;
use proptest::prelude::*;
use std::time::Duration;

/// The keys a run clicks: both panel keys, a letter, the moves the panels take, a key they leave to the app, a latch.
const KEYS: [&str; 7] = ["clip", "fill", "c-0x0010", "esc", "space", "tab", "ctrl"];
/// The Win latch and the V key, the platform's history chord in this run.
const CHORD: (u16, u16, &str) = (0xE05B, 0x2F, "c-0x002F");

/// One thing that happens in a run.
#[derive(Clone, Debug)]
enum Step {
    /// A click on a key.
    Key(&'static str),
    /// The history chord, Win then V.
    Chord,
    /// The word lane's chip asks for Quick-fill.
    Chip,
    /// A module closes its own panel: the clipboard's when true, else Quick-fill's.
    Shut(bool),
    /// A late word from a module, whatever its panel is now: the clipboard's when the first is true, saying the second.
    Stale(bool, bool),
    /// A fill of these parts: a Tab for true, else text.
    Fill(Vec<bool>),
    /// The relabel timer runs this many ms later.
    Wait(u64),
    /// The relabel timer runs only past a waiting part's bound, as after the keyboard was minimised.
    Late,
    /// A click lands outside our windows, then a UI call checks nothing types.
    Away,
    /// A click lands outside our windows, heard only by whatever comes next, such as a new fill.
    Click,
    /// The keyboard restarts, as a settings change does.
    Restart,
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        3 => prop::sample::select(KEYS.to_vec()).prop_map(Step::Key),
        1 => Just(Step::Chord),
        1 => Just(Step::Chip),
        1 => any::<bool>().prop_map(Step::Shut),
        1 => any::<(bool, bool)>().prop_map(|(clip, open)| Step::Stale(clip, open)),
        2 => vec(any::<bool>(), 0..5).prop_map(Step::Fill),
        2 => (0u64..400).prop_map(Step::Wait),
        1 => Just(Step::Late),
        1 => Just(Step::Away),
        1 => Just(Step::Click),
        1 => Just(Step::Restart),
    ]
}

/// What a run tracks: the fills asked, and whether a stale word may have left the keys apart from the modules.
#[derive(Default)]
struct Run {
    asked: Vec<u32>,
    stale: bool,
}

/// Runs `step`.
fn run(t: &mut Two, step: Step, r: &mut Run) {
    let rig = &t.q.b.rig;
    match step {
        Step::Key(id) => {
            let _ = rig.tap(id);
        }
        Step::Chord => {
            let _ = rig.tap("win");
            let _ = rig.press(CHORD.2);
        }
        Step::Chip => t.q.b.send(&FillOpen),
        Step::Shut(true) if t.clip.is_open() => t.clip.set(false, clip_said),
        Step::Shut(false) if t.fill.is_open() => t.fill.set(false, fill_said),
        Step::Shut(_) => {}
        Step::Stale(clip, open) => {
            r.stale = true;
            if clip {
                t.q.b.send(&clip_said(open));
            } else {
                t.q.b.send(&fill_said(open));
            }
        }
        Step::Fill(tabs) => ask(t, &tabs, r),
        Step::Wait(ms) => t.q.relabel_in(ms),
        Step::Late => stop(t, true),
        Step::Away => stop(t, false),
        Step::Click => t.q.b.send(&PointerPressed { ours: false }),
        Step::Restart => {
            t.q.b.rig.restart(None);
            r.stale = false;
        }
    }
}

/// Stops a waiting fill, past its bound when `late`, else by a click outside our windows: nothing types after it.
fn stop(t: &Two, late: bool) {
    t.q.strokes();
    if late {
        let past = GAP + super::late() + Duration::from_millis(1);
        t.q.b.rig.fakes.clock.advance(past);
    } else {
        t.q.b.send(&PointerPressed { ours: false });
    }
    t.q.relabel_in(0);
    assert!(t.q.strokes().is_empty(), "nothing types once stopped");
}

/// Asks fill `r.asked.len() + 1` of `tabs`: a Tab for true, else text.
fn ask(t: &Two, tabs: &[bool], r: &mut Run) {
    let parts = tabs
        .iter()
        .map(|&tab| if tab { FillPart::Tab } else { line("x") });
    let round = u32::try_from(r.asked.len()).unwrap() + 1;
    r.asked.push(round);
    t.q.fill(round, parts.collect());
}

/// The rounds of the answers in `heard`.
fn answers(heard: Vec<Heard>) -> impl Iterator<Item = u32> {
    heard.into_iter().filter_map(|h| match h {
        Heard::Typed(e) => Some(e.round),
        _ => None,
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn any_run_keeps_one_panel_open_at_most_and_answers_every_fill_once(steps in vec(step(), 1..24)) {
        let mut t = Two::new();
        let chord = (scans(&[CHORD.0]), Scan::new(CHORD.1));
        t.q.b.rig.fakes.shortcuts.set_history(Some(chord));
        let (mut r, mut answered) = (Run::default(), Vec::new());
        for step in steps {
            run(&mut t, step, &mut r);
            if r.stale {
                t.one_at_most();
            } else {
                t.shown();
            }
            answered.extend(answers(t.q.heard()));
        }
        for _ in 0..5 {
            t.q.relabel_in(super::gap_ms());
        }
        answered.extend(answers(t.q.heard()));
        answered.sort_unstable();
        prop_assert_eq!(answered, r.asked);
    }
}

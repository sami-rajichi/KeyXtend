//! Properties over any run of strip views, QML reads and fades, the module's hides and the keyboard's modes: the kept
//! strip is always the newest view's, a held tick never brings back an older round, a keyboard in the tray shows no
//! strip, and each change wakes QML once per read.

use super::{Sent, heard, sel, usual};
use crate::Rig;
use kx_module_api::Bus;
use kx_module_api::ports::{Board, ChipsView, Legend, Screen, SelChip, StripSpot};
use kx_ui::{Mode, Wake};
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;
use serde_json::Value;

/// Random runs per property.
const CASES: u32 = 128;
/// Most steps in one run.
const MAX_STEPS: usize = 40;

/// One thing the hub meets.
#[derive(Copy, Clone, Debug)]
enum Step {
    /// The module shows its next round: ticked with this keycap or not, by a sane line or not, with keycaps or none.
    View(Option<SelChip>, bool, bool),
    /// The module hides the strip.
    Hidden,
    /// QML reads the strip.
    Read,
    /// QML's fade ended for the newest round, moved by this much.
    Faded(i64),
    /// The keyboard goes into this mode.
    Mode(Mode),
}

fn step() -> impl Strategy<Value = Step> {
    let acted = proptest::option::of(select(SelChip::ALL.to_vec()));
    let modes = select(vec![Mode::Shown, Mode::Minimised, Mode::Away]);
    prop_oneof![
        4 => (acted, proptest::bool::weighted(0.8), proptest::bool::weighted(0.9))
            .prop_map(|(a, sane, chips)| Step::View(a, sane, chips)),
        1 => Just(Step::Hidden),
        3 => Just(Step::Read),
        2 => (-1..=1_i64).prop_map(Step::Faded),
        2 => modes.prop_map(Step::Mode),
    ]
}

/// What the run expects: the newest round shown, the strip kept with whether it is ticked, whether a wake waits for
/// QML's read, and whether the module was told the keyboard is in the tray.
#[derive(Default)]
struct Model {
    round: u32,
    kept: Option<(u32, bool)>,
    armed: bool,
    tray: bool,
}

impl Model {
    /// The hub keeps `now` after a view: the wakes it must give, one unless nothing showed before or after or a wake
    /// already waits.
    fn keep(&mut self, now: Option<(u32, bool)>) -> usize {
        let changed = now.is_some() || self.kept.is_some();
        self.kept = now;
        let wake = changed && !self.armed;
        self.armed |= wake;
        usize::from(wake)
    }
}

/// The strip of round `round`, ticked with `acted`, by the usual line when `sane` or on an unknown screen, with every
/// keycap when `chips` or none.
fn shown(round: u32, acted: Option<SelChip>, sane: bool, chips: bool) -> ChipsView {
    let spot = if sane {
        usual()
    } else {
        StripSpot {
            screen: Screen::default(),
            ..usual()
        }
    };
    let chips = if chips {
        SelChip::ALL.to_vec()
    } else {
        Vec::new()
    };
    ChipsView::Shown {
        round,
        chips,
        spot,
        acted,
        legend: Legend::Both,
    }
}

/// Plays `step` on the hub as the module, QML or the keyboard would, and returns the wakes the model expects.
fn play(
    rig: &Rig,
    bus: &Bus,
    sent: &Sent,
    model: &mut Model,
    step: Step,
) -> Result<usize, TestCaseError> {
    match step {
        // The module shows nothing while the keyboard is in the tray.
        Step::View(..) if model.tray => Ok(0),
        Step::View(acted, sane, chips) => {
            model.round += 1;
            bus.publish(&shown(model.round, acted, sane, chips))
                .unwrap();
            Ok(model.keep((sane && chips).then_some((model.round, acted.is_some()))))
        }
        Step::Hidden => {
            bus.publish(&ChipsView::Hidden).unwrap();
            Ok(model.keep(None))
        }
        Step::Read => {
            read(rig, model)?;
            Ok(0)
        }
        Step::Faded(by) => {
            rig.hub.sel_faded(i64::from(model.round) + by);
            if model
                .kept
                .is_some_and(|(r, ticked)| ticked && by == 0 && r == model.round)
            {
                model.kept = None;
            }
            Ok(0)
        }
        Step::Mode(mode) => moved(rig, bus, sent, model, mode),
    }
}

/// The keyboard goes into `mode`: the hub tells the place at a change, and the module hides the strip for the tray.
fn moved(
    rig: &Rig,
    bus: &Bus,
    sent: &Sent,
    model: &mut Model,
    mode: Mode,
) -> Result<usize, TestCaseError> {
    rig.hub.keep_mode(mode);
    let Some(&told) = sent.places().last() else {
        return Ok(0);
    };
    prop_assert_eq!(told, mode.board(), "the place told is the mode now");
    model.tray = told == Board::Tray;
    if !model.tray {
        return Ok(0);
    }
    bus.publish(&ChipsView::Hidden).unwrap();
    Ok(model.keep(None))
}

/// QML reads the strip: it is the newest round's or none, ticked as kept, and none while the keyboard is away.
fn read(rig: &Rig, model: &mut Model) -> Result<(), TestCaseError> {
    let json = sel(rig);
    model.armed = false;
    let away = rig.hub.mode() == Mode::Away;
    prop_assert!(
        !away || json.is_null(),
        "a keyboard in the tray shows a strip"
    );
    let Some((round, ticked)) = model.kept else {
        prop_assert_eq!(&json, &Value::Null);
        return Ok(());
    };
    prop_assert_eq!(round, model.round, "an older round shows");
    prop_assert_eq!(&json["round"], &Value::from(round));
    prop_assert_eq!(json["acted"].is_string(), ticked);
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    /// Any run keeps the newest round's strip or none, never brings back an older round under a held tick, shows none
    /// with the keyboard in the tray, and wakes QML once per read.
    #[test]
    fn the_newest_round_shows_none_in_the_tray_and_each_read_wakes_once(steps in vec(step(), 1..=MAX_STEPS)) {
        let rig = Rig::new();
        let (bus, sent, _subs) = heard(&rig);
        let mut model = Model::default();
        for step in steps.into_iter().chain([Step::Read]) {
            let want = play(&rig, &bus, &sent, &mut model, step)?;
            let woke = rig.wakes().into_iter().filter(|w| *w == Wake::Sel).count();
            prop_assert_eq!(woke, want, "{:?}", step);
        }
    }
}

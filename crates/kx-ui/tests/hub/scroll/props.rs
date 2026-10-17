//! Properties of the scroll state over random scroll changes, boxes ours or not and QML reads: it matches a model in
//! which a module that is not live is still with no outline, and changes after a read send one `Wake::Scroll`, never two.

use super::scroll;
use crate::assist::{heard, woke};
use kx_module_api::ports::{Dir, Rect, ScrollAimed, ScrollChanged};
use kx_ui::Wake;
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;
use serde_json::{Value, json};

/// Random runs per property.
const CASES: u32 = 256;
/// Most steps in one run.
const MAX_STEPS: usize = 40;
/// Farthest a box's corner goes on each axis, in pixels; small, so boxes repeat.
const REACH: i32 = 3;

/// One random step.
#[derive(Clone, Debug)]
enum Step {
    /// The scroll module's state.
    Changed(ScrollChanged),
    /// The box to outline, or none, and whether it is ours.
    Aimed(ScrollAimed),
    /// QML reads the state.
    Read,
}

fn dir() -> impl Strategy<Value = Option<Dir>> {
    prop_oneof![Just(None), select(Dir::ALL.to_vec()).prop_map(Some)]
}

fn area() -> impl Strategy<Value = Option<Rect>> {
    let rect = (0..=REACH, 0..=REACH).prop_map(|(x, y)| Rect { x, y, w: 1, h: 1 });
    prop_oneof![Just(None), rect.prop_map(Some)]
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        2 => (any::<bool>(), dir(), any::<bool>()).prop_map(|(live, dir, fast)| {
            Step::Changed(ScrollChanged { live, dir, fast })
        }),
        3 => (area(), any::<bool>()).prop_map(|(area, ours)| Step::Aimed(ScrollAimed { area, ours })),
        2 => Just(Step::Read),
    ]
}

/// What the hub should show, as its docs say.
#[derive(Default)]
struct Model {
    now: Option<ScrollChanged>,
    area: Option<Rect>,
    ours: bool,
}

impl Model {
    fn play(&mut self, step: &Step) {
        match step {
            Step::Changed(now) => {
                self.now = Some(*now);
                if !now.live {
                    (self.area, self.ours) = (None, false);
                }
            }
            Step::Aimed(aimed) if self.now.is_some_and(|n| n.live) => {
                (self.area, self.ours) = (aimed.area, aimed.ours && aimed.area.is_some());
            }
            Step::Aimed(_) | Step::Read => {}
        }
    }

    fn json(&self) -> Value {
        let now = self.now.filter(|n| n.live).unwrap_or(ScrollChanged::OFF);
        json!({ "live": now.live, "dir": now.dir, "fast": now.fast, "area": self.area, "ours": self.ours })
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn the_state_follows_the_model_and_one_wake_waits_per_read(
        steps in vec(step(), 1..=MAX_STEPS),
    ) {
        let (rig, bus, _subs) = heard();
        prop_assert_eq!(scroll(&rig), Model::default().json());
        rig.wakes();
        let (mut model, mut woken, mut changed) = (Model::default(), 0, false);
        for step in steps.iter().chain(&[Step::Read]) {
            match step {
                Step::Changed(now) => bus.publish(now).unwrap(),
                Step::Aimed(aimed) => bus.publish(aimed).unwrap(),
                Step::Read => {}
            }
            let before = model.json();
            model.play(step);
            changed |= model.json() != before;
            woken += woke(&rig, Wake::Scroll);
            prop_assert!(woken <= 1, "two wakes before a read: {steps:?}");
            if !matches!(step, Step::Read) {
                continue;
            }
            prop_assert_eq!(scroll(&rig), model.json());
            prop_assert_eq!(woken, usize::from(changed), "one wake for the changes since the last read");
            (woken, changed) = (0, false);
        }
    }
}

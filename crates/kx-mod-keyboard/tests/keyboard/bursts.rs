//! Any run of presses, ticks, let-gos and leaves on the face (spec §4.6): a tick types only while a key is held, and
//! no latch outlives its burst.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

use crate::held::{A, times};
use crate::host::GOLDEN;
use crate::rig::Rig;
use proptest::collection::vec;
use proptest::prelude::*;

/// Keys a run holds: a letter, Backspace, a latch and a key that never repeats.
const KEYS: [&str; 4] = [A, "bksp", "shift", "esc"];

/// One pointer or timer input on the face.
#[derive(Clone, Debug)]
enum Input {
    /// The pointer goes down on a key of `KEYS`.
    Press(usize),
    /// The hold timer ticks this many ms later.
    Tick(u64),
    /// The pointer lets go of a key of `KEYS`.
    LetGo(usize),
    /// Qt takes the press away, or the keyboard hides.
    Leave,
}

/// After a let-go or leave of the press `held` made: once that press typed, no latch is left down by its burst.
fn latches_went(r: &Rig, held: Option<(usize, usize)>) -> Result<(), TestCaseError> {
    let Some((k, sent)) = held else {
        return Ok(());
    };
    let typed = r.fakes.injector.strokes().len() > sent;
    let shift = r.state()["keys"]["shift"]["on"].clone();
    prop_assert!(
        KEYS[k] == "shift" || !typed || shift == false,
        "Shift outlived the burst"
    );
    Ok(())
}

fn input() -> impl Strategy<Value = Input> {
    let (delay, _) = times();
    prop_oneof![
        (0..KEYS.len()).prop_map(Input::Press),
        (0..delay).prop_map(Input::Tick),
        (0..delay).prop_map(Input::Tick),
        (0..KEYS.len()).prop_map(Input::LetGo),
        Just(Input::Leave),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn a_burst_never_outlives_the_press(shift in any::<bool>(), inputs in vec(input(), 0..40)) {
        let r = Rig::new(&GOLDEN, None);
        if shift {
            r.tap("shift").unwrap();
        }
        // The key held and how many strokes had gone out when it went down.
        let mut held: Option<(usize, usize)> = None;
        for i in inputs {
            let sent = r.fakes.injector.strokes().len();
            match i {
                Input::Press(k) => held = r.press(KEYS[k]).is_ok().then_some((k, sent)),
                Input::Tick(ms) => {
                    r.tick_in(ms);
                    let now = r.fakes.injector.strokes().len();
                    prop_assert!(held.is_some() || now == sent, "a tick typed with nothing held");
                }
                Input::LetGo(k) => {
                    r.let_go(KEYS[k]).unwrap();
                    latches_went(&r, held.take())?;
                }
                Input::Leave => {
                    r.leave();
                    latches_went(&r, held.take())?;
                }
            }
        }
    }
}

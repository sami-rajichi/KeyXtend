//! G2's clicks as the owner makes them (ADR-0024): the button held for `[g2] press_ms` with a slip of `slip_px` inside
//! the key while it is down, so a keyboard that drops a slipped or held click fails the gate.
#![cfg(windows)]

use windows::Win32::Foundation::{POINT, RECT};

use crate::mouse;
use crate::win::sleep_ms;

/// One step of a click as a hand makes it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Step {
    /// The pointer goes to the point and the button goes down there.
    Press(POINT),
    /// The pointer slips to the point, the button still down.
    Slip(POINT),
    /// The pointer goes to the point and the button comes up there.
    Release(POINT),
    /// The hand rests this many ms.
    Wait(u64),
}

/// Where a click on the key boxed by `key` at `at` slips to: `slip_px` right and down, or `None` off the key.
pub fn slip_to(at: POINT, key: RECT, slip_px: i32) -> Option<POINT> {
    let to = POINT {
        x: at.x + slip_px,
        y: at.y + slip_px,
    };
    let inside = (key.left..key.right).contains(&to.x) && (key.top..key.bottom).contains(&to.y);
    inside.then_some(to)
}

/// A click from `at` to `to`: down, half the hold, the slip, the rest of the hold, then up where it slipped.
pub fn steps(at: POINT, to: POINT, press_ms: u64) -> [Step; 5] {
    let half = press_ms / 2;
    [
        Step::Press(at),
        Step::Wait(half),
        Step::Slip(to),
        Step::Wait(press_ms - half),
        Step::Release(to),
    ]
}

/// Makes `steps` with `act`; when one fails with the button down, it still lets go where the pointer last went, so
/// nothing stays pressed.
pub fn perform(
    steps: &[Step],
    mut act: impl FnMut(Step) -> Result<(), String>,
) -> Result<(), String> {
    let mut down = None;
    for &step in steps {
        if let Err(e) = act(step) {
            let Some(at) = down else {
                return Err(e);
            };
            return match act(Step::Release(at)) {
                Ok(()) => Err(e),
                Err(r) => Err(format!("{e}; letting go: {r}")),
            };
        }
        down = match step {
            Step::Press(p) | Step::Slip(p) => Some(p),
            Step::Release(_) => None,
            Step::Wait(_) => down,
        };
    }
    Ok(())
}

/// Makes `step` with the real mouse; a release after a mouse-guard stop still goes out, on the key.
pub fn act(step: Step) -> Result<(), String> {
    match step {
        Step::Press(p) => mouse::press_at(p),
        Step::Slip(p) => mouse::move_to(p),
        Step::Release(p) => mouse::release_at(p),
        Step::Wait(ms) => {
            sleep_ms(ms);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AT: POINT = POINT { x: 100, y: 200 };
    const TO: POINT = POINT { x: 102, y: 202 };
    /// A key's box around `AT`, 20 px each way.
    const KEY: RECT = RECT {
        left: 80,
        top: 180,
        right: 120,
        bottom: 220,
    };

    #[test]
    fn a_click_holds_for_press_ms_and_slips_inside_the_key_before_it_lets_go() {
        let want = [
            Step::Press(AT),
            Step::Wait(200),
            Step::Slip(TO),
            Step::Wait(201),
            Step::Release(TO),
        ];
        assert_eq!(steps(AT, TO, 401), want, "the waits add up to the hold");
    }

    #[test]
    fn a_slip_goes_right_and_down_and_never_leaves_the_key() {
        assert_eq!(slip_to(AT, KEY, 2), Some(TO));
        assert_eq!(slip_to(AT, KEY, 19), Some(POINT { x: 119, y: 219 }));
        assert_eq!(slip_to(AT, KEY, 20), None, "on the key's far edge");
    }

    /// Makes `steps`, failing at the step `fails` names, and answers what was done and the result.
    fn made(fails: Option<usize>) -> (Vec<Step>, Result<(), String>) {
        let mut done = Vec::new();
        let result = perform(&steps(AT, TO, 400), |s| {
            done.push(s);
            if fails == Some(done.len() - 1) {
                Err("refused".to_string())
            } else {
                Ok(())
            }
        });
        (done, result)
    }

    #[test]
    fn every_step_is_made_in_order() {
        let (done, result) = made(None);
        assert_eq!(result, Ok(()));
        assert_eq!(done, steps(AT, TO, 400));
    }

    #[test]
    fn a_step_that_fails_with_the_button_down_still_lets_go_where_the_pointer_last_went() {
        let (done, result) = made(Some(2));
        assert_eq!(result, Err("refused".to_string()));
        let want = [
            Step::Press(AT),
            Step::Wait(200),
            Step::Slip(TO),
            Step::Release(AT),
        ];
        assert_eq!(done, want, "the slip never went out");
        let (done, _) = made(Some(3));
        assert_eq!(done.last(), Some(&Step::Release(TO)), "after the slip");
    }

    #[test]
    fn a_press_that_fails_lets_go_of_nothing() {
        let (done, result) = made(Some(0));
        assert!(result.is_err());
        assert_eq!(done, [Step::Press(AT)]);
    }
}

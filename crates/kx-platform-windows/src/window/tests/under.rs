//! Laying our windows just under another, as the outline goes just under the keyboard, on hidden windows only.

use std::sync::PoisonError;

use kx_platform::ui::{UiError, WindowGuard};

use super::super::guard::guard;
use super::super::top::below;
use super::super::win::top_level;
use super::z_order::Z_ORDER;
use super::{Hidden, WinGuard};

/// True when `b` comes right after `a` in one snapshot of the z-order.
fn just_under(b: &Hidden, a: &Hidden) -> bool {
    let z = top_level();
    let at = |w: &Hidden| z.iter().position(|&x| x == w.0).unwrap();
    at(b) == at(a) + 1
}

#[test]
fn a_window_goes_just_under_another_and_stays_there_guarded() {
    let _z = Z_ORDER.lock().unwrap_or_else(PoisonError::into_inner);
    let keyboard = Hidden::topmost("KeyXtend test: over", None);
    let outline = Hidden::topmost("KeyXtend test: under", None);
    let other = Hidden::topmost("KeyXtend test: between", None);
    below(outline.0, keyboard.0).unwrap();
    assert!(just_under(&outline, &keyboard));
    guard(keyboard.0).unwrap();
    guard(outline.0).unwrap();
    below(other.0, keyboard.0).unwrap();
    below(outline.0, keyboard.0).unwrap();
    assert!(just_under(&outline, &keyboard), "the guard leaves it there");
    assert!(just_under(&other, &outline));
}

#[test]
fn hidden_windows_cannot_be_laid_under() {
    let _over = Hidden::topmost("KeyXtend test: hidden over", None);
    let _under = Hidden::topmost("KeyXtend test: hidden under", None);
    let laid = WinGuard.lay_under("KeyXtend test: hidden under", "KeyXtend test: hidden over");
    assert_eq!(laid, Err(UiError::NoWindow));
}

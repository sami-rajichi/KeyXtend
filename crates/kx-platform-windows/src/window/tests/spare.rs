//! A guard run spares Snip's windows (ADR-0059), on hidden windows only: while a system dialog shows, Qt drops them out
//! of the topmost band, and no guard may lift a frozen screen back over the dialog and the keyboard.

use std::sync::PoisonError;

use windows::Win32::UI::WindowsAndMessaging::HWND_NOTOPMOST;

use super::super::guard::guarded;
use super::super::win::guard_these;
use super::Hidden;
use super::z_order::{Z_ORDER, above, listed, put_after, topmost};

/// The test's frozen screen and Snip panel, whose titles the guard run spares.
const SCREEN: &str = "KeyXtend test: spared screen";
const PANEL: &str = "KeyXtend test: spared panel";

#[test]
fn a_spared_frozen_screen_dropped_out_of_the_band_stays_out_and_under_the_keyboard() {
    let _z = Z_ORDER.lock().unwrap_or_else(PoisonError::into_inner);
    let kb = Hidden::topmost("KeyXtend test: spare keyboard", None);
    let screen = Hidden::topmost(SCREEN, None);
    assert_eq!(guard_these(listed(&[&kb, &screen]), &[SCREEN], None), Ok(1));
    assert!(guarded(kb.0) && !guarded(screen.0), "only the keyboard");
    put_after(&screen, HWND_NOTOPMOST);
    assert!(!topmost(&screen), "it stays out of the band");
    assert!(topmost(&kb) && above(&kb, &screen));
}

/// The trap the spare ends: a guarded frozen screen dropped as a dialog shows goes back over the keyboard.
#[test]
fn a_guarded_frozen_screen_dropped_out_of_the_band_comes_back_over_the_keyboard() {
    let _z = Z_ORDER.lock().unwrap_or_else(PoisonError::into_inner);
    let kb = Hidden::topmost("KeyXtend test: unspared keyboard", None);
    let screen = Hidden::topmost(SCREEN, None);
    guard_these(listed(&[&kb, &screen]), &[], None).unwrap();
    put_after(&screen, HWND_NOTOPMOST);
    assert!(topmost(&screen) && above(&screen, &kb));
}

/// The panel's shadow owns it and its tip is owned by it; the shadow's title is every shadow's, as Qt makes them.
#[test]
fn a_spared_windows_shadow_and_tip_are_spared_and_all_stay_out_of_the_band() {
    let _z = Z_ORDER.lock().unwrap_or_else(PoisonError::into_inner);
    let kb = Hidden::topmost("KeyXtend test: spare keyboard two", None);
    let shadow = Hidden::tip("KeyXtend test: spare shadow", None);
    let panel = Hidden::topmost(PANEL, Some(shadow.0));
    let tip = Hidden::tip("KeyXtend test: spare tip", Some(panel.0));
    let all = [&kb, &shadow, &panel, &tip];
    assert_eq!(guard_these(listed(&all), &[PANEL], None), Ok(1));
    put_after(&panel, HWND_NOTOPMOST);
    put_after(&shadow, HWND_NOTOPMOST);
    for (window, name) in [(&shadow, "shadow"), (&panel, "panel"), (&tip, "tip")] {
        assert!(!guarded(window.0), "the {name} is spared");
        assert!(!topmost(window), "the {name} stays out of the band");
        assert!(above(&kb, window), "the {name} stays under the keyboard");
    }
}

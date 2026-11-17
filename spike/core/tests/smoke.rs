//! Smoke checks that need no window and send no input.

use kx_mod_keyboard::BoardSpec;
use kx_module_api::ports::LayoutSource;
use kx_platform_windows::WinLayouts;

/// Letters every layout types on the board's character keys, at least.
const LETTERS: usize = 26;

#[test]
fn settings_load_and_the_board_keys_type_in_the_front_layout() {
    spike_core::config::load().unwrap();
    let spec = BoardSpec::shipped().unwrap();
    let scans = spec.char_scans();
    let installed = WinLayouts.installed();
    let front = WinLayouts
        .foreground()
        .or_else(|| installed.first().copied());
    let chars = WinLayouts.keys(front.expect("a layout"), &scans);
    assert_eq!(chars.len(), scans.len());
    let typing = chars.iter().filter(|c| !c.base.is_empty()).count();
    assert!(typing >= LETTERS, "only {typing} keys type");
    assert!(!installed.is_empty());
}

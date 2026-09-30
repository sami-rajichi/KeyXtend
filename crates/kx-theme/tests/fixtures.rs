//! Writes the look fixtures the QML tests read (P3 Task 17) from the shipped tokens, so QML tests never hardcode a
//! value. A token change fails this test until it runs with `KX_BLESS=1`, which rewrites `fixtures/`.

#![allow(
    clippy::expect_used,
    reason = "these helpers run only in tests, where a failed setup should fail the test"
)]

use std::path::Path;

use kx_platform::ui::{SysColors, SysLook};
use kx_test_support::golden::assert_golden;
use kx_theme::contrast::WCAG_AA;
use kx_theme::{LookChoice, Mode, ModeChoice, Themes, look_view};

/// The fixtures folder, under this crate.
const DIR: &str = "fixtures";
/// What starts each fixture, so QML imports it as a JavaScript library and reads `lk`.
const HEAD: &str = ".pragma library\n// Written by kx-theme's fixtures test from the tokens; do not edit.\nvar lk = ";
/// What ends each fixture.
const TAIL: &str = ";\n";
/// A sample high-contrast set, as `[r, g, b, a]`.
const HC: SysColors = SysColors {
    window: [0x00, 0x00, 0x00, 0xFF],
    window_text: [0xFF, 0xFF, 0xFF, 0xFF],
    highlight: [0x1A, 0xEB, 0xFF, 0xFF],
    highlight_text: [0x00, 0x00, 0x00, 0xFF],
    button: [0x00, 0x00, 0x00, 0xFF],
    button_text: [0xFF, 0xFF, 0x00, 0xFF],
    gray_text: [0x3F, 0xF2, 0x3F, 0xFF],
};

/// Theme `id` in `mode`, keeping its own colours, with full motion.
fn choice(id: &str, mode: Mode) -> LookChoice {
    let mode = match mode {
        Mode::Light => ModeChoice::Light,
        Mode::Dark => ModeChoice::Dark,
    };
    LookChoice {
        theme: id.into(),
        mode,
        follow_accent: false,
        reduced_motion: false,
        min_contrast: WCAG_AA,
    }
}

/// Checks fixture `name` against the look for `choice` on `sys`, or writes it when blessing.
fn fixture(t: &Themes, name: &str, choice: &LookChoice, sys: &SysLook) {
    let view = look_view(t, choice, sys).expect("a shipped theme");
    let json = serde_json::to_string_pretty(&view).expect("the look is JSON");
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join(DIR).join(name);
    assert_golden(file, &format!("{HEAD}{json}{TAIL}"));
}

#[test]
fn the_qml_look_fixtures_match_the_tokens() {
    let t = Themes::shipped().expect("tokens load");
    let plain = SysLook::default();
    for theme in &t.list {
        for mode in Mode::ALL {
            let id = &theme.look.id;
            let name = format!("look-{id}-{}.js", mode.key());
            fixture(&t, &name, &choice(id, mode), &plain);
        }
    }
    let reduced = LookChoice {
        reduced_motion: true,
        ..choice("native", Mode::Light)
    };
    fixture(&t, "look-native-light-reduced.js", &reduced, &plain);
    let hc = SysLook {
        contrast: Some(HC),
        ..plain
    };
    fixture(
        &t,
        "look-native-light-contrast.js",
        &choice("native", Mode::Light),
        &hc,
    );
}

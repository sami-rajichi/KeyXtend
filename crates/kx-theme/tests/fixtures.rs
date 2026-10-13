//! Writes the look fixtures the QML tests read (P3 Task 17) from the shipped tokens, so QML tests never hardcode a
//! value. A token change fails this test until it runs with `KX_BLESS=1`, and so does a stray file in `fixtures/`.

#![allow(
    clippy::expect_used,
    reason = "these helpers run only in tests, where a failed setup should fail the test"
)]

use std::path::{Path, PathBuf};

use kx_platform::ui::SysLook;
use kx_test_support::golden::assert_golden;
use kx_theme::contrast::WCAG_AA;
use kx_theme::{LookChoice, Mode, ModeChoice, Themes, look_view};

/// The sample high-contrast set the unit tests use.
#[path = "../src/hc/sample.rs"]
mod sample;

/// The fixtures folder, under this crate.
const DIR: &str = "fixtures";
/// What starts each fixture, so QML imports it as a JavaScript library and reads `lk`.
const HEAD: &str = ".pragma library\n// Written by kx-theme's fixtures test from the tokens; do not edit.\nvar lk = ";
/// What ends each fixture.
const TAIL: &str = ";\n";

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

/// The fixtures folder.
fn dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(DIR)
}

/// Checks fixture `name` against the look for `choice` on `sys`, or writes it when blessing; returns the name.
fn fixture(t: &Themes, name: String, choice: &LookChoice, sys: &SysLook) -> String {
    let view = look_view(t, choice, sys, None).expect("a shipped theme");
    let json = serde_json::to_string_pretty(&view).expect("the look is JSON");
    assert_golden(dir().join(&name), &format!("{HEAD}{json}{TAIL}"));
    name
}

/// Fails on a file in the fixtures folder that no look writes, such as one left by a removed theme.
fn no_orphans(written: &[String]) {
    let files = std::fs::read_dir(dir()).expect("the fixtures folder reads");
    for f in files.flatten() {
        let name = f.file_name().to_string_lossy().into_owned();
        assert!(
            written.contains(&name),
            "{DIR}/{name} is no longer written; delete it"
        );
    }
}

#[test]
fn the_qml_look_fixtures_match_the_tokens() {
    let t = Themes::shipped().expect("tokens load");
    let plain = SysLook::default();
    let mut written = Vec::new();
    for theme in &t.list {
        for mode in Mode::ALL {
            let id = &theme.look.id;
            let name = format!("look-{id}-{}.js", mode.key());
            written.push(fixture(&t, name, &choice(id, mode), &plain));
        }
    }
    let reduced = LookChoice {
        reduced_motion: true,
        ..choice("native", Mode::Light)
    };
    written.push(fixture(
        &t,
        "look-native-light-reduced.js".into(),
        &reduced,
        &plain,
    ));
    let hc = SysLook {
        contrast: Some(sample::HC),
        ..plain
    };
    let native = choice("native", Mode::Light);
    written.push(fixture(
        &t,
        "look-native-light-contrast.js".into(),
        &native,
        &hc,
    ));
    no_orphans(&written);
}

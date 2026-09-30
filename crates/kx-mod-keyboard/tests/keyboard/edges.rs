//! Typing at the edges: a send the system takes in part, a table that lags the app in front, two dead keys
//! in a row, and a settings change while modifiers are latched.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

use crate::typing::{Rig, chord, tap, text, unicode};
use kx_mod_keyboard::{KeyboardSettings, Latch, Modifiers, Tapped};
use kx_module_api::ports::{FaceError, InjectError, LayoutId, TextRoute};
use kx_platform_fake::{FakeLayoutSource, LayoutFile};

/// A refusal after one of four events went out.
const PART: InjectError = InjectError::Refused { sent: 1, total: 4 };

#[test]
fn a_dead_key_taken_in_part_starts_the_wait_and_uses_the_latches() {
    let mut r = Rig::new("fr-fr", |_| ());
    r.tap("altgr").unwrap();
    r.injector.set_error(Some(PART));
    assert_eq!(r.char(0x03), Err(FaceError::Inject(PART)), "still shown");
    assert!(r.typing.waiting(), "the app may wait for the accent");
    assert!(!r.typing.latches().is_on(Latch::AltGr));
    r.injector.set_error(None);
    r.char(0x12).unwrap();
    assert_eq!(r.sent(), [tap(0x12)], "the letter goes as a key press");
}

#[test]
fn text_taken_in_part_uses_the_latches_so_a_retry_is_not_shifted_again() {
    let mut r = Rig::new("ar-tn", |_| ());
    r.tap("shift").unwrap();
    r.injector.set_error(Some(PART));
    assert_eq!(r.char(0x30), Err(FaceError::Inject(PART)));
    assert!(!r.typing.latches().is_on(Latch::Shift));
    r.injector.set_error(None);
    r.char(0x30).unwrap();
    assert_eq!(r.sent(), [unicode("\u{644}\u{627}")], "لا, not لآ");
}

/// The id of golden layout `name`.
fn id(name: &str) -> LayoutId {
    LayoutFile::golden(name).unwrap().id
}

#[test]
fn a_click_whose_result_needs_the_apps_layout_waits_while_the_table_lags() {
    let mut r = Rig::new("fr-fr", |_| ());
    r.source = FakeLayoutSource::golden(&["fr-fr", "en-us"]).unwrap();
    r.source.set_foreground(Some(id("en-us")));
    assert_eq!(r.char(0x1A), Ok(Tapped::Stale), "EN would type [");
    assert!(!r.typing.waiting());
    r.tap("ctrl").unwrap();
    assert_eq!(r.char(0x10), Ok(Tapped::Stale), "Ctrl+A would be Ctrl+Q");
    assert!(r.typing.latches().is_on(Latch::Ctrl), "nothing went out");
    r.tap("ctrl").unwrap();
    assert_eq!(r.char(0x12), Ok(Tapped::Done), "text types what it shows");
    assert_eq!(r.sent(), [unicode("e")]);
}

#[test]
fn with_no_installed_layout_in_front_the_table_shown_stands() {
    let mut r = Rig::new("fr-fr", |_| ());
    r.source.set_foreground(None);
    assert_eq!(
        r.char(0x1A),
        Ok(Tapped::Done),
        "no app in front, or our own window"
    );
    r.source.set_foreground(Some(LayoutId::new(1)));
    assert_eq!(
        r.char(0x12),
        Ok(Tapped::Done),
        "a layout Follow cannot show"
    );
    assert_eq!(r.sent(), [tap(0x1A), tap(0x12)]);
}

#[test]
fn two_dead_keys_in_a_row_keep_the_wait_until_a_letter() {
    let mut r = Rig::new("fr-fr", |_| ());
    r.char(0x1A).unwrap();
    r.tap("altgr").unwrap();
    r.char(0x03).unwrap();
    assert!(r.typing.waiting(), "^ then the dead ~");
    r.char(0x12).unwrap();
    assert!(!r.typing.waiting());
    let want = [tap(0x1A), chord(&[0xE038], 0x03), tap(0x12)];
    assert_eq!(r.sent(), want);
}

#[test]
fn a_settings_change_keeps_the_latched_modifiers() {
    let mut r = Rig::new("en-us", |s| s.modifiers = Modifiers::Stay);
    r.tap("shift").unwrap();
    let mut settings = KeyboardSettings::from_defaults().unwrap();
    settings.key_apps = vec!["notepad.exe".into()];
    r.typing.update(&settings);
    assert!(
        r.typing.latches().is_on(Latch::Shift),
        "kept across the change"
    );
    r.injector.set_target(Some("Notepad.exe"));
    r.char(0x10).unwrap();
    assert!(!r.typing.latches().is_on(Latch::Shift), "now release");
    assert_eq!(r.sent(), [text("Q", TextRoute::Keys)]);
}

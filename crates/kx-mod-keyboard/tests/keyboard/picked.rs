//! A picked word on the typing state (ADR-0033 decision 2): Backspaces, then the text by the route a letter takes now;
//! refused whole while Ctrl, Alt or Win is latched or past `text_cap`, and told as one `Erase` per Backspace and one
//! `Text`.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

use crate::told::text;
use crate::typing::{Rig, tap, unicode};
use kx_mod_keyboard::{Io, Latch};
use kx_module_api::Redacted;
use kx_module_api::ports::{InjectError, Piece, TextRoute};
use kx_platform_fake::Stroke;

/// Backspace's scan code.
const BKSP: u16 = 0x0E;
/// The smallest `text_cap` the settings take.
const CAP: u16 = 32;

/// Types a picked word on `r`: `erase` Backspaces, then `t`; true when any of it reached the app.
fn word(r: &mut Rig, erase: u16, t: &str) -> bool {
    let io = Io {
        injector: &r.injector,
        source: &r.source,
        table: &r.table,
        shortcuts: &r.shortcuts,
        layer: &[],
    };
    r.typing.type_word(&io, erase, &Redacted::new(t.into()))
}

#[test]
fn a_word_erases_then_types_as_unicode_and_is_told_so() {
    let mut r = Rig::new("en-us", |_| ());
    assert!(word(&mut r, 3, "hello "));
    let want = [tap(BKSP), tap(BKSP), tap(BKSP), unicode("hello ")];
    assert_eq!(r.sent(), want);
    let told = [Piece::Erase, Piece::Erase, Piece::Erase, text("hello ")];
    assert_eq!(r.typing.told(), told);
}

#[test]
fn a_listed_app_gets_the_word_as_key_presses() {
    let mut r = Rig::new("en-us", |s| s.key_apps = vec!["app.exe".into()]);
    r.injector.set_target(Some("APP.EXE"));
    word(&mut r, 1, "hi");
    let keys = Stroke::Text(Redacted::new("hi".into()), TextRoute::Keys);
    assert_eq!(r.sent(), [tap(BKSP), keys]);
}

#[test]
fn ctrl_alt_or_win_latched_refuses_the_word_whole() {
    for latch in ["ctrl", "alt", "win"] {
        let mut r = Rig::new("en-us", |_| ());
        r.tap(latch).unwrap();
        assert!(!word(&mut r, 2, "hello"), "{latch}");
        assert!(r.sent().is_empty() && r.typing.told().is_empty(), "{latch}");
        assert!(!r.typing.latches().held(&[]).is_empty(), "{latch} stays");
    }
}

#[test]
fn a_one_shot_shift_lets_go_after_the_word_and_leaves_its_letters_as_picked() {
    let mut r = Rig::new("en-us", |_| ());
    r.tap("shift").unwrap();
    assert!(word(&mut r, 0, "hello"));
    assert_eq!(r.sent(), [unicode("hello")]);
    assert!(!r.typing.latches().is_on(Latch::Shift));
}

#[test]
fn a_word_ends_a_dead_keys_wait() {
    let mut r = Rig::new("fr-fr", |_| ());
    r.char(0x1A).unwrap();
    assert!(r.typing.waiting());
    word(&mut r, 0, "été");
    assert!(!r.typing.waiting());
}

#[test]
fn an_erase_or_a_text_past_the_cap_types_nothing_and_the_cap_counts_characters() {
    let mut r = Rig::new("en-us", |s| s.text_cap = CAP);
    let long = "a".repeat(usize::from(CAP) + 1);
    assert!(!word(&mut r, CAP + 1, ""));
    assert!(!word(&mut r, 0, &long));
    assert!(r.sent().is_empty() && r.typing.told().is_empty());
    let wide = "😀".repeat(usize::from(CAP));
    assert!(word(&mut r, CAP, &wide), "64 UTF-16 units, 32 characters");
    assert_eq!(r.sent().len(), usize::from(CAP) + 1);
}

#[test]
fn a_refused_word_tells_nothing_and_keeps_the_latches() {
    let mut r = Rig::new("en-us", |_| ());
    r.tap("shift").unwrap();
    r.injector
        .set_error(Some(InjectError::Refused { sent: 0, total: 2 }));
    assert!(!word(&mut r, 1, "hi"));
    assert!(r.typing.told().is_empty());
    assert!(r.typing.latches().is_on(Latch::Shift));
}

#[test]
fn an_empty_word_types_nothing() {
    let mut r = Rig::new("en-us", |_| ());
    r.tap("shift").unwrap();
    assert!(!word(&mut r, 0, ""));
    assert!(r.sent().is_empty());
    assert!(r.typing.latches().is_on(Latch::Shift));
}

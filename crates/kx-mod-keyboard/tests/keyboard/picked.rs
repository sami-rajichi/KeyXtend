//! A picked word on the typing state (ADR-0033 decision 2): Backspaces, then the text by the route a letter takes now;
//! refused whole while a shortcut latch is on or a dead key waits, for a hidden mark or past `text_cap`, and
//! told as one `Erase` per Backspace and one `Text`.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

use crate::told::{BKSP, text};
use crate::typing::{Rig, tap, unicode};
use kx_mod_keyboard::{Io, KeyboardSettings, Latch, Pause, TEXT_CAP};
use kx_module_api::Redacted;
use kx_module_api::ports::{InjectError, Piece, TextRoute};
use kx_platform_fake::{FakeInjector, Stroke};
use std::sync::Mutex;
use std::time::Duration;

/// The smallest `text_cap` the settings take.
const CAP: u16 = *TEXT_CAP.start();

/// Each wait a picked word asks for, with how many strokes had gone out by then; it never sleeps.
struct Waits<'a> {
    injector: &'a FakeInjector,
    seen: Mutex<Vec<(usize, Duration)>>,
}

impl Pause for Waits<'_> {
    fn wait(&self, gap: Duration) {
        let sent = self.injector.strokes().len();
        self.seen.lock().unwrap().push((sent, gap));
    }
}

/// Types a picked word on `r`: `erase` Backspaces, then `t`; true when any of it reached the app.
fn word(r: &mut Rig, erase: u16, t: &str) -> bool {
    timed(r, erase, t).0
}

/// Types a picked word on `r` as `word` does, and gives the waits it asked for.
fn timed(r: &mut Rig, erase: u16, t: &str) -> (bool, Vec<(usize, Duration)>) {
    let waits = Waits {
        injector: &r.injector,
        seen: Mutex::default(),
    };
    let io = Io {
        injector: &r.injector,
        source: &r.source,
        table: &r.table,
        shortcuts: &r.shortcuts,
        layer: &[],
    };
    let went = r
        .typing
        .type_word(&io, erase, &Redacted::new(t.into()), &waits);
    (went, waits.seen.into_inner().unwrap())
}

/// `ms` milliseconds, the `[keyboard] erase_gap_ms` unit.
fn ms(ms: u64) -> Duration {
    Duration::from_millis(ms)
}

#[test]
fn a_pick_erasing_three_waits_after_each_backspace_and_types_after_the_last_wait() {
    let mut r = Rig::new("en-us", |_| ());
    let gap = ms(KeyboardSettings::from_defaults().unwrap().erase_gap_ms);
    let (went, waits) = timed(&mut r, 3, "hello ");
    assert!(went);
    assert_eq!(waits, [(1, gap), (2, gap), (3, gap)], "after each tap");
    let taps = [tap(BKSP), tap(BKSP), tap(BKSP), unicode("hello ")];
    assert_eq!(r.sent(), taps, "the text after the last wait");
}

#[test]
fn a_pick_that_erases_nothing_or_a_zero_gap_waits_for_nothing() {
    let mut r = Rig::new("en-us", |_| ());
    assert_eq!(timed(&mut r, 0, "lp "), (true, Vec::new()), "no erase");
    let mut r = Rig::new("en-us", |s| s.erase_gap_ms = 0);
    assert_eq!(timed(&mut r, 2, "hi"), (true, Vec::new()), "a zero gap");
    assert_eq!(r.sent(), [tap(BKSP), tap(BKSP), unicode("hi")]);
}

#[test]
fn a_new_gap_takes_effect_through_update() {
    let mut r = Rig::new("en-us", |_| ());
    let mut s = KeyboardSettings::from_defaults().unwrap();
    s.erase_gap_ms = *kx_mod_keyboard::ERASE_GAP_MS.end();
    r.typing.update(&s);
    let (_, waits) = timed(&mut r, 1, "a");
    assert_eq!(waits, [(1, ms(s.erase_gap_ms))]);
}

#[test]
fn a_word_erases_then_types_as_unicode_and_is_told_so() {
    let mut r = Rig::new("en-us", |_| ());
    assert!(word(&mut r, 3, "hello "));
    let want = [tap(BKSP), tap(BKSP), tap(BKSP), unicode("hello ")];
    assert_eq!(r.sent(), want);
    let told = [Piece::Erase, Piece::Erase, Piece::Erase, text("hello ")];
    assert_eq!(r.typing.take_pieces(), told);
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
        assert!(
            r.sent().is_empty() && r.typing.take_pieces().is_empty(),
            "{latch}"
        );
        assert!(!r.typing.latches().held(&[]).is_empty(), "{latch} stays");
    }
}

#[test]
fn altgr_on_a_layout_without_altgr_characters_refuses_the_word_as_right_alt() {
    let mut r = Rig::new("en-us", |_| ());
    r.tap("altgr").unwrap();
    assert!(!word(&mut r, 1, "hi"));
    assert!(r.sent().is_empty());
    let mut fr = Rig::new("fr-fr", |_| ());
    fr.tap("altgr").unwrap();
    assert!(word(&mut fr, 0, "hi"), "AltGr types characters here");
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
fn a_dead_key_waiting_refuses_the_word_whole() {
    // The app still holds the accent, so Backspaces and text would compose with it.
    let mut r = Rig::new("fr-fr", |_| ());
    r.char(0x1A).unwrap();
    r.sent();
    r.typing.take_pieces();
    assert!(!word(&mut r, 1, "été"));
    assert!(r.sent().is_empty() && r.typing.take_pieces().is_empty());
    assert!(r.typing.waiting(), "the accent still waits");
}

#[test]
fn a_control_character_in_the_text_refuses_the_word_whole() {
    for t in ["hi\n", "a\tb", "\r", "x\u{7f}", "y\u{85}"] {
        let mut r = Rig::new("en-us", |_| ());
        assert!(!word(&mut r, 1, t), "{t:?}");
        assert!(r.sent().is_empty() && r.typing.take_pieces().is_empty());
    }
}

#[test]
fn a_new_text_cap_takes_effect_through_update() {
    let mut r = Rig::new("en-us", |_| ());
    let long = "a".repeat(usize::from(CAP) + 1);
    assert!(word(&mut r, 0, &long), "the default cap is larger");
    let mut s = KeyboardSettings::from_defaults().unwrap();
    s.text_cap = CAP;
    r.typing.update(&s);
    assert!(!word(&mut r, 0, &long));
    assert!(word(&mut r, 0, &long[1..]));
}

#[test]
fn an_erase_or_a_text_past_the_cap_types_nothing_and_the_cap_counts_characters() {
    let mut r = Rig::new("en-us", |s| s.text_cap = CAP);
    let long = "a".repeat(usize::from(CAP) + 1);
    assert!(!word(&mut r, CAP + 1, ""));
    assert!(!word(&mut r, 0, &long));
    assert!(r.sent().is_empty() && r.typing.take_pieces().is_empty());
    let wide = "😀".repeat(usize::from(CAP));
    assert!(
        word(&mut r, CAP, &wide),
        "two UTF-16 units each, one character each"
    );
    assert_eq!(r.sent().len(), usize::from(CAP) + 1);
}

#[test]
fn a_refused_word_tells_nothing_and_keeps_the_latches() {
    let mut r = Rig::new("en-us", |_| ());
    r.tap("shift").unwrap();
    r.injector
        .set_error(Some(InjectError::Refused { sent: 0, total: 2 }));
    assert!(!word(&mut r, 1, "hi"));
    assert!(r.typing.take_pieces().is_empty());
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

#[test]
fn a_hidden_mark_in_the_text_refuses_the_word_whole() {
    // A line separator, a right-to-left override, an isolate, a zero-width space and a BOM: none draws on a chip.
    for t in [
        "a\u{2028}b",
        "\u{202e}hi",
        "x\u{2067}",
        "y\u{200b}z",
        "\u{feff}w",
    ] {
        let mut r = Rig::new("en-us", |_| ());
        assert!(!word(&mut r, 1, t), "{t:?}");
        assert!(r.sent().is_empty() && r.typing.take_pieces().is_empty());
    }
    let mut r = Rig::new("en-us", |_| ());
    assert!(
        word(&mut r, 0, "می\u{200c}خواهم"),
        "ZWNJ is a Persian letter joiner"
    );
}

#[test]
fn a_backspace_the_app_takes_only_in_part_stops_the_word_after_one_erase() {
    let mut r = Rig::new("en-us", |_| ());
    r.injector
        .set_error(Some(InjectError::Refused { sent: 1, total: 2 }));
    assert!(word(&mut r, 3, "hi"), "the part that went counts");
    assert_eq!(
        r.typing.take_pieces(),
        [Piece::Erase],
        "one Erase told, no second Backspace and no text"
    );
}

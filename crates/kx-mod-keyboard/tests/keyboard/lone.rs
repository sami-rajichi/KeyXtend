//! The lone Win press (owner, 2026-10-17): a second click on Win with nothing used since the first lets go and sends
//! Win alone, so Start opens; a key, Caps Lock, a mouse click or another latch between lets it go quietly.

use crate::typing::{Rig, chord, tap, unicode};
use kx_mod_keyboard::{Latch, Modifiers, TapError, Tapped};
use kx_module_api::ports::InjectError;

const WIN: u16 = 0xE05B;
const SHIFT: u16 = 0x2A;
const CTRL: u16 = 0x1D;
const E: u16 = 0x12;
const S: u16 = 0x1F;
const CAPS: u16 = 0x3A;
const MODES: [Modifiers; 2] = [Modifiers::Release, Modifiers::Stay];

/// English typing in `mode`.
fn rig(mode: Modifiers) -> Rig {
    Rig::new("en-us", |s| s.modifiers = mode)
}

/// True while Win is latched in `r`.
fn win_on(r: &Rig) -> bool {
    r.typing.latches().is_on(Latch::Win)
}

#[test]
fn win_clicked_twice_with_nothing_between_sends_win_alone() {
    for mode in MODES {
        let mut r = rig(mode);
        r.tap("win").unwrap();
        assert!(r.sent().is_empty() && win_on(&r), "the first click latches");
        assert_eq!(r.tap("win"), Ok(Tapped::Done));
        assert_eq!(r.sent(), [tap(WIN)], "{mode:?}");
        assert!(!win_on(&r), "{mode:?}");
    }
}

#[test]
fn win_used_by_a_key_lets_go_quietly_in_stay_mode() {
    let mut r = rig(Modifiers::Stay);
    r.tap("win").unwrap();
    r.char(E).unwrap();
    assert_eq!(r.sent(), [chord(&[WIN], E)], "Win+E");
    r.tap("win").unwrap();
    assert!(r.sent().is_empty() && !win_on(&r), "no Start");
}

#[test]
fn in_release_mode_the_key_lets_win_go_and_the_next_click_latches_it_again() {
    let mut r = rig(Modifiers::Release);
    r.tap("win").unwrap();
    r.char(E).unwrap();
    assert!(!win_on(&r));
    r.tap("win").unwrap();
    assert_eq!(r.sent(), [chord(&[WIN], E)], "only the shortcut");
    assert!(win_on(&r));
}

#[test]
fn another_latch_or_caps_lock_between_lets_win_go_quietly() {
    let between: [&[&str]; 3] = [&["shift"], &["shift", "shift"], &["caps", "caps"]];
    for mode in MODES {
        for ids in between {
            let mut r = rig(mode);
            r.tap("win").unwrap();
            for id in ids {
                r.tap(id).unwrap();
            }
            let caps = r.sent();
            r.tap("win").unwrap();
            assert!(r.sent().is_empty() && !win_on(&r), "{mode:?} {ids:?}");
            assert!(caps.iter().all(|s| *s == tap(CAPS)), "{caps:?}");
        }
    }
}

#[test]
fn win_shift_s_goes_out_as_one_shortcut() {
    let mut r = rig(Modifiers::Release);
    for id in ["win", "shift"] {
        r.tap(id).unwrap();
    }
    r.char(S).unwrap();
    assert_eq!(r.sent(), [chord(&[SHIFT, WIN], S)]);
    assert!(!win_on(&r));
}

#[test]
fn a_latch_on_before_win_lets_go_after_the_lone_press_as_its_mode_says() {
    for mode in MODES {
        let mut r = rig(mode);
        for id in ["ctrl", "win", "win"] {
            r.tap(id).unwrap();
        }
        assert_eq!(r.sent(), [tap(WIN)], "Win alone, without Ctrl");
        let stay = mode == Modifiers::Stay;
        assert_eq!(r.typing.latches().is_on(Latch::Ctrl), stay, "{mode:?}");
        r.char(E).unwrap();
        let next = if stay {
            chord(&[CTRL], E)
        } else {
            unicode("e")
        };
        assert_eq!(r.sent(), [next], "{mode:?}: the key after it");
    }
}

#[test]
fn a_mouse_click_uses_win_in_stay_mode_and_lets_it_go_in_release_mode() {
    let mut r = rig(Modifiers::Stay);
    r.tap("win").unwrap();
    assert!(!r.typing.clicked(), "Win stays");
    r.tap("win").unwrap();
    assert!(
        r.sent().is_empty() && !win_on(&r),
        "Win+click, then no Start"
    );
    let mut r = rig(Modifiers::Release);
    r.tap("win").unwrap();
    assert!(r.typing.clicked());
    assert!(!win_on(&r));
}

#[test]
fn no_other_latch_sends_itself_when_clicked_twice() {
    for mode in MODES {
        for id in ["shift", "rshift", "ctrl", "alt", "altgr"] {
            let mut r = rig(mode);
            r.tap(id).unwrap();
            r.tap(id).unwrap();
            assert!(r.sent().is_empty(), "{mode:?} {id}");
        }
    }
}

#[test]
fn a_lone_press_refused_whole_keeps_win_latched_and_unused() {
    let mut r = rig(Modifiers::Release);
    r.tap("win").unwrap();
    let refused = InjectError::Refused { sent: 0, total: 2 };
    r.injector.set_error(Some(refused));
    assert_eq!(r.tap("win"), Err(TapError::Inject(refused)));
    assert!(win_on(&r) && r.typing.latches().unused(Latch::Win));
    r.injector.set_error(None);
    r.tap("win").unwrap();
    assert_eq!(r.sent(), [tap(WIN)], "the next click sends it");
}

#[test]
fn a_lone_press_taken_in_part_lets_win_go() {
    let mut r = rig(Modifiers::Release);
    r.tap("win").unwrap();
    let part = InjectError::Refused { sent: 1, total: 2 };
    r.injector.set_error(Some(part));
    assert_eq!(r.tap("win"), Err(TapError::Inject(part)));
    assert!(!win_on(&r), "never retried, so Win never goes twice");
}

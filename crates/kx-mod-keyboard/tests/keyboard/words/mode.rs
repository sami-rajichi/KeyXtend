//! The typing mode on the bus: told at start, again at each restart, and on each change of Shift, Caps Lock, a
//! shortcut latch or a dead key's wait, from a click, a mouse click that used the latches or the relabel tick; never
//! when nothing changed.

use super::Desk;
use crate::held::A;
use kx_module_api::ports::{FaceAction, InjectError, LatchesAsk, LatchesUsed, TypingMode};

/// The mode with Shift latched.
const SHIFT: TypingMode = TypingMode {
    shift: true,
    caps: false,
    shortcut: false,
    dead: false,
};

/// The mode with Caps Lock on.
const CAPS: TypingMode = TypingMode {
    shift: false,
    caps: true,
    ..SHIFT
};

/// The mode with a shortcut latch on.
fn shortcut() -> TypingMode {
    TypingMode {
        shortcut: true,
        ..TypingMode::default()
    }
}

#[test]
fn the_mode_is_told_at_start_and_at_each_change_only() {
    let d = Desk::new(None);
    assert_eq!(d.modes(), [TypingMode::default()], "at start");
    d.rig.tap("shift").unwrap();
    assert_eq!(d.modes(), [SHIFT]);
    d.rig.tap(A).unwrap();
    assert_eq!(
        d.modes(),
        [TypingMode::default()],
        "the one-shot Shift let go"
    );
    d.rig.tap(A).unwrap();
    d.rig.act(FaceAction::Relabel).unwrap();
    assert!(d.modes().is_empty(), "nothing changed");
}

#[test]
fn ctrl_alt_and_win_each_make_a_shortcut() {
    let d = Desk::new(None);
    d.modes();
    for latch in ["ctrl", "alt", "win"] {
        d.rig.tap(latch).unwrap();
        assert_eq!(d.modes(), [shortcut()], "{latch}");
        d.rig.tap(latch).unwrap();
        assert_eq!(d.modes(), [TypingMode::default()], "{latch} let go");
    }
}

#[test]
fn the_relabel_tick_tells_caps_lock_once_it_moves() {
    let d = Desk::new(None);
    d.modes();
    d.rig.fakes.source.set_caps(true);
    d.rig.act(FaceAction::Relabel).unwrap();
    let caps = TypingMode {
        caps: true,
        ..TypingMode::default()
    };
    assert_eq!(d.modes(), [caps]);
    d.rig.act(FaceAction::Relabel).unwrap();
    assert!(d.modes().is_empty());
}

#[test]
fn a_mouse_click_that_used_the_latches_tells_the_mode() {
    let d = Desk::new(None);
    d.rig.tap("shift").unwrap();
    d.modes();
    d.bus.publish(&LatchesUsed).unwrap();
    assert_eq!(d.modes(), [TypingMode::default()]);
}

#[test]
fn asking_for_the_latches_does_not_tell_the_mode_again_but_a_restart_does() {
    let mut d = Desk::new(None);
    d.rig.tap("shift").unwrap();
    d.modes();
    d.bus.publish(&LatchesAsk).unwrap();
    assert!(d.modes().is_empty());
    d.rig.restart(None);
    assert_eq!(d.modes(), [SHIFT], "the latches survive a restart");
}

#[test]
fn caps_lock_reaches_the_mode_in_the_tick_that_also_switched_the_layout() {
    let d = Desk::new(None);
    d.modes();
    d.rig.fakes.source.set_caps(true);
    d.rig.switch("fr-fr");
    let caps = TypingMode {
        caps: true,
        ..TypingMode::default()
    };
    assert_eq!(d.modes(), [caps]);
}

#[test]
fn a_dead_key_tells_its_wait_and_the_next_key_ends_it() {
    let d = Desk::new(None);
    d.rig.switch("fr-fr");
    d.modes();
    d.rig.tap("c-0x001A").unwrap();
    let dead = TypingMode {
        dead: true,
        ..TypingMode::default()
    };
    assert_eq!(d.modes(), [dead], "the French ^ waits");
    d.rig.tap("c-0x0012").unwrap();
    assert_eq!(d.modes(), [TypingMode::default()], "ê ends it");
}

#[test]
fn a_caps_click_tells_caps_lock_at_once_and_the_tick_corrects_it() {
    let d = Desk::new(None);
    d.modes();
    d.rig.tap("caps").unwrap();
    assert_eq!(d.modes(), [CAPS], "no tick between");
    d.rig.fakes.source.set_caps(true);
    d.rig.act(FaceAction::Relabel).unwrap();
    assert!(d.modes().is_empty(), "the system agrees");
    d.rig.tap("caps").unwrap();
    assert_eq!(d.modes(), [TypingMode::default()]);
    d.rig.act(FaceAction::Relabel).unwrap();
    assert_eq!(d.modes(), [CAPS], "the system still has it on");
}

#[test]
fn a_caps_click_that_never_went_out_changes_no_mode() {
    let d = Desk::new(None);
    d.modes();
    let refused = InjectError::Refused { sent: 0, total: 1 };
    d.rig.fakes.injector.set_error(Some(refused));
    d.rig.tap("caps").unwrap();
    assert!(d.modes().is_empty());
}

#[test]
fn a_caps_press_tells_caps_lock_at_once_as_a_click_does() {
    let d = Desk::new(None);
    d.modes();
    d.rig.press("caps").unwrap();
    assert_eq!(d.modes(), [CAPS]);
}

//! The typing mode on the bus: told at start, again at each restart and each ask, and on each change of Shift, Caps
//! Lock, a shortcut latch or a dead key's wait, from a click, a mouse click that used the latches or the relabel
//! tick; never when nothing changed.

use super::{Desk, plain};
use crate::assist::ear;
use crate::held::A;
use kx_mod_keyboard::KeyboardSettings;
use kx_module_api::ports::{
    FaceAction, InjectError, LatchesAsk, LatchesChanged, LatchesUsed, TypingMode, TypingModeAsk,
};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// The mode with Shift latched.
fn shift() -> TypingMode {
    TypingMode {
        shift: true,
        ..plain()
    }
}

/// The mode with Caps Lock on.
fn caps() -> TypingMode {
    TypingMode {
        caps: true,
        ..plain()
    }
}

/// The mode with a shortcut latch on.
fn shortcut() -> TypingMode {
    TypingMode {
        shortcut: true,
        ..plain()
    }
}

#[test]
fn the_mode_is_told_at_start_and_at_each_change_only() {
    let d = Desk::new(None);
    assert_eq!(d.modes(), [plain()], "at start");
    d.rig.tap("shift").unwrap();
    assert_eq!(d.modes(), [shift()]);
    d.rig.tap(A).unwrap();
    assert_eq!(d.modes(), [plain()], "the one-shot Shift let go");
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
        assert_eq!(d.modes(), [plain()], "{latch} let go");
    }
}

#[test]
fn the_relabel_tick_tells_caps_lock_once_it_moves() {
    let d = Desk::new(None);
    d.modes();
    d.rig.fakes.source.set_caps(true);
    d.rig.act(FaceAction::Relabel).unwrap();
    assert_eq!(d.modes(), [caps()]);
    d.rig.act(FaceAction::Relabel).unwrap();
    assert!(d.modes().is_empty());
}

#[test]
fn a_mouse_click_that_used_the_latches_tells_the_mode() {
    let d = Desk::new(None);
    d.rig.tap("shift").unwrap();
    d.modes();
    d.bus.publish(&LatchesUsed).unwrap();
    assert_eq!(d.modes(), [plain()]);
}

#[test]
fn asking_for_the_latches_does_not_tell_the_mode_again_but_a_restart_does() {
    let mut d = Desk::new(None);
    d.rig.tap("shift").unwrap();
    d.modes();
    d.bus.publish(&LatchesAsk).unwrap();
    assert!(d.modes().is_empty());
    d.rig.restart(None);
    assert_eq!(d.modes(), [shift()], "the latches survive a restart");
}

#[test]
fn caps_lock_reaches_the_mode_in_the_tick_that_also_switched_the_layout() {
    let d = Desk::new(None);
    d.modes();
    d.rig.fakes.source.set_caps(true);
    d.rig.switch("fr-fr");
    assert_eq!(d.modes(), [caps()]);
}

#[test]
fn a_dead_key_tells_its_wait_and_the_next_key_ends_it() {
    let d = Desk::new(None);
    d.rig.switch("fr-fr");
    d.modes();
    d.rig.tap("c-0x001A").unwrap();
    let dead = TypingMode {
        dead: true,
        ..plain()
    };
    assert_eq!(d.modes(), [dead], "the French ^ waits");
    d.rig.tap("c-0x0012").unwrap();
    assert_eq!(d.modes(), [plain()], "ê ends it");
}

#[test]
fn a_caps_click_tells_caps_lock_at_once_and_the_tick_corrects_it() {
    let d = Desk::new(None);
    d.modes();
    d.rig.tap("caps").unwrap();
    assert_eq!(d.modes(), [caps()], "no tick between");
    d.rig.fakes.source.set_caps(true);
    d.rig.act(FaceAction::Relabel).unwrap();
    assert!(d.modes().is_empty(), "the system agrees");
    d.rig.tap("caps").unwrap();
    assert_eq!(d.modes(), [plain()]);
    d.rig.act(FaceAction::Relabel).unwrap();
    assert_eq!(d.modes(), [caps()], "the system still has it on");
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
    assert_eq!(d.modes(), [caps()]);
}

#[test]
fn the_caps_flip_outlasts_the_state_read_until_the_tick() {
    // The app reads the state after each click, and the system may not have the new Caps Lock yet.
    let d = Desk::new(None);
    d.modes();
    d.rig.tap("caps").unwrap();
    assert_eq!(d.modes(), [caps()]);
    d.rig.tap(A).unwrap();
    assert!(d.modes().is_empty(), "the read after the click kept it");
    assert_eq!(
        d.rig.state()["keys"]["caps"]["on"],
        true,
        "lit from the flip"
    );
    d.rig.act(FaceAction::Relabel).unwrap();
    assert_eq!(d.modes(), [plain()], "the tick reads the system");
}

#[test]
fn two_caps_clicks_with_a_state_read_between_tell_no_wrong_value() {
    let d = Desk::new(None);
    d.modes();
    d.rig.tap("caps").unwrap();
    d.rig.tap("caps").unwrap();
    assert_eq!(d.modes(), [caps(), plain()]);
    d.rig.act(FaceAction::Relabel).unwrap();
    assert!(d.modes().is_empty(), "the system is off too");
}

#[test]
fn asking_for_the_mode_tells_it_again_and_not_the_latches() {
    let d = Desk::new(None);
    d.rig.tap("ctrl").unwrap();
    d.modes();
    let latches: Arc<Mutex<Vec<()>>> = Arc::default();
    let _ear = ear(&d.bus, &latches, |_: &LatchesChanged| ());
    let typed = d.rig.fakes.injector.strokes().len();
    d.bus.publish(&TypingModeAsk).unwrap();
    assert_eq!(d.modes(), [shortcut()], "told unchanged, for a later start");
    d.bus.publish(&TypingModeAsk).unwrap();
    assert_eq!(d.modes(), [shortcut()], "each ask is answered");
    assert!(
        latches.lock().unwrap().is_empty(),
        "the latches stay untold"
    );
    assert_eq!(d.rig.fakes.injector.strokes().len(), typed, "nothing typed");
}

#[test]
fn a_caps_flip_left_by_a_hidden_window_gives_way_one_relabel_period_on() {
    // The window hid before its tick, so only a wake's state read comes.
    let d = Desk::new(None);
    d.modes();
    d.rig.tap("caps").unwrap();
    let period = KeyboardSettings::from_defaults().unwrap().relabel_ms;
    d.rig.fakes.clock.advance(Duration::from_millis(period - 1));
    assert_eq!(d.rig.state()["keys"]["caps"]["on"], true, "the flip holds");
    d.rig.fakes.clock.advance(Duration::from_millis(1));
    assert_eq!(d.rig.state()["keys"]["caps"]["on"], false, "the system's");
    d.bus.publish(&LatchesUsed).unwrap();
    assert_eq!(d.modes(), [caps(), plain()], "the mode follows the read");
}

#[test]
fn the_mode_carries_the_text_cap_the_settings_give() {
    let d = Desk::new(Some(("text_cap", toml::Value::Integer(300))));
    let want = TypingMode {
        cap: Some(300),
        ..plain()
    };
    assert_eq!(d.modes(), [want]);
}

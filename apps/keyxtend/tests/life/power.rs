//! The Power key over the whole app: with the keyboard and the power module running, a click opens the fake dialog
//! once; with the power module switched off, the key is dimmed and nothing opens (plan edge case 12).

use crate::rig::{Rig, WAIT, hear};
use keyxtend::boot::Boot;
use kx_mod_keyboard::view::POWER_OFF_TIP;
use kx_module_api::ModuleState;
use kx_module_api::ports::{FaceAction, KeyId, PowerAsk, PowerAsked, PowerChanged};
use serde_json::Value;

/// The Power key.
const KEY: &str = "power";

/// A click on the Power key.
fn click(boot: &Boot) {
    let tap = FaceAction::Tap {
        id: KeyId::new(KEY),
        back: false,
    };
    let _ = boot.hub.act(tap);
}

/// The Power key's state as the keyboard draws it.
fn key(boot: &Boot) -> Value {
    let state: Value = serde_json::from_str(&boot.hub.state_json()).unwrap();
    state["keys"][KEY].clone()
}

#[test]
fn a_click_on_the_power_key_opens_the_dialog_once() {
    let rig = Rig::new();
    let mut boot = rig.load();
    boot.start().unwrap();
    // Ears after the start, so the keyboard hears each state before the test does.
    let bus = boot.kernel.bus();
    let (_told, told) = hear::<PowerChanged>(&bus);
    let (_asked, asked) = hear::<PowerAsked>(&bus);
    bus.publish(&PowerAsk).unwrap();
    while !told.recv_timeout(WAIT).unwrap().live {}
    assert_eq!(key(&boot)["dim"], false, "the key lights");
    click(&boot);
    assert!(rig.power.wait_opened(1, WAIT), "the dialog opened");
    assert_eq!(asked.try_iter().count(), 1, "one ask per click");
    assert_eq!(rig.power.opened(), 1);
}

#[test]
fn with_the_power_module_switched_off_the_key_is_dimmed_and_nothing_opens() {
    let rig = Rig::new();
    let section = format!("[kernel]\ndisabled = [\"{}\"]\n", kx_mod_power::ID);
    std::fs::write(rig.files().settings, section).unwrap();
    let mut boot = rig.load();
    boot.start().unwrap();
    let state = boot.kernel.state(kx_mod_power::ID);
    assert_ne!(state, Some(ModuleState::Active), "switched off");
    let (_asked, asked) = hear::<PowerAsked>(&boot.kernel.bus());
    let k = key(&boot);
    let dim = (Value::Bool(true), Value::from(POWER_OFF_TIP));
    assert_eq!(
        (k["dim"].clone(), k["tip"].clone()),
        dim,
        "dimmed, saying why"
    );
    click(&boot);
    assert_eq!(asked.try_iter().count(), 0, "the keyboard asks nobody");
    assert_eq!(rig.power.opened(), 0, "no power module runs to open it");
}

//! The app from start to stop over the fake platform, without Qt: every module starts and types, Quit stops them and
//! ends the UI, a second copy's commands reach the running one, the kernel's thread keeps going after a refusal, the
//! language shown always matches the legends, the pointer hook, the scroll target, the power dialog, the box watch and
//! the sealed store reach only the modules holding their gate, the shortcuts reach the keyboard's layer, the pad
//! scrolls, the Power key opens the dialog, and prediction opens the language in front.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

#[path = "../common/fakes.rs"]
mod fakes;
mod lang;
mod pick;
mod power;
mod predict;
mod rig;
mod scroll;
mod second;
mod ships;
mod stop;

use keyxtend::boot::{Boot, modules};
use keyxtend::drain;
use keyxtend::log::open;
use kx_kernel::KernelSettings;
use kx_module_api::ModuleId;
use kx_module_api::ModuleState::{Active, Pending};
use kx_module_api::ports::{FaceAction, KeyId};
use kx_ui::{Request, Wake};
use rig::{Rig, char_key, watch};
use std::sync::{Arc, mpsc};

/// The modules the app ships, in the order it adds them.
const SHIPPED: [ModuleId; 7] = [
    kx_mod_layouts::ID,
    kx_mod_keyboard::ID,
    kx_mod_mouse::ID,
    kx_mod_scroll::ID,
    kx_mod_power::ID,
    kx_mod_predict::ID,
    kx_ui::ID,
];
/// The home row's first letter key: `a` in English, `q` in French.
const A_KEY: u16 = 0x1E;
/// What JSON says for nothing.
const NULL: &str = "null";

#[test]
fn the_app_ships_the_layouts_keyboard_mouse_scroll_power_predict_and_ui_modules() {
    let boot = Rig::new().load();
    let ids: Vec<ModuleId> = modules(&boot.hub).iter().map(|m| m.manifest().id).collect();
    assert_eq!(ids, SHIPPED);
}

#[test]
fn every_module_starts_types_and_stops_when_the_user_quits() {
    let rig = Rig::new();
    let mut boot = rig.load();
    assert!(
        SHIPPED
            .iter()
            .all(|&id| boot.kernel.state(id) == Some(Pending))
    );
    boot.start().unwrap();
    assert!(
        SHIPPED
            .iter()
            .all(|&id| boot.kernel.state(id) == Some(Active))
    );
    let wakes = watch(&boot.hub);
    let tap = FaceAction::Tap {
        id: KeyId::new(char_key(A_KEY)),
        back: false,
    };
    let _ = boot.hub.act(tap);
    assert!(!rig.injector.strokes().is_empty(), "it typed");
    let Boot {
        kernel,
        hub,
        requests,
        heard,
    } = boot;
    let thread = drain::spawn(kernel, Arc::clone(&hub), requests).unwrap();
    assert!(hub.quit(), "the tray's Quit reaches the kernel's thread");
    thread.join().unwrap();
    assert!(wakes.lock().unwrap().contains(&Wake::End), "the UI ends");
    assert_eq!(hub.board_json(), NULL, "the ui module let the face go");
    assert!(!hub.quit(), "the queue ended with Quit");
    drop(heard);
}

#[test]
fn the_hub_hears_what_the_kernel_says_while_it_loads() {
    let rig = Rig::new();
    std::fs::write(rig.files().settings, "[kernel\nbroken").unwrap();
    let boot = rig.load();
    let notices = boot.hub.take_notices();
    assert_eq!(notices.len(), 1, "the damaged file's notice");
    assert!(!notices[0].retry, "it names no module to try again");
    assert!(
        rig.files().broken.exists(),
        "the damaged file was set aside"
    );
}

#[test]
fn a_refused_try_again_is_logged_and_quit_still_ends_the_ui() {
    let rig = Rig::new();
    let mut boot = rig.load();
    boot.start().unwrap();
    let wakes = watch(&boot.hub);
    let settings = KernelSettings {
        disabled: Vec::new(),
        log_level: "warn".to_owned(),
        log_max_mb: 1,
    };
    let files = rig.files();
    let log = open(&settings, &files).unwrap();
    let (send, requests) = mpsc::channel();
    send.send(Request::Retry(kx_mod_keyboard::ID)).unwrap();
    send.send(Request::Quit).unwrap();
    drop(send);
    let hub = Arc::clone(&boot.hub);
    tracing::dispatcher::with_default(&log.dispatch, || {
        drain::drain(boot.kernel, &hub, &requests);
    });
    let text = std::fs::read_to_string(&files.log).unwrap();
    assert!(text.contains("try again was refused"), "{text}");
    assert!(text.contains("keyboard"), "it names the module: {text}");
    // The mouse, scroll, power and prediction modules' states come from their own threads, at start and at stop, with
    // the overlay's, keys', scroll's and chip lane's wakes.
    let own = |w: &&Wake| !matches!(w, Wake::Assist | Wake::Keys | Wake::Scroll | Wake::Words);
    let seen: Vec<Wake> = wakes.lock().unwrap().iter().filter(own).copied().collect();
    let order = [Wake::Settings, Wake::End];
    assert_eq!(seen, order, "the modules stop, then the UI ends");
    let last = wakes.lock().unwrap().last().copied();
    assert_eq!(
        last,
        Some(Wake::End),
        "the mouse, scroll, power and prediction modules told their stops first"
    );
    assert_eq!(hub.board_json(), NULL, "Quit stopped the modules");
}

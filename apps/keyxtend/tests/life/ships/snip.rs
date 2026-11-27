//! Snip's grants (ADR-0056 decision 10, ADR-0057 decision 5, ADR-0058): its feature is a default and the build ships
//! it, the keyboard and the ui hold `Snip` as an event gate only, the module alone holds `ScreenCapture`, `SaveFile`
//! and `Microphone` and asks for the freeze, the recorder, the microphone's passes, the Snips folder, the dialogs and
//! the player, each port reaches the module alone but the player, which voice may reach too, and, handed by the app,
//! the module alone gets it, in either run.

use super::voice::held;
use super::{asking, defaults, features, holders, reached};
use crate::rig::{APP, OS, Rig};
use keyxtend::boot::{Services, modules};
use keyxtend::hidden::Run;
use keyxtend::snip::Ports;
use kx_module_api::ports::{
    FileAskKey, MediaOpenKey, MicPassKey, MicPasses, RecorderKey, ScreenFreezeKey, SnipFolderKey,
};
use kx_module_api::{Capability, ModuleId, ServiceKey};
use std::collections::BTreeSet;
use std::sync::Arc;

/// The snip module's feature.
const SNIP_FEATURE: &str = "mod-snip";
/// The modules that hold `Snip` as an event gate only.
const GATES: [ModuleId; 2] = [kx_mod_keyboard::ID, kx_ui::ID];

/// The snip module alone.
fn snip() -> BTreeSet<ModuleId> {
    BTreeSet::from([kx_mod_snip::ID])
}

#[test]
fn mod_snip_is_a_default_feature_and_the_default_build_ships_it() {
    assert!(defaults(&features()).contains(&SNIP_FEATURE));
    let shipped = modules(&Rig::new().load().hub, OS, &Run::Normal);
    assert!(shipped.iter().any(|m| m.manifest().id == kx_mod_snip::ID));
}

#[test]
fn snip_goes_to_the_module_and_as_an_event_gate_to_the_keyboard_and_the_ui() {
    let want: BTreeSet<ModuleId> = GATES.into_iter().chain([kx_mod_snip::ID]).collect();
    assert_eq!(holders(Capability::Snip), want, "ADR-0057 decision 5");
}

#[test]
fn only_snip_holds_screen_capture_save_file_and_the_microphone() {
    for cap in [Capability::ScreenCapture, Capability::SaveFile] {
        assert_eq!(holders(cap), snip(), "{cap:?}: ADR-0056 decision 10");
    }
    let mic = holders(Capability::Microphone);
    assert_eq!(mic, snip(), "ADR-0058 decision 2");
}

#[test]
fn only_snip_asks_for_its_ports() {
    let keys = [
        ScreenFreezeKey::ID,
        RecorderKey::ID,
        MicPassKey::ID,
        SnipFolderKey::ID,
        FileAskKey::ID,
        MediaOpenKey::ID,
    ];
    assert_eq!(asking(&keys), snip(), "the others' Snip is an event gate");
}

#[test]
fn each_snip_port_reaches_snip_alone_but_the_player_which_voice_may_reach_and_never_asks_for() {
    let rig = Rig::new();
    let ports = rig.snip.ports(&Run::Normal);
    let passes: Arc<dyn MicPasses> = Arc::new(ports.seal.clone());
    let freeze = reached::<ScreenFreezeKey>(&rig, &ports.freeze);
    assert_eq!(freeze, snip(), "ScreenCapture");
    let folder = reached::<SnipFolderKey>(&rig, &ports.folder);
    assert_eq!(folder, snip(), "ScreenCapture");
    let recorder = reached::<RecorderKey>(&rig, &ports.recorder);
    assert_eq!(recorder, snip(), "ScreenCapture");
    let passes = reached::<MicPassKey>(&rig, &passes);
    assert_eq!(passes, snip(), "Microphone: ADR-0058 decision 2");
    let dialogs = reached::<FileAskKey>(&rig, &ports.dialogs);
    assert_eq!(dialogs, snip(), "SaveFile");
    let both = BTreeSet::from([kx_mod_voice::ID, kx_mod_snip::ID]);
    let player = reached::<MediaOpenKey>(&rig, &ports.player);
    assert_eq!(player, both, "OpenFolder: ADR-0058 decision 3");
}

/// The holders of the port `pick` names among `run`'s Snip ports, beyond its own, once the app started, then once it
/// stopped, with module `off` switched off.
fn reach<T: ?Sized>(
    run: &Run,
    off: Option<ModuleId>,
    pick: impl Fn(&mut Ports) -> &mut Arc<T>,
) -> (usize, usize) {
    let api = Arc::clone(pick(&mut Rig::new().snip.ports(run)));
    held(&api, |s: &mut Services, a| *pick(&mut s.snip) = a, run, off)
}

/// Checks the snip module holds the port `pick` names in either run and lets it go at stop, and no other module
/// gets it; a hidden run keeps no settings, so only a normal one switches Snip off.
fn alone<T: ?Sized>(port: &str, pick: impl Fn(&mut Ports) -> &mut Arc<T> + Copy) {
    for run in [Run::Normal, Run::hidden(APP.into())] {
        let (running, stopped) = reach(&run, None, pick);
        assert!(running > 0, "{run:?}: the snip module holds the {port}");
        assert_eq!(stopped, 0, "{run:?}: and lets the {port} go at stop");
    }
    let others = reach(&Run::Normal, Some(kx_mod_snip::ID), pick);
    assert_eq!(others, (0, 0), "no other module gets the {port}");
}

#[test]
fn the_app_hands_each_snip_port_to_the_snip_module_alone_in_either_run() {
    // The app wraps the recorder and the seal before the module gets them, so the test above pins their reach and
    // `snip/seal.rs` the wrap.
    alone("freeze", |p| &mut p.freeze);
    alone("folder", |p| &mut p.folder);
    alone("dialogs", |p| &mut p.dialogs);
    alone("player", |p| &mut p.player);
}

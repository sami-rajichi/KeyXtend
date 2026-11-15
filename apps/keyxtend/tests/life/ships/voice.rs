//! Voice typing's grants (ADR-0042, ADR-0046): its feature is a default and the build ships it, the keyboard and the ui
//! hold `Voice` as an event gate only, the module alone holds the microphone, the network and the folder reveal and asks
//! for speech, the model files, the session watch and its shelf, speech and the model files reach it alone, its shelf
//! reaches only the secret store's holders and only the voice module holds it in a run, and the session watch needs no
//! gate.

use super::{asking, defaults, features, holders, reached};
use crate::rig::{OS, Rig};
use keyxtend::boot::modules;
use keyxtend::hidden::Run;
use kx_module_api::ports::{
    ModelFiles, ModelFilesKey, SealedShelf, SessionWatch, SessionWatchKey, Speech, SpeechKey,
    VoiceShelfKey,
};
use kx_module_api::{Capability, ModuleId, ServiceKey};
use kx_platform_fake::{FakeModelFiles, FakeSessionWatch, FakeShelf, FakeSpeech};
use std::collections::BTreeSet;
use std::sync::Arc;

/// The voice module's feature.
const VOICE_FEATURE: &str = "mod-voice";
/// The modules that hold `Voice` as an event gate only.
const GATES: [ModuleId; 2] = [kx_mod_keyboard::ID, kx_ui::ID];

/// Every shipped module's id.
fn shipped() -> BTreeSet<ModuleId> {
    let shipped = modules(&Rig::new().load().hub, OS, &Run::Normal);
    shipped.iter().map(|m| m.manifest().id).collect()
}

/// The voice module alone.
fn voice() -> BTreeSet<ModuleId> {
    BTreeSet::from([kx_mod_voice::ID])
}

#[test]
fn mod_voice_is_a_default_feature_and_the_default_build_ships_it() {
    assert!(defaults(&features()).contains(&VOICE_FEATURE));
    assert!(shipped().contains(&kx_mod_voice::ID));
}

#[test]
fn voice_goes_to_the_module_and_as_an_event_gate_to_the_keyboard_and_the_ui() {
    let want: BTreeSet<ModuleId> = GATES.into_iter().chain([kx_mod_voice::ID]).collect();
    assert_eq!(holders(Capability::Voice), want, "ADR-0042 decision 7");
}

#[test]
fn only_the_voice_module_holds_the_microphone_the_network_and_the_folder_reveal() {
    for cap in [
        Capability::Microphone,
        Capability::Network,
        Capability::OpenFolder,
    ] {
        assert_eq!(holders(cap), voice(), "{cap:?}");
    }
}

#[test]
fn speech_and_the_model_files_reach_the_voice_module_alone() {
    let rig = Rig::new();
    let speech: Arc<dyn Speech> = Arc::new(FakeSpeech::new());
    let files: Arc<dyn ModelFiles> = Arc::new(FakeModelFiles::new());
    assert_eq!(reached::<SpeechKey>(&rig, &speech), voice(), "ADR-0042");
    assert_eq!(reached::<ModelFilesKey>(&rig, &files), voice(), "ADR-0042");
}

#[test]
fn only_the_voice_module_asks_for_speech_the_model_files_the_session_or_its_shelf() {
    let keys = [
        SpeechKey::ID,
        ModelFilesKey::ID,
        SessionWatchKey::ID,
        VoiceShelfKey::ID,
    ];
    assert_eq!(asking(&keys), voice(), "the others' Voice is an event gate");
}

#[test]
fn the_voice_shelf_reaches_only_the_secret_store_holders() {
    let shelf: Arc<dyn SealedShelf> = Arc::new(FakeShelf::new());
    let got = reached::<VoiceShelfKey>(&Rig::new(), &shelf);
    assert_eq!(got, holders(Capability::SecretStore), "its gate");
    assert!(got.contains(&kx_mod_voice::ID), "{got:?}");
}

/// The voice shelf's holders beyond the test's and the kernel's own, after the shipped app's start, then after its stop,
/// with module `off` switched off.
fn shelf_holders(off: Option<ModuleId>) -> (usize, usize) {
    let rig = Rig::new();
    if let Some(id) = off {
        let section = format!("[kernel]\ndisabled = [\"{id}\"]\n");
        std::fs::write(rig.files().settings, section).unwrap();
    }
    let shelf: Arc<dyn SealedShelf> = Arc::new(FakeShelf::new());
    let mut services = rig.services();
    services.voice_shelf = Arc::clone(&shelf);
    let mut boot = rig.load_with(&Run::Normal, services);
    let own = Arc::strong_count(&shelf);
    boot.start().unwrap();
    let running = Arc::strong_count(&shelf) - own;
    boot.kernel.stop_all();
    (running, Arc::strong_count(&shelf) - own)
}

#[test]
fn only_the_voice_module_gets_its_shelf_and_lets_it_go_at_stop() {
    let (running, stopped) = shelf_holders(None);
    assert!(running > 0, "the voice module holds it while it runs");
    assert_eq!(stopped, 0, "and lets it go at stop");
    let others = shelf_holders(Some(kx_mod_voice::ID));
    assert_eq!(others, (0, 0), "no other shipped module gets it");
}

#[test]
fn the_session_watch_needs_no_capability_and_reaches_every_module() {
    assert_eq!(SessionWatchKey::CAPABILITY, None, "ADR-0042 decision 9");
    let watch: Arc<dyn SessionWatch> = Arc::new(FakeSessionWatch::new());
    assert_eq!(reached::<SessionWatchKey>(&Rig::new(), &watch), shipped());
}

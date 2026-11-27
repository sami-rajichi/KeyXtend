//! Voice typing's grants (ADR-0042, ADR-0046, ADR-0047, ADR-0058): its feature is a default and the build ships it,
//! the keyboard and the ui hold `Voice` as an event gate only, the module alone holds speech and the network, Snip
//! alone the microphone, and both the folder reveal; it alone asks for speech, cloud speech, the model files, the
//! session watch and its shelf, and speech, cloud speech and the model files reach it alone; `OpenLink` and
//! `ReadClipboard` each gain exactly the voice module, its shelf and the cloud port reach only it in a run, either run,
//! and the session watch needs no gate.

use super::clip::readers;
use super::sel::linkers;
use super::{asking, defaults, features, holders, reached};
use crate::rig::{APP, OS, Rig};
use keyxtend::boot::{Services, modules};
use keyxtend::hidden::Run;
use kx_module_api::ports::{
    CloudSpeech, CloudSpeechKey, ModelFiles, ModelFilesKey, SealedShelf, SessionWatch,
    SessionWatchKey, Speech, SpeechKey, VoiceShelfKey,
};
use kx_module_api::{Capability, ModuleId, ServiceKey};
use kx_platform_fake::{FakeCloud, FakeModelFiles, FakeSessionWatch, FakeShelf, FakeSpeech};
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
fn voice_alone_holds_speech_and_the_network_snip_alone_the_microphone_and_both_the_folder_reveal() {
    assert_eq!(holders(Capability::Speech), voice(), "ADR-0058 decision 1");
    assert_eq!(holders(Capability::Network), voice(), "ADR-0047");
    let snip = BTreeSet::from([kx_mod_snip::ID]);
    assert_eq!(holders(Capability::Microphone), snip, "ADR-0058 decision 2");
    let both = BTreeSet::from([kx_mod_voice::ID, kx_mod_snip::ID]);
    assert_eq!(
        holders(Capability::OpenFolder),
        both,
        "ADR-0056 decision 10"
    );
}

#[test]
fn open_link_and_read_clipboard_each_gain_exactly_the_voice_module() {
    let sets = [
        (Capability::OpenLink, linkers()),
        (Capability::ReadClipboard, readers()),
    ];
    for (cap, mut was) in sets {
        assert!(was.remove(&kx_mod_voice::ID), "{cap:?}: voice holds it");
        let now = holders(cap);
        let gained: BTreeSet<ModuleId> = now.difference(&was).copied().collect();
        assert_eq!(gained, voice(), "{cap:?}: ADR-0047 decision 11");
        assert!(now.is_superset(&was), "{cap:?}: none lost");
    }
}

#[test]
fn speech_cloud_speech_and_the_model_files_reach_the_voice_module_alone() {
    let rig = Rig::new();
    let speech: Arc<dyn Speech> = Arc::new(FakeSpeech::new());
    let cloud: Arc<dyn CloudSpeech> = Arc::new(FakeCloud::new());
    let files: Arc<dyn ModelFiles> = Arc::new(FakeModelFiles::new());
    assert_eq!(reached::<SpeechKey>(&rig, &speech), voice(), "ADR-0058");
    assert_eq!(reached::<CloudSpeechKey>(&rig, &cloud), voice(), "ADR-0047");
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
fn only_the_voice_module_asks_for_cloud_speech() {
    assert_eq!(
        asking(&[CloudSpeechKey::ID]),
        voice(),
        "ADR-0047 decision 11"
    );
}

#[test]
fn the_voice_shelf_reaches_only_the_secret_store_holders() {
    let shelf: Arc<dyn SealedShelf> = Arc::new(FakeShelf::new());
    let got = reached::<VoiceShelfKey>(&Rig::new(), &shelf);
    assert_eq!(got, holders(Capability::SecretStore), "its gate");
    assert!(got.contains(&kx_mod_voice::ID), "{got:?}");
}

/// The holders of `api`, which `put` sets among the services, beyond the test's and the kernel's own after the
/// shipped app's start in `run`, then after its stop, with module `off` switched off.
pub(super) fn held<T: ?Sized>(
    api: &Arc<T>,
    put: impl FnOnce(&mut Services, Arc<T>),
    run: &Run,
    off: Option<ModuleId>,
) -> (usize, usize) {
    let rig = Rig::new();
    if let Some(id) = off {
        let section = format!("[kernel]\ndisabled = [\"{id}\"]\n");
        std::fs::write(rig.files().settings, section).unwrap();
    }
    let mut services = rig.services(run);
    put(&mut services, Arc::clone(api));
    let mut boot = rig.load_with(run, services);
    let own = Arc::strong_count(api);
    boot.start().unwrap();
    let running = Arc::strong_count(api) - own;
    boot.kernel.stop_all();
    (running, Arc::strong_count(api) - own)
}

#[test]
fn only_the_voice_module_gets_its_shelf_and_lets_it_go_at_stop() {
    let shelf: Arc<dyn SealedShelf> = Arc::new(FakeShelf::new());
    let put = |s: &mut Services, a| s.voice_shelf = a;
    let (running, stopped) = held(&shelf, put, &Run::Normal, None);
    assert!(running > 0, "the voice module holds it while it runs");
    assert_eq!(stopped, 0, "and lets it go at stop");
    let others = held(&shelf, put, &Run::Normal, Some(kx_mod_voice::ID));
    assert_eq!(others, (0, 0), "no other shipped module gets it");
}

#[test]
fn the_app_hands_cloud_speech_to_the_voice_module_alone_in_either_run() {
    let cloud: Arc<dyn CloudSpeech> = Arc::new(FakeCloud::new());
    let put = |s: &mut Services, a| s.cloud = a;
    for run in [Run::Normal, Run::hidden(APP.into())] {
        let (running, stopped) = held(&cloud, put, &run, None);
        assert!(running > 0, "{run:?}: the voice module holds it");
        assert_eq!(stopped, 0, "{run:?}: and lets it go at stop");
    }
    let others = held(&cloud, put, &Run::Normal, Some(kx_mod_voice::ID));
    assert_eq!(others, (0, 0), "no other shipped module gets it");
}

#[test]
fn the_session_watch_needs_no_capability_and_reaches_every_module() {
    assert_eq!(SessionWatchKey::CAPABILITY, None, "ADR-0042 decision 9");
    let watch: Arc<dyn SessionWatch> = Arc::new(FakeSessionWatch::new());
    assert_eq!(reached::<SessionWatchKey>(&Rig::new(), &watch), shipped());
}

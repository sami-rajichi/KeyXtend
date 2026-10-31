//! The `ui` module: its manifest, and a start that hands its settings and the face to the hub.

use crate::{FakeFace, Rig, provide};
use kx_module_api::Capability::{KeyboardUi, ReadClipboard, ReadFocusedText};
use kx_module_api::ports::KeyboardFaceKey;
use kx_module_api::{Module, ModuleCx, ModuleError, ServiceError, ServiceKey, SettingsError};
use kx_test_support::host::FakeHost;
use kx_ui::{ID, MANIFEST, UiModule, Wake};
use std::sync::Arc;

#[test]
fn it_needs_the_face_reads_the_lane_and_clipboard_and_owns_ui() {
    assert_eq!(ID.as_str(), "ui");
    assert_eq!(MANIFEST.requires, [KeyboardFaceKey::ID]);
    assert!(MANIFEST.provides.is_empty());
    assert_eq!(
        MANIFEST.capabilities,
        [KeyboardUi, ReadFocusedText, ReadClipboard]
    );
    assert_eq!(KeyboardFaceKey::CAPABILITY, Some(KeyboardUi));
    assert!(MANIFEST.settings.is_some_and(|s| s.version == 1));
}

#[test]
fn without_keyboard_ui_it_cannot_get_the_face() {
    let rig = Rig::new();
    let mut host = FakeHost::new(&MANIFEST);
    provide(&mut host, Arc::new(FakeFace::default()));
    host.granted.clear();
    let mut module = UiModule::new(Arc::clone(&rig.hub));
    let err = module.start(&mut ModuleCx::new(&mut host)).err();
    let refused = ServiceError::NotGranted(KeyboardFaceKey::ID, KeyboardUi);
    assert_eq!(err, Some(ModuleError::Service(refused)));
    assert_eq!(rig.hub.board_json(), "null", "nothing was attached");
}

#[test]
fn a_start_hands_the_settings_and_face_to_the_hub_and_a_stop_takes_the_face_back() {
    let rig = Rig::new();
    let mut host = FakeHost::new(&MANIFEST);
    host.put("tip_ms", 700.into());
    provide(&mut host, Arc::new(FakeFace::default()));
    let mut module = UiModule::new(Arc::clone(&rig.hub));
    assert!(module.start(&mut ModuleCx::new(&mut host)).is_ok());
    assert_eq!(rig.hub.settings().tip_ms, 700);
    assert_eq!(rig.hub.board_json(), r#"{"board":1}"#);
    assert_eq!(rig.wakes(), [Wake::Settings]);
    module.stop();
    assert_eq!(rig.hub.board_json(), "null");
    assert_eq!(rig.hub.settings().tip_ms, 700, "the last settings stay");
}

#[test]
fn bad_settings_or_no_face_fail_the_start() {
    let rig = Rig::new();
    let mut module = UiModule::new(Arc::clone(&rig.hub));
    let mut host = FakeHost::new(&MANIFEST);
    host.put("page_rows", 0.into());
    provide(&mut host, Arc::new(FakeFace::default()));
    let err = module.start(&mut ModuleCx::new(&mut host)).err();
    assert!(
        matches!(err, Some(ModuleError::Settings(SettingsError::Invalid(_)))),
        "{err:?}"
    );
    let mut bare = FakeHost::new(&MANIFEST);
    let err = module.start(&mut ModuleCx::new(&mut bare)).err();
    let missing = ServiceError::Missing(KeyboardFaceKey::ID);
    assert_eq!(err, Some(ModuleError::Service(missing)));
    assert_eq!(rig.hub.board_json(), "null", "nothing was attached");
}

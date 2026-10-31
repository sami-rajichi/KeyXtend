//! The module's start on its settings, its compiled-in board and the three typing services.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

use crate::host::{FakeHost, fake, store};
use kx_mod_keyboard::spec::char_id;
use kx_mod_keyboard::{
    BoardSpec, ID, KeyboardModule, KeyboardSettings, MANIFEST, Ports, Tapped, Typing,
};
use kx_module_api::Capability::{
    InjectInput, KeyboardUi, ReadClipboard, ReadFocusedText, WriteClipboard,
};
use kx_module_api::ports::{
    ClipKey, ClipPanel, FaceAction, InputInjectorKey, KeyId, KeyboardFace, KeyboardFaceKey,
    LayoutsKey, PasteAsk, Scan, ShortcutsKey, TextRoute, TypeWord,
};
use kx_module_api::{
    Bus, Capability, Event, EventError, Module, ModuleCx, ModuleError, Redacted, ServiceError,
    ServiceKey, SettingsError,
};
use kx_platform_fake::Stroke;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;

/// The capabilities the module declares, from its manifest.
fn all() -> Vec<Capability> {
    MANIFEST.capabilities.to_vec()
}

fn start(host: &mut FakeHost) -> Result<(), ModuleError> {
    KeyboardModule::default().start(&mut ModuleCx::new(host))
}

#[test]
fn it_starts_on_the_defaults_the_shipped_board_and_the_typing_services() {
    let mut host = fake(None);
    store(&mut host, None);
    host.granted = all();
    assert_eq!(start(&mut host), Ok(()));
}

#[test]
fn it_provides_the_face_behind_keyboard_ui() {
    let mut host = fake(None);
    store(&mut host, None);
    host.granted = all();
    start(&mut host).unwrap();
    let (gate, service) = &host.services[&KeyboardFaceKey::ID];
    assert_eq!(*gate, Some(KeyboardUi));
    assert!(service.downcast_ref::<Arc<dyn KeyboardFace>>().is_some());
}

#[test]
fn without_keyboard_ui_it_cannot_offer_the_face() {
    let mut host = fake(None);
    store(&mut host, None);
    host.granted = vec![InjectInput];
    let refused = ServiceError::NotGranted(KeyboardFaceKey::ID, KeyboardUi);
    assert_eq!(start(&mut host), Err(ModuleError::Service(refused)));
    assert!(!host.services.contains_key(&KeyboardFaceKey::ID));
}

#[test]
fn without_read_focused_text_it_cannot_hear_picked_words() {
    let mut host = fake(None);
    store(&mut host, None);
    host.granted = vec![InjectInput, KeyboardUi];
    let refused = EventError::NotGranted(TypeWord::NAME, ReadFocusedText);
    assert_eq!(start(&mut host), Err(ModuleError::Event(refused)));
}

/// The capabilities the module declares but `gone`.
fn all_but(gone: Capability) -> Vec<Capability> {
    all().into_iter().filter(|&c| c != gone).collect()
}

#[test]
fn without_write_clipboard_it_cannot_hear_paste_requests() {
    let mut host = fake(None);
    store(&mut host, None);
    host.granted = all_but(WriteClipboard);
    let refused = EventError::NotGranted(PasteAsk::NAME, WriteClipboard);
    assert_eq!(start(&mut host), Err(ModuleError::Event(refused)));
}

/// How many keys the open clipboard panel hears when the module holds `granted`, and what reached the app.
fn panel_keys(granted: Vec<Capability>) -> (usize, Vec<Stroke>) {
    let mut host = fake(None);
    let fakes = store(&mut host, None);
    host.granted = granted;
    start(&mut host).unwrap();
    let bus = Bus::new(Arc::clone(&host.core));
    let heard = Arc::new(AtomicUsize::new(0));
    let h = Arc::clone(&heard);
    let _ear = bus
        .subscribe(move |_: &ClipKey| {
            h.fetch_add(1, SeqCst);
        })
        .unwrap();
    bus.publish(&ClipPanel { open: true }).unwrap();
    let (_, service) = &host.services[&KeyboardFaceKey::ID];
    let face = service.downcast_ref::<Arc<dyn KeyboardFace>>().unwrap();
    let id = KeyId::new(char_id(Scan::new(0x10)));
    face.act(FaceAction::Tap { id, back: false }).unwrap();
    (heard.load(SeqCst), fakes.injector.strokes())
}

#[test]
fn with_read_clipboard_the_open_panel_hears_a_key_the_app_never_gets() {
    assert_eq!(panel_keys(all()), (1, Vec::new()));
}

#[test]
fn without_read_clipboard_it_cannot_hear_the_panel() {
    let mut host = fake(None);
    store(&mut host, None);
    host.granted = all_but(ReadClipboard);
    let refused = EventError::NotGranted(ClipPanel::NAME, ReadClipboard);
    assert_eq!(start(&mut host), Err(ModuleError::Event(refused)));
}

#[test]
fn without_inject_input_it_cannot_reach_the_injector() {
    let mut host = fake(None);
    store(&mut host, None);
    let refused = ServiceError::NotGranted(InputInjectorKey::ID, InjectInput);
    assert_eq!(start(&mut host), Err(ModuleError::Service(refused)));
}

#[test]
fn without_the_layouts_service_it_fails_to_start() {
    let mut host = fake(None);
    store(&mut host, Some(LayoutsKey::ID));
    host.granted = all();
    let missing = ServiceError::Missing(LayoutsKey::ID);
    assert_eq!(start(&mut host), Err(ModuleError::Service(missing)));
}

#[test]
fn without_the_shortcuts_service_it_fails_to_start() {
    let mut host = fake(None);
    store(&mut host, Some(ShortcutsKey::ID));
    host.granted = all();
    let missing = ServiceError::Missing(ShortcutsKey::ID);
    assert_eq!(start(&mut host), Err(ModuleError::Service(missing)));
}

#[test]
fn bad_settings_stop_the_start() {
    let mut host = fake(Some(("size", 9.0.into())));
    assert!(matches!(
        start(&mut host),
        Err(ModuleError::Settings(SettingsError::Invalid(_)))
    ));
}

#[test]
fn its_handle_writes_its_own_section() {
    let mut host = fake(None);
    host.handle().set("size", 1.25.into()).unwrap();
    let writes = host.sink.writes();
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0].0, ID);
}

#[test]
fn a_tap_through_the_kernels_ports_types_in_the_layout_shown() {
    let mut host = fake(None);
    let fakes = store(&mut host, None);
    host.granted = all();
    let ports = Ports::from_cx(&ModuleCx::new(&mut host), fakes.pause.clone()).unwrap();
    let settings = KeyboardSettings::from_defaults().unwrap();
    let mut typing = Typing::new(&BoardSpec::shipped().unwrap(), &settings);
    let q = char_id(Scan::new(0x10));
    let table = ports.layouts.current();
    assert_eq!(
        typing.tap(&ports.io(&table, &[]), &q, false),
        Ok(Tapped::Done)
    );
    let ids: Vec<_> = ports.layouts.list().iter().map(|l| l.id).collect();
    let (en, fr) = (ids[0], ids[1]);
    assert_eq!(table.label.id, en);
    assert!(fakes.layouts.show(fr));
    let table = ports.layouts.current();
    typing.tap(&ports.io(&table, &[]), &q, false).unwrap();
    let text = |t: &str| Stroke::Text(Redacted::new(t.into()), TextRoute::Unicode);
    assert_eq!(fakes.injector.strokes(), [text("q"), text("a")], "AZERTY");
}

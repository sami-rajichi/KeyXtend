//! The `KeyboardUi` gate (ADR-0033): only a module holding it may offer or get the keyboard face, or send and hear
//! the pointer notes; another grant never opens it.

use kx_module_api::Capability::{InjectInput, KeyboardUi};
use kx_module_api::ports::{
    AssistNote, FaceAction, FaceError, FaceReply, KeyboardFace, KeyboardFaceKey, Point,
    PointerMoved,
};
use kx_module_api::{
    Capability, Event, EventError, Host, Manifest, ModuleCx, ModuleId, ServiceError, ServiceKey,
};
use kx_test_support::host::FakeHost;
use std::sync::{Arc, Mutex};

/// A ui module that draws the face, declaring `InjectInput` too; each test grants it some, as the kernel's policy
/// would.
static UI: Manifest = Manifest {
    id: ModuleId::new("ui"),
    version: env!("CARGO_PKG_VERSION"),
    requires: &[KeyboardFaceKey::ID],
    provides: &[],
    capabilities: &[KeyboardUi, InjectInput],
    settings: None,
};

/// A keyboard that offers the face, declaring the same two.
static KEYBOARD: Manifest = Manifest {
    id: ModuleId::new("keyboard"),
    version: env!("CARGO_PKG_VERSION"),
    requires: &[],
    provides: &[KeyboardFaceKey::ID],
    capabilities: &[KeyboardUi, InjectInput],
    settings: None,
};

/// A point for the notes; nothing reads it.
const AT: Point = Point { x: 1, y: 2 };

/// A face with an empty board.
struct Blank;

impl KeyboardFace for Blank {
    fn board_json(&self) -> String {
        String::new()
    }

    fn state_json(&self, _: usize) -> String {
        String::new()
    }

    fn lang(&self) -> String {
        String::new()
    }

    fn row_px(&self) -> f64 {
        0.0
    }

    fn act(&self, _: FaceAction) -> Result<FaceReply, FaceError> {
        Ok(FaceReply::default())
    }
}

/// A ui host granted `granted`, holding the blank face as the keyboard would offer it.
fn ui(granted: &[Capability]) -> FakeHost {
    let face: Arc<dyn KeyboardFace> = Arc::new(Blank);
    let mut host = FakeHost::new(&UI);
    host.granted.extend(granted);
    let stored = (KeyboardFaceKey::CAPABILITY, Box::new(face) as _);
    host.services.insert(KeyboardFaceKey::ID, stored);
    host
}

/// The refusal of `E` for want of `KeyboardUi`.
fn refused<E: Event>() -> EventError {
    EventError::NotGranted(E::NAME, KeyboardUi)
}

#[test]
fn a_module_without_keyboard_ui_cannot_get_the_face() {
    let mut host = ui(&[InjectInput]);
    let cx = ModuleCx::new(&mut host);
    let refused = ServiceError::NotGranted(KeyboardFaceKey::ID, KeyboardUi);
    assert_eq!(cx.service::<KeyboardFaceKey>().err(), Some(refused));
}

#[test]
fn a_module_holding_keyboard_ui_gets_the_face() {
    let mut host = ui(&[KeyboardUi]);
    let face = ModuleCx::new(&mut host)
        .service::<KeyboardFaceKey>()
        .unwrap();
    assert_eq!(face.act(FaceAction::Relabel), Ok(FaceReply::default()));
}

#[test]
fn a_keyboard_without_keyboard_ui_cannot_offer_the_face() {
    let mut host = FakeHost::new(&KEYBOARD);
    host.granted.push(InjectInput);
    let refused = ServiceError::NotGranted(KeyboardFaceKey::ID, KeyboardUi);
    let offer = ModuleCx::new(&mut host).provide::<KeyboardFaceKey>(Arc::new(Blank));
    assert_eq!(offer, Err(refused));
    host.granted.push(KeyboardUi);
    let offer = ModuleCx::new(&mut host).provide::<KeyboardFaceKey>(Arc::new(Blank));
    assert_eq!(offer, Ok(()));
}

#[test]
fn without_keyboard_ui_the_pointer_notes_neither_go_nor_come() {
    let bus = ui(&[InjectInput]).bus();
    let note = bus.publish(&AssistNote::Fired { at: AT });
    assert_eq!(note, Err(refused::<AssistNote>()));
    let moved = bus.publish(&PointerMoved { at: AT });
    assert_eq!(moved, Err(refused::<PointerMoved>()));
    let heard = bus.subscribe(|_: &AssistNote| {}).err();
    assert_eq!(heard, Some(refused::<AssistNote>()));
}

#[test]
fn with_keyboard_ui_a_note_reaches_its_listener() {
    let bus = ui(&[KeyboardUi]).bus();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::clone(&seen);
    let _sub = bus
        .subscribe(move |n: &AssistNote| log.lock().unwrap().push(*n))
        .unwrap();
    bus.publish(&AssistNote::HoldEnd).unwrap();
    assert_eq!(bus.publish(&PointerMoved { at: AT }), Ok(()));
    assert_eq!(*seen.lock().unwrap(), [AssistNote::HoldEnd]);
}

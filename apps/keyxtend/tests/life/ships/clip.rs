//! The clipboard history's grants (ADR-0034): its feature is a default and the build ships it, it holds the three
//! gates, the keyboard adds both clipboard gates and the ui the read one, and the watch, the writer and the shelf each
//! reach only the modules holding their gate.

use super::{defaults, features, holders, reached};
use crate::rig::Rig;
use keyxtend::boot::modules;
use kx_module_api::ports::{
    Clipboard, ClipboardKey, ClipboardWriter, ClipboardWriterKey, SealedShelf, SealedShelfKey,
};
use kx_module_api::{Capability, ModuleId};
use kx_platform_fake::{FakeClipboard, FakeClipboardWriter, FakeShelf};
use std::collections::BTreeSet;
use std::sync::Arc;

/// The clipboard module's feature.
const CLIP_FEATURE: &str = "mod-clipboard";

#[test]
fn mod_clipboard_is_a_default_feature_and_the_default_build_ships_it() {
    assert!(defaults(&features()).contains(&CLIP_FEATURE));
    let shipped = modules(&Rig::new().load().hub);
    let ids: Vec<ModuleId> = shipped.iter().map(|m| m.manifest().id).collect();
    assert!(ids.contains(&kx_mod_clipboard::ID), "{ids:?}");
}

#[test]
fn the_clipboard_holds_read_write_and_the_secret_store() {
    let want = [
        Capability::ReadClipboard,
        Capability::WriteClipboard,
        Capability::SecretStore,
    ];
    assert_eq!(kx_mod_clipboard::MANIFEST.capabilities, want);
}

#[test]
fn the_clipboard_gates_go_to_the_clipboard_the_keyboard_the_ui_and_predict() {
    let (clip, keyboard, ui) = (kx_mod_clipboard::ID, kx_mod_keyboard::ID, kx_ui::ID);
    let read = BTreeSet::from([clip, keyboard, ui]);
    assert_eq!(
        holders(Capability::ReadClipboard),
        read,
        "the panel's events"
    );
    let write = BTreeSet::from([clip, keyboard]);
    assert_eq!(holders(Capability::WriteClipboard), write, "the paste ask");
    let secret = BTreeSet::from([clip, kx_mod_predict::ID]);
    assert_eq!(holders(Capability::SecretStore), secret, "the sealed files");
}

#[test]
fn the_clipboard_watch_reaches_only_the_read_clipboard_holders() {
    let watch: Arc<dyn Clipboard> = Arc::new(FakeClipboard::new());
    let got = reached::<ClipboardKey>(&Rig::new(), &watch);
    assert_eq!(got, holders(Capability::ReadClipboard));
}

#[test]
fn the_clipboard_writer_reaches_only_the_write_clipboard_holders() {
    let writer: Arc<dyn ClipboardWriter> = Arc::new(FakeClipboardWriter::new());
    let got = reached::<ClipboardWriterKey>(&Rig::new(), &writer);
    assert_eq!(got, holders(Capability::WriteClipboard));
}

#[test]
fn the_sealed_shelf_reaches_only_the_secret_store_holders() {
    let shelf: Arc<dyn SealedShelf> = Arc::new(FakeShelf::new());
    let got = reached::<SealedShelfKey>(&Rig::new(), &shelf);
    assert_eq!(got, holders(Capability::SecretStore));
}

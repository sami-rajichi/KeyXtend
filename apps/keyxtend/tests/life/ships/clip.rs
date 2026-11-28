//! The clipboard history's grants (ADR-0034): its feature is a default and the build ships it, it holds the three
//! gates, the keyboard adds both clipboard gates, the ui the read one, Quick-fill the read one for Paste's take, voice
//! typing both for Copy and the key page's Paste (ADR-0047), Snip the write one for Copy (ADR-0056), the clipboard
//! alone asks for the watch, and the watch, the writer and the shelf each reach only the modules holding their gate.

use super::{asking, defaults, features, holders, reached};
use crate::rig::{OS, Rig};
use keyxtend::boot::modules;
use keyxtend::hidden::Run;
use kx_module_api::ports::{
    Clipboard, ClipboardKey, ClipboardWriter, ClipboardWriterKey, SealedShelf, SealedShelfKey,
};
use kx_module_api::{Capability, ModuleId, ServiceKey};
use kx_platform_fake::{FakeClipboard, FakeClipboardWriter, FakeShelf};
use std::collections::BTreeSet;
use std::sync::Arc;

/// The clipboard module's feature.
const CLIP_FEATURE: &str = "mod-clipboard";

/// The modules holding `ReadClipboard`: the panel's events and the Paste takes, written once for every test.
pub(super) fn readers() -> BTreeSet<ModuleId> {
    let (clip, keyboard, ui) = (kx_mod_clipboard::ID, kx_mod_keyboard::ID, kx_ui::ID);
    BTreeSet::from([clip, keyboard, ui, kx_mod_vault::ID, kx_mod_voice::ID])
}

#[test]
fn mod_clipboard_is_a_default_feature_and_the_default_build_ships_it() {
    assert!(defaults(&features()).contains(&CLIP_FEATURE));
    let shipped = modules(&Rig::new().load().hub, OS, &Run::Normal);
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
fn the_clipboard_gates_go_to_the_clipboard_the_keyboard_the_ui_quick_fill_predict_voice_and_snip() {
    let (clip, keyboard) = (kx_mod_clipboard::ID, kx_mod_keyboard::ID);
    assert_eq!(
        holders(Capability::ReadClipboard),
        readers(),
        "the panel's events and the Paste takes"
    );
    let write = BTreeSet::from([clip, keyboard, kx_mod_voice::ID, kx_mod_snip::ID]);
    assert_eq!(
        holders(Capability::WriteClipboard),
        write,
        "the paste ask, the bar's Copy and Snip's Copy"
    );
    let secret = BTreeSet::from([clip, kx_mod_predict::ID, kx_mod_voice::ID]);
    assert_eq!(holders(Capability::SecretStore), secret, "the sealed files");
}

#[test]
fn only_the_clipboard_asks_for_the_watch_so_the_takes_read_through_it() {
    let watch = asking(&[ClipboardKey::ID]);
    assert_eq!(watch, BTreeSet::from([kx_mod_clipboard::ID]), "ADR-0047");
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

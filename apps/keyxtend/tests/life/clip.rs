//! The clipboard history over the whole app: it starts with its services and Quit lets them go, a copy shows in the
//! panel the keyboard's clip key opens, Insert sets the clipboard and the keyboard pastes it, the hub opens the history
//! folder under the data folder, and switched off it touches nothing.

use crate::rig::{Rig, WAIT, hear};
use keyxtend::boot::Boot;
use keyxtend::clip::CLIP_DIR;
use keyxtend::drain;
use keyxtend::words::SEALED_DIR;
use kx_module_api::ModuleState::Active;
use kx_module_api::Redacted;
use kx_module_api::ports::{
    ClipNote, ClipPart, ClipView, FaceAction, KeyId, KeyName, PASTE_ID, PasteAsk, Pasted,
    ShortcutId, ShortcutInfo, Step,
};
use kx_platform::PORTABLE_DIR;
use kx_platform_fake::ShelfCall;
use kx_ui::clip_acts;
use std::sync::Arc;

/// The board's clip key, as `board.toml` names it.
const CLIP_KEY: &str = "clip";
/// The text the user copies.
const HELLO: &str = "hello";

/// The paste shortcut, Ctrl+V, as the platform's table lists it.
fn paste() -> ShortcutInfo {
    let name = |n: &str| KeyName::new(n).unwrap();
    ShortcutInfo {
        id: ShortcutId::new(PASTE_ID).unwrap(),
        steps: vec![Step {
            held: vec![name("ctrl")],
            key: name("v"),
        }],
    }
}

#[test]
fn the_clipboard_starts_with_its_services_and_quit_lets_them_go() {
    let rig = Rig::new();
    let mut boot = rig.load();
    boot.start().unwrap();
    assert_eq!(boot.kernel.state(kx_mod_clipboard::ID), Some(Active));
    assert!(rig.clip.wait_watching(WAIT), "it watches the clipboard");
    assert!(rig.pics.running(), "it opened a picture session");
    assert!(rig.shelf.calls(ShelfCall::Folders) > 0, "it read the shelf");
    let Boot {
        kernel,
        hub,
        requests,
        heard,
        ..
    } = boot;
    let thread = drain::spawn(kernel, Arc::clone(&hub), requests).unwrap();
    assert!(hub.quit());
    thread.join().unwrap();
    assert!(!rig.clip.watching() && !rig.pics.running(), "Quit let go");
    drop(heard);
}

#[test]
fn a_copy_shows_in_the_panel_and_insert_pastes_it_through_the_keyboard() {
    let rig = Rig::new();
    rig.shortcuts.set_known(vec![paste()]);
    let mut boot = rig.load();
    let bus = boot.kernel.bus();
    let ((_v, views), (_a, asks), (_p, answers)) = (
        hear::<ClipView>(&bus),
        hear::<PasteAsk>(&bus),
        hear::<Pasted>(&bus),
    );
    boot.start().unwrap();
    assert!(rig.clip.wait_watching(WAIT));
    let hello = ClipPart::Text(Redacted::new(HELLO.to_owned()));
    let parts = vec![hello.clone()];
    rig.clip.push(ClipNote::Copied { parts, app: None });
    let _ = boot.hub.act(FaceAction::Tap {
        id: KeyId::new(CLIP_KEY),
        back: false,
    });
    assert_eq!(views.recv_timeout(WAIT).unwrap().0.rows.len(), 1);
    assert!(
        boot.hub.clip_json().contains(HELLO),
        "the hub keeps the view"
    );
    boot.hub.clip_act(clip_acts::TICK, 0);
    boot.hub.clip_act(clip_acts::INSERT, 0);
    let ask = asks.recv_timeout(WAIT).unwrap();
    let answer = answers.recv_timeout(WAIT).unwrap();
    assert_eq!(
        (answer.round, answer.ok),
        (ask.round, true),
        "the keyboard answered"
    );
    assert_eq!(rig.writer.writes(), [vec![hello]]);
    let sent: Vec<ShortcutId> = rig.shortcuts.sent().into_iter().map(|s| s.0).collect();
    assert_eq!(sent, [paste().id], "one plain paste");
}

#[test]
fn open_folder_shows_the_history_folder_under_the_data_folder() {
    let rig = Rig::new();
    let boot = rig.load();
    assert!(boot.hub.open_folder());
    assert!(rig.reveal.wait_shown(1, WAIT));
    let want = rig.exe.path().join(PORTABLE_DIR).join(CLIP_DIR);
    assert!(
        want.is_dir(),
        "the boot makes the folder, so Open folder always opens"
    );
    assert_eq!(boot.no_clip_dir, None);
    assert_eq!(rig.reveal.shown(), [want]);
}

#[test]
fn a_file_in_the_history_folders_place_does_not_stop_the_boot() {
    let rig = Rig::new();
    std::fs::write(rig.exe.path().join(PORTABLE_DIR).join(CLIP_DIR), b"").unwrap();
    let mut boot = rig.load();
    assert!(
        boot.no_clip_dir.is_some(),
        "the boot keeps why, for the log"
    );
    boot.start().unwrap();
    // The real shelf refuses a file in the folder's place, so the clipboard may fail; the other modules still start.
    assert_eq!(boot.kernel.state(kx_mod_keyboard::ID), Some(Active));
}

#[test]
fn the_history_and_the_learned_words_keep_their_keys_in_folders_apart() {
    assert_ne!(CLIP_DIR, SEALED_DIR, "each folder holds its own key file");
}

#[test]
fn with_the_clipboard_switched_off_nothing_watches_reads_or_draws() {
    let rig = Rig::new();
    let section = format!("[kernel]\ndisabled = [\"{}\"]\n", kx_mod_clipboard::ID);
    std::fs::write(rig.files().settings, section).unwrap();
    let mut boot = rig.load();
    boot.start().unwrap();
    assert_ne!(boot.kernel.state(kx_mod_clipboard::ID), Some(Active));
    assert_eq!(boot.kernel.state(kx_mod_keyboard::ID), Some(Active));
    assert_eq!(rig.clip.watches(), 0, "no watch");
    assert_eq!(rig.shelf.calls(ShelfCall::Folders), 0, "no shelf");
    assert_eq!(rig.pics.opens(), 0, "no pictures");
}

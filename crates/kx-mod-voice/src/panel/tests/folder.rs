//! The models folder through the panel: the first Download asks for it on the drive with the most room, the folder
//! list refuses a climbing path (edge 38), and a read-only folder, a full drive or a missing one downloads nothing
//! with a note (edge 21); a name another file holds needs Delete file first.

use kx_module_api::Redacted;
use kx_module_api::ports::{Fingerprint, FolderScan};
use kx_test_support::paths::native_path;

use super::super::rig::{drive, model_file};
use super::*;
use crate::folder::OFFER;

#[test]
fn the_first_download_asks_for_the_folder_on_the_drive_with_the_most_room() {
    let mut rig = Rig::new();
    let out = rig.with(|p, f| p.download_for("en", f));
    assert_eq!(rig.panel.page(), Some(Page::Tab(VoiceTab::Models)));
    assert!(
        !out.iter().any(|d| matches!(d, Do::Fetch { .. })),
        "the folder comes first"
    );
    let offer = models(&rig.view().unwrap()).offer.clone().unwrap();
    let dir = native_path(r"D:\").join(OFFER[0]).join(OFFER[1]);
    assert_eq!(offer.path.expose(), &dir.to_string_lossy());
    assert_eq!(
        rig.click(&VoiceClick::Confirm(true)),
        [Do::Make(dir.clone())]
    );
    let path = toml::Value::String(dir.to_string_lossy().into());
    let made = [Do::Set(crate::settings::MODELS_DIR, path), Do::Scan];
    assert_eq!(
        rig.panel.made(&dir, Ok(())),
        made,
        "the folder is read first"
    );
    let out = rig.scan(Vec::new());
    let parakeet = row("parakeet-v3-q8");
    assert!(
        matches!(&out[..], [Do::Fetch { row, .. }] if *row == parakeet.id),
        "the default for English"
    );
    assert_eq!(rig.panel.folder().dir(), Some(dir.as_path()));
}

#[test]
fn a_copy_already_in_a_new_folder_is_not_downloaded_again() {
    let parakeet = row("parakeet-v3-q8");
    let mut rig = Rig::new();
    rig.with(|p, f| p.download_for("en", f));
    rig.click(&VoiceClick::Confirm(true));
    let dir = native_path(r"D:\").join(OFFER[0]).join(OFFER[1]);
    rig.panel.made(&dir, Ok(()));
    let read = rig.scan(vec![model_file("mine.gguf", parakeet.bytes)]);
    let file = dir.join("mine.gguf");
    let want = Do::Print {
        file,
        bytes: parakeet.bytes,
    };
    assert_eq!(read, [want], "no download while reading");
    let out = rig.print("mine.gguf", parakeet.bytes, parakeet.print);
    assert!(
        !out.iter().any(|d| matches!(d, Do::Fetch { .. })),
        "{out:?}"
    );
    assert!(!rig.panel.fetches().busy());
}

#[test]
fn a_download_asked_while_the_folder_is_read_waits_for_it() {
    let parakeet = row("parakeet-v3-q8");
    let mut rig = Rig::foldered();
    assert!(rig.panel.folder().reading(), "not scanned yet");
    let out = rig.with(|p, f| p.download_for("en", f));
    assert!(
        !out.iter().any(|d| matches!(d, Do::Fetch { .. })),
        "{out:?}"
    );
    let out = rig.scan(Vec::new());
    assert!(
        matches!(&out[..], [Do::Fetch { row, .. }] if *row == parakeet.id),
        "{out:?}"
    );
}

#[test]
fn a_name_another_file_holds_offers_delete_file_not_download() {
    let small = row("whisper-small-q5km");
    let mut rig = Rig::foldered();
    rig.scan(vec![model_file(&small.file, small.bytes)]);
    let stranger = Fingerprint::new([7; Fingerprint::BYTES]);
    rig.print(&small.file, small.bytes, stranger);
    rig.click(&VoiceClick::Open(VoiceTab::Models));
    let at = place(&mut rig, &small.name);
    rig.click(&VoiceClick::Row(at));
    let acts = rig.view().unwrap().acts;
    assert!(acts.contains(&PanelButton::Delete) && !acts.contains(&PanelButton::Download));
    let out = rig.click(&VoiceClick::Button(PanelButton::Delete));
    assert_eq!(
        out,
        [Do::Remove(Path::new(&dir()).join(&small.file)), Do::Scan]
    );
    assert!(rig.with(|p, f| p.download(&small.id, f)).is_empty());
    let view = rig.view().unwrap();
    assert_eq!(view.note, Some(BarHint::Taken));
    assert!(
        view.acts.contains(&PanelButton::Delete),
        "the row is ticked for Delete file"
    );
}

#[test]
fn no_to_the_folder_question_downloads_nothing() {
    let mut rig = Rig::new();
    rig.with(|p, f| p.download_for("en", f));
    assert!(rig.click(&VoiceClick::Confirm(false)).is_empty());
    assert_eq!(models(&rig.view().unwrap()).offer, None);
}

#[test]
fn the_folder_list_picks_a_folder_and_refuses_a_climbing_one() {
    let mut rig = Rig::new();
    rig.with(|p, f| p.download_for("en", f));
    rig.click(&VoiceClick::Button(PanelButton::Change));
    let out = rig.click(&VoiceClick::Row(1));
    assert_eq!(out, [Do::Browse(native_path(r"D:\"))]);
    let names = ["..", "Models"]
        .map(|n| Redacted::new(n.to_owned()))
        .to_vec();
    rig.panel.browsed(
        &native_path(r"D:\"),
        Ok(FolderScan {
            folders: names,
            files: Vec::new(),
        }),
    );
    assert!(
        rig.click(&VoiceClick::Row(0)).is_empty(),
        "a climbing folder"
    );
    assert_eq!(rig.view().unwrap().note, Some(BarHint::Folder));
    rig.click(&VoiceClick::Row(1));
    let mine = native_path(r"D:\").join("Models");
    assert_eq!(
        rig.click(&VoiceClick::Button(PanelButton::Here)),
        [Do::Make(mine.clone())]
    );
    rig.panel.made(&mine, Ok(()));
    assert!(
        rig.scan(Vec::new())
            .iter()
            .any(|d| matches!(d, Do::Fetch { .. })),
        "the waiting download starts"
    );
}

#[test]
fn a_folder_with_a_control_character_is_refused() {
    let mut rig = Rig::new();
    rig.with(|p, f| p.download_for("en", f));
    rig.click(&VoiceClick::Button(PanelButton::Change));
    rig.click(&VoiceClick::Row(1));
    let names = vec![Redacted::new("a\u{7}b".to_owned())];
    rig.panel.browsed(
        &native_path(r"D:\"),
        Ok(FolderScan {
            folders: names,
            files: Vec::new(),
        }),
    );
    assert!(rig.click(&VoiceClick::Row(0)).is_empty());
    assert_eq!(rig.view().unwrap().note, Some(BarHint::Folder));
}

#[test]
fn a_read_only_folder_or_a_full_drive_downloads_nothing_and_says_so() {
    let mut rig = Rig::new();
    rig.with(|p, f| p.download_for("en", f));
    rig.click(&VoiceClick::Confirm(true));
    let dir = native_path(r"D:\").join(OFFER[0]).join(OFFER[1]);
    assert!(rig.panel.made(&dir, Err(ModelError::ReadOnly)).is_empty());
    assert_eq!(rig.view().unwrap().note, Some(BarHint::Folder));
    let mut full = Rig::foldered();
    full.drives = vec![drive(r"D:\", MB_TEST)];
    full.scan(Vec::new());
    let out = full.with(|p, f| p.download_for("en", f));
    assert!(
        !out.iter().any(|d| matches!(d, Do::Fetch { .. })),
        "min_free_mb not met"
    );
    assert_eq!(full.view().unwrap().note, Some(BarHint::Full));
    let mut gone = Rig::foldered();
    gone.drives = vec![drive(r"E:\", 900_000 * MB_TEST)];
    gone.scan(Vec::new());
    gone.with(|p, f| p.download_for("en", f));
    assert_eq!(
        gone.view().unwrap().note,
        Some(BarHint::Folder),
        "a missing drive"
    );
}

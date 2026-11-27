//! Snip's files: the dialogs need `SaveFile`, playing `OpenFolder` and a kept copy `WriteClipboard`, as the kernel
//! checks. The `example_` tests show the ports' contracts on the stand-in `Stub`: a snip reaches a chosen place only
//! through the Save dialog's answer, the `unsaved` part lists and deletes only its own, and a shot encodes with no
//! gate.

use super::stub::{limits, unsaved, video};
use super::{NOW, host, refuses};
use kx_module_api::Capability::{OpenFolder, SaveFile, ScreenCapture, WriteClipboard};
use kx_module_api::ports::{
    AskError, BGRX_BYTES, ClipPart, ClipboardWriterKey, EncodeAsk, FileAskKey, FileKind,
    ImageCodec, MediaOpenKey, PickLimits, PictureConfig, PicturesKey, SaveError, Shot, SnipFile,
    SnipFolderKey,
};
use kx_module_api::{ModuleCx, Redacted};

/// A picture the user would name in the Save dialog.
#[allow(clippy::unwrap_used, reason = "a plain drive path named as a picture")]
fn picture() -> SnipFile {
    SnipFile::new(r"D:\Pictures\a.png".into(), FileKind::Png).unwrap()
}

#[test]
fn the_dialogs_need_save_file_and_playing_needs_open_folder() {
    let mut bare = host(&[ScreenCapture, WriteClipboard]);
    let cx = ModuleCx::new(&mut bare);
    refuses::<FileAskKey>(&cx, SaveFile);
    refuses::<MediaOpenKey>(&cx, OpenFolder);
    let mut granted = host(&[SaveFile, OpenFolder]);
    let cx = ModuleCx::new(&mut granted);
    let reached = (cx.service::<FileAskKey>(), cx.service::<MediaOpenKey>());
    assert!(reached.0.is_ok() && reached.1.is_ok());
}

/// On the stand-in: the answer writes only its own kind, and copies only a video of `unsaved`.
#[test]
fn example_a_snip_reaches_a_chosen_place_only_through_the_dialogs_answer() {
    let mut granted = host(&[SaveFile]);
    let dialogs = ModuleCx::new(&mut granted).service::<FileAskKey>().unwrap();
    let answer = dialogs.save(&picture(), &limits()).unwrap().unwrap();
    assert_eq!(answer.file(), &picture(), "the stub's user keeps the name");
    let png = Redacted::new(vec![1, 2, 3]);
    assert_eq!(answer.png(&png), Ok(()));
    let copied = answer.copy(&video("a.mp4"));
    assert_eq!(
        copied,
        Err(SaveError::BadPath),
        "a picture's answer takes no video"
    );
    let film = SnipFile::new(r"D:\Videos\a.mp4".into(), FileKind::Mp4).unwrap();
    let answer = dialogs.save(&film, &limits()).unwrap().unwrap();
    assert_eq!(answer.copy(&video("a.mp4")), Ok(()));
    assert_eq!(
        answer.copy(&film),
        Err(SaveError::BadPath),
        "not of `unsaved`"
    );
}

/// On the stand-in, as every dialog host must: limits `check` refuses show no dialog.
#[test]
fn example_the_dialogs_refuse_limits_outside_their_range() {
    let mut granted = host(&[SaveFile]);
    let dialogs = ModuleCx::new(&mut granted).service::<FileAskKey>().unwrap();
    let bad = PickLimits {
        job_mb: 0,
        ..limits()
    };
    let saved = dialogs.save(&picture(), &bad).err();
    assert_eq!(saved, Some(AskError::Range("job_mb")));
    let folder = dialogs.folder(&unsaved(), &bad);
    assert_eq!(folder, Err(AskError::Range("job_mb")));
}

/// On the stand-in: `unsaved` lists files that lie in it with their marks, and marks or deletes nothing elsewhere.
#[test]
fn example_the_unsaved_part_lists_its_videos_with_ages_and_deletes_only_its_own() {
    let mut granted = host(&[ScreenCapture]);
    let folder = ModuleCx::new(&mut granted)
        .service::<SnipFolderKey>()
        .unwrap();
    let listed = folder.unsaved().unwrap();
    assert!(!listed.is_empty(), "the stub keeps one");
    assert!(
        listed.iter().all(|f| f.file.lies_in(&unsaved())),
        "{listed:?}"
    );
    assert!(listed.iter().all(|f| f.copied.is_some()), "a copied one");
    assert_eq!(folder.mark_copied(&video("a.mp4")), Ok(()));
    assert_eq!(folder.prune_marks(), Ok(()));
    assert_eq!(folder.remove(&video("a.mp4")), Ok(()));
    let outside = SnipFile::new(r"D:\Videos\a.mp4".into(), FileKind::Mp4).unwrap();
    assert_eq!(folder.mark_copied(&outside), Err(SaveError::BadPath));
    assert_eq!(folder.remove(&outside), Err(SaveError::BadPath));
}

/// The pictures port needs no gate; on the stand-in, a shot within the bound encodes.
#[test]
fn example_a_picture_is_encoded_by_the_media_worker_with_no_gate() {
    let mut bare = host(&[]);
    let pictures = ModuleCx::new(&mut bare).service::<PicturesKey>().unwrap();
    let cfg = PictureConfig {
        deadline_ms: 1,
        idle_s: 1,
        memory_mb: 1,
        max_pixels: 1,
        restart_ms: 1,
        restart_max_ms: 1,
    };
    let shot = Shot::new(1, 1, vec![0; BGRX_BYTES]).unwrap();
    let png = pictures.open(cfg).encode(EncodeAsk { shot });
    assert!(
        png.is_ok_and(|p| !p.expose().is_empty()),
        "ADR-0056 decision 2"
    );
}

/// The writer needs `WriteClipboard`; on the stand-in, a kept copy takes a PNG part and a DIB part together.
#[test]
fn example_a_kept_copy_and_its_dib_go_through_the_writers_own_gate() {
    let mut bare = host(&[ScreenCapture, SaveFile]);
    refuses::<ClipboardWriterKey>(&ModuleCx::new(&mut bare), WriteClipboard);
    let mut granted = host(&[WriteClipboard]);
    let writer = ModuleCx::new(&mut granted)
        .service::<ClipboardWriterKey>()
        .unwrap();
    let part = |codec| ClipPart::Image {
        codec,
        bytes: Redacted::new(vec![0; 4]),
    };
    let parts = [part(ImageCodec::Png), part(ImageCodec::Dib)];
    assert_eq!(
        writer.write_kept(&parts, NOW),
        Ok(()),
        "both histories keep it"
    );
    assert!(
        writer.write(&parts, NOW).is_err(),
        "the stub takes only kept copies"
    );
}

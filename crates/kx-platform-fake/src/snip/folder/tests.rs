//! The fake Snips folder, held to the port: a table in memory, the unsaved part listed with ages by the clock, deletes
//! and copy marks only there, free room and failures as set, a hidden run with no folder, and a `Debug` that shows no
//! path or bytes.

use super::*;
use crate::FakeClock;
use crate::snip::tests::{PNG, SNIPS, UNSAVED, dir, file, png, rig};
use kx_module_api::ports::FileKind;
use std::time::Duration;

/// A video's length.
const LENGTH_MS: u32 = 4_200;
/// The room the test sets.
const FREE: u64 = 3 << 30;
/// A day, the shortest `copied_days`.
const DAY: Duration = Duration::from_hours(24);
/// An hour.
const HOUR: Duration = Duration::from_hours(1);

/// A video `LENGTH_MS` long.
fn video() -> Stored {
    Stored::Video {
        length_ms: LENGTH_MS,
    }
}

#[test]
fn a_new_folder_names_its_snips_holds_nothing_and_has_room_without_end() {
    let (_, folder) = rig();
    assert_eq!(folder.snips(), Some(dir(SNIPS)));
    assert_eq!(folder.unsaved(), Ok(Vec::new()));
    assert!(folder.files().is_empty());
    assert_eq!(folder.free(), Ok(u64::MAX));
}

#[test]
fn the_unsaved_part_lists_pictures_and_videos_there_with_ages_by_the_clock() {
    let (clock, folder) = rig();
    let old = file(UNSAVED, "a.mp4", FileKind::Mp4);
    let new = file(UNSAVED, "b.png", FileKind::Png);
    let saved = file(SNIPS, "c.png", FileKind::Png);
    folder.put(old.clone(), video());
    clock.advance(DAY);
    folder.put(new.clone(), png());
    folder.put(saved.clone(), png());
    clock.advance(HOUR);
    let ages = [(old, DAY + HOUR), (new.clone(), HOUR)];
    let want = ages.map(|(file, age)| UnsavedFile {
        file,
        age,
        copied: None,
    });
    assert_eq!(folder.unsaved(), Ok(want.to_vec()));
    assert_eq!(folder.files().len(), 3, "the saved one is held, not listed");
    let ahead = clock.now().saturating_add(DAY);
    folder.put_at(new, png(), ahead);
    let ages: Vec<_> = folder.unsaved().unwrap().iter().map(|u| u.age).collect();
    assert_eq!(
        ages,
        [DAY + HOUR, Duration::ZERO],
        "a future write is no age"
    );
}

#[test]
fn remove_deletes_only_a_file_directly_in_the_unsaved_part() {
    let (_, folder) = rig();
    let there = file(UNSAVED, "a.mp4", FileKind::Mp4);
    let saved = file(SNIPS, "c.mp4", FileKind::Mp4);
    folder.put(there.clone(), video());
    folder.put(saved.clone(), video());
    assert_eq!(folder.remove(&saved), Err(SaveError::BadPath));
    assert_eq!(folder.remove(&there), Ok(()));
    assert_eq!(
        folder.remove(&there),
        Ok(()),
        "one already gone is no error"
    );
    assert_eq!(folder.files(), std::slice::from_ref(&saved));
    assert_eq!(folder.removes(), [saved, there.clone(), there], "every ask");
}

#[test]
fn a_test_waits_for_removes_asked_on_another_thread() {
    let (_, folder) = rig();
    assert!(!folder.wait_removes(1, Duration::ZERO), "none yet");
    let other = folder.clone();
    let there = file(UNSAVED, "a.mp4", FileKind::Mp4);
    let asking = std::thread::spawn(move || other.remove(&there));
    assert!(folder.wait_removes(1, crate::snip::tests::WITHIN));
    assert_eq!(asking.join().unwrap(), Ok(()));
}

#[test]
fn failures_and_room_come_as_set() {
    let (_, folder) = rig();
    let there = file(UNSAVED, "a.mp4", FileKind::Mp4);
    folder.put(there.clone(), video());
    folder.set_list_error(Some(SaveError::Missing));
    folder.set_remove_error(Some(SaveError::ReadOnly));
    folder.set_free(Err(SaveError::Io));
    assert_eq!(folder.unsaved(), Err(SaveError::Missing));
    assert_eq!(folder.remove(&there), Err(SaveError::ReadOnly));
    assert_eq!(folder.free(), Err(SaveError::Io));
    assert_eq!(
        folder.stored(&there),
        Some(video()),
        "a refused delete keeps it"
    );
    folder.set_list_error(None);
    folder.set_remove_error(None);
    folder.set_free(Ok(FREE));
    assert_eq!(folder.free(), Ok(FREE));
    assert_eq!(folder.remove(&there), Ok(()));
    assert_eq!(folder.unsaved(), Ok(Vec::new()));
}

#[test]
fn a_file_held_for_a_few_removes_goes_at_the_next() {
    let (_, folder) = rig();
    let there = file(UNSAVED, "a.mp4", FileKind::Mp4);
    folder.put(there.clone(), video());
    folder.refuse_removes(SaveError::Io, 2);
    assert_eq!(folder.remove(&there), Err(SaveError::Io));
    assert_eq!(folder.remove(&there), Err(SaveError::Io));
    assert_eq!(folder.stored(&there), Some(video()), "still held");
    assert_eq!(folder.remove(&there), Ok(()));
    assert_eq!(folder.stored(&there), None);
    assert_eq!(folder.removes().len(), 3, "refused ones listed too");
}

#[test]
fn a_hidden_run_has_no_folder() {
    let clock = FakeClock::new(Mono::default());
    let folder = FakeSnipFolder::hidden(Arc::new(clock));
    let there = file(UNSAVED, "a.mp4", FileKind::Mp4);
    assert_eq!(folder.snips(), None);
    assert_eq!(folder.unsaved(), Err(SaveError::NoFolder));
    assert_eq!(folder.remove(&there), Err(SaveError::NoFolder));
    assert_eq!(folder.free(), Err(SaveError::NoFolder));
}

#[test]
fn clones_share_one_table_and_debug_shows_no_path_or_bytes() {
    let (_, folder) = rig();
    let port: Arc<dyn SnipFolder> = Arc::new(folder.clone());
    let picture = file(UNSAVED, "a.png", FileKind::Png);
    folder.put(picture.clone(), png());
    assert_eq!(port.unsaved().unwrap().len(), 1);
    assert_eq!(folder.stored(&picture), Some(png()));
    let shown = format!("{folder:?}");
    assert!(shown.contains("FakeSnipFolder"), "{shown}");
    assert!(!shown.contains("Alex"), "{shown}");
    assert!(!shown.contains(&PNG[1].to_string()), "{shown}");
}

#[test]
fn a_copy_mark_is_listed_with_its_age_and_goes_first_with_its_video() {
    let (clock, folder) = rig();
    let there = file(UNSAVED, "a.mp4", FileKind::Mp4);
    folder.put(there.clone(), video());
    assert_eq!(folder.mark_copied(&there), Ok(()));
    assert_eq!(folder.marked(&there), Some(clock.now()));
    clock.advance(HOUR);
    assert_eq!(folder.unsaved().unwrap()[0].copied, Some(HOUR));
    folder.refuse_removes(SaveError::Io, 1);
    assert_eq!(folder.remove(&there), Err(SaveError::Io));
    assert!(folder.marks().is_empty(), "the mark goes before the video");
    assert_eq!(folder.unsaved().unwrap()[0].copied, None);
    assert_eq!(folder.remove(&there), Ok(()));
}

#[test]
fn a_mark_is_made_only_on_a_held_video_of_the_unsaved_part() {
    let (_, folder) = rig();
    let picture = file(UNSAVED, "a.png", FileKind::Png);
    let saved = file(SNIPS, "b.mp4", FileKind::Mp4);
    let gone = file(UNSAVED, "c.mp4", FileKind::Mp4);
    let there = file(UNSAVED, "d.mp4", FileKind::Mp4);
    folder.put(picture.clone(), png());
    folder.put(saved.clone(), video());
    folder.put(there.clone(), video());
    assert_eq!(folder.mark_copied(&picture), Err(SaveError::BadPath));
    assert_eq!(folder.mark_copied(&saved), Err(SaveError::BadPath));
    assert_eq!(folder.mark_copied(&gone), Err(SaveError::Missing));
    folder.set_mark_error(Some(SaveError::Full));
    assert_eq!(folder.mark_copied(&there), Err(SaveError::Full));
    assert!(folder.marks().is_empty());
    let clock = FakeClock::new(Mono::default());
    let hidden = FakeSnipFolder::hidden(Arc::new(clock));
    assert_eq!(hidden.mark_copied(&there), Err(SaveError::NoFolder));
    assert_eq!(hidden.prune_marks(), Err(SaveError::NoFolder));
}

#[test]
fn prune_drops_only_the_marks_whose_video_is_gone() {
    let (clock, folder) = rig();
    let held = file(UNSAVED, "a.mp4", FileKind::Mp4);
    let gone = file(UNSAVED, "b.mp4", FileKind::Mp4);
    folder.put(held.clone(), video());
    folder.mark_at(held.clone(), clock.now());
    folder.mark_at(gone, clock.now());
    folder.set_list_error(Some(SaveError::Missing));
    assert_eq!(folder.prune_marks(), Err(SaveError::Missing));
    folder.set_list_error(None);
    assert_eq!(folder.prune_marks(), Ok(()));
    assert_eq!(folder.marks(), [held]);
}

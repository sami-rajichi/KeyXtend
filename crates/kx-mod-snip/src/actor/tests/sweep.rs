//! The sweep of the unsaved part at each start of the module and of a snip (edge 15): a video marked copied stays
//! `copied_days` from its mark, which outlives the app; every unmarked file, an old mark's video and a mark without its
//! video go, but a file a job still uses waits; nothing outside the unsaved part is touched.

use std::time::Duration;

use kx_module_api::ports::{AskKey, FileKind, Keep, SaveError, SnipClick, SnipFile, StripKey};
use kx_module_api::{Clock, Redacted};
use kx_platform_fake::{Pick, Stored};

use super::super::Leftovers;
use super::fakes::{CHOSEN, Fakes, SNIPS, UNSAVED, file};
use super::rig::{Rig, held, release};

/// A day.
const DAY: Duration = Duration::from_hours(24);
/// A video in the fake folder.
const VIDEO: Stored = Stored::Video { length_ms: 1_000 };

/// `files` in path order.
fn sorted(mut files: Vec<SnipFile>) -> Vec<SnipFile> {
    files.sort_by(|a, b| a.expose().cmp(b.expose()));
    files
}

/// The fakes of an app started again: the same folder, and no memory of the last run.
fn restarted(fake: &Fakes) -> Fakes {
    let mut next = fake.clone();
    next.leftovers = Leftovers::default();
    next
}

#[test]
fn the_start_deletes_all_but_young_marked_videos_and_their_marks() {
    let old = file(UNSAVED, "1.mp4", FileKind::Mp4);
    let young = file(UNSAVED, "2.mp4", FileKind::Mp4);
    let crashed = file(UNSAVED, "3.mp4", FileKind::Mp4);
    let picture = file(UNSAVED, "4.png", FileKind::Png);
    let orphan = file(UNSAVED, "5.mp4", FileKind::Mp4);
    let saved = file(SNIPS, "kept.mp4", FileKind::Mp4);
    let chosen = file(CHOSEN, "mine.mp4", FileKind::Mp4);
    let rig = Rig::build(|fake, _| {
        let (folder, start) = (&fake.folder, fake.clock.now());
        fake.clock.advance(DAY * 2);
        for video in [&old, &saved, &chosen, &young, &crashed] {
            folder.put_at(video.clone(), VIDEO, start);
        }
        folder.put(picture.clone(), Stored::Png(Redacted::new(vec![1])));
        folder.mark_at(old.clone(), start);
        folder.mark_at(young.clone(), fake.clock.now());
        folder.mark_at(orphan.clone(), start);
    });
    let gone = sorted(vec![old, crashed, picture]);
    assert_eq!(sorted(rig.fake.folder.removes()), gone);
    assert_eq!(
        rig.fake.folder.marks(),
        std::slice::from_ref(&young),
        "the orphan mark went too"
    );
    let kept = sorted(vec![chosen, young, saved]);
    let left = sorted(rig.fake.folder.files());
    assert_eq!(left, kept, "nothing outside the unsaved part");
}

#[test]
fn a_mark_just_under_copied_days_keeps_its_video() {
    let video = file(UNSAVED, "1.mp4", FileKind::Mp4);
    let rig = Rig::build(|fake, _| {
        fake.folder.put(video.clone(), VIDEO);
        fake.clock.advance(DAY * 2);
        fake.folder.mark_at(video.clone(), fake.clock.now());
        fake.clock
            .advance(DAY.saturating_sub(Duration::from_secs(1)));
    });
    assert!(
        rig.fake.folder.removes().is_empty(),
        "its file is older, its mark is not"
    );
}

#[test]
fn a_copied_video_outlives_an_app_restart_until_its_days_pass() {
    let mut rig = Rig::new();
    rig.recorded(1_000);
    let recording = rig.fake.recorder.runs()[0].file.clone();
    rig.keep(Keep::Copy);
    rig.advance(Duration::from_secs(2));
    let next = Rig::with(restarted(&rig.fake), |_, _| {});
    assert_eq!(next.fake.folder.files(), [recording], "copied, so kept");
    next.fake.clock.advance(DAY);
    let after = Rig::with(restarted(&next.fake), |_, _| {});
    assert!(after.fake.folder.files().is_empty());
    assert!(after.fake.folder.marks().is_empty());
}

#[test]
fn a_recording_a_crash_left_goes_at_the_next_start_however_young() {
    let video = file(UNSAVED, "1.mp4", FileKind::Mp4);
    let rig = Rig::build(|fake, _| fake.folder.put(video.clone(), VIDEO));
    assert_eq!(rig.fake.folder.removes(), [video]);
}

/// A rig whose recording's delete was refused once at its end.
fn refused() -> Rig {
    let mut rig = Rig::new();
    rig.recorded(1_000);
    let recording = rig.fake.recorder.runs()[0].file.clone();
    rig.fake.folder.refuse_removes(SaveError::Io, 1);
    rig.act(SnipClick::Strip(StripKey::Close));
    rig.act(SnipClick::Ask(AskKey::Drop));
    let left = rig.fake.folder.files();
    assert_eq!(left, [recording], "held by another program");
    rig
}

#[test]
fn a_delete_refused_is_made_at_the_next_start_of_the_app_or_the_module() {
    let rig = refused();
    let next = Rig::with(restarted(&rig.fake), |_, _| {});
    assert!(next.fake.folder.files().is_empty(), "unmarked, so it goes");
    let rig = refused();
    rig.fake.folder.set_list_error(Some(SaveError::Io));
    let next = Rig::with(rig.fake.clone(), |_, _| {});
    assert!(
        next.fake.folder.files().is_empty(),
        "known missed, even unlisted"
    );
}

#[test]
fn each_snip_start_sweeps_with_no_restart() {
    let mut rig = Rig::new();
    rig.recorded(1_000);
    rig.keep(Keep::Copy);
    rig.act(SnipClick::Strip(StripKey::Close));
    rig.advance(DAY);
    assert_eq!(rig.fake.folder.files().len(), 1, "no snip started yet");
    rig.key();
    assert!(rig.fake.folder.files().is_empty(), "its days passed");
    assert!(rig.fake.folder.marks().is_empty());
    let mut rig = refused();
    rig.key();
    assert!(
        rig.fake.folder.files().is_empty(),
        "the missed delete again"
    );
}

#[test]
fn a_file_a_job_still_uses_waits_for_a_later_sweep() {
    let chosen = file(CHOSEN, "clip.mp4", FileKind::Mp4);
    let mut rig = Rig::build(|fake, _| fake.dialogs.queue_save(Pick::File(chosen.clone())));
    rig.recorded(1_000);
    let recording = rig.fake.recorder.runs()[0].file.clone();
    rig.actor.ports.spawn = held;
    rig.keep(Keep::Save);
    let next = Rig::with(rig.fake.clone(), |_, _| {});
    let left = sorted(next.fake.folder.files());
    assert_eq!(
        left,
        [recording],
        "the module started again, the dialog still open"
    );
    release();
    assert!(next.fake.leftovers.lock().unwrap().used.is_empty());
    Rig::with(next.fake.clone(), |_, _| {});
    assert_eq!(
        next.fake.folder.files(),
        [chosen],
        "swept once the job let go"
    );
}

#[test]
fn a_hidden_run_sweeps_nothing() {
    let rig = Rig::with(Fakes::hidden(), |_, _| {});
    assert!(rig.fake.folder.removes().is_empty());
}

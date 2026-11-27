//! Quit: the key goes dark and the keyboard comes back; a Copy or Save in flight finishes, then what the snip made
//! goes but a video on the clipboard, even a re-timed copy that arrives as Quit is handled; a run that answers after
//! Quit, or that the actor drops, is cancelled and its video deleted.

use std::time::Duration;

use kx_module_api::ports::{
    BarKey, ClipError, FileKind, Keep, RecKey, SnipClick, SnipFile, SnipStage, StripKey,
};
use kx_platform_fake::Pick;

use super::super::{Back, Msg};
use super::ends::{WHOLE, since_start};
use super::fakes::{CHOSEN, file};
use super::rig::{Rig, drop_held, held, release};

#[test]
fn quit_ends_the_snip_with_the_key_dark_and_the_keyboard_back() {
    let mut rig = Rig::new();
    rig.snap();
    rig.send(Msg::Quit);
    assert_eq!(since_start(&rig), WHOLE);
    let mut rig = Rig::new();
    rig.record();
    rig.send(Msg::Quit);
    assert_eq!(since_start(&rig), WHOLE);
    assert_eq!(rig.fake.recorder.runs()[0].cancels, 1);
    assert!(rig.fake.folder.files().is_empty());
}

#[test]
fn quit_with_a_save_in_flight_lets_it_write_then_deletes_the_recording() {
    let chosen = file(CHOSEN, "clip.mp4", FileKind::Mp4);
    let mut rig = Rig::build(|fake, _| fake.dialogs.queue_save(Pick::File(chosen.clone())));
    rig.recorded(2_000);
    let recording = rig.fake.recorder.runs()[0].file.clone();
    rig.actor.ports.spawn = held;
    rig.keep(Keep::Save);
    rig.send(Msg::Quit);
    let reading = rig.fake.folder.stored(&recording).is_some();
    assert!(reading, "the job still reads it");
    release();
    rig.pump();
    let left = rig.fake.folder.files();
    assert_eq!(left, [chosen], "written, then the recording goes");
}

/// A recorded video at twice its speed on its strip, its Save asked of `chosen` and held.
fn sped_up_save(chosen: &SnipFile) -> Rig {
    let mut rig = Rig::build(|fake, _| fake.dialogs.queue_save(Pick::File(chosen.clone())));
    rig.recorded(4_000);
    rig.act(SnipClick::Strip(StripKey::Speed));
    rig.actor.ports.spawn = held;
    rig.keep(Keep::Save);
    rig
}

#[test]
fn quit_with_a_sped_up_save_in_flight_leaves_no_retimed_copy() {
    let chosen = file(CHOSEN, "fast.mp4", FileKind::Mp4);
    let mut rig = sped_up_save(&chosen);
    rig.send(Msg::Quit);
    release();
    rig.pump();
    assert_eq!(rig.fake.recorder.speeds().len(), 1);
    let left = rig.fake.folder.files();
    assert_eq!(left, [chosen], "neither the recording nor its copy stays");
}

#[test]
fn a_retimed_copy_whose_answer_comes_after_quit_is_deleted() {
    let chosen = file(CHOSEN, "fast.mp4", FileKind::Mp4);
    let mut rig = sped_up_save(&chosen);
    release();
    rig.send(Msg::Quit);
    let left = rig.fake.folder.files();
    assert_eq!(
        left,
        [chosen],
        "the job ended before Quit, its answer after"
    );
}

#[test]
fn quit_with_a_copy_in_flight_keeps_the_video_the_clipboard_holds() {
    let mut rig = Rig::new();
    rig.recorded(2_000);
    let recording = rig.fake.recorder.runs()[0].file.clone();
    rig.actor.ports.spawn = held;
    rig.keep(Keep::Copy);
    rig.send(Msg::Quit);
    release();
    rig.pump();
    assert_eq!(rig.fake.clip.writes().len(), 1);
    let left = rig.fake.folder.files();
    let one = std::slice::from_ref(&recording);
    assert_eq!(left, one, "a copied video stays its days");
    assert!(rig.fake.folder.marked(&recording).is_some());
}

#[test]
fn a_run_that_starts_after_quit_is_cancelled_and_its_video_deleted() {
    let mut rig = Rig::new();
    rig.take(BarKey::VideoRegion);
    rig.act(SnipClick::Strip(StripKey::Record));
    rig.actor.ports.spawn = held;
    rig.advance(Duration::from_secs(u64::from(rig.actor.held.countdown_s)));
    rig.send(Msg::Quit);
    release();
    rig.pump();
    assert_eq!(rig.fake.recorder.runs()[0].cancels, 1);
    assert!(rig.fake.folder.files().is_empty());
}

#[test]
fn a_run_out_in_a_call_at_quit_is_cancelled_once_the_call_ends() {
    let mut rig = Rig::new();
    rig.record();
    rig.actor.ports.spawn = held;
    rig.act(SnipClick::Record(RecKey::Sound));
    rig.send(Msg::Quit);
    assert_eq!(rig.fake.recorder.runs()[0].cancels, 0, "the call holds it");
    release();
    rig.pump();
    assert_eq!(rig.fake.recorder.runs()[0].cancels, 1);
    assert!(rig.fake.folder.files().is_empty());
}

/// A recorded video whose Copy the clipboard answers with `error`, the job held until Quit.
fn lost_at_quit(error: ClipError) -> Rig {
    let mut rig = Rig::build(|fake, _| fake.clip.queue(Err(error)));
    rig.recorded(2_000);
    rig.actor.ports.spawn = held;
    rig.keep(Keep::Copy);
    rig.send(Msg::Quit);
    release();
    rig.pump();
    rig
}

#[test]
fn quit_with_a_copy_that_never_reached_the_clipboard_deletes_its_video() {
    for rig in [
        lost_at_quit(ClipError::Refused),
        lost_at_quit(ClipError::Busy),
    ] {
        let left = rig.fake.folder.files();
        assert!(left.is_empty(), "never copied, so not kept: {left:?}");
    }
}

#[test]
fn quit_while_a_busy_clipboard_waits_deletes_the_video_it_was_to_copy() {
    let mut rig = Rig::build(|fake, _| fake.clip.queue(Err(ClipError::Busy)));
    rig.recorded(2_000);
    rig.keep(Keep::Copy);
    assert!(rig.actor.keeping.next().is_some(), "a try waits");
    rig.send(Msg::Quit);
    assert!(rig.fake.folder.files().is_empty());
}

/// A recorded video on its strip that a 1x Copy put on the clipboard; the clipboard answers the next write by `next`.
fn copied_once(next: Option<ClipError>) -> (Rig, SnipFile) {
    let mut rig = Rig::build(|fake, _| {
        fake.clip.queue(Ok(()));
        fake.clip.queue(next.map_or(Ok(()), Err));
    });
    rig.recorded(2_000);
    rig.keep(Keep::Copy);
    assert_eq!(rig.fake.clip.writes().len(), 1, "the first Copy is done");
    let recording = rig.fake.recorder.runs()[0].file.clone();
    (rig, recording)
}

/// True when `video` stays in the folder, marked copied.
fn still_copied(rig: &Rig, video: &SnipFile) -> bool {
    let folder = &rig.fake.folder;
    folder.stored(video).is_some() && folder.marked(video).is_some()
}

#[test]
fn quit_while_a_second_copy_waits_on_a_busy_clipboard_keeps_the_copied_video() {
    let (mut rig, recording) = copied_once(Some(ClipError::Busy));
    rig.keep(Keep::Copy);
    assert!(rig.actor.keeping.next().is_some(), "a try waits");
    rig.act(SnipClick::Strip(StripKey::Close));
    let shown = matches!(rig.stage(), SnipStage::Strip(_));
    assert!(shown, "Close waits for the Copy");
    rig.send(Msg::Quit);
    assert!(still_copied(&rig, &recording), "the clipboard holds it");
}

#[test]
fn a_second_copy_failing_after_quit_keeps_the_copied_video() {
    let (mut rig, recording) = copied_once(Some(ClipError::Refused));
    rig.actor.ports.spawn = held;
    rig.keep(Keep::Copy);
    rig.send(Msg::Quit);
    release();
    rig.pump();
    assert_eq!(rig.fake.clip.tries(), 2);
    assert!(still_copied(&rig, &recording), "the clipboard holds it");
}

#[test]
fn a_retimed_copy_dropped_after_quit_keeps_the_video_a_1x_copy_put_on_the_clipboard() {
    let (mut rig, recording) = copied_once(None);
    rig.act(SnipClick::Strip(StripKey::Speed));
    rig.actor.ports.spawn = held;
    rig.keep(Keep::Copy);
    rig.send(Msg::Quit);
    drop_held();
    rig.pump();
    assert!(rig.fake.recorder.speeds().is_empty(), "the job never ran");
    assert!(still_copied(&rig, &recording), "the clipboard holds it");
}

#[test]
fn a_dropped_retimed_copy_names_no_video_it_copied() {
    let mut rig = Rig::new();
    rig.recorded(4_000);
    rig.act(SnipClick::Strip(StripKey::Speed));
    rig.actor.ports.spawn = held;
    rig.keep(Keep::Copy);
    drop_held();
    let Some(Msg::Back(Back::Kept(kept))) = rig.next() else {
        panic!("its reply answers");
    };
    assert!(kept.into.is_some(), "the copy it was to make");
    assert!(kept.copied.is_none(), "the recording is not what it copies");
}

#[test]
fn an_actor_dropped_while_recording_cancels_the_run_and_deletes_its_video() {
    let mut rig = Rig::new();
    rig.record();
    let Rig { actor, fake, .. } = rig;
    drop(actor);
    assert_eq!(fake.recorder.runs()[0].cancels, 1);
    assert!(fake.folder.files().is_empty());
}

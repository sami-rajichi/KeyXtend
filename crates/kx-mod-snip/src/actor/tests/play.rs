//! Play at the speed picked: the re-timed copy written once per speed in a job, reused by Play and Copy, while the keys
//! wait, the one that asked shown waiting, and the strip stays on top. A speed job, a player or a job that fails says
//! so (edge 8).

use kx_module_api::ports::{
    AskKey, ClipPart, FileKind, Keep, PlayError, RecordError, SnipClick, SnipFile, SnipNote,
    SnipStage, StripKey,
};
use kx_platform_fake::{Pick, RecordCall};

use super::fakes::{CHOSEN, file};
use super::rig::{Rig, held, refuse, release};

/// A recorded video on its strip at 2×.
fn twice() -> Rig {
    let mut rig = Rig::new();
    rig.recorded(4_000);
    rig.act(SnipClick::Strip(StripKey::Speed));
    rig
}

/// The recording's file.
fn recording(rig: &Rig) -> SnipFile {
    rig.fake.recorder.runs()[0].file.clone()
}

/// Clicks Play.
fn play(rig: &mut Rig) {
    rig.act(SnipClick::Strip(StripKey::Play));
}

/// The strip shows with nothing over it, as before Play.
fn on_top(rig: &Rig) -> bool {
    let page = rig.page();
    matches!(page.stage, SnipStage::Strip(_)) && !page.dialog
}

#[test]
fn play_at_two_times_writes_the_copy_once_and_play_and_copy_reuse_it() {
    let mut rig = twice();
    play(&mut rig);
    let speeds = rig.fake.recorder.speeds();
    assert_eq!(speeds.len(), 1);
    assert_eq!(speeds[0].from, recording(&rig));
    let played = rig.fake.player.played();
    let copy = played[0].clone();
    assert_ne!(copy, recording(&rig), "the copy plays");
    assert!(on_top(&rig));
    play(&mut rig);
    rig.keep(Keep::Copy);
    assert_eq!(rig.fake.recorder.speeds().len(), 1, "written once");
    assert_eq!(rig.fake.player.played(), [copy.clone(), copy.clone()]);
    let parts = rig.fake.clip.writes()[0].clone();
    let path = copy.expose().to_path_buf();
    assert!(matches!(&parts[..], [ClipPart::Files(p)] if *p.expose() == [path]));
    rig.act(SnipClick::Strip(StripKey::Close));
    assert_eq!(rig.fake.folder.files(), [copy], "the copied copy stays");
}

#[test]
fn while_plays_copy_is_written_the_keys_wait_and_the_strip_stays_on_top() {
    let mut rig = twice();
    rig.actor.ports.spawn = held;
    play(&mut rig);
    for key in [StripKey::Speed, StripKey::Copy, StripKey::Close] {
        rig.act(SnipClick::Strip(key));
    }
    assert!(on_top(&rig) && rig.fake.player.played().is_empty());
    rig.drain();
    assert_eq!(rig.fake.player.played().len(), 1, "it plays once ready");
    assert!(on_top(&rig));
}

#[test]
fn edge_8_a_speed_job_that_fails_says_why_and_nothing_plays() {
    for (error, note) in [
        (RecordError::Gone, SnipNote::SpeedFailed),
        (RecordError::Full, SnipNote::DiskLow),
    ] {
        let mut rig = Rig::build(|fake, _| fake.recorder.fail(RecordCall::Speed, error));
        rig.recorded(4_000);
        rig.act(SnipClick::Strip(StripKey::Speed));
        play(&mut rig);
        assert_eq!(rig.page().note, Some(note));
        assert!(rig.fake.player.played().is_empty() && on_top(&rig));
        play(&mut rig);
        assert_eq!(
            rig.fake.player.played().len(),
            1,
            "the next Play tries again"
        );
    }
}

#[test]
fn edge_8_a_player_that_refuses_or_a_job_that_never_starts_says_so() {
    let refused: [fn(&mut Rig); 2] = [
        |rig| rig.fake.player.set_error(Some(PlayError::Refused(5))),
        |rig| rig.actor.ports.spawn = refuse,
    ];
    for refuse in refused {
        let mut rig = twice();
        refuse(&mut rig);
        play(&mut rig);
        assert_eq!(rig.page().note, Some(SnipNote::PlayFailed));
        assert!(on_top(&rig));
    }
}

#[test]
fn close_unkept_deletes_the_recording_and_plays_copy() {
    let mut rig = twice();
    play(&mut rig);
    rig.act(SnipClick::Strip(StripKey::Close));
    rig.act(SnipClick::Ask(AskKey::Drop));
    assert_eq!(rig.stage(), SnipStage::Idle);
    assert!(rig.fake.folder.files().is_empty(), "nothing unchosen stays");
}

#[test]
fn the_strip_shows_which_key_waits_only_while_its_job_writes_the_copy() {
    let waits = |rig: &Rig| match rig.page().stage {
        SnipStage::Strip(t) => t.waits,
        _ => None,
    };
    for key in [StripKey::Play, StripKey::Copy, StripKey::Save] {
        let mut rig = twice();
        let chosen = file(CHOSEN, "clip.mp4", FileKind::Mp4);
        rig.fake.dialogs.queue_save(Pick::File(chosen));
        rig.actor.ports.spawn = held;
        rig.act(SnipClick::Strip(key));
        assert_eq!(waits(&rig), None, "{key:?} before its job runs");
        release();
        let writing = rig.next().expect("the job says it writes");
        rig.actor.handle(writing);
        assert_eq!(waits(&rig), Some(key), "{key:?} writing");
        assert!(
            on_top(&rig),
            "{key:?}: no dialog shows while the copy is written"
        );
        rig.pump();
        rig.drain();
        assert_eq!(waits(&rig), None, "{key:?} written");
    }
    let mut rig = twice();
    rig.act(SnipClick::Strip(StripKey::Save));
    let pages = rig.views();
    let waited = pages
        .iter()
        .any(|p| matches!(&p.stage, SnipStage::Strip(t) if t.waits.is_some()));
    assert!(!waited, "a Save cancelled in its dialog never waits");
}

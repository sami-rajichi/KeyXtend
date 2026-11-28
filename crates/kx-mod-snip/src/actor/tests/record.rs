//! Recordings: Record checks the room first (edge 8), the run takes the snip's held values and the app's pass, a stop
//! by itself keeps the video with a note (edges 8 and 9), a dead worker's or a lost call's video goes (edge 13), no
//! video encoders dim both video ways (edge 14), and Cancel deletes, even a run that ended unheard. Cancel and a
//! switch-off never wait behind queued calls.

use std::sync::Arc;
use std::time::Duration;

use kx_module_api::ports::{
    BarKey, Dim, Gear, MB, RecKey, RecordEnd, RecordFacts, RecordWaits, Recorder, SnipClick,
    SnipNote, SnipStage, StripKey,
};
use toml::Value;

use super::super::run::{Ended, Run};
use super::rig::{Rig, held, refuse};

/// The dims of the strip's or the bar's keys told last, by key.
pub(super) fn strip_dim(rig: &Rig, key: StripKey) -> Option<Dim> {
    let SnipStage::Strip(taken) = rig.stage() else {
        panic!("a strip shows");
    };
    taken
        .keys
        .into_iter()
        .find(|k| k.key == key)
        .and_then(|k| k.dim)
}

#[test]
fn the_run_records_the_part_with_the_snips_held_values_and_the_apps_pass() {
    let switch = *RecordWaits::SWITCH_MS.end();
    let mut rig = Rig::build(|_, host| {
        host.put("mic_on", Value::Boolean(true));
        host.put("fps", Value::Integer(15));
        host.put("max_min", Value::Integer(7));
        host.put("switch_ms", Value::Integer(switch.into()));
    });
    rig.record();
    let ask = &rig.fake.recorder.starts()[0];
    let held = &rig.actor.held;
    assert_eq!((ask.fps, ask.max_min, ask.waits.switch_ms), (15, 7, switch));
    assert_eq!(
        (ask.min_free_mb, ask.job_mb, ask.waits),
        (held.min_free_mb, held.record_mb, held.record_waits())
    );
    assert!(
        ask.mic
            .as_ref()
            .is_some_and(|pass| rig.fake.seal.opens(pass))
    );
    assert!(!ask.sound);
}

#[test]
fn edge_8_record_dims_with_a_note_under_min_free_mb() {
    let mut rig = Rig::new();
    let floor = u64::from(rig.actor.settings.min_free_mb) * MB;
    rig.fake.folder.set_free(Ok(floor - 1));
    rig.take(BarKey::VideoRegion);
    assert_eq!(strip_dim(&rig, StripKey::Record), Some(Dim::DiskLow));
    assert_eq!(rig.page().note, Some(SnipNote::DiskLow));
    rig.act(SnipClick::Strip(StripKey::Record));
    assert!(
        matches!(rig.stage(), SnipStage::Strip(_)),
        "a dimmed Record does nothing"
    );
    assert!(rig.fake.recorder.starts().is_empty());
}

#[test]
fn edge_8_a_drive_filling_mid_recording_stops_and_keeps_the_video_with_a_note() {
    let mut rig = Rig::new();
    rig.record();
    assert!(rig.fake.recorder.end(RecordEnd::Full));
    rig.pump();
    let SnipStage::Strip(taken) = rig.stage() else {
        panic!("the video's strip shows");
    };
    assert!(taken.length_ms.is_some());
    assert_eq!(rig.page().note, Some(SnipNote::DiskLow));
    assert!(
        rig.fake
            .folder
            .stored(&rig.fake.recorder.runs()[0].file)
            .is_some()
    );
}

#[test]
fn edge_9_the_longest_video_stops_keeps_it_and_says_so() {
    let mut rig = Rig::new();
    rig.record();
    assert!(rig.fake.recorder.end(RecordEnd::Longest));
    rig.pump();
    assert!(matches!(rig.stage(), SnipStage::Strip(_)));
    assert_eq!(rig.page().note, Some(SnipNote::Longest));
    assert!(
        rig.fake
            .folder
            .stored(&rig.fake.recorder.runs()[0].file)
            .is_some()
    );
}

#[test]
fn edge_13_a_dead_worker_notes_deletes_the_broken_video_and_drops_the_run() {
    let mut rig = Rig::new();
    rig.record();
    assert!(rig.fake.recorder.die());
    rig.pump();
    let run = &rig.fake.recorder.runs()[0];
    assert!(run.dropped, "the run goes");
    assert!(rig.fake.folder.stored(&run.file).is_none());
    assert_eq!(rig.stage(), SnipStage::Idle);
    assert_eq!(rig.page().note, Some(SnipNote::RecordFailed));
}

#[test]
fn edge_14_without_video_encoders_both_video_ways_dim_with_a_tip() {
    let mut rig = Rig::build(|fake, _| {
        fake.recorder.set_facts(RecordFacts {
            video: false,
            mic: Gear::Ready,
            sound: Gear::Ready,
        });
    });
    rig.key();
    let SnipStage::Pick(picking) = rig.stage() else {
        panic!("the bar shows");
    };
    let dims: Vec<_> = picking
        .keys
        .iter()
        .filter_map(|k| k.dim.map(|d| (k.key, d)))
        .collect();
    assert_eq!(
        dims,
        [
            (BarKey::VideoRegion, Dim::NoVideo),
            (BarKey::VideoFull, Dim::NoVideo)
        ]
    );
    rig.act(SnipClick::Bar(BarKey::VideoFull));
    let SnipStage::Pick(picking) = rig.stage() else {
        panic!("still picking");
    };
    assert!(
        picking.keys.iter().any(|k| k.key == BarKey::Region && k.on),
        "a dimmed way is not picked"
    );
}

#[test]
fn a_missing_microphone_dims_mic_and_records_without_it() {
    let mut rig = Rig::build(|fake, host| {
        host.put("mic_on", Value::Boolean(true));
        fake.recorder.set_facts(RecordFacts {
            video: true,
            mic: Gear::Missing,
            sound: Gear::Ready,
        });
    });
    rig.take(BarKey::VideoRegion);
    assert_eq!(strip_dim(&rig, StripKey::Mic), Some(Dim::NoMic));
    rig.act(SnipClick::Strip(StripKey::Record));
    rig.advance(Duration::from_secs(u64::from(rig.actor.held.countdown_s)));
    assert!(rig.fake.recorder.starts()[0].mic.is_none());
}

#[test]
fn stop_shows_the_videos_length_and_cancel_deletes_it() {
    let mut rig = Rig::new();
    rig.recorded(1_500);
    let SnipStage::Strip(taken) = rig.stage() else {
        panic!("the strip shows");
    };
    assert_eq!(taken.length_ms, Some(1_500));
    assert!(rig.fake.recorder.runs()[0].dropped, "a run that ended goes");
    let mut rig = Rig::new();
    rig.record();
    rig.act(SnipClick::Record(RecKey::Cancel));
    let run = &rig.fake.recorder.runs()[0];
    assert_eq!(run.cancels, 1);
    assert!(rig.fake.folder.files().is_empty());
    assert_eq!(rig.stage(), SnipStage::Idle);
}

#[test]
fn the_bar_holds_its_clock_until_the_runs_first_frame() {
    let mut rig = Rig::new();
    rig.take(BarKey::VideoRegion);
    rig.act(SnipClick::Strip(StripKey::Record));
    for _ in 0..rig.actor.held.countdown_s {
        rig.advance(Duration::from_secs(1));
    }
    let live = |rig: &Rig| match rig.stage() {
        SnipStage::Record(r) => (r.live, r.elapsed_ms),
        other => panic!("it records: {other:?}"),
    };
    assert_eq!(live(&rig), (false, 0), "no frame yet");
    assert!(rig.fake.recorder.time(), "the first frame");
    rig.pump();
    assert_eq!(live(&rig), (true, 0), "the clock starts at the first frame");
}

#[test]
fn a_sound_switched_mid_recording_reaches_the_run() {
    let mut rig = Rig::new();
    rig.record();
    rig.act(SnipClick::Record(RecKey::Sound));
    rig.act(SnipClick::Record(RecKey::Mic));
    let heard = &rig.fake.recorder.runs()[0].heard;
    let last = heard.last().unwrap();
    assert!(last.mic && last.sound, "{heard:?}");
}

#[test]
fn a_switch_off_goes_ahead_of_a_switch_on_still_queued() {
    let mut rig = Rig::new();
    rig.record();
    rig.actor.ports.spawn = held;
    rig.act(SnipClick::Record(RecKey::Mic));
    rig.act(SnipClick::Record(RecKey::Sound));
    rig.act(SnipClick::Record(RecKey::Sound));
    rig.drain();
    let heard = &rig.fake.recorder.runs()[0].heard;
    assert!(
        heard.iter().all(|h| !h.sound),
        "the sound never went on: {heard:?}"
    );
    assert!(heard.last().is_some_and(|h| h.mic), "{heard:?}");
}

#[test]
fn cancel_drops_the_calls_queued_and_acts_once_the_run_is_back() {
    let mut rig = Rig::new();
    rig.record();
    rig.actor.ports.spawn = held;
    rig.act(SnipClick::Record(RecKey::Mic));
    rig.act(SnipClick::Record(RecKey::Sound));
    rig.act(SnipClick::Record(RecKey::Cancel));
    assert_eq!(
        rig.fake.recorder.runs()[0].cancels,
        0,
        "the mic's call holds the run"
    );
    rig.drain();
    let run = &rig.fake.recorder.runs()[0];
    assert_eq!(run.cancels, 1);
    assert!(
        run.heard.iter().all(|h| !h.sound),
        "the sound's call was dropped"
    );
    assert!(rig.fake.folder.files().is_empty());
}

// The platform may end a run just before its note stream marks it ended; the fake's run below ends with no ear.
#[test]
fn cancel_deletes_a_run_that_ended_before_its_last_note_was_heard_at_once() {
    let mut rig = Rig::new();
    rig.record();
    let ask = rig.fake.recorder.starts()[0].clone();
    rig.act(SnipClick::Record(RecKey::Cancel));
    let inner = rig.fake.recorder.start(ask, Box::new(|_| {})).unwrap();
    let file = inner.file().clone();
    let folder = Arc::new(rig.fake.folder.clone());
    let run = Run::new(
        inner,
        Ended::default(),
        folder,
        Arc::clone(&rig.fake.leftovers),
    );
    assert!(
        rig.fake.recorder.end(RecordEnd::Stopped),
        "it ends and keeps its video"
    );
    run.cancel();
    assert!(
        rig.fake.folder.stored(&file).is_none(),
        "gone now, not at the next start"
    );
    assert!(rig.fake.leftovers.lock().unwrap().missed.is_empty());
}

#[test]
fn a_call_whose_job_does_not_start_cancels_the_run_and_ends_with_a_note() {
    let mut rig = Rig::new();
    rig.record();
    rig.actor.ports.spawn = refuse;
    rig.act(SnipClick::Record(RecKey::Sound));
    assert_eq!(rig.fake.recorder.runs()[0].cancels, 1);
    assert!(rig.fake.folder.files().is_empty());
    assert_eq!(rig.stage(), SnipStage::Idle);
    assert_eq!(rig.page().note, Some(SnipNote::RecordFailed));
}

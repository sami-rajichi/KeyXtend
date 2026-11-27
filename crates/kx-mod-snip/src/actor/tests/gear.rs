//! What the system's gear and drive say about a recording: edge 11's tips for a microphone the privacy switch refuses
//! and no sound device, a switch-on that fails heard as lost with a note unless the run already ended, a microphone
//! failing at the start bringing the strip back with Mic off, a refused start reaching the flow as a fault, a drive
//! that cannot be read taken as not low, and a Speed that fills the drive.

use std::time::Duration;

use kx_module_api::ports::{
    BarKey, Dim, Gear, Keep, RecKey, RecordEnd, RecordError, RecordFacts, SaveError, SnipClick,
    SnipLit, SnipNote, SnipStage, StripKey,
};
use kx_platform_fake::RecordCall;
use toml::Value;

use super::record::strip_dim;
use super::rig::{Rig, held, release};
use crate::settings::keys;

/// The record bar's `key` told last: lit, and why it dims.
fn rec_key(rig: &Rig, key: RecKey) -> (bool, Option<Dim>) {
    let SnipStage::Record(recording) = rig.stage() else {
        panic!("a recording shows: {:?}", rig.stage());
    };
    let cap = recording.keys.into_iter().find(|k| k.key == key).unwrap();
    (cap.on, cap.dim)
}

/// A rig whose microphone the privacy switch refuses and which has no sound device.
fn refused_gear() -> Rig {
    let facts = RecordFacts {
        video: true,
        mic: Gear::Refused,
        sound: Gear::Missing,
    };
    Rig::build(|fake, _| fake.recorder.set_facts(facts))
}

#[test]
fn edge_11_a_refused_microphone_and_no_sound_device_dim_with_their_tips() {
    let mut rig = refused_gear();
    rig.take(BarKey::VideoRegion);
    assert_eq!(strip_dim(&rig, StripKey::Mic), Some(Dim::MicRefused));
    assert_eq!(strip_dim(&rig, StripKey::Sound), Some(Dim::NoSound));
    let mut rig = refused_gear();
    rig.record();
    assert_eq!(rec_key(&rig, RecKey::Mic).1, Some(Dim::MicRefused));
    assert_eq!(rec_key(&rig, RecKey::Sound).1, Some(Dim::NoSound));
}

#[test]
fn a_switch_on_that_fails_is_heard_as_lost_and_its_key_goes_off() {
    let mut rig = Rig::build(|_, host| host.put(keys::SOUND_ON, Value::Boolean(false)));
    rig.record();
    rig.fake
        .recorder
        .fail(RecordCall::Sound, RecordError::NoSound);
    rig.act(SnipClick::Record(RecKey::Sound));
    assert!(!rec_key(&rig, RecKey::Sound).0, "lost, so off");
    assert_eq!(rig.page().note, Some(SnipNote::SoundFailed));
    assert_eq!(rig.fake.recorder.runs()[0].cancels, 0, "it records on");
}

#[test]
fn a_microphone_failing_at_the_start_brings_the_strip_back_with_mic_off() {
    let mut rig = Rig::build(|fake, host| {
        host.put(keys::MIC_ON, Value::Boolean(true));
        fake.recorder
            .fail(RecordCall::Start, RecordError::MicRefused);
    });
    let countdown = Duration::from_secs(u64::from(rig.actor.held.countdown_s));
    rig.take(BarKey::VideoRegion);
    rig.act(SnipClick::Strip(StripKey::Record));
    rig.advance(countdown);
    assert_eq!(rig.page().note, Some(SnipNote::MicFailed));
    let SnipStage::Strip(taken) = rig.stage() else {
        panic!("the strip shows again: {:?}", rig.stage());
    };
    let mic = taken.keys.iter().find(|k| k.key == StripKey::Mic).unwrap();
    assert!(!mic.on, "Mic is off");
    assert_eq!(rig.actor.flow.lit(), SnipLit::On, "the snip goes on");
    rig.act(SnipClick::Strip(StripKey::Record));
    rig.advance(countdown);
    let starts = rig.fake.recorder.starts();
    assert!(starts[1].mic.is_none(), "it records without the microphone");
}

#[test]
fn a_switch_that_fails_after_the_run_ended_cancels_nothing() {
    let mut rig = sound_off_held(RecordError::Gone);
    assert!(
        rig.fake.recorder.end(RecordEnd::Stopped),
        "the run ends first"
    );
    rig.pump();
    release();
    rig.pump();
    assert!(
        matches!(rig.stage(), SnipStage::Strip(_)),
        "the video is kept"
    );
    assert_eq!(rig.fake.recorder.runs()[0].cancels, 0);
}

#[test]
fn a_switch_failure_heard_after_the_run_ended_cancels_nothing() {
    let mut rig = sound_off_held(RecordError::NoSound);
    release();
    let late = rig.next().expect("the switch's answer");
    assert!(rig.fake.recorder.end(RecordEnd::Stopped));
    rig.pump();
    rig.send(late);
    assert!(
        matches!(rig.stage(), SnipStage::Strip(_)),
        "the video is kept"
    );
    assert_eq!(rig.fake.recorder.runs()[0].cancels, 0);
}

/// A recording whose Sound key was clicked on, its switch held in a job that fails with `error`.
fn sound_off_held(error: RecordError) -> Rig {
    let mut rig = Rig::build(|_, host| host.put(keys::SOUND_ON, Value::Boolean(false)));
    rig.record();
    rig.actor.ports.spawn = held;
    rig.fake.recorder.fail(RecordCall::Sound, error);
    rig.act(SnipClick::Record(RecKey::Sound));
    rig
}

#[test]
fn a_refused_start_reaches_the_flow_as_a_fault() {
    let mut rig = Rig::build(|fake, _| fake.recorder.fail(RecordCall::Start, RecordError::Full));
    rig.take(BarKey::VideoRegion);
    rig.act(SnipClick::Strip(StripKey::Record));
    rig.advance(Duration::from_secs(u64::from(rig.actor.held.countdown_s)));
    assert_eq!(rig.stage(), SnipStage::Idle);
    assert_eq!(rig.page().note, Some(SnipNote::DiskLow));
}

#[test]
fn a_drive_that_cannot_be_read_is_not_low() {
    let mut rig = Rig::build(|fake, _| fake.folder.set_free(Err(SaveError::Io)));
    rig.take(BarKey::VideoRegion);
    assert_eq!(strip_dim(&rig, StripKey::Record), None);
    assert_eq!(rig.page().note, None);
}

#[test]
fn a_speed_that_fills_the_drive_says_the_disk_is_low() {
    let mut rig = Rig::new();
    rig.recorded(4_000);
    rig.act(SnipClick::Strip(StripKey::Speed));
    rig.fake.recorder.fail(RecordCall::Speed, RecordError::Full);
    rig.keep(Keep::Copy);
    assert_eq!(rig.page().note, Some(SnipNote::DiskLow));
}

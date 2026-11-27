//! A capture window the ui could not keep out of captures (`SnipClick::Bare`): both video ways and Record dim after a
//! hidden run's reason, a countdown ends, a recording stops and keeps its video, each with a note saying why, and a new
//! actor learns it again.

use kx_module_api::ports::{
    BarKey, Dim, Gear, Keycap, RecordFacts, SnipClick, SnipNote, SnipStage, StripKey,
};

use super::fakes::Fakes;
use super::rig::Rig;

/// Both video ways, dimmed for `dim`.
fn video(dim: Dim) -> [(BarKey, Dim); 2] {
    [(BarKey::VideoRegion, dim), (BarKey::VideoFull, dim)]
}

/// Each dimmed key of `keys` with its reason.
fn dims<K: Copy>(keys: &[Keycap<K>]) -> Vec<(K, Dim)> {
    keys.iter()
        .filter_map(|k| k.dim.map(|d| (k.key, d)))
        .collect()
}

/// The capture bar's dimmed keys, told last.
fn bar(rig: &Rig) -> Vec<(BarKey, Dim)> {
    let SnipStage::Pick(picking) = rig.stage() else {
        panic!("the bar shows");
    };
    dims(&picking.keys)
}

#[test]
fn bare_dims_both_video_ways_and_record_and_a_dimmed_record_does_nothing() {
    let mut rig = Rig::new();
    rig.take(BarKey::VideoRegion);
    rig.act(SnipClick::Bare);
    let SnipStage::Strip(taken) = rig.stage() else {
        panic!("the strip shows");
    };
    assert_eq!(dims(&taken.keys), [(StripKey::Record, Dim::Bare)]);
    rig.act(SnipClick::Strip(StripKey::Record));
    assert!(matches!(rig.stage(), SnipStage::Strip(_)), "no countdown");
    rig.act(SnipClick::Strip(StripKey::Close));
    rig.key();
    assert_eq!(bar(&rig), video(Dim::Bare), "kept for the next snip");
}

#[test]
fn hidden_comes_before_bare_and_bare_before_no_video() {
    let mut rig = Rig::with(Fakes::hidden(), |_, _| {});
    rig.key();
    rig.act(SnipClick::Bare);
    assert_eq!(bar(&rig), video(Dim::Hidden));
    let mut rig = Rig::build(|fake, _| {
        fake.recorder.set_facts(RecordFacts {
            video: false,
            mic: Gear::Ready,
            sound: Gear::Ready,
        });
    });
    rig.key();
    rig.act(SnipClick::Bare);
    assert_eq!(bar(&rig), video(Dim::Bare));
}

#[test]
fn bare_during_the_countdown_ends_the_snip_with_no_video() {
    let mut rig = Rig::new();
    rig.take(BarKey::VideoRegion);
    rig.act(SnipClick::Strip(StripKey::Record));
    assert!(matches!(rig.stage(), SnipStage::Count(_)));
    rig.act(SnipClick::Bare);
    assert_eq!(rig.stage(), SnipStage::Idle);
    assert_eq!(rig.page().note, Some(SnipNote::Bare), "it says why");
    assert!(rig.fake.folder.files().is_empty(), "nothing recorded");
}

#[test]
fn bare_while_recording_stops_it_keeps_the_video_and_says_why() {
    let mut rig = Rig::new();
    rig.record();
    rig.act(SnipClick::Bare);
    let SnipStage::Strip(taken) = rig.stage() else {
        panic!("the video's strip shows");
    };
    assert!(taken.length_ms.is_some(), "the video so far is kept");
    assert_eq!(rig.page().note, Some(SnipNote::Bare), "it says why");
    assert_eq!(rig.fake.folder.files().len(), 1);
}

#[test]
fn a_new_actor_knows_nothing_until_the_next_bare() {
    let mut rig = Rig::new();
    rig.key();
    assert_eq!(bar(&rig), [], "nothing heard yet");
    rig.act(SnipClick::Bare);
    assert_eq!(bar(&rig), video(Dim::Bare));
}

//! A recorded video's strip (ADR-0060): Play, Speed, Copy, Save and Close with no Edit, the length shown at each speed,
//! Play opening the recording while the strip stays on top (edge 8), and Save at a speed writing the re-timed copy.

use kx_module_api::ports::{
    FileKind, Keep, PlayError, SnipClick, SnipNote, SnipStage, Speed, StripKey, Taken,
};
use kx_platform_fake::{Pick, Stored};

use super::fakes::{CHOSEN, file};
use super::rig::{Rig, refuse};

/// The strip told last.
fn taken(rig: &Rig) -> Taken {
    let SnipStage::Strip(taken) = rig.stage() else {
        panic!("the strip shows");
    };
    taken
}

/// Its keys, in order.
fn keys(rig: &Rig) -> Vec<StripKey> {
    taken(rig).keys.iter().map(|cap| cap.key).collect()
}

#[test]
fn after_stop_the_strip_shows_play_speed_copy_save_and_close_with_no_edit() {
    use StripKey::{Close, Copy, Play, Save, Speed as Faster};
    let mut rig = Rig::new();
    rig.recorded(1_500);
    assert_eq!(keys(&rig), [Play, Faster, Copy, Save, Close]);
    let shown = taken(&rig);
    assert_eq!(
        (shown.length_ms, shown.speed),
        (Some(1_500), Some(Speed::PLAIN))
    );
    assert!(taken(&rig).keys.iter().all(|cap| cap.dim.is_none()));
}

#[test]
fn speed_shows_each_speed_with_the_length_at_it() {
    let mut rig = Rig::new();
    rig.recorded(4_000);
    let speed = |times| Speed::new(times).unwrap();
    for (times, length_ms) in [(2, 2_000), (4, 1_000), (1, 4_000)] {
        rig.act(SnipClick::Strip(StripKey::Speed));
        let shown = taken(&rig);
        assert_eq!(
            (shown.speed, shown.length_ms),
            (Some(speed(times)), Some(length_ms))
        );
    }
}

#[test]
fn play_opens_the_recording_while_the_strip_stays_on_top() {
    let mut rig = Rig::new();
    rig.recorded(1_000);
    rig.act(SnipClick::Strip(StripKey::Play));
    let recording = rig.fake.recorder.runs()[0].file.clone();
    assert_eq!(rig.fake.player.played(), std::slice::from_ref(&recording));
    assert!(
        matches!(rig.stage(), SnipStage::Strip(_)),
        "the strip stays"
    );
    assert!(!rig.page().dialog, "nothing takes the top from it");
}

#[test]
fn edge_8_a_player_that_refuses_or_never_starts_says_so_and_the_strip_stays() {
    let refused: [fn(&mut Rig); 2] = [
        |rig| rig.fake.player.set_error(Some(PlayError::Refused(5))),
        |rig| rig.actor.ports.spawn = refuse,
    ];
    for refuse in refused {
        let mut rig = Rig::new();
        rig.recorded(1_000);
        refuse(&mut rig);
        rig.act(SnipClick::Strip(StripKey::Play));
        let page = rig.page();
        assert_eq!(page.note, Some(SnipNote::PlayFailed));
        assert!(matches!(page.stage, SnipStage::Strip(_)) && !page.dialog);
    }
}

#[test]
fn save_at_four_times_writes_the_retimed_copy_and_close_leaves_only_it() {
    let chosen = file(CHOSEN, "fast.mp4", FileKind::Mp4);
    let mut rig = Rig::build(|fake, _| fake.dialogs.queue_save(Pick::File(chosen.clone())));
    rig.recorded(4_000);
    rig.act(SnipClick::Strip(StripKey::Speed));
    rig.act(SnipClick::Strip(StripKey::Speed));
    rig.keep(Keep::Save);
    let recording = rig.fake.recorder.runs()[0].file.clone();
    assert_eq!(rig.fake.recorder.speeds()[0].from, recording);
    assert!(matches!(
        rig.fake.folder.stored(&chosen),
        Some(Stored::Video { length_ms: 1_000 })
    ));
    assert_eq!(
        taken(&rig).done,
        Some(Keep::Save),
        "the strip stays with its tick"
    );
    rig.act(SnipClick::Strip(StripKey::Close));
    assert_eq!(rig.stage(), SnipStage::Idle, "kept, so Close asks nothing");
    assert_eq!(
        rig.fake.folder.files(),
        [chosen],
        "the recording and its copy go"
    );
}

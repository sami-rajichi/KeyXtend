//! The key and mouse sounds through the hub: each event's takes in turn at its gain under the `[ui]` switches and
//! volume, after a key's frame mark, and never on a repeat, a top-bar button or a list key.

use crate::{FakeFace, Rig};
use kx_module_api::Bus;
use kx_module_api::ports::{AssistNote, FaceAction, FaceReply, KeyId, Point};
use kx_platform::ui::{ClipId, SpeakerConfig};
use kx_test_support::host::{FakeCore, FakeHost};
use kx_theme::sounds::Event;
use kx_ui::{MANIFEST, UiSettings, sounds};
use std::sync::Arc;

/// The volume that plays each sound at its own level.
const FULL: f32 = 100.0;
/// How far two gains may differ and still be one.
const SAME_GAIN: f32 = 1e-6;
/// Where a note happened; no sound depends on it.
const AT: Point = Point { x: 10, y: 20 };

/// The `[ui]` defaults.
fn defaults() -> UiSettings {
    UiSettings::from_defaults().unwrap()
}

/// A click on key `id`.
fn tap(id: &str) -> FaceAction {
    FaceAction::Tap {
        id: KeyId::new(id),
        back: false,
    }
}

/// `event`'s first `n` plays at `volume`: its takes in turn, each at the event's gain times the volume.
fn takes(event: Event, n: usize, volume: u8) -> Vec<(ClipId, f32)> {
    let Some(cue) = sounds::cue(event) else {
        return Vec::new();
    };
    let gain = cue.gain * f32::from(volume) / FULL;
    cue.clips
        .iter()
        .cycle()
        .take(n)
        .map(|&id| (id, gain))
        .collect()
}

/// The first play of each of `events` at the default volume.
fn first_of(events: impl IntoIterator<Item = Event>) -> Vec<(ClipId, f32)> {
    let volume = defaults().sound_volume;
    events
        .into_iter()
        .flat_map(|e| takes(e, 1, volume))
        .collect()
}

/// Fails unless `got` plays the clips of `want` in order, each at its gain.
fn assert_plays(got: &[(ClipId, f32)], want: &[(ClipId, f32)]) {
    let ids = |plays: &[(ClipId, f32)]| plays.iter().map(|p| p.0).collect::<Vec<_>>();
    assert_eq!(ids(got), ids(want), "the clips");
    for ((_, g), (_, w)) in got.iter().zip(want) {
        assert!((g - w).abs() <= SAME_GAIN, "gain {g}, not {w}");
    }
}

/// A rig whose face is attached with the `[ui]` defaults that `change` changed.
fn rig_with(change: impl FnOnce(&mut UiSettings)) -> (Rig, Arc<FakeFace>, FakeHost) {
    let rig = Rig::new();
    let face = Arc::new(FakeFace::default());
    let mut host = FakeHost::new(&MANIFEST);
    let mut settings = defaults();
    change(&mut settings);
    rig.hub.attach(settings, face.clone(), host.handle());
    (rig, face, host)
}

/// Sets the `[ui]` switch of `event` to `on`.
fn switch(s: &mut UiSettings, event: Event, on: bool) {
    match event {
        Event::Key => s.sound_key = on,
        Event::Rclick => s.sound_rclick = on,
        Event::Pick => s.sound_pick = on,
        Event::Drop => s.sound_drop = on,
    }
}

/// Asks the hub to sound every event once.
fn sound_all(rig: &Rig) {
    for event in Event::ALL {
        rig.hub.sound(event);
    }
}

#[test]
fn the_speaker_is_prepared_once_at_start_with_every_clip_the_voices_and_the_idle_time() {
    let (rig, _face, _host) = Rig::with_face();
    rig.hub.sent(tap("a"), || {});
    sound_all(&rig);
    let ids: Vec<ClipId> = sounds::clips().iter().map(|c| c.id).collect();
    let config = SpeakerConfig {
        voices: sounds::voices(),
        idle_ms: defaults().sound_idle_ms,
    };
    assert_eq!(rig.speaker.prepared(), [(ids, config)]);
}

#[test]
fn a_new_idle_time_prepares_the_speaker_again_and_the_same_one_does_not() {
    let (rig, face, mut host) = rig_with(|_| {});
    assert_eq!(rig.speaker.prepared().len(), 1, "the defaults keep it");
    let mut settings = defaults();
    settings.sound_idle_ms += 1;
    rig.hub.attach(settings.clone(), face, host.handle());
    let prepared = rig.speaker.prepared();
    assert_eq!(prepared.len(), 2);
    assert_eq!(prepared[1].0, prepared[0].0, "every clip again");
    assert_eq!(prepared[1].1.idle_ms, settings.sound_idle_ms);
}

#[test]
fn a_tap_plays_the_key_sound_once_after_its_frame_mark() {
    let (rig, face, _host) = Rig::with_face();
    let mut at_mark = None;
    rig.hub.sent(tap("a"), || {
        at_mark = Some((face.acts().len(), rig.speaker.plays().len()));
    });
    assert_eq!(
        at_mark,
        Some((1, 0)),
        "the key went out, nothing sounded yet"
    );
    assert_plays(&rig.speaker.take_plays(), &first_of([Event::Key]));
}

#[test]
fn taps_on_any_key_play_the_key_takes_in_turn_then_start_again() {
    let (rig, _face, _host) = Rig::with_face();
    let n = sounds::cue(Event::Key).map_or(0, |c| c.clips.len()) + 1;
    for id in ["a", "rclick", "grab"].into_iter().cycle().take(n) {
        rig.hub.sent(tap(id), || {});
    }
    let volume = defaults().sound_volume;
    assert_plays(&rig.speaker.take_plays(), &takes(Event::Key, n, volume));
}

#[test]
fn a_held_key_sounds_once_at_its_press_and_never_on_its_ticks() {
    let (rig, face, _host) = Rig::with_face();
    face.answer(Ok(FaceReply {
        holding: true,
        ..FaceReply::default()
    }));
    let id = KeyId::new("bksp");
    rig.hub.sent(FaceAction::Press { id: id.clone() }, || {});
    for _ in 0..3 {
        rig.hub.act(FaceAction::Tick);
    }
    rig.hub.sent(FaceAction::Tick, || {});
    rig.hub.act(FaceAction::LetGo { id });
    rig.hub.act(FaceAction::Leave);
    assert_plays(&rig.speaker.take_plays(), &first_of([Event::Key]));
}

#[test]
fn the_top_bar_and_the_language_list_play_nothing() {
    let (rig, _face, _host) = Rig::with_face();
    let bar = [
        FaceAction::Smaller,
        FaceAction::Bigger,
        FaceAction::Preset { index: 0 },
        FaceAction::LangPick { index: 0 },
        FaceAction::Relabel,
    ];
    for action in bar {
        rig.hub.act(action.clone());
        rig.hub.sent(action, || {});
    }
    rig.hub.panel_key("right");
    assert!(rig.hub.flip_mode() && rig.hub.step_theme());
    assert!(rig.speaker.plays().is_empty());
}

#[test]
fn a_fired_hold_a_pick_up_and_a_drop_each_play_their_sound_once() {
    let (rig, _face, _host) = Rig::with_face();
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let heard = rig.hub.listen(&bus).unwrap();
    for (note, event) in [
        (AssistNote::Fired { at: AT }, Event::Rclick),
        (AssistNote::Picked { at: AT }, Event::Pick),
        (AssistNote::Dropped { at: AT }, Event::Drop),
    ] {
        bus.publish(&note).unwrap();
        assert_plays(&rig.speaker.take_plays(), &first_of([event]));
    }
    drop(heard);
    bus.publish(&AssistNote::Fired { at: AT }).unwrap();
    assert!(rig.speaker.plays().is_empty(), "the hub stopped hearing");
}

#[test]
fn a_hold_starting_ending_or_a_cancelled_carry_plays_nothing() {
    let (rig, _face, _host) = Rig::with_face();
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let _heard = rig.hub.listen(&bus).unwrap();
    let start = AssistNote::HoldStart {
        at: AT,
        ring_after_ms: 150,
        hold_ms: 1_500,
    };
    for note in [start, AssistNote::HoldEnd, AssistNote::Cancelled] {
        bus.publish(&note).unwrap();
    }
    assert!(rig.speaker.plays().is_empty());
}

#[test]
fn a_sound_switched_off_never_plays_and_the_others_still_do() {
    for off in Event::ALL {
        let (rig, _face, _host) = rig_with(|s| switch(s, off, false));
        rig.hub.sent(tap("a"), || {});
        sound_all(&rig);
        let keys = if off == Event::Key { 0 } else { 2 };
        let mut want = takes(Event::Key, keys, defaults().sound_volume);
        let mouse = Event::ALL.into_iter().filter(|&e| e != Event::Key);
        want.extend(first_of(mouse.filter(|&e| e != off)));
        assert_plays(&rig.speaker.take_plays(), &want);
    }
}

#[test]
fn volume_0_never_plays_and_50_plays_at_half_each_gain() {
    let (rig, _face, _host) = rig_with(|s| s.sound_volume = 0);
    rig.hub.sent(tap("a"), || {});
    sound_all(&rig);
    assert!(rig.speaker.plays().is_empty());
    let (rig, _face, _host) = rig_with(|s| s.sound_volume = 50);
    sound_all(&rig);
    let half = Event::ALL.into_iter().filter_map(sounds::cue);
    let half: Vec<_> = half.map(|c| (c.clips[0], c.gain / 2.0)).collect();
    assert_plays(&rig.speaker.take_plays(), &half);
}

#[test]
fn a_settings_change_applies_at_the_next_play() {
    let (rig, face, mut host) = Rig::with_face();
    rig.hub.sound(Event::Drop);
    let mut settings = defaults();
    settings.sound_volume *= 2;
    rig.hub
        .attach(settings.clone(), face.clone(), host.handle());
    rig.hub.sound(Event::Drop);
    let louder = settings.sound_volume;
    settings.sound_drop = false;
    rig.hub.attach(settings, face, host.handle());
    rig.hub.sound(Event::Drop);
    let mut want = first_of([Event::Drop]);
    want.extend(takes(Event::Drop, 2, louder).into_iter().skip(1));
    assert_plays(&rig.speaker.take_plays(), &want);
}

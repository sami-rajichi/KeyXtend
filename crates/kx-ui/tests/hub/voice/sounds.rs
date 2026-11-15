//! The dictation's start and stop sounds (edge 40): the bar asks for them as it starts and stops listening, each plays
//! its take at the `[ui]` volume, and none plays with `[voice] sounds` off, before any view, or at volume 0.

use super::{heard, listening_bar, page};
use crate::sound::{assert_plays, rig_with, takes};
use kx_theme::sounds::Event;
use kx_ui::UiSettings;

/// The volume a test sets.
const VOLUME: u8 = 40;

#[test]
fn the_start_and_stop_sounds_play_their_takes_at_the_ui_volume() {
    let (rig, _face, _host) = rig_with(|s| s.sound_volume = VOLUME);
    let heard = heard(&rig);
    heard.shown(&rig, page(1, listening_bar(), None));
    rig.hub.voice_chime(true);
    rig.hub.voice_chime(false);
    let want = [
        takes(Event::VoiceStart, 1, VOLUME),
        takes(Event::VoiceStop, 1, VOLUME),
    ]
    .concat();
    assert_plays(&rig.speaker.take_plays(), &want);
}

#[test]
fn sounds_off_plays_none() {
    let (rig, _face, _host) = rig_with(|_| {});
    let heard = heard(&rig);
    let mut quiet = page(1, listening_bar(), None);
    quiet.sounds = false;
    heard.shown(&rig, quiet);
    rig.hub.voice_chime(true);
    rig.hub.voice_chime(false);
    assert!(rig.speaker.take_plays().is_empty());
}

#[test]
fn no_sound_plays_before_any_view() {
    let (rig, _face, _host) = rig_with(|_| {});
    rig.hub.voice_chime(true);
    assert!(rig.speaker.take_plays().is_empty());
    let defaults = UiSettings::from_defaults().unwrap();
    assert!(defaults.sound_volume > 0, "the default volume plays");
}

#[test]
fn no_sound_plays_at_volume_zero() {
    let (rig, _face, _host) = rig_with(|s| s.sound_volume = 0);
    let heard = heard(&rig);
    heard.shown(&rig, page(1, listening_bar(), None));
    rig.hub.voice_chime(true);
    rig.hub.voice_chime(false);
    assert!(rig.speaker.take_plays().is_empty());
}

#[test]
fn the_key_sound_switch_leaves_the_dictation_sounds_alone() {
    let (rig, _face, _host) = rig_with(|s| s.sound_key = false);
    let heard = heard(&rig);
    heard.shown(&rig, page(1, listening_bar(), None));
    rig.hub.voice_chime(true);
    let volume = UiSettings::from_defaults().unwrap().sound_volume;
    assert_plays(
        &rig.speaker.take_plays(),
        &takes(Event::VoiceStart, 1, volume),
    );
}

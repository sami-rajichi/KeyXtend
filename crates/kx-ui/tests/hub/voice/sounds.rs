//! The dictation's start and stop sounds (edge 40): the bar asks for them as it starts and stops listening, each plays
//! its take at the `[ui]` volume, and none plays with `[voice] sounds` off, before any view, or at volume 0. A press on a
//! key page's line plays the key sound once, as on Quick-fill's lines; a late or wrong click plays none.

use super::cloud::typing::typed_panel;
use super::{heard, listening_bar, page};
use crate::sound::{assert_plays, rig_with, takes};
use kx_module_api::Redacted;
use kx_module_api::ports::{BarButton, CloudLine, VoiceAct, VoiceBar, VoiceClick};
use kx_theme::sounds::Event;
use kx_ui::UiSettings;
use kx_ui::i18n::keys::{bar_button_id, cloud_line_id};
use kx_ui::voice_acts::{BAR, CARET, PICK};

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

#[test]
fn a_press_on_a_line_plays_the_key_sound_once_and_its_caret_none() {
    let (rig, _face, _host) = rig_with(|_| {});
    let heard = heard(&rig);
    let address = cloud_line_id(CloudLine::Address);
    let key_page = typed_panel(0, CloudLine::Address, 0, &[]);
    heard.shown(&rig, page(1, VoiceBar::Hidden, Some(key_page)));
    rig.speaker.take_plays();
    rig.hub.voice_act(PICK, address, 24, 1);
    let volume = UiSettings::from_defaults().unwrap().sound_volume;
    assert_plays(&rig.speaker.take_plays(), &takes(Event::Key, 1, volume));
    rig.hub.voice_act(CARET, address, 4, 1);
    assert!(rig.speaker.take_plays().is_empty(), "the place is silent");
    let caret = |at| VoiceAct {
        round: 1,
        click: VoiceClick::Caret {
            line: CloudLine::Address,
            at: Redacted::new(at),
        },
    };
    assert_eq!(
        heard.sent.acts(),
        [caret(24), caret(4)],
        "both place the caret"
    );
}

/// A click QML sent before any view, or with an unknown kind, id or round, sounds nothing, as it does nothing.
#[test]
fn a_late_or_wrong_click_plays_no_key_sound() {
    let (rig, _face, _host) = rig_with(|_| {});
    let close = bar_button_id(BarButton::Close);
    rig.hub.voice_act(BAR, close, 0, 1);
    assert!(rig.speaker.take_plays().is_empty(), "before any view");
    let heard = heard(&rig);
    heard.shown(&rig, page(1, listening_bar(), None));
    rig.speaker.take_plays();
    for (kind, id, round) in [
        ("no-such-kind", close, 1),
        (BAR, "no-such-id", 1),
        (BAR, close, -1),
    ] {
        rig.hub.voice_act(kind, id, 0, round);
        assert!(rig.speaker.take_plays().is_empty(), "{kind} {id} {round}");
    }
    assert!(heard.sent.acts().is_empty(), "none was sent");
}

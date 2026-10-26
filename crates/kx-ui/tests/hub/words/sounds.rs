//! Taps on the chip lane, its menu and the email list play the key sound once each, as a key does, under the same
//! `[ui] sound_key` switch and volume; the lane's own `again` ask never sounds, and a tap out of range still does.

use super::heard;
use crate::Rig;
use crate::sound::{assert_plays, rig_with, takes};
use kx_module_api::ports::WordsAsk;
use kx_theme::sounds::Event;
use kx_ui::UiSettings;
use kx_ui::asks::{AGAIN, CANCEL, FORGET, MORE, PAGE, PAUSE, PICK, POWER, UNDO};

/// Each tap kind QML sends: a chip, every ask with the argument its tap gives, and a tap QML handled alone; gives
/// how many taps it made.
fn tap_each(rig: &Rig) -> usize {
    rig.hub.pick_chip(1, 0);
    let asks = [
        (FORGET, 0),
        (CANCEL, 0),
        (PAUSE, 1),
        (POWER, 0),
        (UNDO, 0),
        (MORE, 0),
        (PAGE, 1),
        (PICK, 0),
    ];
    for (kind, arg) in asks {
        rig.hub.words_ask(kind, arg);
    }
    rig.hub.tapped();
    asks.len() + 2
}

/// The `[ui] sound_volume` default.
fn volume() -> u8 {
    UiSettings::from_defaults().unwrap().sound_volume
}

#[test]
fn each_chip_lane_tap_plays_the_key_sound_once() {
    let (rig, _face, _host) = rig_with(|_| {});
    let (_bus, sent, _subs) = heard(&rig);
    let n = tap_each(&rig);
    assert_eq!(sent.picks().len(), 1, "the chip went out");
    assert_plays(&rig.speaker.take_plays(), &takes(Event::Key, n, volume()));
}

#[test]
fn the_lanes_own_again_never_sounds() {
    let (rig, _face, _host) = rig_with(|_| {});
    let (_bus, sent, _subs) = heard(&rig);
    assert!(rig.speaker.plays().is_empty(), "the ask as the hub listens");
    sent.asks();
    rig.hub.words_ask(AGAIN, 0);
    assert_eq!(sent.asks(), [WordsAsk::Again]);
    assert!(rig.speaker.plays().is_empty());
}

#[test]
fn a_tap_out_of_range_sends_nothing_but_still_sounds() {
    let (rig, _face, _host) = rig_with(|_| {});
    let (_bus, sent, _subs) = heard(&rig);
    sent.asks();
    rig.hub.pick_chip(-1, 0);
    rig.hub.pick_chip(1, 256);
    rig.hub.words_ask(PAUSE, 2);
    rig.hub.words_ask("mystery", 0);
    assert!(sent.picks().is_empty() && sent.asks().is_empty());
    assert_plays(&rig.speaker.take_plays(), &takes(Event::Key, 4, volume()));
}

#[test]
fn the_key_switch_off_or_volume_0_silences_every_chip_lane_tap() {
    let quiet: [fn(&mut UiSettings); 2] = [|s| s.sound_key = false, |s| s.sound_volume = 0];
    for change in quiet {
        let (rig, _face, _host) = rig_with(change);
        let (_bus, _sent, _subs) = heard(&rig);
        tap_each(&rig);
        assert!(rig.speaker.plays().is_empty());
    }
}

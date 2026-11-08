//! Clicks on the strip: a keycap of the live strip sends `ChipAct` with the round QML drew; an old round, a ticked strip
//! or a chip not shown sends nothing quietly, an unknown chip warns once, and every click sounds as a key does.

use super::{heard, strip, ticked, usual, view};
use crate::Rig;
use crate::logs::warnings_of;
use kx_module_api::ports::{ChipAct, ChipsView, Legend, SelChip};
use kx_ui::i18n::keys::sel_chip_id;

#[test]
fn a_click_on_the_live_strip_sends_its_chip_and_round() {
    let rig = Rig::new();
    let (bus, sent, _subs) = heard(&rig);
    bus.publish(&view(9)).unwrap();
    for chip in SelChip::ALL {
        rig.hub.sel_act(9, sel_chip_id(chip));
        assert_eq!(sent.acts(), [ChipAct { round: 9, chip }], "{chip:?}");
    }
}

#[test]
fn a_click_on_an_old_round_a_ticked_strip_or_a_chip_not_shown_sends_nothing_quietly() {
    let rig = Rig::new();
    let (bus, sent, _subs) = heard(&rig);
    let read_only = ChipsView::Shown {
        round: 10,
        chips: vec![SelChip::Copy, SelChip::Translate],
        spot: usual(),
        acted: None,
        legend: Legend::Both,
    };
    let (copy, cut) = (sel_chip_id(SelChip::Copy), sel_chip_id(SelChip::Cut));
    let late = warnings_of(|| {
        rig.hub.sel_act(1, copy);
        bus.publish(&view(9)).unwrap();
        for round in [8, 10, -9, i64::from(u32::MAX) + 9] {
            rig.hub.sel_act(round, copy);
        }
        bus.publish(&read_only).unwrap();
        rig.hub.sel_act(10, cut);
        bus.publish(&ticked(11, SelChip::Copy)).unwrap();
        rig.hub.sel_act(11, sel_chip_id(SelChip::Paste));
    });
    assert!(
        sent.acts().is_empty(),
        "no strip, old rounds, no Cut, a tick"
    );
    assert!(late.is_empty(), "a late click is no fault: {late:?}");
}

#[test]
fn an_unknown_chip_sends_nothing_and_warns_once_each() {
    let rig = Rig::new();
    let (bus, sent, _subs) = heard(&rig);
    bus.publish(&view(2)).unwrap();
    let bad = ["", "Copy", "copy ", "select-all", "نسخ"];
    let warned = warnings_of(|| bad.into_iter().for_each(|k| rig.hub.sel_act(2, k)));
    assert!(sent.acts().is_empty());
    assert_eq!(warned.len(), bad.len(), "{warned:?}");
    assert!(warned.iter().all(|w| !w.contains("نسخ")), "{warned:?}");
}

#[test]
fn a_strip_redrawn_takes_clicks_on_its_new_round_only() {
    let rig = Rig::new();
    let (bus, sent, _subs) = heard(&rig);
    bus.publish(&view(3)).unwrap();
    bus.publish(&strip(4, usual(), None, Legend::Icons))
        .unwrap();
    rig.hub.sel_act(3, sel_chip_id(SelChip::Cut));
    rig.hub.sel_act(4, sel_chip_id(SelChip::Cut));
    let want = ChipAct {
        round: 4,
        chip: SelChip::Cut,
    };
    assert_eq!(sent.acts(), [want]);
}

/// Each click plays the key sound once, a dropped or unknown one too, as the user pressed a keycap all the same.
#[cfg(feature = "qt")]
#[test]
fn each_click_plays_the_key_sound_once_and_none_with_key_sounds_off() {
    use crate::sound::{assert_plays, rig_with, takes};
    use kx_theme::sounds::Event;
    let (rig, _face, _host) = rig_with(|_| {});
    let (bus, _sent, _subs) = heard(&rig);
    bus.publish(&view(1)).unwrap();
    rig.speaker.take_plays();
    rig.hub.sel_act(1, sel_chip_id(SelChip::Copy));
    rig.hub.sel_act(0, sel_chip_id(SelChip::Copy));
    rig.hub.sel_act(1, "mystery");
    let volume = kx_ui::UiSettings::from_defaults().unwrap().sound_volume;
    assert_plays(&rig.speaker.take_plays(), &takes(Event::Key, 3, volume));
    let (quiet, _face, _host) = rig_with(|s| s.sound_key = false);
    let (bus, _sent, _subs) = heard(&quiet);
    bus.publish(&view(1)).unwrap();
    quiet.speaker.take_plays();
    quiet.hub.sel_act(1, sel_chip_id(SelChip::Copy));
    assert!(quiet.speaker.take_plays().is_empty(), "key sounds are off");
}

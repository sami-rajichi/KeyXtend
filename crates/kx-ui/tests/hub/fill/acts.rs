//! Clicks on the panel: each kind QML names sends its `FillClick` by the id it gives, with the round QML drew, which
//! the vault checks. An unknown kind or id, or an argument or round out of range, sends nothing, with one warning that
//! holds no content; a click with no view kept is late and goes quietly.

use super::{heard, listed};
use crate::Rig;
use crate::logs::{lines_of, warnings_of};
use kx_module_api::ports::{
    EntryKind, FieldButton, FillAct, FillButton, FillClick, FillFilter, RowButton,
};
use kx_ui::fill_acts::{ANSWER, BUTTON, CARET, CHECKS, FIELD, FILTER, KIND, PAGE, ROW, WHEEL};
use kx_ui::i18n::keys::{
    entry_kind_id, field_button_id, fill_button_id, fill_filter_id, row_button_id,
};
use std::collections::BTreeSet;

/// Every kind QML names.
const KINDS: [&str; 10] = [
    ROW, FIELD, FILTER, KIND, PAGE, BUTTON, ANSWER, CHECKS, CARET, WHEEL,
];
/// An id that appears nowhere else.
const SECRET: &str = "zebra-quartz";

/// The click the hub sends for `kind`, `id` and `arg` on the view of round 9.
fn sent(kind: &str, id: &str, arg: i32) -> Vec<FillAct> {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, listed(9));
    rig.hub.fill_act(kind, id, arg, 9);
    heard.sent.acts()
}

/// `click` on round 9.
fn on_nine(click: FillClick) -> Vec<FillAct> {
    vec![FillAct { round: 9, click }]
}

#[test]
fn each_kind_is_a_plain_name_of_its_own() {
    let unique: BTreeSet<&str> = KINDS.into_iter().collect();
    assert_eq!(unique.len(), KINDS.len(), "no two kinds share a name");
    let plain = |k: &str| !k.is_empty() && k.bytes().all(|b| b.is_ascii_lowercase());
    assert!(KINDS.into_iter().all(plain), "{KINDS:?}");
}

#[test]
fn each_kind_sends_its_click_with_the_round_qml_drew() {
    let cases = [
        (
            ROW,
            "both",
            2,
            FillClick::Row {
                row: 2,
                button: RowButton::Both,
            },
        ),
        (
            FIELD,
            "show",
            1,
            FillClick::Field {
                field: 1,
                button: FieldButton::Show,
            },
        ),
        (FILTER, "labels", 0, FillClick::Filter(FillFilter::Labels)),
        (KIND, "wifi", 0, FillClick::Kind(EntryKind::Wifi)),
        (PAGE, "", -1, FillClick::Page { back: true }),
        (PAGE, "", 1, FillClick::Page { back: false }),
        (
            BUTTON,
            "back-up-now",
            0,
            FillClick::Button(FillButton::BackUpNow),
        ),
        (ANSWER, "", 1, FillClick::Answer(true)),
        (ANSWER, "", 0, FillClick::Answer(false)),
        (CHECKS, "", 1, FillClick::Checks(true)),
        (CHECKS, "", 0, FillClick::Checks(false)),
        (CARET, "", 0, FillClick::Caret { at: 0.into() }),
        (CARET, "", 7, FillClick::Caret { at: 7.into() }),
    ];
    for (kind, id, arg, click) in cases {
        assert_eq!(sent(kind, id, arg), on_nine(click), "{kind} {id} {arg}");
    }
}

#[test]
fn the_wheel_sends_the_click_a_page_button_does() {
    for (arg, back) in [(-1, true), (1, false)] {
        let click = FillClick::Page { back };
        assert_eq!(sent(WHEEL, "", arg), on_nine(click), "{arg}");
    }
}

#[test]
fn an_older_rounds_click_goes_with_its_round_for_the_vault_to_drop() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, listed(9));
    rig.hub.fill_act(ROW, "insert", 0, 8);
    let click = FillClick::Row {
        row: 0,
        button: RowButton::Insert,
    };
    assert_eq!(heard.sent.acts(), [FillAct { round: 8, click }]);
}

#[test]
fn every_id_of_every_list_names_its_click() {
    for b in RowButton::ALL {
        let click = FillClick::Row { row: 4, button: b };
        assert_eq!(sent(ROW, row_button_id(b), 4), on_nine(click));
    }
    for b in FieldButton::ALL {
        let click = FillClick::Field {
            field: 3,
            button: b,
        };
        assert_eq!(sent(FIELD, field_button_id(b), 3), on_nine(click));
    }
    for f in FillFilter::ALL {
        assert_eq!(
            sent(FILTER, fill_filter_id(f), 0),
            on_nine(FillClick::Filter(f))
        );
    }
    for k in EntryKind::ALL {
        assert_eq!(sent(KIND, entry_kind_id(k), 0), on_nine(FillClick::Kind(k)));
    }
    for b in FillButton::ALL {
        assert_eq!(
            sent(BUTTON, fill_button_id(b), 0),
            on_nine(FillClick::Button(b))
        );
    }
}

#[test]
fn a_bad_click_sends_nothing_and_warns_once_without_content() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, listed(9));
    let past = i32::from(u8::MAX) + 1;
    let bad = [
        ("mystery", "", 0, 9),
        ("", "", 0, 9),
        ("Row", "both", 0, 9),
        (ROW, "Both", 0, 9),
        (ROW, SECRET, 0, 9),
        (ROW, "both", -1, 9),
        (ROW, "both", past, 9),
        (FIELD, "show", past, 9),
        (FIELD, SECRET, 0, 9),
        (FILTER, SECRET, 0, 9),
        (KIND, "", 0, 9),
        (PAGE, "", 0, 9),
        (WHEEL, "", 0, 9),
        (BUTTON, SECRET, 0, 9),
        (ANSWER, "", 2, 9),
        (ANSWER, "", -1, 9),
        (CHECKS, "", 5, 9),
        (CARET, "", -1, 9),
        (BUTTON, "close", 0, -1),
        (BUTTON, "close", 0, i64::from(u32::MAX) + 1),
    ];
    let mut warned = Vec::new();
    let lines = lines_of(|| {
        warned = warnings_of(|| {
            for (kind, id, arg, round) in bad {
                rig.hub.fill_act(kind, id, arg, round);
            }
        });
    });
    assert!(heard.sent.acts().is_empty());
    assert_eq!(warned.len(), bad.len(), "{warned:?}");
    assert!(lines.iter().all(|l| !l.contains(SECRET)), "{lines:?}");
}

/// A caret tells a hidden line's length, so no log line holds its number, on a bad round or a bad place.
#[test]
fn a_bad_caret_click_warns_without_its_number() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, listed(9));
    let bad = [(CARET, "", 5, -1), (CARET, "", -5, 9)];
    let mut warned = Vec::new();
    let lines = lines_of(|| {
        warned = warnings_of(|| {
            for (kind, id, arg, round) in bad {
                rig.hub.fill_act(kind, id, arg, round);
            }
        });
    });
    assert!(heard.sent.acts().is_empty());
    assert_eq!(warned.len(), bad.len(), "{warned:?}");
    assert!(lines.iter().all(|l| !l.contains('5')), "{lines:?}");
}

/// A click that races the panel's close finds no view kept: the vault shows nothing to click, so it is late.
#[test]
fn with_no_view_a_click_sends_nothing_quietly() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.panel(true);
    heard.sent.clear();
    let warned = warnings_of(|| rig.hub.fill_act(BUTTON, "close", 0, 0));
    assert!(heard.sent.acts().is_empty(), "nothing was drawn");
    assert!(warned.is_empty(), "a late click is no fault: {warned:?}");
}

/// Each click but a caret's and the wheel's sounds as a key does, a bad one too; a caret's line sounded itself.
#[cfg(feature = "qt")]
#[test]
fn each_click_but_a_carets_or_the_wheels_plays_the_key_sound_once() {
    use crate::sound::{assert_plays, rig_with, takes};
    use kx_theme::sounds::Event;
    let (rig, _face, _host) = rig_with(|_| {});
    let heard = heard(&rig);
    heard.shown(&rig, listed(1));
    rig.speaker.take_plays();
    for kind in KINDS {
        rig.hub.fill_act(kind, "", 0, 1);
    }
    let volume = kx_ui::UiSettings::from_defaults().unwrap().sound_volume;
    let want = takes(Event::Key, KINDS.len() - 2, volume);
    assert_plays(&rig.speaker.take_plays(), &want);
}

/// The wheel turns a page silently, as the scroll pad does, and a page button still sounds.
#[cfg(feature = "qt")]
#[test]
fn the_wheel_pages_silently_and_a_page_button_sounds() {
    use crate::sound::{assert_plays, rig_with, takes};
    use kx_theme::sounds::Event;
    let (rig, _face, _host) = rig_with(|_| {});
    let heard = heard(&rig);
    heard.shown(&rig, listed(1));
    rig.speaker.take_plays();
    rig.hub.fill_act(WHEEL, "", 1, 1);
    assert!(rig.speaker.take_plays().is_empty(), "no sound for a notch");
    let click = FillClick::Page { back: false };
    assert_eq!(heard.sent.acts(), [FillAct { round: 1, click }]);
    rig.hub.fill_act(PAGE, "", 1, 1);
    let volume = kx_ui::UiSettings::from_defaults().unwrap().sound_volume;
    assert_plays(&rig.speaker.take_plays(), &takes(Event::Key, 1, volume));
    assert_eq!(heard.sent.acts(), [FillAct { round: 1, click }]);
}

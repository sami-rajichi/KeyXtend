//! Clicks on the panel: each kind QML names sends its `ClipAct`, a tick carries the round QML drew and goes only
//! while that view is kept, and the filters follow `Filter::ORDER`. An unknown kind, a row or filter out of range, or
//! a tick with no view sends nothing, with one warning that holds no content.

use super::{heard, page, row, shown};
use crate::Rig;
use crate::logs::{lines_of, warnings_of};
use kx_module_api::ports::{ClipAct, ClipView, Filter};
use kx_ui::clip_acts::{
    BACK, CLOSE, DELETE, DELETE_ALL, FILTER, INSERT, NO, PAGE, PIN, PREVIEW, TICK, YES,
};
use std::collections::BTreeSet;

/// Every kind QML names.
const KINDS: [&str; 12] = [
    TICK, FILTER, PAGE, INSERT, PIN, DELETE, DELETE_ALL, YES, NO, PREVIEW, BACK, CLOSE,
];
/// A title that appears nowhere else.
const SECRET: &str = "zebra-quartz";

#[test]
fn each_kind_is_a_plain_name_of_its_own() {
    let unique: BTreeSet<&str> = KINDS.into_iter().collect();
    assert_eq!(unique.len(), KINDS.len(), "no two kinds share a name");
    let plain = |k: &str| {
        !k.is_empty()
            && !k.starts_with('-')
            && k.bytes().all(|b| b.is_ascii_lowercase() || b == b'-')
    };
    assert!(KINDS.into_iter().all(plain), "{KINDS:?}");
}

#[test]
fn each_kind_sends_its_act_and_a_tick_the_round_qml_drew() {
    let rig = Rig::new();
    let (bus, acts, _subs) = heard(&rig);
    shown(&rig, &bus, &acts, page(9, vec![row(7, "a"), row(8, "b")]));
    let cases = [
        (TICK, 1, ClipAct::Tick { round: 9, row: 1 }),
        (TICK, 0, ClipAct::Tick { round: 9, row: 0 }),
        (PAGE, -1, ClipAct::Page { back: true }),
        (PAGE, 1, ClipAct::Page { back: false }),
        (INSERT, 0, ClipAct::Insert),
        (PIN, 0, ClipAct::Pin),
        (DELETE, 0, ClipAct::Delete),
        (DELETE_ALL, 0, ClipAct::DeleteAll),
        (YES, 0, ClipAct::Confirm(true)),
        (NO, 0, ClipAct::Confirm(false)),
        (PREVIEW, 0, ClipAct::Preview),
        (BACK, 0, ClipAct::Back),
        (CLOSE, 0, ClipAct::Close),
    ];
    for (kind, arg, want) in cases {
        rig.hub.clip_act(kind, arg, 9);
        assert_eq!(acts.take(), [want], "{kind} {arg}");
    }
}

#[test]
fn a_filter_click_names_its_place_in_the_order() {
    let rig = Rig::new();
    let (bus, acts, _subs) = heard(&rig);
    shown(&rig, &bus, &acts, page(1, vec![row(7, "a")]));
    for (at, filter) in (0..).zip(Filter::ORDER) {
        rig.hub.clip_act(FILTER, at, 1);
        assert_eq!(acts.take(), [ClipAct::Filter(filter)]);
    }
}

#[test]
fn a_bad_click_sends_nothing_and_warns_once_without_content() {
    let rig = Rig::new();
    let (bus, acts, _subs) = heard(&rig);
    shown(&rig, &bus, &acts, page(9, vec![row(7, SECRET)]));
    let order = i32::try_from(Filter::ORDER.len()).unwrap();
    let bad = [
        ("mystery", 0),
        ("", 0),
        ("Tick", 0),
        (TICK, 1),
        (TICK, -1),
        (TICK, i32::from(u8::MAX) + 1),
        (FILTER, order),
        (FILTER, -1),
        (PAGE, 0),
    ];
    let mut warned = Vec::new();
    let lines = lines_of(|| {
        warned = warnings_of(|| {
            for (kind, arg) in bad {
                rig.hub.clip_act(kind, arg, 9);
            }
        });
    });
    assert!(acts.take().is_empty());
    assert_eq!(warned.len(), bad.len(), "{warned:?}");
    assert!(lines.iter().all(|l| !l.contains(SECRET)), "{lines:?}");
}

#[test]
fn with_no_view_a_tick_sends_nothing_and_close_still_goes() {
    let rig = Rig::new();
    let (_bus, acts, _subs) = heard(&rig);
    rig.hub.clip_act(TICK, 0, 0);
    assert!(acts.take().is_empty(), "no round to name");
    rig.hub.clip_act(CLOSE, 0, 0);
    assert_eq!(acts.take(), [ClipAct::Close]);
}

/// A view that lands before QML redraws moves the rows, so a tick from the drawn view would name another entry.
#[test]
fn a_tick_from_an_older_view_is_dropped_quietly_and_other_clicks_still_go() {
    let rig = Rig::new();
    let (bus, acts, _subs) = heard(&rig);
    shown(&rig, &bus, &acts, page(9, vec![row(7, "a"), row(8, "b")]));
    let newer = page(10, vec![row(6, "new"), row(7, "a"), row(8, "b")]);
    bus.publish(&ClipView::new(newer)).unwrap();
    let stale = [9, 11, -1, i64::from(u32::MAX) + 10];
    let warned = warnings_of(|| stale.into_iter().for_each(|r| rig.hub.clip_act(TICK, 0, r)));
    assert!(acts.take().is_empty(), "the row moved since it was drawn");
    assert!(warned.is_empty(), "a late click is no fault: {warned:?}");
    rig.hub.clip_act(TICK, 2, 10);
    assert_eq!(acts.take(), [ClipAct::Tick { round: 10, row: 2 }]);
    rig.hub.clip_act(CLOSE, 0, 9);
    assert_eq!(
        acts.take(),
        [ClipAct::Close],
        "only a row's click needs its view"
    );
}

/// Each click sounds as a key does, a bad one too, and so does Open folder.
#[cfg(feature = "qt")]
#[test]
fn each_click_plays_the_key_sound_once() {
    use crate::sound::{assert_plays, rig_with, takes};
    use kx_theme::sounds::Event;
    let (rig, _face, _host) = rig_with(|_| {});
    let (bus, acts, _subs) = heard(&rig);
    shown(&rig, &bus, &acts, page(1, vec![row(7, "a")]));
    rig.speaker.take_plays();
    for kind in KINDS {
        rig.hub.clip_act(kind, 0, 1);
    }
    rig.hub.clip_act("mystery", 0, 1);
    rig.hub.open_folder();
    let volume = kx_ui::UiSettings::from_defaults().unwrap().sound_volume;
    let want = takes(Event::Key, KINDS.len() + 2, volume);
    assert_plays(&rig.speaker.take_plays(), &want);
}

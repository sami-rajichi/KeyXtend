//! Snip's capture windows are kept out of every capture: each is shielded by its title once Qt made it, before it
//! shows, whatever `keyboard` says; one whose shield is refused stays hidden while a countdown or recording runs, and
//! the module hears `Bare`.

use super::{heard, page, picking, rect, screens};
use crate::Rig;
use kx_module_api::ports::{
    Countdown, Keycap, Part, RecKey, Recording, SnipClick, SnipSpot, SnipStage,
};
use kx_platform::ui::UiError;
use kx_platform_fake::GuardCall;
use kx_ui::snip_acts::CAPTURE;
use serde_json::{Value, json};

/// The part a video films.
fn part() -> Part {
    Part::new(rect(1, 1, 4, 3)).unwrap()
}

/// Recording the part, with Stop on its bar.
fn recording() -> SnipStage {
    SnipStage::Record(Recording {
        part: part(),
        screen: rect(0, 0, 8, 6),
        spot: SnipSpot::Top,
        elapsed_ms: 0,
        live: true,
        keys: vec![Keycap {
            key: RecKey::Stop,
            on: false,
            dim: None,
        }],
    })
}

/// What QML must keep hidden now.
fn hidden(rig: &Rig) -> Value {
    super::json(rig)["hide"].clone()
}

#[test]
fn every_capture_window_is_shielded_by_its_title() {
    let rig = Rig::new();
    let _heard = heard(&rig);
    for window in CAPTURE {
        let title = format!("Snip {window}");
        assert!(rig.hub.snip_shield(window, &title), "{window}");
        assert!(rig.guard.calls().contains(&GuardCall::Shield(title, true)));
    }
    assert_eq!(rig.guard.shielded().len(), CAPTURE.len());
    assert!(
        !rig.hub.snip_shield("keyboard", "Keyboard"),
        "only Snip's capture windows"
    );
    assert_eq!(rig.guard.shielded().len(), CAPTURE.len());
}

#[test]
fn a_window_whose_shield_is_refused_stays_hidden_while_a_countdown_or_recording_runs() {
    let rig = Rig::new();
    let heard = heard(&rig);
    rig.guard.set_shield_error(Some(UiError::Refused(5)));
    assert!(!rig.hub.snip_shield("edge", "Snip edge"));
    assert!(!rig.hub.snip_shield("record", "Snip record"));
    rig.guard.set_shield_error(None);
    assert!(rig.hub.snip_shield("bar", "Snip bar"));
    heard.shown(&rig, picking(1, screens()));
    assert_eq!(
        hidden(&rig),
        json!([]),
        "a picture is cut from the frozen screen"
    );
    let count = SnipStage::Count(Countdown {
        part: part(),
        left_s: 3,
    });
    heard.shown(&rig, page(2, Vec::new(), count));
    assert_eq!(hidden(&rig), json!(["edge", "record"]));
    heard.shown(&rig, page(3, Vec::new(), recording()));
    assert_eq!(hidden(&rig), json!(["edge", "record"]));
    assert!(
        rig.hub.snip_shield("edge", "Snip edge"),
        "a later try holds"
    );
    assert_eq!(hidden(&rig), json!(["record"]));
}

#[test]
fn a_refusal_during_a_snip_tells_the_module_at_once_and_one_before_waits_for_the_start() {
    let rig = Rig::new();
    let heard = heard(&rig);
    rig.guard.set_shield_error(Some(UiError::Refused(5)));
    assert!(!rig.hub.snip_shield("strip", "Snip strip"));
    assert_eq!(heard.sent.clicks(), [], "no snip runs");
    heard.shown(&rig, page(1, Vec::new(), recording()));
    assert!(!rig.hub.snip_shield("record", "Snip record"));
    assert_eq!(heard.sent.clicks(), [SnipClick::Bare], "a new refusal");
    assert!(!rig.hub.snip_shield("record", "Snip record"));
    assert_eq!(heard.sent.clicks(), [], "told once");
}

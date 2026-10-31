//! The platform's clipboard-history chord clicked on our keyboard (spec §7, ADR-0034 decision 7): it opens our panel
//! while the clipboard module wants it, and goes to the system otherwise (plan edge case 14).

use super::{Clip, Heard, text};
use crate::assist::scans;
use crate::rig::STATE;
use kx_module_api::ports::{HistoryAsk, HistoryWanted, Scan};
use kx_platform_fake::Stroke;

/// The board's Win latch, and the key the fake's chord takes with it.
const WIN: u16 = 0xE05B;
const V: u16 = 0x2F;
/// The V key's id.
const V_KEY: &str = "c-0x002F";

/// The keyboard on a platform whose history chord is Win with V.
fn chorded(c: Clip) -> Clip {
    let chord = (scans(&[WIN]), Scan::new(V));
    c.b.rig.fakes.shortcuts.set_history(Some(chord));
    c
}

/// Win+V as clicked on our keyboard.
fn win_v(c: &Clip) {
    c.b.rig.tap("win").unwrap();
    c.b.rig.press(V_KEY).unwrap();
}

/// Win+V as it reaches the app.
fn sent() -> Vec<Stroke> {
    vec![Stroke::Chord(scans(&[WIN]), Scan::new(V))]
}

#[test]
fn the_chord_opens_our_panel_only_while_the_module_wants_it() {
    let c = chorded(Clip::quiet(None));
    win_v(&c);
    assert!(c.heard().is_empty(), "never told: the system's own");
    assert_eq!(c.strokes(), sent());
    c.want(true);
    c.b.rig.tap("win").unwrap();
    assert_eq!(c.b.rig.press(V_KEY), Ok(STATE));
    assert_eq!(c.heard(), [Heard::Open]);
    assert!(c.strokes().is_empty());
    assert!(!c.on("win"), "Win let go as after a key");
    c.want(false);
    win_v(&c);
    assert!(c.heard().is_empty(), "off again: the system's own");
    assert_eq!(c.strokes(), sent());
}

#[test]
fn the_chord_toggles_the_open_panel_and_v_alone_still_searches() {
    let c = chorded(Clip::quiet(None));
    c.want(true);
    c.open(true);
    c.b.rig.press(V_KEY).unwrap();
    win_v(&c);
    assert_eq!(c.heard(), [text("v"), Heard::Open]);
    assert!(c.strokes().is_empty());
}

#[test]
fn another_chord_or_a_platform_without_one_goes_to_the_app() {
    let c = chorded(Clip::quiet(None));
    c.want(true);
    c.b.rig.tap("ctrl").unwrap();
    c.b.rig.press(V_KEY).unwrap();
    assert_eq!(c.strokes(), [Stroke::Chord(scans(&[0x1D]), Scan::new(V))]);
    c.b.rig.fakes.shortcuts.set_history(None);
    win_v(&c);
    assert_eq!(c.strokes(), sent());
    assert!(c.heard().is_empty());
}

#[test]
fn a_restart_forgets_the_wanted_chord_until_the_module_answers_again() {
    let mut c = chorded(Clip::quiet(None));
    c.want(true);
    c.b.rig.restart(Some(("lang_key", "cycle".into())));
    assert_eq!(c.heard(), [Heard::Ask]);
    win_v(&c);
    assert_eq!(c.strokes(), sent());
}

/// The keyboard subscribes before it asks, so a clipboard module that answers at once is heard.
#[test]
fn a_clipboard_module_already_running_hands_over_the_chord_at_start() {
    let c = chorded(Clip::after(None, |bus| {
        let reply = bus.clone();
        let answer = move |_: &HistoryAsk| reply.publish(&HistoryWanted { on: true }).unwrap();
        vec![bus.subscribe(answer).unwrap()]
    }));
    win_v(&c);
    assert_eq!(c.heard(), [Heard::Ask, Heard::Open]);
}

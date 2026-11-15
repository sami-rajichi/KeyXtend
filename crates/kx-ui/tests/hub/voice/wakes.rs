//! The Voice object's wakes and the hub's first words: a view wakes once per read, a level wakes only the meter and
//! only while the bar listens, the start asks the module and tells it the rows, and the pointer leaves a bar that hides.

use super::{heard, home, json, listening, listening_bar, page, panel};
use crate::Rig;
use kx_module_api::ports::{VoiceAct, VoiceBar, VoiceClick};
use kx_ui::{VoiceWake, voice_acts};

#[test]
fn one_view_wakes_the_bar_once_per_read() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.view(page(1, listening_bar(), None));
    heard.view(page(2, VoiceBar::Transcribing, None));
    assert_eq!(heard.sent.woken(), [VoiceWake::View], "one wake for both");
    assert_eq!(json(&rig)["round"], 2, "the newest view");
    heard.view(page(3, VoiceBar::Hidden, None));
    assert_eq!(
        heard.sent.woken(),
        [VoiceWake::View],
        "read, so it wakes again"
    );
    assert!(
        rig.wakes().is_empty(),
        "the keyboard's object hears nothing"
    );
}

#[test]
fn levels_wake_the_meter_once_per_read_and_never_the_panel() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, page(1, listening_bar(), Some(panel(home()))));
    heard.level(64);
    heard.level(u8::MAX);
    assert_eq!(heard.sent.woken(), [VoiceWake::Level]);
    assert!((rig.hub.voice_level() - 1.0).abs() < f64::EPSILON, "full");
    heard.level(0);
    assert_eq!(
        heard.sent.woken(),
        [VoiceWake::Level],
        "read, so it wakes again"
    );
    assert!(rig.hub.voice_level().abs() < f64::EPSILON);
    assert_eq!(json(&rig)["round"], 1, "the view stays");
}

#[test]
fn a_bar_that_stops_listening_shows_no_old_level() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, page(1, listening_bar(), None));
    heard.level(200);
    heard.view(page(2, VoiceBar::Transcribing, None));
    assert!(rig.hub.voice_level().abs() < f64::EPSILON);
}

#[test]
fn before_any_view_qml_reads_null() {
    let rig = Rig::new();
    let _heard = heard(&rig);
    assert_eq!(rig.hub.voice_json(), "null");
}

#[test]
fn the_start_asks_the_module_and_tells_it_the_rows_a_page_holds() {
    let rig = Rig::new();
    let heard = listening(&rig);
    assert_eq!(heard.sent.asks(), 1, "a late starter asks");
    let rows = VoiceClick::Rows(u8::try_from(rig.hub.settings().page_rows).unwrap());
    let want = VoiceAct {
        round: 0,
        click: rows,
    };
    assert_eq!(heard.sent.acts(), [want]);
}

#[test]
fn the_pointer_leaves_a_bar_that_hides() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, page(4, listening_bar(), None));
    rig.hub.voice_act(voice_acts::OVER, "", 1, 4);
    heard.sent.acts();
    heard.view(page(5, VoiceBar::Hidden, None));
    let left = VoiceAct {
        round: 5,
        click: VoiceClick::Over(false),
    };
    assert_eq!(heard.sent.acts(), [left]);
    heard.view(page(6, VoiceBar::Hidden, None));
    assert!(heard.sent.acts().is_empty(), "it left once");
}

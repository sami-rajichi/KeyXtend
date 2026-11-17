//! Clicks on the bar and the panel: each kind sends its click with the round of the view QML drew, the gear opens the
//! panel, Start sends the set text of the language in front, a pad page turns the open panel, a bad or late click
//! sends nothing, and no log line holds a text.

use super::{coded, heard, home, listening_bar, page, panel};
use crate::Rig;
use crate::logs::lines_of;
use crate::texts::rig;
use kx_module_api::ports::{
    BarButton, BarText, CompareSheet, DeviceLabel, DeviceName, FetchButton, PanelButton,
    ScrollPage, VoiceAct, VoiceBar, VoiceClick, VoicePanel, VoiceSheet, VoiceTab, VoiceText,
    VoiceView,
};
use kx_module_api::{Redacted, Zeroizing};
use kx_ui::i18n::keys::VOICE_READ_TEXT;
use kx_ui::voice_acts::{
    BAR, BUTTON, COMPARE, CONFIRM, FETCH, FLIP, GEAR, MINUS, OPEN, OVER, PAGE, PLUS, ROW, STEP,
    WHEEL,
};
use kx_ui::{I18n, Wake, find};

/// The round of the view the tests click on.
const ROUND: u32 = 9;

/// `click` on the view of `round`.
const fn act(round: u32, click: VoiceClick) -> VoiceAct {
    VoiceAct { round, click }
}

/// Compare's set text, ready on six models.
fn ready() -> VoiceSheet {
    VoiceSheet::Compare(CompareSheet::Ready {
        models: 6,
        device: DeviceLabel::Auto(DeviceName::Cpu),
        recording: false,
    })
}

#[test]
fn each_kind_sends_its_click_with_the_round_qml_drew() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, page(ROUND, listening_bar(), Some(panel(home()))));
    let fetch = VoiceClick::Fetch {
        row: 2,
        button: FetchButton::Pause,
    };
    let cases = [
        (BAR, "stop", 0, VoiceClick::Bar(BarButton::Stop)),
        (BAR, "undo", 0, VoiceClick::Bar(BarButton::Undo)),
        (OVER, "", 1, VoiceClick::Over(true)),
        (OPEN, "history", 0, VoiceClick::Open(VoiceTab::History)),
        (ROW, "", 4, VoiceClick::Row(4)),
        (PAGE, "", -1, VoiceClick::Page { back: true }),
        (WHEEL, "", 1, VoiceClick::Page { back: false }),
        (FETCH, "pause", 2, fetch),
        (FLIP, "", 3, VoiceClick::Flip(3)),
        (STEP, PLUS, 1, VoiceClick::Step { row: 1, up: true }),
        (STEP, MINUS, 0, VoiceClick::Step { row: 0, up: false }),
        (
            BUTTON,
            "delete-all",
            0,
            VoiceClick::Button(PanelButton::DeleteAll),
        ),
        (CONFIRM, "", 1, VoiceClick::Confirm(true)),
        (CONFIRM, "", 0, VoiceClick::Confirm(false)),
    ];
    for (kind, id, arg, want) in cases {
        rig.hub.voice_act(kind, id, arg, i64::from(ROUND - 1));
        assert_eq!(heard.sent.acts(), [act(ROUND - 1, want)], "{kind} {id}");
    }
}

#[test]
fn the_gear_opens_the_panel_and_sends_no_click() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, page(ROUND, listening_bar(), None));
    rig.hub.voice_act(GEAR, "", 0, i64::from(ROUND));
    assert_eq!(heard.sent.opens(), 1);
    assert!(heard.sent.acts().is_empty());
    assert!(rig.wakes().is_empty(), "the keyboard is not asked to show");
}

/// Compare's set text on a panel for the language of code `iso`.
fn ready_in(iso: &str) -> VoicePanel {
    let mut panel = panel(ready());
    panel.lang = coded(iso);
    panel
}

#[test]
fn start_sends_the_set_text_of_the_panels_language_not_its_words_or_the_face() {
    let (rig, _face, _host) = rig("en", "ar", "en");
    let heard = heard(&rig);
    heard.shown(&rig, page(ROUND, VoiceBar::Hidden, Some(ready_in("fr"))));
    rig.hub.voice_act(COMPARE, "", 0, i64::from(ROUND));
    let french = I18n::new(find("fr").unwrap()).text(VOICE_READ_TEXT);
    let want = VoiceClick::Compare(Redacted::new(french));
    assert_eq!(heard.sent.acts(), [act(ROUND, want)]);
}

#[test]
fn a_language_with_no_set_text_starts_compare_with_none() {
    let (rig, _face, _host) = rig("en", "en", "auto");
    let heard = heard(&rig);
    heard.shown(&rig, page(ROUND, VoiceBar::Hidden, Some(ready_in("de"))));
    rig.hub.voice_act(COMPARE, "", 0, i64::from(ROUND));
    let want = VoiceClick::Compare(Redacted::new(String::new()));
    assert_eq!(heard.sent.acts(), [act(ROUND, want)]);
}

#[test]
fn a_bad_click_sends_nothing() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, page(ROUND, listening_bar(), None));
    let past = i64::from(u32::MAX) + 1;
    let bad = [
        (BAR, "gear", 0, 9),
        ("nope", "", 0, 9),
        (ROW, "", -1, 9),
        (ROW, "", 256, 9),
        (PAGE, "", 0, 9),
        (CONFIRM, "", 2, 9),
        (OVER, "", -1, 9),
        (STEP, "up", 0, 9),
        (OPEN, "home", 0, 9),
        (FETCH, "stop", 0, 9),
        (BUTTON, "send", 0, 9),
        (BAR, "stop", 0, -1),
        (BAR, "stop", 0, past),
        (GEAR, "", 0, -1),
    ];
    for (kind, id, arg, round) in bad {
        rig.hub.voice_act(kind, id, arg, round);
    }
    assert!(heard.sent.acts().is_empty());
    assert_eq!(heard.sent.opens(), 0);
}

/// What the user dictated, as no log line or debug print may hold it.
const SECRET: &str = "s3cret-dictation";

#[test]
fn no_text_name_or_set_text_reaches_a_log_line_or_a_debug_print() {
    let (rig, _face, _host) = rig("en", "en", "auto");
    let heard = heard(&rig);
    let text = VoiceText::Shown {
        text: Redacted::new(Zeroizing::new(SECRET.to_owned())),
        rtl: false,
        arabic: false,
    };
    let bar = VoiceBar::Text(BarText {
        text,
        hint: None,
        undo: true,
    });
    let shown = page(ROUND, bar, Some(panel(ready())));
    let view = VoiceView::new(shown.clone());
    assert!(!format!("{view:?}").contains(SECRET), "{view:?}");
    let lines = lines_of(|| {
        heard.shown(&rig, shown);
        let json: Zeroizing<String> = rig.hub.voice_json();
        assert!(json.contains(SECRET), "QML reads it");
        assert_eq!(
            json.capacity(),
            json.len(),
            "sized once, so no grown copy is left"
        );
        rig.hub.voice_act(COMPARE, "", 0, i64::from(ROUND));
        rig.hub.voice_act(SECRET, SECRET, 0, i64::from(ROUND));
        rig.hub.voice_act(BAR, SECRET, -7, -7);
    });
    assert!(lines.iter().all(|l| !l.contains(SECRET)), "{lines:?}");
    let read = I18n::new(find("en").unwrap()).text(VOICE_READ_TEXT);
    let sent = heard.sent.acts();
    assert!(
        !format!("{sent:?}").contains(&read),
        "the set text goes redacted"
    );
}

#[test]
fn a_click_before_any_view_is_dropped() {
    let rig = Rig::new();
    let heard = heard(&rig);
    rig.hub.voice_act(BAR, "stop", 0, 0);
    rig.hub.voice_act(GEAR, "", 0, 0);
    assert!(heard.sent.acts().is_empty());
    assert_eq!(heard.sent.opens(), 0);
}

#[test]
fn a_pad_page_tap_turns_the_open_panel_and_else_our_open_list() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, page(ROUND, listening_bar(), Some(panel(home()))));
    heard.bus.publish(&ScrollPage { back: true }).unwrap();
    assert_eq!(
        heard.sent.acts(),
        [act(ROUND, VoiceClick::Page { back: true })]
    );
    assert!(rig.wakes().is_empty(), "no list of ours turns");
    heard.view(page(ROUND + 1, listening_bar(), None));
    heard.bus.publish(&ScrollPage { back: false }).unwrap();
    assert!(heard.sent.acts().is_empty(), "the bar has no page");
    assert_eq!(rig.wakes(), [Wake::Page { back: false }]);
}

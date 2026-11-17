//! The bar as QML reads it: each state's words in the keyboard's language, its buttons in one order in every language
//! with the dimmed ones in place, its text with its direction or dots, and a note with the action it names.

use super::{arabic, button, buttons, english, heard, json, listening_bar, page};
use crate::Rig;
use crate::texts::{rig, switch};
use kx_module_api::ports::{
    BarButton, BarHint, BarNote, BarPlace, BarText, Listening, VoiceBar, VoiceText,
};
use kx_module_api::{Redacted, Zeroizing};
use kx_ui::VoiceWake;
use kx_ui::i18n::keys::{VOICE_MASK, WINDOW_VOICE};
use serde_json::{Value, json};

/// The bar QML reads for `bar`.
fn bar_json(bar: VoiceBar) -> Value {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.view(page(1, bar, None));
    json(&rig)["bar"].clone()
}

/// The `[id, on]` of each of `bar`'s buttons.
fn ids(bar: &Value) -> Vec<(String, bool)> {
    let all = buttons(&bar["buttons"]);
    all.into_iter()
        .map(|[_, id, _, on, _]| (id, on == "true"))
        .collect()
}

/// `ids` as `ids` lists them.
fn want(ids: &[(&str, bool)]) -> Vec<(String, bool)> {
    ids.iter().map(|(id, on)| ((*id).to_owned(), *on)).collect()
}

#[test]
fn listening_shows_the_language_and_the_timer_between_cancel_and_stop() {
    let bar = bar_json(listening_bar());
    assert_eq!(bar["state"], "listening");
    assert_eq!(bar["word"], "Listening…");
    let lang = json!({"name": "English", "rtl": false, "arabic": false});
    assert_eq!(bar["lang"], lang);
    assert_eq!(bar["elapsedMs"], 1200);
    assert_eq!(bar["waveMs"], 300);
    assert_eq!(bar["hint"], Value::Null);
    assert_eq!(ids(&bar), want(&[("cancel", true), ("stop", true)]));
    let [cancel, stop] = [0, 1].map(|i| buttons(&bar["buttons"])[i].clone());
    assert_eq!(cancel, button("bar", "cancel", "Cancel", true, "plain"));
    assert_eq!(
        stop,
        button("bar", "stop", "Stop", true, "primary"),
        "Stop is the strongest button, drawn dark (owner, 2026-11-14)"
    );
}

#[test]
fn listening_that_hears_nothing_yet_says_so_and_keeps_listening() {
    let bar = bar_json(VoiceBar::Listening(Listening {
        lang: arabic(),
        elapsed_ms: 6000,
        nothing: true,
        wave_ms: 0,
    }));
    assert_eq!(bar["hint"], "Nothing heard yet");
    assert_eq!(bar["lang"]["rtl"], true);
    assert_eq!(
        bar["lang"]["name"], "Arabic",
        "named in the keyboard's words"
    );
}

#[test]
fn transcribing_keeps_stop_in_its_place_dimmed() {
    let bar = bar_json(VoiceBar::Transcribing);
    assert_eq!(bar["state"], "transcribing");
    assert_eq!(bar["word"], "Transcribing…");
    assert_eq!(ids(&bar), want(&[("cancel", true), ("stop", false)]));
    assert_eq!(
        buttons(&bar["buttons"])[1][4],
        "primary",
        "still dark, dimmed"
    );
}

#[test]
fn a_text_shows_with_its_direction_then_copy_undo_and_close() {
    let text = VoiceText::Shown {
        text: Redacted::new(Zeroizing::new("مرحبا\u{202e}".to_owned())),
        rtl: true,
        arabic: true,
    };
    let bar = bar_json(VoiceBar::Text(BarText {
        text,
        hint: Some(BarHint::TooLong),
        undo: false,
    }));
    let shown = json!({"text": "مرحبا", "rtl": true, "arabic": true, "masked": false});
    assert_eq!(bar["text"], shown, "hidden marks go");
    let hint = "Stopped at the longest dictation, and the text was typed.";
    assert_eq!(bar["hint"], json!({"id": "too-long", "text": hint}));
    let order = [("copy", true), ("undo", false), ("close", true)];
    assert_eq!(ids(&bar), want(&order), "Undo insert dims in its place");
}

#[test]
fn a_password_boxs_text_shows_as_dots_with_no_copy() {
    let bar = bar_json(VoiceBar::Text(BarText {
        text: VoiceText::Masked,
        hint: Some(BarHint::Password),
        undo: true,
    }));
    let dots = crate::voice::english_text(VOICE_MASK);
    let shown = json!({"text": dots, "rtl": false, "arabic": false, "masked": true});
    assert_eq!(bar["text"], shown);
    assert_eq!(bar["hint"]["id"], "password");
    assert_eq!(ids(&bar), want(&[("undo", true), ("close", true)]));
}

#[test]
fn a_note_names_its_action_and_download_carries_the_models_size() {
    let bar = bar_json(VoiceBar::Note(BarNote {
        hint: BarHint::NoModel {
            lang: english(),
            bytes: 739_508_576,
        },
        act: Some(BarButton::Download),
        mic: false,
    }));
    assert_eq!(bar["state"], "note");
    assert_eq!(bar["hint"]["text"], "No model for English yet");
    assert_eq!(bar["mic"], false);
    let download = button("bar", "download", "Download · 740 MB", true, "primary");
    let close = button("bar", "close", "Close", true, "plain");
    assert_eq!(buttons(&bar["buttons"]), [download, close]);
}

#[test]
fn a_note_while_the_microphone_is_open_keeps_the_red_dot() {
    let bar = bar_json(VoiceBar::Note(BarNote {
        hint: BarHint::Nothing,
        act: Some(BarButton::Listen),
        mic: true,
    }));
    assert_eq!(bar["mic"], true);
    assert_eq!(buttons(&bar["buttons"])[0][2], "Listen again");
    let bare = bar_json(VoiceBar::Note(BarNote {
        hint: BarHint::Language(english()),
        act: None,
        mic: false,
    }));
    assert_eq!(ids(&bare), want(&[("close", true)]), "the gear acts");
}

#[test]
fn the_view_carries_the_place_the_sounds_and_the_bars_words() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let mut top = page(3, VoiceBar::Hidden, None);
    top.place = BarPlace::Top;
    top.sounds = false;
    heard.view(top);
    let view = json(&rig);
    assert_eq!(view["place"], "top");
    assert_eq!(view["sounds"], false);
    assert_eq!(view["bar"], json!({"state": "hidden"}));
    assert_eq!(view["panel"], Value::Null);
    let words = &view["words"];
    assert_eq!(words["window"], rig.hub.i18n().text(WINDOW_VOICE));
    assert_eq!(words["title"], "Dictation");
    assert_eq!(words["gear"], "Dictation settings");
    assert_eq!(words["copied"], "Copied");
    assert_eq!(
        words["timer"], "%1:%2",
        "the timer's minutes, then its seconds, as Qt's arg fills them"
    );
}

#[test]
fn the_bar_speaks_the_keyboards_language_and_keeps_its_button_order() {
    let (rig, _face, _host) = rig("en", "ar", "auto");
    let heard = heard(&rig);
    heard.view(page(1, listening_bar(), None));
    let view = json(&rig);
    assert_eq!(view["bar"]["word"], "جارٍ الاستماع…");
    assert_eq!(view["words"]["title"], "إملاء");
    let order = [("cancel", true), ("stop", true)];
    assert_eq!(ids(&view["bar"]), want(&order), "Cancel stays left");
}

#[test]
fn a_layout_change_words_a_kept_view_again() {
    let (rig, face, _host) = rig("en", "en", "auto");
    let heard = heard(&rig);
    heard.shown(&rig, page(1, listening_bar(), None));
    switch(&rig, &face, "fr");
    assert_eq!(heard.sent.woken(), [VoiceWake::View]);
    assert_eq!(json(&rig)["bar"]["word"], "Écoute…");
}

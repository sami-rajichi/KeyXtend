//! The Voice panel's frame and home list: the header with Back and Close, each page's row with its present value, Try
//! again dimmed until the module lists it, the note band, pages and ticks, in the keyboard's language.

use super::{acting, arabic, button, buttons, heard, home, panel, panel_json};
use crate::Rig;
use crate::texts::rig;
use kx_module_api::Redacted;
use kx_module_api::ports::{
    BarHint, Date, DeviceLabel, DeviceName, Pages, PanelButton, VoiceHome, VoiceSheet,
};
use kx_ui::i18n::keys::{VOICE_AGAIN_HINT, WINDOW_VOICE_PANEL};
use serde_json::{Value, json};

/// Home rows as `(tab, label, value)`.
fn rows(panel: &Value) -> Vec<(String, String, String)> {
    let rows = panel["sheet"]["rows"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let text = |r: &Value, k: &str| r[k].as_str().unwrap_or_default().to_owned();
    rows.iter()
        .map(|r| (text(r, "tab"), text(r, "label"), text(r, "value")))
        .collect()
}

/// `(tab, label, value)` rows as `rows` lists them.
fn want(rows: [(&str, &str, &str); 5]) -> Vec<(String, String, String)> {
    let own = |s: &str| s.to_owned();
    rows.iter()
        .map(|(t, l, v)| (own(t), own(l), own(v)))
        .collect()
}

#[test]
fn the_home_list_names_each_page_with_its_value() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let p = panel_json(&rig, &heard, panel(home()));
    assert_eq!(p["page"], "home");
    assert_eq!(p["sheet"]["is"], "home");
    assert_eq!(p["window"], rig.hub.i18n().text(WINDOW_VOICE_PANEL));
    assert_eq!(p["title"], "Dictation");
    assert_eq!(p["sub"], "Language: English");
    assert_eq!(p["back"], Value::Null, "home has no Back");
    let expected = want([
        ("models", "Models", "No model ready"),
        ("device", "Device", "Automatic (processor)"),
        ("compare", "Compare models", "Not run yet"),
        ("history", "History", "12 texts"),
        ("options", "Options", "All at defaults"),
    ]);
    assert_eq!(rows(&p), expected);
}

#[test]
fn home_values_name_the_model_the_card_the_last_run_and_the_changes() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let sheet = VoiceSheet::Home(VoiceHome {
        model: Some(Redacted::new("Whisper small".to_owned())),
        device: DeviceLabel::Auto(DeviceName::Card("Radeon 780M".to_owned())),
        compared: Date::new(2026, 11, 12),
        history: 1,
        changed: 3,
    });
    let values: Vec<String> = rows(&panel_json(&rig, &heard, panel(sheet)))
        .into_iter()
        .map(|(_, _, v)| v)
        .collect();
    let expected = [
        "Whisper small",
        "Automatic (Radeon 780M)",
        "Last run: 12/11/2026",
        "1 text",
        "Changed: 3",
    ];
    assert_eq!(values, expected);
}

#[test]
fn a_device_picked_by_hand_reads_as_itself() {
    let rig = Rig::new();
    let heard = heard(&rig);
    for (device, word) in [
        (DeviceLabel::Cpu, "Processor"),
        (DeviceLabel::Card("RTX 4060".to_owned()), "RTX 4060"),
    ] {
        let sheet = VoiceSheet::Home(VoiceHome {
            model: None,
            device,
            compared: None,
            history: 0,
            changed: 0,
        });
        let p = panel_json(&rig, &heard, panel(sheet));
        assert_eq!(rows(&p)[1].2, word);
    }
}

#[test]
fn try_again_dims_until_the_module_lists_it() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let p = panel_json(&rig, &heard, panel(home()));
    assert_eq!(p["line"], super::english_text(VOICE_AGAIN_HINT));
    let dim = button("button", "again", "Try again", false, "primary");
    assert_eq!(buttons(&p["foot"]), [dim]);
    let lit = acting(home(), &[PanelButton::Close, PanelButton::Again]);
    let p = panel_json(&rig, &heard, lit);
    assert_eq!(buttons(&p["foot"])[0][3], "true");
}

#[test]
fn every_page_has_close_and_a_page_but_home_has_back() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let p = panel_json(&rig, &heard, acting(home(), &[]));
    let close = json!({"kind": "button", "id": "close", "arg": 0, "label": "Close", "on": true, "tone": "plain"});
    assert_eq!(p["close"], close, "Close always acts");
    let options = panel(VoiceSheet::Options(Vec::new()));
    let p = panel_json(&rig, &heard, options);
    assert_eq!(p["back"]["id"], "back");
    assert_eq!(p["back"]["label"], "Back");
    assert_eq!(p["back"]["on"], true);
}

#[test]
fn the_note_band_pages_and_ticks_carry_over() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let mut shown = panel(home());
    shown.note = Some(BarHint::Full);
    shown.pages = Pages { at: 1, count: 3 };
    shown.ticked = 2;
    let p = panel_json(&rig, &heard, shown);
    let full = "Not enough room on the drive, so nothing was downloaded.";
    assert_eq!(p["note"], json!({"id": "full", "text": full}));
    assert_eq!(p["pages"], json!({"at": 1, "count": 3}));
    assert_eq!(p["ticked"], 2);
    assert_eq!(p["lang"]["name"], "English");
}

#[test]
fn the_panel_speaks_the_keyboards_language_and_names_the_one_in_front() {
    let (rig, _face, _host) = rig("en", "ar", "auto");
    let heard = heard(&rig);
    let mut shown = panel(home());
    shown.lang = arabic();
    let p = panel_json(&rig, &heard, shown);
    assert_eq!(p["title"], "إملاء");
    assert!(
        p["sub"].as_str().unwrap().contains("العربية"),
        "{}",
        p["sub"]
    );
    assert_eq!(p["lang"]["rtl"], true);
}

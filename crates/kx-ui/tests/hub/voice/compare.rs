//! Compare models: the set text in the language in front with its models and Start, Cancel while the user reads, each
//! model's turn, and the results' table with what Compare keeps; a language with no set text asks for free speech.

use super::{acting, button, buttons, english_text, heard, panel, panel_json};
use crate::Rig;
use crate::texts::rig;
use kx_module_api::Redacted;
use kx_module_api::ports::{
    CompareRow, CompareRun, CompareSheet, Date, DeviceLabel, DeviceName, PanelButton, RunStep,
    VoiceSheet,
};
use kx_ui::i18n::keys::{VOICE_INTRO, VOICE_INTRO_FREE, VOICE_KEPT, VOICE_READ_TEXT};
use serde_json::{Value, json};

/// Compare's set text, ready on `models` models.
fn ready(models: u16, recording: bool) -> VoiceSheet {
    VoiceSheet::Compare(CompareSheet::Ready {
        models,
        device: DeviceLabel::Auto(DeviceName::Cpu),
        recording,
    })
}

/// `name`, redacted.
fn name(name: &str) -> Redacted<String> {
    Redacted::new(name.to_owned())
}

#[test]
fn the_set_text_shows_with_its_models_and_start() {
    let (rig, _face, _host) = rig("en", "en", "auto");
    let heard = heard(&rig);
    let p = panel_json(&rig, &heard, panel(ready(6, false)));
    assert_eq!(
        (&p["page"], &p["title"]),
        (&json!("compare"), &json!("Compare models"))
    );
    assert_eq!(p["sub"], "For English");
    let sheet = json!({"is": "ready", "intro": english_text(VOICE_INTRO),
        "text": english_text(VOICE_READ_TEXT), "line": "6 ready models · Automatic (processor)", "recording": null});
    assert_eq!(p["sheet"], sheet);
    let start = button("compare", "", "Start reading", true, "primary");
    assert_eq!(buttons(&p["foot"]), [start]);
    let none = panel_json(&rig, &heard, panel(ready(0, false)));
    assert_eq!(buttons(&none["foot"])[0][3], "false", "no ready model");
    assert_eq!(
        none["sheet"]["line"],
        "0 ready models · Automatic (processor)"
    );
}

#[test]
fn while_the_user_reads_start_goes_and_cancel_stays() {
    let (rig, _face, _host) = rig("en", "en", "auto");
    let heard = heard(&rig);
    let p = panel_json(&rig, &heard, acting(ready(1, true), &[PanelButton::Cancel]));
    assert_eq!(
        p["sheet"]["recording"],
        "Read the text now, then press Stop on the bar."
    );
    assert_eq!(p["sheet"]["line"], "1 ready model · Automatic (processor)");
    assert_eq!(
        buttons(&p["foot"]),
        [button("button", "cancel", "Cancel", true, "plain")]
    );
}

/// The set text's sheet for a layout in `layout` with `[ui] language` `setting`.
fn read_for(layout: &str, setting: &str) -> Value {
    let (rig, _face, _host) = rig("en", layout, setting);
    let heard = heard(&rig);
    panel_json(&rig, &heard, panel(ready(6, false)))["sheet"].clone()
}

#[test]
fn the_set_text_follows_the_layout_in_front_and_a_language_with_none_speaks_freely() {
    let sheet = read_for("fr", "en");
    let text = sheet["text"].as_str().unwrap().to_owned();
    assert!(
        text.starts_with("Le train"),
        "French to read, English words: {text}"
    );
    assert_eq!(sheet["intro"], english_text(VOICE_INTRO));
    let sheet = read_for("de", "auto");
    assert_eq!(sheet["text"], Value::Null);
    assert_eq!(sheet["intro"], english_text(VOICE_INTRO_FREE));
}

#[test]
fn each_model_shows_its_turn_and_the_step_names_the_one_writing() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let run = |n: &str, step| CompareRun {
        name: name(n),
        step,
    };
    let rows = vec![
        run("Whisper small", RunStep::Done { wait_ms: 2100 }),
        run("Whisper medium", RunStep::Writing),
        run("Cohere", RunStep::Waiting),
    ];
    let sheet = VoiceSheet::Compare(CompareSheet::Running {
        at: 2,
        of: 3,
        name: name("Whisper medium"),
        rows,
    });
    let p = panel_json(&rig, &heard, acting(sheet, &[PanelButton::Cancel]));
    let top = json!({"id": "step", "text": "Model 2 of 3", "name": "Whisper medium"});
    assert_eq!(p["top"], top);
    let rows = &p["sheet"]["rows"];
    assert_eq!(
        rows[0],
        json!({"name": "Whisper small", "mark": {"id": "done", "text": "Done · 2.1 s"}})
    );
    assert_eq!(
        rows[1]["mark"],
        json!({"id": "writing", "text": "Writing…"})
    );
    assert_eq!(rows[2]["mark"]["id"], "waiting");
    assert_eq!(
        buttons(&p["foot"]),
        [button("button", "cancel", "Cancel", true, "plain")],
        "a plain Cancel, as the mock-up draws it"
    );
}

/// The results: one scored model, ticked, and one with free speech, on the processor.
fn results() -> VoiceSheet {
    let scored = CompareRow {
        name: name("Whisper small"),
        wrong: Some(3),
        words: 41,
        wait_ms: 2100,
        mb: 512,
        text: None,
        tick: true,
    };
    let free = CompareRow {
        wrong: None,
        text: Some(name("hello there")),
        tick: false,
        ..scored.clone()
    };
    VoiceSheet::Compare(CompareSheet::Results {
        day: Date::new(2026, 11, 12).unwrap(),
        device: DeviceLabel::Cpu,
        rows: vec![scored, free],
    })
}

#[test]
fn the_results_show_each_models_numbers_and_what_is_kept() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let p = panel_json(&rig, &heard, panel(results()));
    assert_eq!(p["sub"], "12/11/2026 · Processor");
    assert_eq!(
        p["top"],
        json!({"id": "kept", "text": english_text(VOICE_KEPT), "name": null})
    );
    let sheet = &p["sheet"];
    assert_eq!(
        sheet["head"],
        json!(["Model", "Words wrong", "Wait", "Memory"])
    );
    let first = json!({"name": "Whisper small", "wrong": "3 of 41", "wait": "2.1 s", "memory": "0.5 GB",
        "text": null, "tick": true});
    assert_eq!(sheet["rows"][0], first);
    assert_eq!(
        (&sheet["rows"][1]["wrong"], &sheet["rows"][1]["text"]),
        (&Value::Null, &json!("hello there"))
    );
    let foot = [
        button("button", "rerun", "Compare again", false, "plain"),
        button("button", "use", "Use for this language", false, "primary"),
    ];
    assert_eq!(buttons(&p["foot"]), foot);
}

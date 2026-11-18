//! Cloud models and the bar through the hub: the Models page's Local | Cloud chips and band (edge 29), a cloud model's
//! row with its service and mark or why it is Unavailable (edge 17), Compare's ticked cloud models and send warning,
//! a cloud model's reason while Compare runs (carry 5), the bar's cloud mark, and a cloud note's buttons (carry 4).

use super::super::{button, buttons, heard, json, listening_bar, page, panel, panel_json};
use super::{groq, marks, pairs};
use crate::Rig;
use kx_module_api::Redacted;
use kx_module_api::ports::{
    BarButton, BarHint, BarNote, CloudBand, CloudFault, CloudRow, CloudWhy, CompareCloud,
    CompareRun, CompareSheet, DeviceLabel, DeviceName, LocalUse, ModelRow, ModelSide, ModelState,
    ModelsSheet, RunStep, ServiceMark, ServiceName, Unavailable, VoiceBar, VoiceSheet,
};
use kx_ui::voice_acts::{BAR, LOCAL, OPEN, SIDE};
use serde_json::{Value, json};

/// A cloud model `name` of `service` standing at `mark`, maybe Unavailable.
fn cloud_model(name: &str, service: ServiceName, mark: ServiceMark, na: bool) -> ModelRow {
    ModelRow {
        name: Redacted::new(name.to_owned()),
        bytes: 0,
        default: false,
        state: ModelState::Ready,
        used: false,
        tick: false,
        fetch: None,
        cloud: Some(CloudRow {
            service,
            mark,
            unavailable: na.then_some(Unavailable::KeepNothing),
        }),
    }
}

/// The Models page's Cloud side with `rows` and `band`.
fn cloud_side(rows: Vec<ModelRow>, band: CloudBand) -> VoiceSheet {
    VoiceSheet::Models(ModelsSheet {
        rows,
        best: None,
        untested: false,
        slow: false,
        folder: None,
        offer: None,
        side: ModelSide::Cloud,
        band: Some(band),
    })
}

/// A list of one JSON button, for `buttons`.
fn one(b: &Value) -> Value {
    Value::Array(vec![b.clone()])
}

#[test]
fn the_cloud_side_lists_each_model_with_its_service_and_where_it_stands() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let router = ServiceName::Named("OpenRouter".to_owned());
    let rows = vec![
        cloud_model("Whisper Large v3 Turbo", groq(), ServiceMark::Ready, false),
        cloud_model("voxtral", router.clone(), ServiceMark::Ready, true),
        cloud_model("nemotron", router, ServiceMark::NeedsKey, false),
    ];
    let p = panel_json(&rig, &heard, panel(cloud_side(rows, CloudBand::Listed)));
    let sides = &p["sheet"]["sides"];
    assert_eq!(
        (sides["name"].as_str(), sides["shown"].as_str()),
        (Some("Where the model runs"), Some("cloud"))
    );
    let chips = [
        button(SIDE, "local", "Local", true, "plain"),
        button(SIDE, "cloud", "Cloud", true, "plain"),
    ];
    assert_eq!(buttons(&sides["chips"]), chips, "edge 29");
    let band = &p["sheet"]["band"];
    let listed = "Models of the services you switched on; nothing leaves before Stop.";
    assert_eq!(band["text"], listed);
    let to = button(OPEN, "cloud", "Cloud services", true, "plain");
    assert_eq!(buttons(&one(&band["button"])), [to]);
    let rows = p["sheet"]["rows"].as_array().cloned().unwrap_or_default();
    assert_eq!(rows[0]["line"], "In the cloud, with your key");
    assert_eq!(
        marks(&rows[0]["marks"]),
        pairs(&[("cloud", "Groq"), ("ready", "Ready")])
    );
    let none = "No host of this model promises to keep nothing.";
    assert_eq!(rows[1]["line"], none, "edge 17");
    let na = pairs(&[("cloud", "OpenRouter"), ("unavailable", "Unavailable")]);
    assert_eq!(marks(&rows[1]["marks"]), na);
    let key = pairs(&[("cloud", "OpenRouter"), ("needs-key", "Needs a key")]);
    assert_eq!(marks(&rows[2]["marks"]), key);
}

#[test]
fn each_band_says_where_cloud_stands() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let off = "Cloud services are off: none is offered and nothing is sent.";
    for (band, text) in [
        (CloudBand::Off, off),
        (CloudBand::NoneOn, "No cloud service is switched on yet."),
    ] {
        let p = panel_json(&rig, &heard, panel(cloud_side(Vec::new(), band)));
        assert_eq!(p["sheet"]["band"]["text"], text);
    }
}

#[test]
fn compare_lists_its_cloud_models_and_warns_where_the_voice_goes_before_start() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let pick = CompareCloud {
        name: Redacted::new("Whisper Large v3 Turbo".to_owned()),
        service: groq(),
        tick: true,
    };
    let sheet = VoiceSheet::Compare(CompareSheet::Ready {
        models: 2,
        device: DeviceLabel::Auto(DeviceName::Cpu),
        recording: false,
        preparing: false,
        cloud: vec![pick],
        send: vec![groq(), ServiceName::Custom],
    });
    let p = panel_json(&rig, &heard, panel(sheet));
    let picks = json!([{"name": "Whisper Large v3 Turbo", "service": "Groq", "tick": true}]);
    assert_eq!(p["sheet"]["picks"], picks);
    let send = "Compare sends your voice to Groq, Custom address at Stop.";
    assert_eq!(p["sheet"]["send"], send);
}

#[test]
fn a_cloud_model_that_failed_while_compare_runs_says_why() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let run = |why| CompareRun {
        name: Redacted::new("Whisper Large v3".to_owned()),
        step: RunStep::Failed,
        why,
    };
    let sheet = VoiceSheet::Compare(CompareSheet::Running {
        at: 2,
        of: 2,
        name: Redacted::new("Whisper small".to_owned()),
        rows: vec![
            run(Some(CloudFault::Key)),
            run(Some(CloudFault::Language)),
            run(None),
        ],
    });
    let p = panel_json(&rig, &heard, panel(sheet));
    let rows = &p["sheet"]["rows"];
    assert_eq!(
        rows[0]["why"],
        "The service refused the key. Check it on the key page."
    );
    assert_eq!(rows[1]["why"], "The service does not take English.");
    assert_eq!(rows[2]["why"], Value::Null);
}

#[test]
fn the_bar_shows_the_cloud_mark_with_its_service_only_while_one_takes_the_voice() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let mut shown = page(1, listening_bar(), None);
    heard.view(shown.clone());
    assert_eq!(json(&rig)["cloud"], Value::Null);
    shown.cloud = Some(groq());
    heard.view(shown);
    let mark = json!({"name": "Groq", "said": "Your voice goes to Groq at Stop"});
    assert_eq!(json(&rig)["cloud"], mark);
}

/// The bar's buttons for a cloud note on `why` with `local`, and Try again when `again`.
fn note_buttons(why: CloudWhy, local: LocalUse, again: bool) -> Vec<[String; 5]> {
    let rig = Rig::new();
    let heard = heard(&rig);
    let note = BarNote {
        hint: BarHint::Cloud { why, local },
        act: again.then_some(BarButton::Again),
        mic: false,
    };
    heard.view(page(1, VoiceBar::Note(note), None));
    let bar = json(&rig)["bar"].clone();
    assert_eq!(bar["state"], "note");
    buttons(&bar["buttons"])
}

#[test]
fn a_cloud_note_offers_try_again_then_the_local_model_dimmed_with_download_when_none_is_ready() {
    let again = button(BAR, "again", "Try again", true, "primary");
    let local = |on| button(LOCAL, "local", "Use the local model", on, "plain");
    let close = button(BAR, "close", "Close", true, "plain");
    let ready = note_buttons(CloudWhy::Key, LocalUse::Ready, true);
    assert_eq!(ready, [again.clone(), local(true), close.clone()]);
    let fetch = note_buttons(
        CloudWhy::Net,
        LocalUse::Download { bytes: 740_000_000 },
        true,
    );
    let download = button(BAR, "download", "Download · 740 MB", true, "plain");
    assert_eq!(
        fetch,
        [again, local(false), download, close.clone()],
        "edge 20"
    );
    let none = note_buttons(CloudWhy::NoLocal, LocalUse::Dimmed, false);
    assert_eq!(none, [local(false), close]);
}

//! The cloud pages through the hub (ADR-0047): each cloud click, the Cloud page with its switch, marks and footer, and
//! the home list's Cloud value, paged five to a page; a card's and the key page's tests are in `card` and `lines`.

use super::{acting, button, buttons, heard, page, panel, panel_json};
use crate::Rig;
use kx_module_api::ports::{
    CloudButton, CloudSheet, DeviceLabel, Fact, FactArg, FactValue, Hundredths, ModelSide,
    PanelButton, ServiceMark, ServiceName, ServiceRow, VoiceBar, VoiceClick, VoiceHome, VoiceSheet,
    VoiceTab,
};
use kx_ui::voice_acts::{CLOUD, KEEP, LOCAL, SERVICE, SIDE};
use serde_json::Value;

mod card;
mod lines;
mod models;

/// The round the tests click on.
const ROUND: u32 = 4;

/// A fact `id` with `args`.
pub fn fact(id: &str, args: Vec<FactArg>) -> Fact {
    Fact {
        id: id.to_owned(),
        args,
        warn: false,
    }
}

/// A count `name` of `n`.
pub fn count(name: &str, n: u32) -> FactArg {
    FactArg {
        name: name.to_owned(),
        value: FactValue::Count(n),
    }
}

/// Money `name` of `h` hundredths of a dollar.
pub fn usd(name: &str, h: u32) -> FactArg {
    FactArg {
        name: name.to_owned(),
        value: FactValue::Money {
            amount: Hundredths(h),
            currency: "USD".to_owned(),
        },
    }
}

/// Groq, by its own name.
pub fn groq() -> ServiceName {
    ServiceName::Named("Groq".to_owned())
}

/// Groq's free allowance: 8 hours a day, 20 requests a minute.
pub fn daily() -> Fact {
    fact("daily", vec![count("hours", 8), count("per_min", 20)])
}

/// The Cloud page, `on` or off, with Groq ready and ticked and the Custom row off with the training warning.
fn cloud(on: bool, acts: &[CloudButton]) -> VoiceSheet {
    let groq = ServiceRow {
        name: groq(),
        mark: ServiceMark::Ready,
        untested: true,
        free: Some(daily()),
        warn: None,
        tick: true,
    };
    let custom = ServiceRow {
        name: ServiceName::Custom,
        mark: ServiceMark::Off,
        untested: false,
        free: None,
        warn: Some("trains".to_owned()),
        tick: false,
    };
    VoiceSheet::Cloud(CloudSheet {
        on,
        rows: vec![groq, custom],
        acts: acts.to_vec(),
    })
}

/// The ids and words of `marks`, a JSON list.
pub fn marks(marks: &Value) -> Vec<(String, String)> {
    let all = marks.as_array().cloned().unwrap_or_default();
    let text = |m: &Value, k: &str| m[k].as_str().unwrap_or_default().to_owned();
    all.iter()
        .map(|m| (text(m, "id"), text(m, "text")))
        .collect()
}

/// `(id, text)` pairs as `marks` lists them.
pub fn pairs(want: &[(&str, &str)]) -> Vec<(String, String)> {
    want.iter()
        .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
        .collect()
}

#[test]
fn each_cloud_click_sends_its_click() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(
        &rig,
        page(ROUND, VoiceBar::Hidden, Some(panel(cloud(true, &[])))),
    );
    let mut cases = vec![
        (CLOUD, "", 1, VoiceClick::Cloud(true)),
        (CLOUD, "", 0, VoiceClick::Cloud(false)),
        (KEEP, "", 1, VoiceClick::KeepNothing(true)),
        (SIDE, "cloud", 0, VoiceClick::Side(ModelSide::Cloud)),
        (SIDE, "local", 0, VoiceClick::Side(ModelSide::Local)),
        (LOCAL, "local", 0, VoiceClick::Local),
    ];
    let ids = [
        "on", "off", "facts", "lines", "paste", "test", "remove", "link",
    ];
    for (id, b) in ids.into_iter().zip(CloudButton::ALL) {
        cases.push((SERVICE, id, 0, VoiceClick::Service(b)));
    }
    for (kind, id, arg, click) in cases {
        rig.hub.voice_act(kind, id, arg, i64::from(ROUND));
        let sent: Vec<VoiceClick> = heard.sent.acts().into_iter().map(|a| a.click).collect();
        assert_eq!(sent, [click], "{kind} {id}");
    }
    for (kind, id, arg) in [(SERVICE, "nope", 0), (SIDE, "", 0), (KEEP, "", 2)] {
        rig.hub.voice_act(kind, id, arg, i64::from(ROUND));
        assert!(heard.sent.acts().is_empty(), "{kind} {id} {arg}");
    }
}

/// The Cloud page on, with Facts, Key and model and Switch off acting.
fn cloud_json() -> Value {
    let rig = Rig::new();
    let heard = heard(&rig);
    let acts = [CloudButton::Facts, CloudButton::Lines, CloudButton::Off];
    panel_json(&rig, &heard, panel(cloud(true, &acts)))
}

#[test]
fn the_cloud_page_shows_its_switch_and_its_footer() {
    let p = cloud_json();
    assert_eq!(
        (p["page"].as_str(), p["sheet"]["is"].as_str()),
        (Some("cloud"), Some("cloud"))
    );
    assert_eq!(p["title"], "Cloud services");
    assert_eq!(p["sub"], "Your own key · nothing leaves before Stop");
    let sheet = &p["sheet"];
    assert_eq!(sheet["switch"]["label"], "Use cloud services");
    assert_eq!(
        (
            sheet["switch"]["on"].as_bool(),
            sheet["switch"]["value"].as_str()
        ),
        (Some(true), Some("On"))
    );
    assert_eq!(sheet["off"], Value::Null);
    let foot = [
        button(SERVICE, "facts", "Facts", true, "plain"),
        button(SERVICE, "lines", "Key and model", true, "plain"),
        button(SERVICE, "off", "Switch off", true, "plain"),
        button(SERVICE, "on", "Switch on", false, "primary"),
    ];
    assert_eq!(buttons(&p["foot"]), foot);
}

#[test]
fn the_cloud_page_lists_each_service_with_its_free_allowance_and_marks() {
    let p = cloud_json();
    let sheet = &p["sheet"];
    let groq = &sheet["rows"][0];
    assert_eq!(groq["name"], "Groq");
    assert_eq!(groq["line"], "Free: 8 hours a day, 20 requests a minute");
    assert_eq!(groq["tag"], "Untested");
    assert_eq!(marks(&groq["marks"]), pairs(&[("on", "Ready")]));
    assert_eq!(groq["tick"], true);
    let custom = &sheet["rows"][1];
    assert_eq!(custom["name"], "Custom address");
    assert_eq!(
        custom["line"],
        "An HTTPS address, a model and a key you type"
    );
    assert_eq!(custom["tag"], Value::Null);
    assert_eq!(
        marks(&custom["marks"]),
        pairs(&[("warn", "Trains by default"), ("off", "Off")])
    );
}

#[test]
fn cloud_services_off_say_so_beside_the_switch() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let p = panel_json(&rig, &heard, panel(cloud(false, &[])));
    let sheet = &p["sheet"];
    assert_eq!(
        (
            sheet["switch"]["on"].as_bool(),
            sheet["switch"]["value"].as_str()
        ),
        (Some(false), Some("Off"))
    );
    assert_eq!(sheet["switch"]["kind"], CLOUD);
    let off = "Cloud services are off: none is offered and nothing is sent.";
    assert_eq!(sheet["off"], off);
}

/// The home list with cloud services `cloud`, its first page.
fn home_with(cloud: Option<u16>) -> VoiceSheet {
    VoiceSheet::Home(VoiceHome {
        model: None,
        device: DeviceLabel::Cpu,
        compared: None,
        history: 0,
        changed: 0,
        cloud,
        tabs: VoiceTab::ALL[..2].to_vec(),
    })
}

#[test]
fn the_home_lists_its_shown_pages_and_cloud_says_off_or_how_many_are_ready() {
    let rig = Rig::new();
    let heard = heard(&rig);
    for (cloud, value) in [
        (None, "Off"),
        (Some(0), "On · No service ready"),
        (Some(1), "On · 1 service ready"),
        (Some(2), "On · 2 services ready"),
    ] {
        let p = panel_json(
            &rig,
            &heard,
            acting(home_with(cloud), &[PanelButton::Close]),
        );
        let rows = p["sheet"]["rows"].as_array().cloned().unwrap_or_default();
        let tabs: Vec<&str> = rows.iter().filter_map(|r| r["tab"].as_str()).collect();
        assert_eq!(tabs, ["models", "cloud"]);
        assert_eq!(rows[1]["label"], "Cloud services");
        assert_eq!(rows[1]["value"], value);
    }
}

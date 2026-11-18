//! A service's card through the hub: its facts filled from the services table's row with the least billed (carry 3),
//! a name from the service's pages kept as its own part (carry 2), the day read, its warning tone once old (edge 15),
//! the Untested line (edge 18), the warning band, and Confirm or the day it was switched on.

use super::super::{button, buttons, heard, panel, panel_json};
use super::{count, daily, fact, groq, usd};
use crate::Rig;
use kx_module_api::ports::{
    CloudCard, Date, FactArg, FactValue, ServiceName, UsePrice, VoiceSheet,
};
use kx_ui::voice_acts::{CONFIRM, SERVICE};
use serde_json::Value;

/// A day for the cards.
pub fn day(d: u8) -> Date {
    Date::new(2026, 11, d).unwrap()
}

/// Groq's card as the services table fills it, read on the 17th, not yet switched on.
pub fn groq_card() -> CloudCard {
    let switch = FactArg {
        name: "switch".to_owned(),
        value: FactValue::Text("Settings → Data Controls".to_owned()),
    };
    let price = |low, high| fact("free-or", vec![usd("low", low), usd("high", high)]);
    CloudCard {
        service: groq(),
        place: fact("us", Vec::new()),
        trains: fact("never", Vec::new()),
        keeps: fact("logs", vec![count("days", 30), switch]),
        free: daily(),
        light: UsePrice {
            hours: 15,
            price: price(60, 167),
        },
        heavy: UsePrice {
            hours: 60,
            price: price(240, 666),
        },
        bills: Some(fact("least", vec![count("seconds", 10)])),
        read: day(17),
        old: false,
        site: Some("groq.com".to_owned()),
        warn: None,
        untested: true,
        on: None,
    }
}

/// The panel QML reads for `card`.
fn card_json(card: CloudCard) -> Value {
    let rig = Rig::new();
    let heard = heard(&rig);
    panel_json(&rig, &heard, panel(VoiceSheet::Card(Box::new(card))))
}

/// The `(id, label, sentence, warn)` of each of `p`'s facts.
fn facts(p: &Value) -> Vec<(String, String, String, bool)> {
    let all = p["sheet"]["facts"].as_array().cloned().unwrap_or_default();
    let text = |v: &Value| v.as_str().unwrap_or_default().to_owned();
    all.iter()
        .map(|f| {
            let warn = f["warn"].as_bool().unwrap_or_default();
            (
                text(&f["id"]),
                text(&f["label"]),
                text(&f["value"]["text"]),
                warn,
            )
        })
        .collect()
}

/// Groq's facts as its card writes them: id, label and sentence.
const GROQ_FACTS: [(&str, &str, &str); 7] = [
    ("where", "Where your voice goes", "United States"),
    ("trains", "Training", "Not used for training"),
    (
        "keeps",
        "What is kept",
        "Nothing by default, but checks may keep requests up to 30 days; “Settings → Data Controls” turns that off",
    ),
    ("free", "Free use", "8 hours a day, 20 requests a minute"),
    (
        "light",
        "Light use · 15 hours a month",
        "Free within the free use, else $0.60 to $1.67 by model",
    ),
    (
        "heavy",
        "Heavy use · 60 hours a month",
        "Free within the free use, else $2.40 to $6.66 by model",
    ),
    (
        "bills",
        "Least billed",
        "At least 10 seconds a request, so short pieces cost more",
    ),
];

#[test]
fn a_card_fills_its_facts_from_the_row_with_the_least_billed() {
    let p = card_json(groq_card());
    assert_eq!(
        (p["page"].as_str(), p["sheet"]["is"].as_str()),
        (Some("card"), Some("card"))
    );
    assert_eq!(
        (p["title"].as_str(), p["sub"].as_str()),
        (Some("Groq"), Some("Read this before you switch it on"))
    );
    let want: Vec<_> = GROQ_FACTS
        .iter()
        .map(|(a, b, c)| ((*a).to_owned(), (*b).to_owned(), (*c).to_owned(), false))
        .collect();
    assert_eq!(facts(&p), want);
    let parts = &p["sheet"]["facts"][2]["value"]["parts"];
    assert_eq!(parts[1]["text"], "Settings → Data Controls");
    assert_eq!(
        (parts[1]["own"].as_bool(), parts[0]["own"].as_bool()),
        (Some(true), Some(false))
    );
}

#[test]
fn a_card_asks_with_the_day_read_its_link_and_untested() {
    let p = card_json(groq_card());
    let read = &p["sheet"]["read"];
    assert_eq!(read["text"], "Facts read on 17/11/2026.");
    assert_eq!(read["old"], Value::Null);
    let link = button(SERVICE, "link", "Open groq.com", true, "plain");
    assert_eq!(buttons(&Value::Array(vec![read["link"].clone()])), [link]);
    assert_eq!(
        p["sheet"]["untested"], "Untested: not yet checked with a real key.",
        "edge 18"
    );
    assert_eq!(p["sheet"]["warn"], Value::Null);
    assert_eq!(
        p["line"],
        "Your voice goes to Groq only after Stop, with your own key."
    );
    let foot = [
        button(CONFIRM, "", "Cancel", true, "plain"),
        button(CONFIRM, "", "Confirm", true, "primary"),
    ];
    assert_eq!(buttons(&p["foot"]), foot);
}

#[test]
fn old_facts_take_the_warning_tone_and_a_card_switched_on_names_its_day() {
    let mut card = groq_card();
    (card.old, card.untested, card.on, card.bills) = (true, false, Some(day(12)), None);
    let p = card_json(card);
    let old = "They may be out of date; check the service’s page.";
    assert_eq!(p["sheet"]["read"]["old"], old, "edge 15");
    assert_eq!(p["sub"], "Switched on 12/11/2026");
    assert_eq!(p["sheet"]["untested"], Value::Null);
    assert_eq!(facts(&p).len(), 6, "no least billed");
    assert_eq!(
        (p["line"].as_null(), buttons(&p["foot"]).len()),
        (Some(()), 0)
    );
}

#[test]
fn a_warning_takes_its_band_and_the_custom_card_names_the_typed_address() {
    let mut card = groq_card();
    card.service = ServiceName::Custom;
    card.site = None;
    card.trains.warn = true;
    let switch = FactArg {
        name: "switch".to_owned(),
        value: FactValue::Text("Use my data for training".to_owned()),
    };
    card.warn = Some(fact("trains", vec![switch]));
    let p = card_json(card);
    assert_eq!(p["title"], "Custom address");
    assert_eq!(p["sheet"]["read"]["link"], Value::Null);
    assert!(facts(&p)[1].3, "the training fact takes the warning tone");
    let warn = &p["sheet"]["warn"];
    let text = "Trains its models on your voice by default. Before you confirm, turn off “Use my data for training” in your account.";
    assert_eq!(warn["text"], text);
    assert_eq!(warn["parts"][1]["own"], true);
    let ask = "Your voice goes to the address you type only after Stop, with your own key.";
    assert_eq!(p["line"], ask);
}

//! A card's fact with a service's own words (ADR-0022, ADR-0047): the words stay a part of their own with no bidi
//! mark, and end the sentence after a colon with no quote mark, so each part reads whole in every language. The order
//! check reads each Arabic cloud line's Latin runs left to right, from `runs`.

use crate::{CONTROLS, all, lang};
use kx_module_api::ports::{Fact, FactArg, FactValue};
use kx_ui::I18n;
use kx_ui::i18n::keys::vars::SITE;
use kx_ui::i18n::keys::{
    FACT_KEEPS, FACT_WARN, VOICE_ADDRESS_HINT, VOICE_ADDRESS_HINT_OWN, VOICE_CUSTOM_LINE,
    VOICE_KEY_LINK,
};
use serde_json::{Value, json};

/// The account switch's name, as Groq's page words it.
const SWITCH: &str = "Settings → Data Controls";
/// The quote marks a sentence could put around a name.
const QUOTES: [char; 6] = ['“', '”', '‘', '«', '»', '"'];

/// The facts that name an account switch: what Groq keeps, and a training warning.
fn named() -> [(&'static str, Fact); 2] {
    let arg = |name: &str, value| FactArg {
        name: name.to_owned(),
        value,
    };
    let switch = arg("switch", FactValue::Text(SWITCH.to_owned()));
    let keeps = Fact {
        id: "logs".to_owned(),
        args: vec![arg("days", FactValue::Count(30)), switch.clone()],
        warn: false,
    };
    let warn = Fact {
        id: "trains".to_owned(),
        args: vec![switch],
        warn: true,
    };
    [(FACT_KEEPS, keeps), (FACT_WARN, warn)]
}

#[test]
fn a_facts_service_name_stays_its_own_part_with_no_bidi_mark() {
    for t in all() {
        for (group, fact) in named() {
            let said = t.voice_fact(group, &fact);
            let own: Vec<&str> = said
                .parts
                .iter()
                .filter(|p| p.own)
                .map(|p| p.text.as_str())
                .collect();
            assert_eq!(own, [SWITCH], "{} {group}", t.lang().tag);
            assert!(!said.text.contains(CONTROLS), "{}", said.text);
            assert!(said.parts.iter().all(|p| !p.text.trim().is_empty()));
        }
    }
    let [(group, keeps), _] = named();
    let ar = lang("ar").voice_fact(group, &keeps);
    assert!(ar.text.contains("30 يومًا"), "{}", ar.text);
}

#[test]
fn a_services_own_words_end_the_sentence_after_a_colon_with_no_quote_mark() {
    for t in all() {
        let tag = t.lang().tag;
        for (group, fact) in named() {
            let said = t.voice_fact(group, &fact);
            let [lead, own] = &said.parts[..] else {
                panic!("{tag} {group}: {:?}", said.parts);
            };
            assert!(own.own && !lead.own, "{tag} {group}: the name ends it");
            let colon = lead.text.trim_end().ends_with(':');
            assert!(colon, "{tag} {group}: {}", lead.text);
            assert!(
                !said.text.contains(&QUOTES[..]),
                "{tag} {group}: {}",
                said.text
            );
        }
    }
}

/// The cloud lines, `$https` filled, that put a web address or a product's name inside a sentence.
const LATIN_LINES: [&str; 5] = [
    VOICE_ADDRESS_HINT,
    VOICE_ADDRESS_HINT_OWN,
    VOICE_CUSTOM_LINE,
    "cloud.address-https",
    "cloud.address-host",
];
/// Where keys are made, on Groq's key page.
const KEYS_SITE: &str = "console.groq.com";
/// The marks that end a sentence after a run, which read on its other side in a right-to-left line.
const ENDS: [char; 4] = ['.', ':', '!', '?'];

/// Each right-to-left cloud line that holds a Latin run or a number, with those runs, for the order check: a card's
/// facts give one entry per part, as the card lays each out.
pub(crate) fn runs(t: &I18n) -> Vec<Value> {
    if !t.lang().rtl {
        return Vec::new();
    }
    let mut texts: Vec<(String, String)> =
        LATIN_LINES.map(|k| (k.to_owned(), t.voice_https(k))).into();
    let site = t.voice_with(VOICE_KEY_LINK, &[(SITE, KEYS_SITE.to_owned())]);
    texts.push((VOICE_KEY_LINK.to_owned(), site));
    for (group, fact) in named() {
        let parts = t.voice_fact(group, &fact).parts;
        let each = parts.into_iter().enumerate();
        texts.extend(each.map(|(i, p)| (format!("{group}-{}-{i}", fact.id), p.text)));
    }
    let with_runs = texts
        .into_iter()
        .map(|(tag, text)| (tag, latin(&text), text));
    let shown = with_runs.filter(|(_, runs, _)| !runs.is_empty());
    shown
        .map(|(tag, runs, text)| json!({ "tag": tag, "text": text, "runs": runs }))
        .collect()
}

/// The runs of `text` outside the Arabic script that hold a Latin letter or a digit, each without the sentence's end.
fn latin(text: &str) -> Vec<String> {
    let arabic = |c: char| ('\u{600}'..='\u{6ff}').contains(&c);
    let runs = text.split(arabic).map(|r| r.trim().trim_end_matches(ENDS));
    let latin = runs.filter(|r| r.chars().any(|c| c.is_ascii_alphanumeric()));
    latin.map(str::to_owned).collect()
}

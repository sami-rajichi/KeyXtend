//! Gemini's row as its pages read 2026-11-17 give it (research file `[N…]` lines): its single JSON request to the
//! Interactions API with the key in `x-goog-api-key`, its key test on the models list, Gemini 3.5 Transcribe with one
//! BCP-47 code per keyboard language, pieces under its 20 MB request cap, and its card: training on the free allowance
//! with its warning, the paid price per minute for light and heavy use, and its rule for the EEA and for age.

use super::*;
use kx_module_api::ports::{Fact, KeyHeader};

/// Its warning's id, which says what the free allowance gives Google [N3][N4].
pub(crate) const FREE_USE: &str = "free-allowance";
/// The model it offers [N2].
const TRANSCRIBE: &str = "models/gemini-3.5-transcribe";
/// The codes it gets for the owner's three languages: Modern Standard Arabic's usual locale, then French and English.
const CODES: [(&str, &str); 3] = [("ar", "ar-SA"), ("fr", "fr-FR"), ("en", "en-US")];
/// The other BCP-47 codes its pages name [N1][N2], by keyboard language.
const NAMED: [(&str, &str); 3] = [("es", "es-ES"), ("hi", "hi-IN"), ("zh", "cmn-Hans-CN")];
/// Light and heavy use in cents a month: 15 and 60 hours at about $0.005 a minute on the paid tier [N3].
const PRICES: [u32; 2] = [450, 1800];

/// The shipped row.
fn gemini() -> &'static Service {
    shipped().get("gemini").unwrap()
}

/// `fact`'s text id and its values by name.
fn values(fact: &Fact) -> (&str, Vec<(&str, &FactValue)>) {
    let args = fact.args.iter().map(|a| (a.name.as_str(), &a.value));
    (fact.id.as_str(), args.collect())
}

#[test]
fn it_posts_one_json_request_to_its_own_address_and_tests_its_key_on_the_models_list() {
    let row = gemini();
    assert_eq!(row.name.as_deref(), Some("Gemini"));
    assert_eq!(row.shape, CloudShape::Gemini, "no form [N1]");
    let shown = |url: Option<HttpsUrl>| url.map(|u| u.expose().to_owned());
    let address = "https://generativelanguage.googleapis.com/v1beta/interactions";
    assert_eq!(shown(row.url("")).as_deref(), Some(address), "[N1]");
    let test = "https://generativelanguage.googleapis.com/v1beta/models";
    assert_eq!(shown(row.test_url("")).as_deref(), Some(test), "[N8]");
    assert_eq!(
        row.header,
        KeyHeader::Named("x-goog-api-key".into()),
        "[N1]"
    );
    assert!(
        row.keep.is_none() && !row.other,
        "no keep-nothing switch, no Other line"
    );
    assert!(row.offered && row.untested, "offered for the owner's check");
}

#[test]
fn it_offers_gemini_35_transcribe_with_one_bcp_47_code_per_language() {
    let ids: Vec<&str> = gemini().models.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(ids, [TRANSCRIBE], "[N2]");
    let model = &gemini().models[0];
    assert!(!model.unnamed && model.shape.is_none() && !model.keeps);
    for (lang, code) in CODES.into_iter().chain(NAMED) {
        assert_eq!(model.code(lang), Some(code), "{lang}");
    }
    assert_eq!(
        model.langs.len(),
        CODES.len() + NAMED.len(),
        "no code its pages do not name"
    );
}

#[test]
fn its_pieces_last_about_7_minutes_and_two_go_at_once() {
    let l = gemini().limits;
    let pieces = (l.max_mb, l.max_s);
    assert_eq!(
        pieces,
        (14, 420),
        "about 14 MB or 7 min raw, under 20 MB a request [N6]"
    );
    let rates = (l.per_min, l.at_once);
    assert_eq!(
        rates,
        (10, 2),
        "its rates are not published [N5]: guesses for the live check"
    );
}

#[test]
fn its_card_warns_what_the_free_allowance_gives_google() {
    let facts = &gemini().facts;
    let trains = (facts.trains.id.as_str(), facts.trains.warn);
    assert_eq!(trains, (FREE_USE, true), "[N3]");
    let warn = facts.warn.as_ref().expect("a warning [N3][N4]");
    assert_eq!(
        (warn.id.as_str(), warn.args.len(), warn.warn),
        (FREE_USE, 0, true)
    );
}

#[test]
fn its_facts_are_its_pages_with_the_paid_price_and_its_rule() {
    let facts = &gemini().facts;
    assert_eq!(facts.read, Date::new(2026, 11, 17).unwrap());
    let host = facts.source.as_ref().map(HttpsUrl::host);
    assert_eq!(host, Some("aistudio.google.com"));
    let ids = [&facts.place, &facts.keeps, &facts.free].map(|f| f.id.as_str());
    assert_eq!(ids, ["unstated", "limited", "no-card"], "[N3][N4][N5]");
    for (fact, cents) in [&facts.light, &facts.heavy].into_iter().zip(PRICES) {
        let price = FactValue::Money {
            amount: Hundredths(cents),
            currency: "USD".into(),
        };
        assert_eq!(values(fact), ("plain", vec![("price", &price)]), "{cents}");
    }
    assert!(facts.bills.is_none());
    let rule = facts
        .rule
        .as_ref()
        .expect("its rule for the EEA and age [N4]");
    assert_eq!(
        (rule.id.as_str(), rule.args.len(), rule.warn),
        ("eea-paid-18", 0, false)
    );
}

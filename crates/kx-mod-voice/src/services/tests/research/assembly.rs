//! `AssemblyAI`'s row as its pages read 2026-11-19 give it (research file `[A…]` lines): its Sync request to the
//! European servers with the key in `Authorization` and the model in its own header, no key test, Universal-3.6 Pro
//! with the Sync API's 32 language codes, a two-minute piece far under its request cap, and its card: the EU, no
//! training there and no warning, a keeping its pages do not state, $50 of credit and the plain hourly price.

use super::*;
use crate::services::KeyTest;
use kx_module_api::ports::KeyHeader;

/// The only Sync model [A1][A6].
const UNIVERSAL: &str = "universal-3-6-pro";
/// The Sync API's language codes [A1], each the keyboard language's own.
const CODES: [&str; 32] = [
    "af", "ar", "ca", "da", "de", "en", "es", "et", "fa", "fi", "fr", "gl", "he", "hi", "it", "ja",
    "ko", "mr", "nl", "nn", "no", "pt", "ro", "ru", "sv", "tr", "ur", "vi", "xh", "yue", "zh",
    "zu",
];
/// Light and heavy use in cents a month: 15 and 60 hours at $0.30 an hour [A3].
const PRICES: [u32; 2] = [450, 1800];
/// Its free credit in cents [A3].
const CREDIT: u32 = 5000;

/// The shipped row.
fn assemblyai() -> &'static Service {
    shipped().get("assemblyai").unwrap()
}

#[test]
fn it_posts_one_sync_request_to_the_european_servers_with_the_model_in_its_own_header() {
    let row = assemblyai();
    assert_eq!(row.name.as_deref(), Some("AssemblyAI"));
    let model_header = "X-AAI-Model".to_owned();
    assert_eq!(row.shape, CloudShape::Assembly { model_header }, "[A1]");
    let address = row.url("").map(|u| u.expose().to_owned());
    let eu = "https://sync.eu.assemblyai.com/v1/transcribe";
    assert_eq!(address.as_deref(), Some(eu), "its EU host [A1][A5]");
    assert_eq!(row.header, KeyHeader::Bearer, "Authorization [A1]");
    assert_eq!(row.test, KeyTest::None, "no call without sound [A1]");
    assert!(row.keep.is_none() && !row.other, "no switch, no Other line");
    assert!(row.offered && row.untested, "offered for the owner's check");
}

#[test]
fn it_offers_universal_36_pro_with_the_32_codes_of_its_sync_api() {
    let ids: Vec<&str> = assemblyai().models.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(ids, [UNIVERSAL], "[A1][A6]");
    let model = &assemblyai().models[0];
    assert_eq!(model.name, "Universal-3.6 Pro");
    assert!(!model.unnamed && model.shape.is_none() && !model.keeps);
    for code in CODES {
        assert_eq!(model.code(code), Some(code), "{code}");
    }
    assert_eq!(model.code("nb"), Some("no"), "Bokmål goes as Norwegian");
    let more: Vec<&str> = model
        .langs
        .keys()
        .map(String::as_str)
        .filter(|l| !CODES.contains(l))
        .collect();
    assert_eq!(more, ["nb"], "no code its pages do not name");
}

#[test]
fn its_pieces_last_two_minutes_under_its_cap_and_its_rates_are_guesses() {
    let l = assemblyai().limits;
    assert_eq!(
        l.max_s, 120,
        "the live limit, far within the 30 s deadline's reach [A1][A2]"
    );
    let rates = (l.per_min, l.at_once);
    assert_eq!(
        rates,
        (20, 2),
        "not stated for Sync [A2]: guesses for the live check"
    );
}

#[test]
fn its_card_is_the_eu_with_no_training_or_warning_its_credit_and_plain_price() {
    let facts = &assemblyai().facts;
    assert_eq!(facts.read, Date::new(2026, 11, 19).unwrap());
    let host = facts.source.as_ref().map(HttpsUrl::host);
    assert_eq!(host, Some("www.assemblyai.com"), "its pricing page [A3]");
    let ids = [&facts.place, &facts.trains, &facts.keeps].map(|f| f.id.as_str());
    assert_eq!(ids, ["eu", "never", "unstated"], "[A5]");
    assert!(
        !facts.trains.warn && facts.warn.is_none(),
        "nothing to warn of"
    );
    let credit = usd(CREDIT);
    assert_eq!(
        values(&facts.free),
        ("credit", vec![("credit", &credit)]),
        "[A3][A4]"
    );
    for (fact, cents) in [&facts.light, &facts.heavy].into_iter().zip(PRICES) {
        let price = usd(cents);
        assert_eq!(values(fact), ("plain", vec![("price", &price)]), "{cents}");
    }
    assert!(facts.bills.is_none() && facts.rule.is_none());
}

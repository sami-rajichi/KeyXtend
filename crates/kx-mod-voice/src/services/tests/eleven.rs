//! `ElevenLabs`' row as its pages read 2026-11-17 give it (research file `[E…]` lines): its own form at its own address
//! with the key in `xi-api-key`, its key test on the subscription, Scribe v3 and v2 but never the deprecated v1, two
//! requests at once, and its card: the US, training by default with its switch named last, history kept, the lower
//! free allowance, its hourly price for light and heavy use, and the free plan's rule.

use super::*;
use kx_module_api::ports::{Fact, FactArg, KeyHeader};

/// Its price an hour in cents, on every plan [E1].
const HOURLY: u32 = 20;
/// The switch that stops training, in its own words [E5][E7].
const SWITCH: &str =
    "Profile menu → Terms and privacy → Data use → Improve the models for everyone";

/// The shipped row.
fn eleven() -> &'static Service {
    shipped().get("elevenlabs").unwrap()
}

/// `fact`'s text id and its values by name.
fn values(fact: &Fact) -> (&str, Vec<(&str, &FactValue)>) {
    let args = fact.args.iter().map(|a| (a.name.as_str(), &a.value));
    (fact.id.as_str(), args.collect())
}

/// An amount of dollars in hundredths.
fn usd(cents: u32) -> FactValue {
    FactValue::Money {
        amount: Hundredths(cents),
        currency: "USD".into(),
    }
}

#[test]
fn it_posts_its_own_form_to_its_own_address_and_tests_its_key_on_the_subscription() {
    let row = eleven();
    assert_eq!(row.name.as_deref(), Some("ElevenLabs"));
    assert_eq!(row.shape, CloudShape::Eleven);
    let shown = |url: Option<HttpsUrl>| url.map(|u| u.expose().to_owned());
    let address = "https://api.elevenlabs.io/v1/speech-to-text";
    assert_eq!(shown(row.url("")).as_deref(), Some(address), "[E3]");
    let test = "https://api.elevenlabs.io/v1/user/subscription";
    assert_eq!(
        shown(row.test_url("")).as_deref(),
        Some(test),
        "[E9]: no sound"
    );
    assert_eq!(row.header, KeyHeader::Named("xi-api-key".into()), "[E3]");
    assert!(
        row.keep.is_none() && !row.other,
        "no keep-nothing switch, no Other line"
    );
    assert!(
        row.offered && row.untested,
        "offered for the owner's free-plan check"
    );
}

#[test]
fn it_offers_scribe_v3_then_v2_never_the_deprecated_v1() {
    let ids: Vec<&str> = eleven().models.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(ids, ["scribe_v3", "scribe_v2"], "[E4]");
    for m in &eleven().models {
        assert!(!m.unnamed && m.shape.is_none() && !m.keeps, "{}", m.id);
        let codes = ["en", "fr", "ar"].map(|l| m.code(l));
        assert_eq!(
            codes,
            [Some("en"), Some("fr"), Some("ar")],
            "{}: ISO 639-1 [E3]",
            m.id
        );
        assert_eq!(
            m.langs.len(),
            3,
            "{}: en-fr-ar until task 24 reads its list",
            m.id
        );
    }
}

#[test]
fn it_sends_two_requests_at_once() {
    let limits = eleven().limits;
    assert_eq!(limits.at_once, 2, "the free plan's concurrency [E8]");
    assert_eq!((limits.max_mb, limits.max_s), (23, 600));
}

#[test]
fn its_card_warns_that_it_trains_by_default_and_names_its_switch_last() {
    let facts = &eleven().facts;
    assert_eq!(
        (facts.trains.id.as_str(), facts.trains.warn),
        ("default", true)
    );
    let warn = facts.warn.as_ref().expect("a warning [E5][E7]");
    let switch = FactArg {
        name: "switch".into(),
        value: FactValue::Text(SWITCH.into()),
    };
    assert_eq!(
        (warn.id.as_str(), &warn.args[..], warn.warn),
        ("trains", &[switch][..], true)
    );
}

#[test]
fn its_facts_are_its_pages_with_the_lower_free_allowance_and_the_free_plans_rule() {
    let facts = &eleven().facts;
    assert_eq!(facts.read, Date::new(2026, 11, 17).unwrap());
    assert_eq!(
        facts.source.as_ref().map(HttpsUrl::host),
        Some("elevenlabs.io")
    );
    let ids = (facts.place.id.as_str(), facts.keeps.id.as_str());
    assert_eq!(ids, ("us", "history"), "[E3][E5][E7]");
    // [E1] gives 4 h 30 min a month, [E2] about 30 min: the card shows the lower one.
    let free = values(&facts.free);
    assert_eq!(
        free,
        ("minutes-month", vec![("minutes", &FactValue::Count(30))])
    );
    for (fact, hours) in [
        (&facts.light, shipped().light_hours()),
        (&facts.heavy, shipped().heavy_hours()),
    ] {
        let price = usd(u32::from(hours) * HOURLY);
        assert_eq!(
            values(fact),
            ("plain", vec![("price", &price)]),
            "{hours} h"
        );
    }
    assert!(facts.bills.is_none());
    let rule = facts.rule.as_ref().expect("its free plan's rule [E6]");
    assert_eq!(
        (rule.id.as_str(), rule.args.len(), rule.warn),
        ("non-commercial", 0, false)
    );
}

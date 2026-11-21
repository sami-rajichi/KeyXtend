//! Groq's row as its pages read 2026-11-21 give it (research file `[G…]` lines): its facts as numbers and flags with
//! their day and source, models that know English, French and Arabic, no training, its one host and its least bill.

use super::*;

#[test]
fn groqs_facts_are_numbers_and_flags_with_their_day_and_source() {
    let groq = shipped().get("groq").unwrap();
    let facts = &groq.facts;
    assert_eq!(facts.read, Date::new(2026, 11, 21).unwrap());
    let spoken = groq
        .models
        .iter()
        .all(|m| ["en", "fr", "ar"].iter().all(|l| m.code(l).is_some()));
    assert!(spoken, "Groq's models know the owner's three languages");
    assert_eq!(
        facts.source.as_ref().map(HttpsUrl::host),
        Some("console.groq.com")
    );
    assert_eq!(
        (facts.place.id.as_str(), facts.trains.id.as_str()),
        ("us", "never")
    );
    assert!(
        facts.warn.is_none() && !facts.trains.warn,
        "Groq trains on nothing"
    );
    let counts = [
        ("hours", &FactValue::Count(8)),
        ("per_min", &FactValue::Count(20)),
    ];
    assert_eq!(values(&facts.free), ("daily", counts.to_vec()));
    let heavy: Vec<&FactValue> = facts.heavy.args.iter().map(|a| &a.value).collect();
    assert_eq!(heavy, [&usd(666), &usd(240)], "high then low, by name");
}

#[test]
fn groq_reaches_its_one_host_and_bills_ten_seconds_at_least() {
    let groq = shipped().get("groq").unwrap();
    let host = groq.url("").map(|u| u.host().to_owned());
    assert_eq!(host.as_deref(), Some("api.groq.com"));
    let ten = [("seconds", &FactValue::Count(10))];
    let bills = groq.facts.bills.as_ref().map(values);
    assert_eq!(bills, Some(("least", ten.to_vec())));
}

//! Each read of `[voice]` drops, alone and named, a service entry the table no longer takes or offers and a cloud pick
//! that no longer names a service the user switched on with a model for its language, so a later table or switch never
//! brings one back by itself.

use super::*;
use crate::cloud::pick_text;
use crate::services::tests::NOT_OFFERED;
use crate::services::{Service, table};
use kx_module_api::ports::Date;

/// The day the entries below were confirmed.
fn day() -> Date {
    Date::new(2026, 11, 17).unwrap()
}

/// The defaults with cloud on.
fn on() -> VoiceSettings {
    let mut s = VoiceSettings::from_defaults().unwrap();
    s.cloud = true;
    s
}

/// An entry at the first terms number, which no row's is below, with `edit` applied.
fn entry(edit: impl FnOnce(&mut ServiceOn)) -> ServiceOn {
    let mut on = ServiceOn::new(day(), 1);
    edit(&mut on);
    on
}

/// An entry at the first terms number with `address` typed.
fn at(address: &str) -> ServiceOn {
    entry(|on| address.clone_into(&mut on.address))
}

/// The shipped row `id`.
fn row(id: &str) -> &'static Service {
    table().unwrap().get(id).unwrap()
}

#[test]
fn a_bad_service_entry_is_dropped_alone_and_named() {
    let at_ten = "2026-11-17T10:00:00".parse().unwrap();
    let own_host = format!("https://{}/v1", row("groq").url("").unwrap().host());
    let bad = [
        ("ovhcloud", entry(|_| {})),
        ("groq", entry(|on| on.terms = 0)),
        ("groq", entry(|on| on.terms = row("groq").terms + 1)),
        ("groq", entry(|on| on.on = at_ten)),
        ("groq", at(&own_host)),
        ("custom", at("http://stt.example.com/v1")),
        ("custom", at("https://192.168.1.20/v1")),
        ("custom", entry(|on| on.model = "a//b".into())),
    ];
    for (id, one) in bad {
        let mut s = on();
        let other = if id == "groq" { "custom" } else { "groq" };
        s.services.insert(other.into(), entry(|_| {}));
        s.services.insert(id.into(), one);
        assert_eq!(s.drop_bad(), [format!("services.{id}")], "{id}");
        let kept: Vec<_> = s.services.keys().collect();
        assert_eq!(kept, [other], "{id}: the other stays");
    }
}

#[test]
fn an_odd_id_names_the_whole_key_and_a_second_read_drops_nothing() {
    let mut s = on();
    s.services.insert(
        "groq".into(),
        entry(|on| on.model = "whisper-large-v4".into()),
    );
    s.services.insert(
        "custom".into(),
        at("https://stt.example.com/v1/audio/transcriptions"),
    );
    s.services.insert("Odd Id".into(), entry(|_| {}));
    assert_eq!(
        s.drop_bad(),
        ["services"],
        "an id that is no name names the whole key"
    );
    assert_eq!(s.services.keys().collect::<Vec<_>>(), ["custom", "groq"]);
    assert!(s.drop_bad().is_empty(), "a second read drops nothing");
}

#[test]
fn a_cloud_pick_stays_only_while_it_names_a_service_on_and_a_model_for_its_language() {
    let mut s = on();
    s.services.insert("groq".into(), entry(|_| {}));
    let mut custom = at("https://stt.example.com/v1/audio/transcriptions");
    custom.model = "my-model".into();
    s.services.insert("custom".into(), custom);
    let (groq, hidden) = (&row("groq").models, &row("openai").models[0].id);
    let (first, last) = (&groq[0].id, &groq[groq.len() - 1].id);
    let picks = [
        ("fr", pick_text("groq", first)),
        ("ar", pick_text("groq", last)),
        ("en", "custom:my-model".into()),
        ("nl", "canary-v2-q8".into()),
        ("de", pick_text("groq", &format!("{first}-gone"))),
        ("es", pick_text("openai", hidden)),
        ("it", "custom:other-model".into()),
        ("pt", "groq:".into()),
        ("tlh", pick_text("groq", first)),
    ];
    s.picks = picks.map(|(l, p)| (l.to_owned(), p)).into();
    let named = ["picks.de", "picks.es", "picks.it", "picks.pt", "picks.tlh"];
    assert_eq!(s.drop_bad(), named);
    assert_eq!(s.picks.keys().collect::<Vec<_>>(), ["ar", "en", "fr", "nl"]);
    s.cloud = false;
    let cloud = ["picks.ar", "picks.en", "picks.fr"];
    assert_eq!(s.drop_bad(), cloud, "edge 21: cloud off drops them");
    assert_eq!(
        s.picks.keys().collect::<Vec<_>>(),
        ["nl"],
        "a local pick stays"
    );
    assert_eq!(
        s.services.len(),
        2,
        "the confirmations stay for the next time cloud is on"
    );
}

#[test]
fn a_saved_entry_of_a_row_not_offered_drops_alone_and_its_picks_go_back_to_local() {
    let models = crate::models::table().unwrap();
    for id in NOT_OFFERED {
        let mut s = on();
        s.services.insert("groq".into(), entry(|_| {}));
        s.services.insert(id.into(), entry(|_| {}));
        let model = row(id)
            .models
            .iter()
            .find(|m| m.code("fr").is_some())
            .unwrap();
        let picks = [
            ("fr", pick_text(id, &model.id)),
            ("en", pick_text("groq", &row("groq").models[0].id)),
            ("nl", "canary-v2-q8".into()),
        ];
        s.picks = picks.map(|(l, p)| (l.to_owned(), p)).into();
        let named = ["picks.fr".to_owned(), format!("services.{id}")];
        assert_eq!(s.drop_bad(), named, "{id}: no crash, each named");
        assert_eq!(s.services.keys().collect::<Vec<_>>(), ["groq"], "{id}");
        assert_eq!(s.picks.keys().collect::<Vec<_>>(), ["en", "nl"], "{id}");
        let local = models.default_for("fr").map(crate::models::Pick::Row);
        assert_eq!(
            models.used("fr", &s.picks),
            local,
            "{id}: French goes local"
        );
        assert!(s.drop_bad().is_empty(), "{id}: a second read drops nothing");
    }
}

#[test]
fn a_listed_pick_of_a_service_merely_off_stays_unnamed_for_that_services_confirm() {
    let mut s = on();
    let pick = pick_text("groq", &row("groq").models[0].id);
    s.picks.insert("fr".into(), pick.clone());
    assert!(s.drop_bad().is_empty(), "owner, 2026-11-21: it waits");
    assert!(s.picks.contains_key("fr"));
    s.services.insert("groq".into(), entry(|on| on.terms = 0));
    let stale = ["picks.fr", "services.groq"];
    assert_eq!(s.drop_bad(), stale, "a stale entry takes its picks along");
    s.picks.insert("fr".into(), pick);
    s.cloud = false;
    assert_eq!(s.drop_bad(), ["picks.fr"], "edge 21: cloud off drops it");
}

#[test]
fn a_typed_rows_pick_goes_while_no_address_is_typed() {
    let mut s = on();
    s.services
        .insert("custom".into(), entry(|on| on.model = "my-model".into()));
    s.picks.insert("en".into(), "custom:my-model".into());
    assert_eq!(s.drop_bad(), ["picks.en"], "edge 13: nowhere to send it");
    assert!(s.services.contains_key("custom"), "the confirmation stays");
}

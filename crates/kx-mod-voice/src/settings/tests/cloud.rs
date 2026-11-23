//! The cloud settings (ADR-0047): off to start with no service on, the wait and tries in the frames' own ranges, the
//! facts' age in its own, and an entry with its day, terms number, typed lines, keep-nothing switch and the terms
//! number its warning was seen at.

use super::*;
use kx_module_api::ports::{CloudAsk, Date};

/// A `services` table holding one entry for `id` with `fields`.
fn services(id: &str, fields: &str) -> toml::Value {
    let table: toml::Table = toml::from_str(&format!("{id} = {{ {fields} }}")).unwrap();
    toml::Value::Table(table)
}

#[test]
fn cloud_starts_off_with_no_service_and_the_plans_numbers() {
    let s = VoiceSettings::from_defaults().unwrap();
    assert!(
        !s.cloud && s.services.is_empty(),
        "nothing is sent until the user switches a service on"
    );
    assert_eq!((s.net_wait_s, s.net_tries, s.facts_days), (60, 2, 180));
}

#[test]
fn the_wait_and_tries_take_the_frames_ranges_and_the_age_its_own() {
    assert_eq!(
        (CloudAsk::WAIT_S, CloudAsk::TRIES, FACTS_DAYS),
        (5..=300, 1..=5, 30..=730)
    );
    let bounds = [
        ("net_wait_s", ends(&CloudAsk::WAIT_S)),
        ("net_tries", ends(&CloudAsk::TRIES)),
        ("facts_days", ends(&FACTS_DAYS)),
    ];
    for (key, (lo, hi)) in bounds {
        assert!(taken(&with(key, lo)) && taken(&with(key, hi)), "{key}");
        let (below, above) = (with(key, lo - 1), with(key, hi + 1));
        assert!(refused(&below, key) || unread(&below), "{key} = {}", lo - 1);
        assert!(refused(&above, key) || unread(&above), "{key} = {}", hi + 1);
    }
    assert!(unread(&with("cloud", "yes")));
}

#[test]
fn a_service_entry_reads_with_its_day_terms_typed_lines_and_keep_switch() {
    let t = with("services", services("groq", "on = 2026-11-17, terms = 1"));
    let s = parse_as::<VoiceSettings>(&t).unwrap();
    let on = &s.services["groq"];
    assert_eq!(
        (on.terms, on.address.as_str(), on.model.as_str()),
        (1, "", "")
    );
    assert_eq!((on.day(), on.keep), (Date::new(2026, 11, 17), KEEP_START));
    const {
        assert!(
            KEEP_START,
            "OpenRouter's keep-nothing switch is on to start"
        );
    };
    let typed = "on = 2026-11-17, terms = 2, address = \"https://stt.example.com/v1\", model = \"m\", keep = false";
    let t = with("services", services("custom", typed));
    let s = parse_as::<VoiceSettings>(&t).unwrap();
    let on = &s.services["custom"];
    assert_eq!((on.terms, on.model.as_str(), on.keep), (2, "m", false));
    for odd in [
        "terms = 1",
        "on = 2026-11-17",
        "on = \"2026-11-17\", terms = 1",
    ] {
        assert!(unread(&with("services", services("groq", odd))), "{odd}");
    }
}

#[test]
fn an_entry_starts_on_its_day_and_terms_and_prints_no_typed_line() {
    let day = Date::new(2026, 11, 17).unwrap();
    let mut on = ServiceOn::new(day, 3);
    assert_eq!((on.day(), on.terms, on.keep), (Some(day), 3, KEEP_START));
    assert!(on.address.is_empty() && on.model.is_empty());
    on.address = "https://stt.example.com/v1".into();
    on.model = "my-own-model".into();
    let shown = format!("{on:?}");
    assert!(
        !shown.contains("stt.example") && !shown.contains("my-own-model"),
        "{shown}"
    );
    assert!(shown.contains("terms: 3"), "{shown}");
    let mut s = VoiceSettings::from_defaults().unwrap();
    s.services.insert("custom".into(), on);
    let back = s.with("facts_days", 30.into()).unwrap();
    assert_eq!(
        back.services, s.services,
        "an entry round-trips through a panel write"
    );
}

#[test]
fn the_services_value_written_back_holds_a_plain_date() {
    let day = Date::new(2026, 11, 17).unwrap();
    let mut s = VoiceSettings::from_defaults().unwrap();
    assert_eq!(
        s.services_value(),
        Some(toml::Value::Table(toml::Table::new()))
    );
    s.services.insert("groq".into(), ServiceOn::new(day, 1));
    let value = s.services_value().unwrap();
    let on = &value["groq"]["on"];
    assert_eq!(
        on.as_datetime().map(ToString::to_string).as_deref(),
        Some("2026-11-17"),
        "{on:?}"
    );
    assert!(
        value["groq"].get("address").is_none(),
        "an empty typed line is left out"
    );
    let back = s.with("services", value).unwrap();
    assert_eq!(back.services, s.services);
}

#[test]
fn the_warning_seen_reads_back_and_is_left_out_until_shown() {
    let day = Date::new(2026, 11, 17).unwrap();
    let mut s = VoiceSettings::from_defaults().unwrap();
    s.services
        .insert("elevenlabs".into(), ServiceOn::new(day, 1));
    let value = s.services_value().unwrap();
    assert!(value["elevenlabs"].get("warned").is_none(), "never shown");
    s.services.get_mut("elevenlabs").unwrap().warned = 1;
    let value = s.services_value().unwrap();
    assert_eq!(value["elevenlabs"]["warned"].as_integer(), Some(1));
    let back = s.with("services", value).unwrap();
    assert_eq!(back.services["elevenlabs"].warned, 1);
}

#[test]
fn a_field_a_later_version_adds_keeps_the_entry_on_a_downgrade() {
    let later = "on = 2026-11-17, terms = 1, model = \"m\", later = 1";
    let t = with("services", services("groq", later));
    let s = parse_as::<VoiceSettings>(&t).unwrap();
    let on = &s.services["groq"];
    assert_eq!((on.terms, on.model.as_str()), (1, "m"), "the rest stays");
}

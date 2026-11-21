//! What a request to a row takes: a model's own form (`OpenAI`'s `languages[]`, ruling 2026-11-17), the keep-nothing
//! field while its switch is on, and an address and test only on the row's own host or the typed one (edge 13).

use super::*;
use kx_module_api::ports::{FormField, FormFields};

/// A form shape of `lang`, a list or not, with `fixed`.
fn form(lang: &str, list: bool, fixed: Vec<FormField>) -> CloudShape {
    CloudShape::Form(FormFields {
        lang: lang.into(),
        list,
        fixed,
        paywall: false,
        region: None,
    })
}

/// The address `typed` gives row `id` of `table`, as text.
fn url(table: &Services, id: &str, typed: &str) -> Option<String> {
    let row = table.get(id).unwrap();
    row.url(typed).map(|u| u.expose().to_owned())
}

#[test]
fn a_model_carries_its_own_form_fields_beside_the_rows() {
    let table = with_rows(OPENAI);
    let row = table.get("openai-test").unwrap();
    let gpt = row.model("gpt-transcribe");
    assert_eq!(
        row.shape_for(gpt, false),
        form("languages", true, Vec::new()),
        "languages[] as a list"
    );
    let whisper = row.model("whisper-1");
    assert_eq!(
        row.shape_for(whisper, false),
        form("language", false, Vec::new())
    );
    assert_eq!(
        row.shape_for(None, false),
        form("language", false, Vec::new()),
        "a typed model takes the row's"
    );
    assert_eq!(
        row.shape_for(gpt, true),
        row.shape_for(gpt, false),
        "no keep field, no switch"
    );
}

#[test]
fn a_rows_paywall_and_region_go_with_every_form_it_sends_a_models_own_too() {
    let rules =
        "shape = \"form\"\npaywall = true\nregion = \"unsupported_country_region_territory\"";
    let walled = OPENAI.replace("shape = \"form\"", rules);
    let table = with_rows(&walled);
    let row = table.get("openai-test").unwrap();
    for model in [row.model("gpt-transcribe"), row.model("whisper-1"), None] {
        let shape = row.shape_for(model, false);
        assert!(
            matches!(&shape, CloudShape::Form(f) if f.paywall && f.region.is_some()),
            "{shape:?}"
        );
    }
    let open = with_rows(OPENAI);
    let row = open.get("openai-test").unwrap();
    let shape = row.shape_for(row.model("gpt-transcribe"), false);
    assert!(
        matches!(&shape, CloudShape::Form(f) if !f.paywall && f.region.is_none()),
        "none by default"
    );
}

#[test]
fn the_keep_nothing_field_goes_last_only_while_its_switch_is_on() {
    let table = with_rows(ROUTER);
    let row = table.get("router-test").unwrap();
    let keep = FormField {
        name: "provider".into(),
        value: "{\"zdr\":true}".into(),
    };
    assert_eq!(row.keep.as_ref(), Some(&keep));
    let turbo = row.model("openai/whisper-large-v3-turbo");
    assert_eq!(
        row.shape_for(turbo, true),
        form("language", false, vec![keep])
    );
    assert_eq!(
        row.shape_for(turbo, false),
        form("language", false, Vec::new())
    );
    assert!(row.model("kept/model").is_some_and(|m| m.keeps));
}

#[test]
fn every_fixed_rows_address_and_test_stay_on_its_own_host() {
    for row in shipped().rows() {
        let Address::Fixed(own) = &row.address else {
            continue;
        };
        assert_eq!(
            row.url("https://evil.example/v1"),
            Some(own.clone()),
            "{}: a typed text is ignored",
            row.id
        );
        if let KeyTest::At(test) = &row.test {
            assert_eq!(test.host(), own.host(), "{}", row.id);
        }
    }
    let groq = shipped().get("groq").unwrap();
    let written = |key| Some(field_of(groq_line(key), key).trim_matches('"'));
    let host = groq.url("").map(|u| u.host().to_owned());
    assert_eq!(host.as_deref(), written("host"), "its row's host");
    let test = groq.test_url("").map(|u| u.expose().to_owned());
    assert_eq!(
        test.as_deref(),
        written("test"),
        "its row's test, as written"
    );
}

#[test]
fn customs_typed_address_is_https_on_a_host_name_only() {
    let good = "https://stt.example.com/v1/audio/transcriptions";
    assert_eq!(url(shipped(), "custom", good).as_deref(), Some(good));
    let bad = [
        "http://stt.example.com/v1/audio/transcriptions",
        "http://192.168.1.20",
        "https://192.168.1.20/v1/audio/transcriptions",
        "https://2130706433/v1",
        "https://user@stt.example.com/v1",
        "stt.example.com/v1",
        "",
    ];
    for text in bad {
        assert_eq!(url(shipped(), "custom", text), None, "{text:?}");
    }
    assert_eq!(
        shipped().get("custom").unwrap().test_url(good),
        None,
        "no test: the first dictation tests it"
    );
}

#[test]
fn a_typed_address_stays_under_its_rows_domain_and_its_test_on_that_host() {
    let table = with_rows(DOMAIN);
    let typed =
        "https://kx-test.speech.example.com/v1/transcriptions:transcribe?api-version=2025-10-15";
    assert_eq!(url(&table, "domain-test", typed).as_deref(), Some(typed));
    for bad in [
        "https://kx-test.speech.example.com.evil.example/x",
        "https://evil.example/v1",
        "https://speech.example.com/x",
        "http://kx-test.speech.example.com/x",
    ] {
        assert_eq!(url(&table, "domain-test", bad), None, "{bad}");
    }
    let row = table.get("domain-test").unwrap();
    let test = |typed: &str| row.test_url(typed).map(|u| u.expose().to_owned());
    let models = "https://kx-test.speech.example.com/v1/models";
    assert_eq!(test(typed).as_deref(), Some(models));
    let port = "https://kx.speech.example.com:8443/stt?x=1";
    assert_eq!(
        test(port).as_deref(),
        Some("https://kx.speech.example.com:8443/v1/models")
    );
    assert_eq!(test("https://evil.example/x"), None);
    assert!(row.model("base").is_some_and(|m| m.unnamed));
}

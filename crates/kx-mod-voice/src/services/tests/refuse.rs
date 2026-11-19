//! The validator refuses a broken row with its rule, so a bad edit fails here with a clear message instead of reaching
//! a request: every field the later rows use is covered.

use super::*;
use ServiceError as E;

/// Groq's address, its test, its form and its limits, which the edits below break.
const ADDRESS: &str = "address = \"https://api.groq.com/openai/v1/audio/transcriptions\"";
const TEST: &str = "test = \"https://api.groq.com/openai/v1/models\"";
const FORM: &str = "form = { lang = \"language\", list = false, fixed = [] }";
const LIMITS: &str = "cap_mb = 25, max_mb = 23, max_s = 600, per_min = 20, at_once = 4";

/// Asserts that one edit of the shipped table, `from` to `to`, is refused with `why`.
#[track_caller]
fn refused(from: &str, to: &str, why: ServiceError) {
    assert_eq!(row_err(from, to), Some(why), "{to}");
}

/// The form with `field` as its one fixed field.
fn fixed(field: &str) -> String {
    FORM.replace("fixed = []", &format!("fixed = [{field}]"))
}

#[test]
fn a_fixed_address_or_test_off_the_rows_own_https_host_is_refused() {
    refused(ADDRESS, &ADDRESS.replace("https://", "http://"), E::Address);
    refused(
        ADDRESS,
        &ADDRESS.replace("api.groq.com", "192.168.1.20"),
        E::Address,
    );
    refused(
        ADDRESS,
        &ADDRESS.replace("api.groq.com", "api.openai.com"),
        E::Host,
    );
    refused(TEST, &TEST.replace("api.groq.com", "evil.example"), E::Host);
    refused(TEST, &TEST.replace("https://", "http://"), E::Test);
    refused("host = \"api.groq.com\"", "host = \"groq.com\"", E::Host);
}

#[test]
fn a_typed_rows_test_must_be_a_path_and_its_domain_a_host_entry() {
    let typed = "address = \"typed\"";
    refused(typed, &format!("{typed}\ntest = \"models\""), E::Test);
    refused(typed, &format!("host = \"a b\"\n{typed}"), E::Host);
}

#[test]
fn only_one_row_may_go_unnamed_and_only_the_custom_one() {
    let domain = edits_of(
        &format!("{TEXT}{DOMAIN}"),
        &[("name = \"Domain Test\"\n", "")],
    );
    let unnamed = |id: &str| {
        Some(ServicesError::Row {
            id: id.into(),
            why: E::Name,
        })
    };
    assert_eq!(
        domain.err(),
        unnamed("domain-test"),
        "a typed row under a domain is named"
    );
    let custom = &TEXT[TEXT.find("[[services]]\nid = \"custom\"").unwrap()..];
    let second = custom.replace("id = \"custom\"", "id = \"custom-two\"");
    let twice = parse(&format!("{TEXT}{second}"));
    assert_eq!(twice.err(), unnamed("custom-two"), "a second Custom row");
}

#[test]
fn a_shape_or_form_that_breaks_a_request_is_refused() {
    let five = ["{ name = \"a\", value = \"1\" }"; 5].join(", ");
    refused("shape = \"form\"", "shape = \"json\"", E::Shape);
    let bare = format!("shape = \"form\"\n{FORM}");
    refused(&bare, "shape = \"eleven\"", E::Form);
    refused("shape = \"form\"", "shape = \"gemini\"", E::Form);
    refused(FORM, "", E::Form);
    refused(
        FORM,
        &FORM.replace("\"language\"", "\"lang uage\""),
        E::Form,
    );
    refused(FORM, &fixed(&five), E::Form);
    let keep = format!("{FORM}\nkeep = {{ name = \"provider\", value = \"a\\nb\" }}");
    refused(FORM, &keep, E::Form);
    refused("header = \"bearer\"", "header = \"x y\"", E::Header);
}

#[test]
fn a_row_of_the_dropped_azure_shape_is_refused_as_an_unknown_shape() {
    // ADR-0050 left three shapes, so a row written as an Azure one names no shape at all.
    let shape = format!("shape = \"form\"\n{FORM}");
    refused(&shape, "shape = \"azure\"", E::Shape);
}

#[test]
fn a_list_paywall_or_region_is_refused_on_elevenlabs_shape_and_an_odd_region_code_anywhere() {
    let form = "shape = \"form\"";
    let eleven = (form, "shape = \"eleven\"");
    assert!(edits(&[eleven]).is_ok(), "its form");
    let list = FORM.replace("list = false", "list = true");
    let listed = edits(&[eleven, (FORM, list.as_str())]);
    let why = matches!(listed, Err(ServicesError::Row { why: E::Form, .. }));
    assert!(why, "its language goes plain: {listed:?}");
    for rule in ["paywall = true", "region = \"unsupported_country\""] {
        refused(form, &format!("shape = \"eleven\"\n{rule}"), E::Form);
    }
    for code in ["unsupported country", ""] {
        refused(FORM, &format!("{FORM}\nregion = \"{code}\""), E::Form);
    }
}

#[test]
fn a_fixed_field_that_asks_for_timings_or_an_odd_reply_is_refused() {
    let verbose = "{ name = \"response_format\", value = \"verbose_json\" }";
    refused(FORM, &fixed(verbose), E::Reply);
    refused(
        FORM,
        &fixed("{ name = \"timestamp_granularities\", value = \"word\" }"),
        E::Reply,
    );
    for plain in [
        "{ name = \"response_format\", value = \"json\" }",
        "{ name = \"timestamps_granularity\", value = \"none\" }",
    ] {
        assert!(edits(&[(FORM, &fixed(plain))]).is_ok(), "{plain}");
    }
}

#[test]
fn a_name_keys_page_terms_or_limit_that_breaks_a_rule_is_refused() {
    let keys = "keys = \"https://console.groq.com/keys\"";
    refused("name = \"Groq\"", "name = \"\"", E::Name);
    refused("name = \"Groq\"", "name = \"Gr\u{202E}oq\"", E::Name);
    refused("name = \"Groq\"\n", "", E::Name);
    refused(keys, &keys.replace("https://", "http://"), E::Keys);
    refused(keys, "", E::Keys);
    refused("terms = 1", "terms = 0", E::Terms);
    refused(
        LIMITS,
        &LIMITS.replace("per_min = 20", "per_min = 0"),
        E::Limits,
    );
    refused(
        LIMITS,
        &LIMITS.replace("cap_mb = 25", "cap_mb = 0"),
        E::Limits,
    );
    refused(
        LIMITS,
        &LIMITS.replace("max_s = 600", "max_s = 601"),
        E::Limits,
    );
}

#[test]
fn too_many_models_or_none_without_an_other_line_is_refused() {
    let too_many = edits(&[("max_models = 10", "max_models = 1")]);
    let models = |id: &str| {
        Some(ServicesError::Row {
            id: id.into(),
            why: E::Models,
        })
    };
    assert_eq!(too_many.err(), models("groq"));
    let custom = "at_once = 2 }\nother = true";
    let none = edits(&[(custom, "at_once = 2 }\nother = false")]);
    assert_eq!(none.err(), models("custom"));
}

#[test]
fn a_bad_id_or_table_wide_value_is_refused() {
    let twice = edits(&[("id = \"custom\"", "id = \"groq\"")]);
    assert_eq!(twice.err(), Some(ServicesError::Id("groq".into())));
    for odd in ["Groq", "nul", "a/b", "a:b", ""] {
        let to = format!("id = \"{odd}\"");
        let err = edits(&[("id = \"groq\"", &to)]).err();
        assert_eq!(err, Some(ServicesError::Id(odd.into())), "{odd}");
    }
    for (from, to) in [
        ("light_hours = 15", "light_hours = 60"),
        ("room_kb = 64", "room_kb = 0"),
        ("max_models = 10", "max_models = 0"),
    ] {
        assert_eq!(
            edits(&[(from, to)]).err(),
            Some(ServicesError::Table),
            "{to}"
        );
    }
}

#[test]
fn a_refusal_names_the_row_and_its_rule_in_plain_words() {
    let err = edits(&[("max_mb = 23", "max_mb = 24")])
        .unwrap_err()
        .to_string();
    let room = "a piece leaves no room for the wav header and the form under the service's cap";
    assert_eq!(err, format!("the service groq is refused: {room}"));
}

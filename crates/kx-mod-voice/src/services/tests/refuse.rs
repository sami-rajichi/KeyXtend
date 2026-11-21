//! The validator refuses a broken row with its rule, so a bad edit fails here with a clear message instead of reaching
//! a request: every field the later rows use is covered.

use super::*;
use ServiceError as E;

/// Asserts that one edit of the shipped table, `from` to `to`, is refused with `why`.
#[track_caller]
fn refused(from: &str, to: &str, why: ServiceError) {
    assert_eq!(row_err(from, to), Some(why), "{to}");
}

/// Groq's form line, which the edits below break.
fn form() -> &'static str {
    groq_line("form")
}

/// Groq's form with `field` as its one fixed field.
fn fixed(field: &str) -> String {
    with_field(form(), "fixed", &format!("[{field}]"))
}

#[test]
fn a_fixed_address_or_test_off_the_rows_own_https_host_is_refused() {
    let (address, test, host_line) = (groq_line("address"), groq_line("test"), groq_line("host"));
    let host = field_of(host_line, "host").trim_matches('"');
    refused(address, &address.replace("https://", "http://"), E::Address);
    refused(address, &address.replace(host, "192.168.1.20"), E::Address);
    refused(address, &address.replace(host, "api.openai.com"), E::Host);
    refused(test, &test.replace(host, "evil.example"), E::Host);
    refused(test, &test.replace("https://", "http://"), E::Test);
    let parent = host.split_once('.').map_or(host, |(_, rest)| rest);
    refused(host_line, &format!("host = \"{parent}\""), E::Host);
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
    let shape = groq_line("shape");
    refused(shape, "shape = \"json\"", E::Shape);
    let bare = format!("{shape}\n{}", form());
    refused(&bare, "shape = \"eleven\"", E::Form);
    refused(shape, "shape = \"gemini\"", E::Form);
    refused(form(), "", E::Form);
    refused(
        form(),
        &with_field(form(), "lang", "\"lang uage\""),
        E::Form,
    );
    refused(form(), &fixed(&five), E::Form);
    let keep = format!(
        "{}\nkeep = {{ name = \"provider\", value = \"a\\nb\" }}",
        form()
    );
    refused(form(), &keep, E::Form);
    refused(groq_line("header"), "header = \"x y\"", E::Header);
}

#[test]
fn a_row_of_the_dropped_azure_shape_is_refused_as_an_unknown_shape() {
    // ADR-0050 left three shapes, so a row written as an Azure one names no shape at all.
    let shape = format!("{}\n{}", groq_line("shape"), form());
    refused(&shape, "shape = \"azure\"", E::Shape);
}

#[test]
fn a_list_paywall_or_region_is_refused_on_elevenlabs_shape_and_an_odd_region_code_anywhere() {
    let shape = groq_line("shape");
    let eleven = (shape, "shape = \"eleven\"");
    assert!(edits(&[eleven]).is_ok(), "its form");
    let list = with_field(form(), "list", "true");
    let listed = edits(&[eleven, (form(), list.as_str())]);
    let why = matches!(listed, Err(ServicesError::Row { why: E::Form, .. }));
    assert!(why, "its language goes plain: {listed:?}");
    for rule in ["paywall = true", "region = \"unsupported_country\""] {
        refused(shape, &format!("shape = \"eleven\"\n{rule}"), E::Form);
    }
    for code in ["unsupported country", ""] {
        refused(form(), &format!("{}\nregion = \"{code}\"", form()), E::Form);
    }
}

#[test]
fn a_fixed_field_that_asks_for_timings_or_an_odd_reply_is_refused() {
    let verbose = "{ name = \"response_format\", value = \"verbose_json\" }";
    refused(form(), &fixed(verbose), E::Reply);
    let word = "{ name = \"timestamp_granularities\", value = \"word\" }";
    refused(form(), &fixed(word), E::Reply);
    for plain in [
        "{ name = \"response_format\", value = \"json\" }",
        "{ name = \"timestamps_granularity\", value = \"none\" }",
    ] {
        assert!(edits(&[(form(), &fixed(plain))]).is_ok(), "{plain}");
    }
}

#[test]
fn a_name_keys_page_terms_or_limit_that_breaks_a_rule_is_refused() {
    let (name, keys, limits) = (groq_line("name"), groq_line("keys"), groq_line("limits"));
    refused(name, "name = \"\"", E::Name);
    refused(name, "name = \"Gr\u{202E}oq\"", E::Name);
    refused(&format!("{name}\n"), "", E::Name);
    refused(keys, &keys.replace("https://", "http://"), E::Keys);
    refused(keys, "", E::Keys);
    refused(groq_line("terms"), "terms = 0", E::Terms);
    for (field, value) in [("per_min", "0"), ("cap_mb", "0"), ("max_s", "601")] {
        refused(limits, &with_field(limits, field, value), E::Limits);
    }
}

#[test]
fn too_many_models_or_none_without_an_other_line_is_refused() {
    let too_many = edits(&[(head_line("max_models"), "max_models = 1")]);
    let models = |id: &str| {
        Some(ServicesError::Row {
            id: id.into(),
            why: E::Models,
        })
    };
    assert_eq!(too_many.err(), models("groq"));
    let (custom, none) = row_edit("custom", "other", "other = false");
    let none = edits(&[(custom, &none)]);
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
    let heavy = field_of(head_line("heavy_hours"), "heavy_hours");
    for (key, to) in [
        ("light_hours", format!("light_hours = {heavy}")),
        ("room_kb", "room_kb = 0".to_owned()),
        ("max_models", "max_models = 0".to_owned()),
    ] {
        let err = edits(&[(head_line(key), &to)]).err();
        assert_eq!(err, Some(ServicesError::Table), "{to}");
    }
}

#[test]
fn a_refusal_names_the_row_and_its_rule_in_plain_words() {
    let limits = groq_line("limits");
    let full = with_field(limits, "max_mb", field_of(limits, "cap_mb"));
    let err = edits(&[(limits, &full)]).unwrap_err().to_string();
    let room = "a piece leaves no room for the wav header and the form under the service's cap";
    assert_eq!(err, format!("the service groq is refused: {room}"));
}

/// The row error one edit of the shipped second file gives.
fn own_err(from: &str, to: &str) -> Option<ServiceError> {
    own_errs(&[(from, to)])
}

/// The row error these edits of the shipped second file give.
fn own_errs(pairs: &[(&str, &str)]) -> Option<ServiceError> {
    match Services::parse(TEXT, &edits_text(OWN, pairs), LANGS) {
        Err(ServicesError::Row { why, .. }) => Some(why),
        _ => None,
    }
}

#[test]
fn a_model_header_goes_on_assemblyais_shape_alone_which_needs_a_plain_one() {
    let named = own_line("assemblyai", "model_header");
    assert_eq!(
        own_err(named, ""),
        Some(E::Form),
        "its model header is needed"
    );
    let spaced = "model_header = \"X AAI-Model\"";
    assert_eq!(own_err(named, spaced), Some(E::Form), "a plain name");
    let with_form = format!("{named}\n{}", form());
    assert_eq!(own_err(named, &with_form), Some(E::Form), "no form");
    let shape = groq_line("shape");
    refused(shape, &format!("{shape}\n{named}"), E::Form);
    refused(
        &format!("{shape}\n{}", form()),
        "shape = \"assembly\"",
        E::Form,
    );
}

#[test]
fn a_model_header_never_names_a_header_the_request_sets_or_the_keys_own() {
    let named = own_line("assemblyai", "model_header");
    for taken in [
        "Authorization",
        "content-type",
        "CONTENT-LENGTH",
        "host",
        "User-Agent",
        "user-AGENT",
    ] {
        let to = format!("model_header = \"{taken}\"");
        assert_eq!(own_err(named, &to), Some(E::Form), "{taken}");
    }
    let key = (own_line("assemblyai", "header"), "header = \"X-Test-Key\"");
    let alone = Services::parse(TEXT, &edits_text(OWN, &[key]), LANGS);
    assert!(alone.is_ok(), "a named key header alone passes");
    let same = (named, "model_header = \"x-test-key\"");
    assert_eq!(own_errs(&[key, same]), Some(E::Form), "nor the key's own");
}

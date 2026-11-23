//! The flag reader: `true` at the exact dotted path, under each named object, counts; any other value, place or body
//! does not, and a broken or too-deep body reads as no flag without a panic.

use crate::net::flag::{any, flagged};
use std::borrow::Cow;

/// `OpenRouter`'s management-key field.
const PATH: &str = "data.is_management_key";
/// A reply that flags it.
const FLAG: &[u8] = br#"{"data":{"is_management_key":true}}"#;

#[test]
fn true_at_the_exact_path_counts_and_any_path_of_a_list_may_hold_it() {
    let both = br#"{"data":{"rate":{"is_management_key":false},"is_management_key":true}}"#;
    assert!(flagged(FLAG, PATH) && flagged(both, PATH));
    let escaped = br#"{"data":{"is_management_key":true}}"#;
    assert!(flagged(escaped, PATH), "an escaped name");
    assert!(flagged(br#"{"a":{"b":{"c":true}}}"#, "a.b.c"));
    let paths = [Cow::Borrowed("data.other"), Cow::Borrowed(PATH)];
    assert!(any(both, &paths));
    assert!(!any(both, &[]), "no paths");
}

#[test]
fn another_value_or_place_is_no_flag() {
    let bodies: [&[u8]; 9] = [
        br#"{"data":{"is_management_key":false}}"#,
        br#"{"data":{"is_management_key":"true"}}"#,
        br#"{"data":{"is_management_key":1}}"#,
        br#"{"data":{"is_management_key":null}}"#,
        br#"{"data":{"is_management_key":[true]}}"#,
        br#"{"data":{"is_management_key":{"value":true}}}"#,
        br#"{"is_management_key":true,"data":{}}"#,
        br#"{"data":[{"is_management_key":true}]}"#,
        br#"{"data":{"label":"kx-test"}}"#,
    ];
    for body in bodies {
        assert!(!flagged(body, PATH), "{}", String::from_utf8_lossy(body));
    }
    assert!(!flagged(br#"{"data":true}"#, PATH), "a path past a value");
    assert!(!flagged(FLAG, "data"), "an object");
}

#[test]
fn a_body_that_is_not_one_json_object_is_no_flag() {
    let listed = [&b"["[..], FLAG, b"]"].concat();
    let trailed = [FLAG, b" x"].concat();
    let cut = &FLAG[..FLAG.len() - 1];
    for body in [&listed[..], &trailed[..], cut, b"", b"true", b"null"] {
        assert!(!flagged(body, PATH), "{}", String::from_utf8_lossy(body));
    }
    let open = br#"{"data":{"is_management_key":true,"x":"#;
    let deep = [&open[..], &b"[".repeat(4096)[..]].concat();
    assert!(!flagged(&deep, PATH), "too deep after the flag");
}

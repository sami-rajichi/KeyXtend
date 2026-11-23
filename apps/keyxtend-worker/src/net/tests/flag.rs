//! The flag reader: `true` at the exact dotted path, under each named object, escaped or not, counts; any other value,
//! place or body does not. An error after the flag voids it without a panic, and a closed deep body reads as the reply
//! reader reads it.

use crate::net::flag::{any, flagged};
use crate::net::reply::object;
use std::borrow::Cow;

/// `OpenRouter`'s management-key field.
const PATH: &str = "data.is_management_key";
/// A reply that flags it.
const FLAG: &[u8] = br#"{"data":{"is_management_key":true}}"#;
/// Nesting far past `serde_json`'s recursion limit.
const DEEP: usize = 4096;

#[test]
fn true_at_the_exact_path_counts_and_any_path_of_a_list_may_hold_it() {
    let both = br#"{"data":{"rate":{"is_management_key":false},"is_management_key":true}}"#;
    assert!(flagged(FLAG, PATH) && flagged(both, PATH));
    let escaped = br#"{"data":{"is_management_key":true}}"#;
    assert!(flagged(escaped, PATH), "escaped names");
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
}

#[test]
fn an_error_after_the_flag_voids_it_and_a_closed_deep_body_reads_as_the_reply_reader_does() {
    let open = br#"{"data":{"is_management_key":true,"x":"#;
    let (into, out) = (b"[".repeat(DEEP), b"]".repeat(DEEP));
    let cut = [&open[..], &into].concat();
    assert!(!flagged(&cut, PATH), "an error after the flag voids it");
    let closed = [&open[..], &into, &out, b"}}"].concat();
    let read = (any(&closed, &[Cow::Borrowed(PATH)]), object(&closed));
    assert_eq!(read, (true, true), "a closed deep body is a valid object");
}

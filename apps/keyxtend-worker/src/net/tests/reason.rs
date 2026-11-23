//! An error reply read in one pass: its code only where its `Coded` says, its words only under their names and within
//! their depth, any other value skipped, and nothing from a body that is not one JSON object.

use crate::net::reason::Reason;
use crate::net::reply::{Coded, ERROR, SAYING, UNCODED};

/// Reads `body` as the upload form's errors are read.
fn form(body: &str) -> Reason {
    Reason::read(body.as_bytes(), &ERROR, SAYING)
}

#[test]
fn a_code_counts_only_where_its_coded_says_in_any_case() {
    let top = form(r#"{"error":{"code":"Insufficient_Quota","type":7}}"#);
    assert!(top.coded(&["insufficient_quota"]));
    let deeper = form(r#"{"error":{"inner":{"code":"insufficient_quota"}}}"#);
    let listed = form(r#"{"error":{"code":["insufficient_quota"]}}"#);
    let elsewhere = form(r#"{"code":"insufficient_quota"}"#);
    for reason in [deeper, listed, elsewhere] {
        assert!(!reason.coded(&["insufficient_quota"]));
    }
}

#[test]
fn words_count_under_their_names_within_their_depth_in_lower_case() {
    let deep = r#"{"detail":[{"msg":"Language NOT supported","loc":["body","language"]}]}"#;
    assert!(form(deep).says(&["language not supported"]));
    assert!(
        form(deep).says(&["language"]),
        "a loc list inside a detail item"
    );
    let too_deep = r#"{"error":{"message":{"message":{"message":{"message":{"message":"x"}}}}}}"#;
    assert!(!form(too_deep).says(&["x"]));
    let unnamed = r#"{"note":"language not supported"}"#;
    assert!(!form(unnamed).says(&["language"]));
}

#[test]
fn a_body_that_is_not_one_json_object_says_nothing() {
    for body in [
        r#"["error"]"#,
        r#""error""#,
        r#"{"error":"x""#,
        r#"{"error":"x"} {}"#,
        "",
        "<html>",
    ] {
        let reason = form(body);
        assert!(
            !reason.says(&["x", "error"]) && !reason.coded(&["x"]),
            "{body}"
        );
    }
}

#[test]
fn kept_texts_come_from_inside_the_coded_object_at_any_depth_and_odd_items_lose_only_themselves() {
    let outside = r#""reason":"out","message":{"reason":"out"}"#;
    let inside = r#""details":["odd",null,{"reason":"In","list":null},{"x":{"reason":"deep"}}]"#;
    let body = format!(r#"{{{outside},"error":{{{inside}}}}}"#);
    let reason = Reason::keeping(body.as_bytes(), &ERROR, SAYING, &["reason"]);
    assert_eq!(reason.kept("reason").collect::<Vec<_>>(), ["In", "deep"]);
    assert_eq!(reason.kept("other").count(), 0);
    assert_eq!(
        form(&body).kept("reason").count(),
        0,
        "kept only when asked"
    );
    let worded = br#"{"detail":[{"msg":"out"}],"error":{"x":{"msg":"in"}}}"#;
    let reason = Reason::keeping(worded, &ERROR, SAYING, &["msg"]);
    assert_eq!(
        reason.kept("msg").collect::<Vec<_>>(),
        ["in"],
        "a word's name too"
    );
}

#[test]
fn other_values_are_skipped_however_large_and_words_alone_need_no_code() {
    let junk = "1,".repeat(100_000);
    let body = format!(r#"{{"junk":[{junk}1],"title":"Payment method required"}}"#);
    let titled = Reason::read(body.as_bytes(), &UNCODED, &["title"]);
    assert!(titled.says(&["payment method"]) && !titled.coded(&[""]));
    let status = Coded {
        object: "error",
        names: &["status"],
    };
    let google = Reason::read(
        br#"{"error":{"status":"PERMISSION_DENIED"}}"#,
        &status,
        SAYING,
    );
    assert!(google.coded(&["permission_denied"]));
}

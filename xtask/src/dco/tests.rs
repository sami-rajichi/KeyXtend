//! Unit and property tests for commit parsing and the DCO sign-off check.

use proptest::prelude::*;

use super::*;
use crate::test_support::dco_config as dco_cfg;

/// Joins fields with [`FIELD_SEP`], for building a raw `git log` record in tests.
fn record(hash: &str, name: &str, email: &str, parents: &str, body: &str) -> String {
    format!("{hash}{FIELD_SEP}{name}{FIELD_SEP}{email}{FIELD_SEP}{parents}{FIELD_SEP}{body}")
}

/// Builds one [`Commit`] for `check` tests.
fn commit(name: &str, email: &str, is_merge: bool, body: &str) -> Commit {
    Commit {
        hash: "abc123def456".to_string(),
        author_name: name.to_string(),
        author_email: email.to_string(),
        is_merge,
        body: body.to_string(),
    }
}

// Parser tests.

#[test]
fn parses_a_body_with_blank_lines_and_a_trailing_newline() {
    let body = "subject\n\nSigned-off-by: A <a@x.com>\n\n";
    let raw = format!(
        "{}{RECORD_SEP}",
        record("deadbeef", "A", "a@x.com", "", body)
    );

    let commits = parse_log(&raw).unwrap();

    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].hash, "deadbeef");
    assert_eq!(commits[0].body, body.trim());
    assert!(!commits[0].is_merge);
}

#[test]
fn a_record_with_missing_fields_is_an_error() {
    let raw = format!("onlyhash{FIELD_SEP}onlyname{RECORD_SEP}");
    assert!(parse_log(&raw).is_err());
}

#[test]
fn two_parents_means_a_merge_commit() {
    let raw = format!(
        "{}{RECORD_SEP}",
        record("m1", "A", "a@x.com", "p1 p2", "merge body")
    );
    let commits = parse_log(&raw).unwrap();
    assert!(commits[0].is_merge);
}

#[test]
fn empty_input_parses_to_no_commits() {
    assert_eq!(parse_log("").unwrap(), vec![]);
}

// Plan edge cases 13-17.

#[test]
fn commit_without_sign_off_fails() {
    let c = commit("Ada Lovelace", "ada@example.com", false, "just a message");
    assert_eq!(check(&[c], &dco_cfg()).len(), 1);
}

#[test]
fn sign_off_with_different_email_fails() {
    let c = commit(
        "Ada Lovelace",
        "ada@example.com",
        false,
        "msg\n\nSigned-off-by: Ada Lovelace <other@example.com>",
    );
    assert_eq!(check(&[c], &dco_cfg()).len(), 1);
}

#[test]
fn trailer_text_mid_sentence_is_not_a_sign_off() {
    let c = commit(
        "Ada Lovelace",
        "ada@example.com",
        false,
        "feat: Signed-off-by: fixed parsing",
    );
    assert_eq!(check(&[c], &dco_cfg()).len(), 1);
}

#[test]
fn lower_case_trailer_passes() {
    let c = commit(
        "Ada Lovelace",
        "ada@example.com",
        false,
        "msg\n\nsigned-off-by: Ada Lovelace <ada@example.com>",
    );
    assert!(check(&[c], &dco_cfg()).is_empty());
}

#[test]
fn merge_and_exempt_author_are_skipped() {
    let merge = commit(
        "Ada Lovelace",
        "ada@example.com",
        true,
        "merge, no sign-off",
    );
    let bot = commit(
        "dependabot[bot]",
        "bot@github.com",
        false,
        "bump, no sign-off",
    );
    assert!(check(&[merge, bot], &dco_cfg()).is_empty());
}

#[test]
fn empty_range_passes() {
    assert!(check(&[], &dco_cfg()).is_empty());
}

// Output tests.

#[test]
fn render_shows_short_hash_author_and_reason() {
    let v = Violation {
        hash: "0123456789abcdef".to_string(),
        author_name: "A".to_string(),
        author_email: "a@x.com".to_string(),
    };
    assert_eq!(render(&v), "0123456 A <a@x.com>: missing sign-off");
}

#[test]
fn passed_is_true_for_no_violations() {
    assert!(passed(&[]));
}

#[test]
fn passed_is_false_with_any_violation() {
    let v = Violation {
        hash: "h".to_string(),
        author_name: "A".to_string(),
        author_email: "a@x.com".to_string(),
    };
    assert!(!passed(&[v]));
}

#[test]
fn summary_counts_and_pluralizes() {
    assert_eq!(summary(&[]), "dco: 0 failing commits");
    let v = Violation {
        hash: "h".to_string(),
        author_name: "A".to_string(),
        author_email: "a@x.com".to_string(),
    };
    assert_eq!(summary(&[v]), "dco: 1 failing commit");
}

// Property test (plan task 9): a matching trailer passes, and its absence fails.

/// Names/emails from a restricted class, non-empty once trimmed: real git identities,
/// never impossible ones, and never characters (`<`, `>`, newline) the trailer needs.
fn ident() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 ._@-]{1,20}"
        .prop_filter("must not be blank", |s: &String| !s.trim().is_empty())
        .prop_map(|s| s.trim().to_string())
}

proptest! {
    #[test]
    fn matching_trailer_passes_and_missing_one_fails(name in ident(), email in ident()) {
        let dco = dco_cfg();

        let signed_off = commit(
            &name,
            &email,
            false,
            &format!("subject\n\nSigned-off-by: {name} <{email}>\n"),
        );
        prop_assert!(check(&[signed_off], &dco).is_empty());

        let unsigned = commit(&name, &email, false, "subject\n\nno trailer here");
        prop_assert!(!check(&[unsigned], &dco).is_empty());
    }
}

// Parser properties: generated records round-trip, and no input makes the parser panic.

/// One generated record's fields: hash, author name, author email, parent hashes, body.
type Fields = (String, String, String, Vec<String>, String);

/// Fields as git could print them; a body never ends in whitespace, which parsing trims.
fn any_fields() -> impl Strategy<Value = Fields> {
    let hash = "[0-9a-f]{40}";
    let parents = prop::collection::vec(hash, 0..4);
    let body = "[A-Za-z0-9 :<>@.\n-]{0,60}".prop_map(|s| s.trim_end().to_string());
    (hash, ident(), ident(), parents, body)
}

/// The [`Commit`] that parsing the record for `fields` must produce.
fn to_commit((hash, author_name, author_email, parents, body): Fields) -> Commit {
    Commit {
        hash,
        author_name,
        author_email,
        is_merge: parents.len() > 1,
        body,
    }
}

proptest! {
    #[test]
    fn generated_records_parse_back_to_the_same_commits(
        all in prop::collection::vec(any_fields(), 0..5),
    ) {
        let mut raw = String::new();
        for (hash, name, email, parents, body) in &all {
            raw.push_str(&record(hash, name, email, &parents.join(" "), body));
            raw.push(RECORD_SEP);
            raw.push('\n');
        }
        let expected: Vec<Commit> = all.into_iter().map(to_commit).collect();
        prop_assert_eq!(parse_log(&raw), Ok(expected));
    }

    #[test]
    fn parse_log_never_panics(
        chars in prop::collection::vec(
            prop_oneof![Just(FIELD_SEP), Just(RECORD_SEP), any::<char>()],
            0..64,
        ),
    ) {
        let _ = parse_log(&chars.into_iter().collect::<String>());
    }
}

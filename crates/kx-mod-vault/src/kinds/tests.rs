//! The kinds and labels tables: each kind's icon, filter and starter labels, each label's hidden start and `@@` feed,
//! and the broken tables a parse refuses.

use super::*;
use std::fmt::Write;

/// The shipped labels table, which the broken kinds tables below lean on.
fn labels() -> &'static str {
    LABELS
}

/// A kinds table with every kind but `drop`, plus `extra` lines.
fn kinds_without(drop: &str, extra: &str) -> String {
    let rows = ["person", "login", "wifi", "card", "note", "other"];
    let mut text = String::new();
    for id in rows.iter().filter(|id| **id != drop) {
        let row = "\nicon = \"kind_x\"\nfilter = \"all\"\nfields = []\n";
        let _ = write!(text, "[[kind]]\nid = \"{id}\"{row}");
    }
    text.push_str(extra);
    text
}

/// True when the kinds `text` beside the shipped labels is refused.
fn kinds_refused(text: &str) -> bool {
    Tables::parse(text, labels()).is_err()
}

#[test]
fn the_shipped_tables_parse() {
    assert!(Tables::parse(KINDS, LABELS).is_ok());
    assert_eq!(tables().kinds().len(), EntryKind::ALL.len());
}

#[test]
fn each_kind_has_its_icon_filter_and_starter_labels() {
    use EntryKind::{Card, Login, Note, Other, Person, Wifi};
    use FillFilter::{All, Cards, Logins, Notes, People};
    let want: [(EntryKind, &str, FillFilter, &[&str]); 6] = [
        (
            Person,
            "kind_person",
            People,
            &["full-name", "email", "phone", "address"],
        ),
        (
            Login,
            "kind_login",
            Logins,
            &["user", "password", "website"],
        ),
        (
            Wifi,
            "kind_wifi",
            FillFilter::Wifi,
            &["network", "password"],
        ),
        (Card, "kind_card", Cards, &["number", "full-name"]),
        (Note, "kind_note", Notes, &["text"]),
        (Other, "kind_other", All, &[]),
    ];
    for (kind, icon, filter, fields) in want {
        let row = tables().kind(kind).unwrap();
        assert_eq!((row.icon.as_str(), row.filter), (icon, filter), "{kind:?}");
        assert_eq!(row.fields, fields, "{kind:?}");
        assert_eq!(tables().kind_by_id(&row.id).map(|r| r.kind), Some(kind));
    }
}

#[test]
fn the_labels_come_in_their_order_with_their_hidden_start_and_feed() {
    let ids: Vec<&str> = tables().labels().iter().map(|l| l.id.as_str()).collect();
    let want = [
        "full-name",
        "email",
        "phone",
        "address",
        "city",
        "postcode",
        "country",
        "birthday",
        "user",
        "password",
        "pin",
        "id-number",
        "number",
        "website",
        "network",
        "text",
    ];
    assert_eq!(ids, want);
    let hidden: Vec<&str> = tables()
        .labels()
        .iter()
        .filter(|l| l.hidden)
        .map(|l| l.id.as_str())
        .collect();
    assert_eq!(hidden, ["password", "pin", "id-number", "number"]);
    let emails: Vec<&str> = tables()
        .labels()
        .iter()
        .filter(|l| l.emails)
        .map(|l| l.id.as_str())
        .collect();
    assert_eq!(emails, ["email"]);
    assert!(
        tables()
            .label("email")
            .is_some_and(|l| l.emails && !l.hidden)
    );
    assert!(tables().label("Email").is_none());
}

#[test]
fn a_kinds_table_lists_every_kind_once() {
    assert!(Tables::parse(&kinds_without("", ""), labels()).is_ok());
    assert!(kinds_refused(&kinds_without("card", "")));
    let twice = "[[kind]]\nid = \"note\"\nicon = \"kind_note\"\nfilter = \"notes\"\nfields = []\n";
    assert!(kinds_refused(&kinds_without("", twice)));
    let unknown = "[[kind]]\nid = \"pet\"\nicon = \"kind_pet\"\nfilter = \"all\"\nfields = []\n";
    assert!(kinds_refused(&kinds_without("", unknown)));
}

#[test]
fn a_kind_starts_only_with_known_labels_once_each_and_names_a_token() {
    let row = |fields: &str, icon: &str| {
        let line = format!(
            "[[kind]]\nid = \"other\"\nicon = \"{icon}\"\nfilter = \"all\"\nfields = [{fields}]\n"
        );
        kinds_without("other", &line)
    };
    assert!(!kinds_refused(&row("\"text\"", "kind_other")));
    assert!(kinds_refused(&row("\"colour\"", "kind_other")));
    assert!(kinds_refused(&row("\"text\", \"text\"", "kind_other")));
    for icon in ["", "Kind", "kind other", "kind-other"] {
        assert!(kinds_refused(&row("", icon)), "{icon:?}");
    }
}

#[test]
fn a_labels_table_names_each_label_once_as_a_text_key_part() {
    let kinds = kinds_without("", "");
    let parse = |text: &str| Tables::parse(&kinds, text).is_ok();
    assert!(parse("[[label]]\nid = \"pet-name\"\n"));
    assert!(!parse("[[label]]\nid = \"pet\"\n[[label]]\nid = \"pet\"\n"));
    for id in [
        "",
        "Pet",
        "pet name",
        "-pet",
        "pet-",
        "pet--name",
        "pet_name",
    ] {
        assert!(!parse(&format!("[[label]]\nid = \"{id}\"\n")), "{id:?}");
    }
    assert!(!parse("[[label]]\nid = \"pet\"\ncolour = true\n"));
}

#[test]
fn the_login_alone_pairs_its_user_and_password_labels() {
    for row in tables().kinds() {
        let want =
            (row.kind == EntryKind::Login).then(|| ["user".to_owned(), "password".to_owned()]);
        assert_eq!(row.pair, want, "{:?}", row.kind);
    }
}

#[test]
fn a_pair_names_two_known_labels_apart() {
    let row = |pair: &str| {
        let line = format!(
            "[[kind]]\nid = \"other\"\nicon = \"kind_other\"\nfilter = \"all\"\nfields = []\npair = [{pair}]\n"
        );
        kinds_without("other", &line)
    };
    assert!(!kinds_refused(&row("\"user\", \"pin\"")));
    assert!(kinds_refused(&row("\"pin\", \"pin\"")));
    assert!(kinds_refused(&row("\"user\", \"colour\"")));
    assert!(kinds_refused(&row("\"user\"")));
}

#[test]
fn only_the_password_label_offers_make_one() {
    let made: Vec<&str> = tables()
        .labels()
        .iter()
        .filter(|l| l.make)
        .map(|l| l.id.as_str())
        .collect();
    assert_eq!(made, ["password"]);
}

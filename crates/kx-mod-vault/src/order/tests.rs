//! The order and filters: last used first, then never used, each A to Z by folded name; each filter's kinds; By label's
//! labels with their counts, own labels joined by their folded text; and every value under one label.

use super::*;
use crate::book::Notebook;
use crate::entry::{Draft, Field, Stamp};
use crate::find::LabelNames;
use crate::fixture::{LIMITS, draft, field, own};
use kx_platform_fake::FakeRandom;

/// A day in ms.
const DAY: i64 = 86_400_000;
/// The time the tests save at.
const NOW: Stamp = Stamp::from_unix_ms(1_762_000_000_000);
/// No held times, so each entry sorts by its own.
const FREE: &Uses = &Uses::new();

/// `drafts` saved in order, each filled at the day it is given, if any.
fn book(drafts: Vec<(Draft, Option<i64>)>) -> Notebook {
    let (mut book, random) = (Notebook::default(), FakeRandom::new(5));
    for (d, day) in drafts {
        let id = book.add(d, NOW, &random, &LIMITS).unwrap();
        if let Some(day) = day {
            book.use_at(id, Stamp::from_unix_ms(NOW.unix_ms() + day * DAY));
        }
    }
    book
}

/// A note named `name`.
fn note(name: &str) -> Draft {
    draft(name, EntryKind::Note, vec![field("text", "x", false)])
}

/// The names of `entries`, in order.
fn names(entries: &[&Entry]) -> Vec<String> {
    entries.iter().map(|e| e.name().expose().clone()).collect()
}

#[test]
fn the_last_used_come_first_then_the_rest_a_to_z_by_folded_name() {
    let b = book(vec![
        (note("zèbre"), None),
        (note("Banque"), Some(1)),
        (note("école"), None),
        (note("Agence"), None),
        (note("Mail"), Some(3)),
        (note("Écrin"), None),
    ]);
    let want = ["Mail", "Banque", "Agence", "école", "Écrin", "zèbre"];
    assert_eq!(names(&sorted(b.entries(), FREE)), want);
}

#[test]
fn arabic_names_sort_by_their_folded_letters() {
    let b = book(vec![
        (note("بنك"), None),
        (note("أمل"), None),
        (note("ثمر"), None),
        (note("تفاح"), None),
    ]);
    assert_eq!(
        names(&sorted(b.entries(), FREE)),
        ["أمل", "بنك", "تفاح", "ثمر"]
    );
}

#[test]
fn each_filter_shows_its_kinds_and_all_shows_every_one() {
    use EntryKind::{Card, Login, Note, Other, Person, Wifi};
    let kinds = [Person, Login, Wifi, Card, Note, Other];
    let shown = |filter| {
        kinds
            .iter()
            .copied()
            .filter(|k| shows(filter, *k))
            .collect::<Vec<_>>()
    };
    assert_eq!(shown(FillFilter::All), kinds);
    assert_eq!(shown(FillFilter::People), [Person]);
    assert_eq!(shown(FillFilter::Logins), [Login]);
    assert_eq!(shown(FillFilter::Wifi), [Wifi]);
    assert_eq!(shown(FillFilter::Cards), [Card]);
    assert_eq!(shown(FillFilter::Notes), [Note]);
    assert_eq!(
        shown(FillFilter::Labels),
        kinds,
        "By label groups every entry"
    );
}

#[test]
fn the_list_is_the_filter_and_the_search_in_order() {
    let login = |name: &str| draft(name, EntryKind::Login, vec![field("user", "alex", false)]);
    let b = book(vec![
        (login("Bank"), None),
        (note("Bank note"), Some(1)),
        (login("Mail"), Some(2)),
    ]);
    let find = Find::new("", &LabelNames::new());
    assert_eq!(
        names(&listed(b.entries(), FillFilter::All, &find, FREE)),
        ["Mail", "Bank note", "Bank"]
    );
    let bank = Find::new("bank", &LabelNames::new());
    assert_eq!(
        names(&listed(b.entries(), FillFilter::Logins, &bank, FREE)),
        ["Bank"]
    );
}

/// A draft of `kind` named `name` with `fields`, filled at `day` if any.
fn at(name: &str, kind: EntryKind, fields: Vec<Field>, day: Option<i64>) -> (Draft, Option<i64>) {
    (draft(name, kind, fields), day)
}

#[test]
fn by_label_counts_values_in_the_tables_order_then_own_labels_a_to_z() {
    use EntryKind::{Other, Person};
    let b = book(vec![
        at(
            "A",
            Person,
            vec![field("phone", "1", false), field("email", "a@b", false)],
            None,
        ),
        at(
            "B",
            Person,
            vec![field("email", "b@b", true), own("Zone", "z", false)],
            None,
        ),
        at(
            "C",
            Other,
            vec![own("code wifi", "c", true), own("Code Wi-Fi", "d", false)],
            Some(1),
        ),
        at("D", Other, vec![own("CODE WIFI", "e", false)], None),
    ]);
    let got: Vec<(Label, u32)> = labels(b.entries(), FREE)
        .into_iter()
        .map(|l| (l.label, l.count))
        .collect();
    let table = |id: &str| Label::Table(id.to_owned());
    let mine = |text: &str| Label::Own(Redacted::new(text.to_owned()));
    // Folded, "code wi-fi" sorts before "code wifi"; the last used entry, C, gives the spelling its group keeps.
    let want = vec![
        (table("email"), 2),
        (table("phone"), 1),
        (mine("Code Wi-Fi"), 1),
        (mine("code wifi"), 2),
        (mine("Zone"), 1),
    ];
    assert_eq!(got, want);
}

#[test]
fn the_values_under_a_label_come_with_their_entry_in_the_lists_order() {
    use EntryKind::{Other, Person};
    let b = book(vec![
        at("A", Person, vec![field("email", "a@b", false)], None),
        at(
            "B",
            Person,
            vec![field("email", "b@b", true), field("phone", "2", false)],
            Some(1),
        ),
        at("C", Other, vec![own("Pin", "1", true)], None),
        at("D", Other, vec![own("PIN", "2", true)], None),
    ]);
    let under = |label: &Label| -> Vec<(String, String)> {
        let rows = values(b.entries(), label, FREE);
        let pair =
            |(e, f): &(&Entry, &Field)| (e.name().expose().clone(), f.value.expose().to_string());
        rows.iter().map(pair).collect()
    };
    let pair = |n: &str, v: &str| (n.to_owned(), v.to_owned());
    let email = under(&Label::Table("email".to_owned()));
    assert_eq!(email, [pair("B", "b@b"), pair("A", "a@b")]);
    let pin = under(&Label::Own(Redacted::new("pin".to_owned())));
    assert_eq!(pin, [pair("C", "1"), pair("D", "2")]);
    assert!(under(&Label::Table("city".to_owned())).is_empty());
}

#[test]
fn a_held_time_sorts_in_place_of_the_entrys_own_so_a_fill_moves_no_row() {
    let mut b = book(vec![(note("Agence"), Some(1)), (note("Banque"), None)]);
    let banque = b.entries()[1].id();
    b.use_at(banque, Stamp::from_unix_ms(NOW.unix_ms() + 2 * DAY));
    assert_eq!(names(&sorted(b.entries(), FREE)), ["Banque", "Agence"]);
    let held = Uses::from([(banque, None)]);
    assert_eq!(names(&sorted(b.entries(), &held)), ["Agence", "Banque"]);
    let find = Find::new("", &LabelNames::new());
    let all = listed(b.entries(), FillFilter::All, &find, &held);
    assert_eq!(names(&all), ["Agence", "Banque"]);
}

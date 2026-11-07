//! Quick-fill's views the Qt Quick tests draw (`qmltest/fixtures/fill.js`), as the hub writes them in each language:
//! a full list and a short one, the empty notebook, no match, By label and a label's values, an entry with a hidden value
//! masked, shown and empty, an Arabic entry, New's kinds and form, Edit with its question, the labels table, a note and
//! the backup and import pages; and the click kinds QML sends to `QuickFill.act`, by the name QML holds each under
//! (QuickFillNames.qml).

mod backups;
mod forms;

use crate::qml::{HEAD, LANGS, fixture, hub, var};
use forms::{edits, entries};
use kx_module_api::ports::{
    EntryKind, EntryRow, FieldValue, FillFilter, FillNote, FillPage, FillPanel, FillView, Hint,
    Label, LabelCount, LabelValue, PageNote, Pages, PlaceKind, RowButton, Sheet, Spot,
};
use kx_module_api::{Bus, Redacted, Secret, Zeroizing};
use kx_test_support::host::FakeCore;
use kx_ui::fill_acts as acts;
use serde_json::{Map, Value, json};
use std::sync::Arc;

/// The fixture's file name.
pub const FILE: &str = "fill.js";
/// An Arabic name, email and own label with its value, which read right to left in the Arabic font.
const ARABIC: [&str; 4] = ["ليلى حداد", "layla@example.com", "رقم الخزانة", "12"];

/// Each click kind by the name QML holds it under.
fn acts() -> Value {
    json!({
        "row": acts::ROW, "field": acts::FIELD, "filter": acts::FILTER, "kind": acts::KIND,
        "page": acts::PAGE, "button": acts::BUTTON, "answer": acts::ANSWER, "checks": acts::CHECKS,
        "caret": acts::CARET, "wheel": acts::WHEEL,
    })
}

/// `text`, redacted.
fn shown(text: &str) -> Redacted<String> {
    Redacted::new(text.to_owned())
}

/// `text` as a secret.
fn secret(text: &str) -> Secret {
    Secret::new(Zeroizing::new(text.to_owned()))
}

/// A table label by its id.
fn table(id: &str) -> Label {
    Label::Table(id.to_owned())
}

/// Row `id` of `kind` named `name`, with `hint` and `buttons`.
fn row(id: u64, kind: EntryKind, name: &str, hint: Hint, buttons: &[RowButton]) -> EntryRow {
    EntryRow {
        id,
        kind,
        name: shown(name),
        hint,
        buttons: buttons.to_vec(),
    }
}

/// A full page: a login, a person, a Wi-Fi, a one-field note and an Arabic person; hidden values glimpse as the mask.
fn rows() -> Vec<EntryRow> {
    let (plain, hidden) = (
        |t: &str| FieldValue::new(secret(t), false, false),
        || FieldValue::new(secret("hidden"), true, false),
    );
    let login = Hint::Values(vec![plain("alex@example.com"), hidden()]);
    let wifi = Hint::Values(vec![plain("HomeNet-5G"), hidden()]);
    let (open, insert) = (&[RowButton::Open][..], &[RowButton::Insert][..]);
    let both = [RowButton::User, RowButton::Password, RowButton::Both];
    vec![
        row(1, EntryKind::Login, "GitHub", login, &both),
        row(2, EntryKind::Person, "Alex Martin", Hint::Fields(6), open),
        row(3, EntryKind::Wifi, "Home Wi-Fi", wifi, open),
        row(4, EntryKind::Note, "Door code", Hint::Fields(1), insert),
        row(
            5,
            EntryKind::Person,
            ARABIC[0],
            Hint::Values(vec![plain(ARABIC[1])]),
            open,
        ),
    ]
}

/// Page 1 of `count` showing `sheet`, under All, with no search, the first row highlighted.
fn page(sheet: Sheet, count: u16) -> FillPage {
    FillPage {
        round: 9,
        sheet,
        filter: FillFilter::All,
        search: shown(""),
        caret: Redacted::new(0),
        pages: Pages { at: 0, count },
        highlight: Some(0),
        note: None,
        waiting: None,
        empty: false,
    }
}

/// The lists: a full page of three with its second row highlighted, a short one, the empty notebook, no match, and a
/// short one with a note and a waiting backup.
fn lists() -> Vec<(&'static str, FillPage)> {
    let mut list = page(Sheet::List(rows()), 3);
    list.highlight = Some(1);
    let short = || page(Sheet::List(rows().into_iter().take(2).collect()), 1);
    let mut empty = page(Sheet::List(Vec::new()), 1);
    (empty.empty, empty.highlight) = (true, None);
    let mut none = page(Sheet::List(Vec::new()), 1);
    (none.search, none.filter, none.highlight) = (shown("zzz"), FillFilter::Logins, None);
    let mut noted = short();
    noted.note = Some(PageNote::plain(FillNote::Typed));
    noted.waiting = Some(Spot {
        kind: PlaceKind::Removable,
        name: shown("USB stick (E:)"),
    });
    vec![
        ("list", list),
        ("short", short()),
        ("empty", empty),
        ("none", none),
        ("noted", noted),
    ]
}

/// By label's labels, then its values pages.
fn labels() -> Vec<(&'static str, FillPage)> {
    let count = |label, count| LabelCount { label, count };
    let mut by = page(
        Sheet::Labels(vec![
            count(table("email"), 3),
            count(table("password"), 2),
            count(Label::Own(shown("Locker")), 1),
            count(Label::Own(shown(ARABIC[2])), 1),
        ]),
        1,
    );
    by.filter = FillFilter::Labels;
    let mut all = vec![("labels", by)];
    all.extend(values());
    all
}

/// A label's values, and the masked values under PIN.
fn values() -> Vec<(&'static str, FillPage)> {
    let value = |name: &str, v: FieldValue| LabelValue {
        name: shown(name),
        value: v,
    };
    let plain = |t: &str| FieldValue::new(secret(t), false, false);
    let emails = vec![
        value("Alex Martin", plain("alex@example.com")),
        value(ARABIC[0], plain(ARABIC[1])),
    ];
    let masked = |t: &str| FieldValue::new(secret(t), true, false);
    let pins = vec![
        value("Bank card", masked("4821")),
        value("Door code", masked("1234")),
    ];
    vec![
        (
            "values",
            page(
                Sheet::Values {
                    label: table("email"),
                    rows: emails,
                },
                1,
            ),
        ),
        (
            "pins",
            page(
                Sheet::Values {
                    label: table("pin"),
                    rows: pins,
                },
                1,
            ),
        ),
    ]
}

/// Every view in language `tag`, by name.
fn views(tag: &str) -> Value {
    let (hub, _host) = hub(tag);
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let _heard = hub.listen(&bus).expect("the hub listens");
    bus.publish(&FillPanel { open: true })
        .expect("the panel opens");
    let all = [
        lists(),
        labels(),
        entries(),
        edits(),
        backups::backups(),
        backups::picks(),
        backups::asks(),
        backups::checks(),
    ]
    .into_iter()
    .flatten();
    let shown = all.map(|(name, p)| {
        bus.publish(&FillView::new(p)).expect("the view is sent");
        let v: Value = serde_json::from_str(&hub.fill_json()).expect("panel JSON");
        (name.to_owned(), v)
    });
    Value::Object(shown.collect::<Map<_, _>>())
}

#[test]
fn the_fill_fixture_matches_the_hub() {
    let all: Map<String, Value> = LANGS.iter().map(|t| ((*t).to_owned(), views(t))).collect();
    let text = format!(
        "{HEAD}{}{}",
        var("acts", &acts(), true),
        var("views", &Value::Object(all), true)
    );
    fixture(FILE.to_owned(), &text);
}

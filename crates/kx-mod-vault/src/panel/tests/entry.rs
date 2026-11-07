//! The entry page: each field with its value or the mask, Show until the page closes, Insert, and its pages.

use kx_module_api::ports::{
    EntryKind, EntrySheet, FieldButton, FieldLine, FillButton, FillClick, Label, PanelKey, Sheet,
};

use super::super::rig::{NOW, Rig};
use super::super::{Do, Fill, Part};
use super::{button, open};
use crate::fixture::{LIMITS, draft, field};

/// Field `field`'s `button`.
fn at(field: u8, button: FieldButton) -> FillClick {
    FillClick::Field { field, button }
}

/// The entry page shown.
fn sheet(r: &Rig) -> &EntrySheet {
    match &r.page.sheet {
        Sheet::Entry(e) => e,
        other => panic!("not an entry page: {other:?}"),
    }
}

/// The values the entry page shows, `None` for the mask.
fn values(r: &Rig) -> Vec<Option<String>> {
    let shown = sheet(r)
        .fields
        .iter()
        .map(|f| f.value.text().map(|t| t.expose().to_string()));
    shown.collect()
}

/// Bank's values as its page shows them before any Show.
fn masked() -> Vec<Option<String>> {
    vec![
        Some("alex@example.com".into()),
        None,
        Some("bank.example".into()),
    ]
}

#[test]
fn the_entry_page_shows_each_field_with_hidden_ones_masked() {
    let mut r = Rig::sample();
    assert_eq!(r.click(open(1)), Do::Show);
    let e = sheet(&r);
    assert_eq!(
        (e.kind, e.name.expose().as_str()),
        (EntryKind::Login, "Bank")
    );
    let labels: Vec<Label> = e.fields.iter().map(|f| f.label.clone()).collect();
    assert_eq!(
        labels,
        ["user", "password", "website"].map(|id| Label::Table(id.into()))
    );
    assert_eq!(values(&r), masked());
    assert!(sheet(&r).fields[1].value.hidden());
    let read_only = |f: &FieldLine| !f.make && !f.value.switches();
    assert!(
        sheet(&r).fields.iter().all(read_only),
        "no Make one or switch here"
    );
}

#[test]
fn show_reveals_a_hidden_value_until_its_page_closes() {
    let mut r = Rig::sample();
    r.click(open(1));
    assert_eq!(r.click(at(1, FieldButton::Show)), Do::Show);
    assert_eq!(values(&r)[1].as_deref(), Some("hunter2"));
    assert!(sheet(&r).fields[1].value.shown());
    assert_eq!(r.click(at(1, FieldButton::Show)), Do::Show, "Hide");
    assert_eq!(values(&r), masked());
    r.click(at(1, FieldButton::Show));
    r.key(&PanelKey::Esc);
    r.click(open(1));
    assert_eq!(values(&r), masked(), "hidden again once the page closed");
    r.click(at(1, FieldButton::Show));
    r.click(button(FillButton::Back));
    r.click(open(1));
    assert_eq!(values(&r), masked());
}

#[test]
fn show_does_nothing_on_a_field_that_is_not_hidden() {
    let mut r = Rig::sample();
    r.click(open(1));
    assert_eq!(r.click(at(0, FieldButton::Show)), Do::Stay);
    for b in [
        FieldButton::Value,
        FieldButton::Label,
        FieldButton::Hidden,
        FieldButton::Paste,
        FieldButton::Make,
    ] {
        assert_eq!(r.click(at(1, b)), Do::Stay, "{b:?} belongs to New and Edit");
    }
    assert_eq!(
        r.click(at(5, FieldButton::Insert)),
        Do::Stay,
        "past the fields"
    );
}

#[test]
fn insert_fills_that_field_and_enter_the_highlighted_one() {
    let mut r = Rig::sample();
    r.click(open(1));
    let bank = r.id("Bank");
    let one = |n| {
        Do::Fill(Fill {
            id: bank,
            parts: vec![Part::Value(n)],
        })
    };
    assert_eq!(r.page.highlight, Some(0));
    assert_eq!(r.click(at(2, FieldButton::Insert)), one(2));
    assert_eq!(r.page.highlight, Some(2), "a click highlights its field");
    assert!(
        matches!(r.page.sheet, Sheet::Entry(_)),
        "a fill keeps the page"
    );
    assert_eq!(r.key(&PanelKey::Up), Do::Show);
    assert_eq!(r.key(&PanelKey::Enter), one(1));
    for key in [
        PanelKey::Left,
        PanelKey::Right,
        PanelKey::Tick,
        PanelKey::Erase,
    ] {
        assert_eq!(r.key(&key), Do::Stay, "{key:?}");
    }
    assert_eq!(r.text("x"), Do::Stay, "letters go nowhere here");
}

#[test]
fn a_long_entry_pages_its_fields_five_at_a_time() {
    let fields = (0..7)
        .map(|n| field("text", &format!("v{n}"), false))
        .collect();
    let mut r = Rig::new(vec![draft("Long", EntryKind::Other, fields)]);
    r.click(open(0));
    assert_eq!(sheet(&r).fields.len(), 5);
    assert_eq!(r.page.pages.count, 2);
    assert_eq!(r.click(FillClick::Page { back: false }), Do::Show);
    assert_eq!(values(&r), [Some("v5".into()), Some("v6".into())]);
    let long = r.id("Long");
    let want = Do::Fill(Fill {
        id: long,
        parts: vec![Part::Value(6)],
    });
    assert_eq!(r.click(at(1, FieldButton::Insert)), want);
}

#[test]
fn an_entry_gone_from_the_notebook_closes_its_page() {
    let mut r = Rig::sample();
    r.click(open(1));
    let bank = r.id("Bank");
    r.book.remove(bank);
    assert_eq!(
        r.key(&PanelKey::Enter),
        Do::Show,
        "never fills the gone entry"
    );
    assert!(matches!(r.page.sheet, Sheet::List(_)));
}

#[test]
fn an_entry_replaced_under_its_id_hides_what_its_page_showed() {
    let mut r = Rig::sample();
    let bank = r.id("Bank");
    r.click(open(1));
    r.click(at(1, FieldButton::Show));
    assert_eq!(values(&r)[1].as_deref(), Some("hunter2"));
    let fields = vec![
        field("user", "sam", false),
        field("password", "new-pass", true),
    ];
    let login = draft("Bank", EntryKind::Login, fields);
    assert_eq!(r.book.edit(bank, login, NOW, &LIMITS), Ok(true));
    r.draw();
    assert_eq!(values(&r), [Some("sam".to_owned()), None]);
}

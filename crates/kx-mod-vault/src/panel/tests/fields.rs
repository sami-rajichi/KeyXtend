//! A form's fields: Add a field from the labels table or with Your own label, Make one, the hidden flag, Paste, and
//! shown values hiding as the form closes.

use kx_module_api::ports::{
    EntryKind, FieldButton, FillButton, FillClick, FillNote, Label, PanelKey, Sheet, Typing,
};

use super::super::Do;
use super::super::rig::{CONFIG, Rig};
use super::edit::{at, limited, new, saved, sheet, values};
use super::{button, open};
use crate::book::Limits;
use crate::fixture::LIMITS;
use crate::make::groups;

#[test]
fn add_a_field_offers_the_labels_table_paged() {
    let mut r = Rig::sample();
    new(&mut r, EntryKind::Other);
    assert!(sheet(&r).fields.is_empty());
    assert_eq!(r.click(button(FillButton::AddField)), Do::Show);
    let first =
        ["full-name", "email", "phone", "address", "city"].map(|id| Label::Table(id.to_owned()));
    assert!(matches!(&r.page.sheet, Sheet::AddField(l) if *l == first));
    assert_eq!(r.page.pages.count, 4, "16 labels");
    r.click(FillClick::Page { back: false });
    assert_eq!(r.click(open(4)), Do::Show, "password");
    let e = sheet(&r);
    assert_eq!(e.fields[0].label, Label::Table("password".to_owned()));
    assert!(e.fields[0].value.hidden(), "hidden as its label starts");
    assert_eq!(e.typing, Some(Typing::Value(0)));
    r.click(button(FillButton::AddField));
    assert_eq!(r.key(&PanelKey::Down), Do::Show);
    assert_eq!(r.key(&PanelKey::Enter), Do::Show, "email");
    assert_eq!(sheet(&r).fields[1].label, Label::Table("email".to_owned()));
}

#[test]
fn your_own_label_takes_letters_on_its_own_line() {
    let mut r = Rig::sample();
    new(&mut r, EntryKind::Login);
    assert_eq!(r.click(button(FillButton::OwnLabel)), Do::Show);
    assert_eq!(sheet(&r).typing, Some(Typing::Label(3)));
    r.text("Code");
    r.key(&PanelKey::Down);
    assert_eq!(sheet(&r).typing, Some(Typing::Value(3)));
    r.text("42");
    r.key(&PanelKey::Up);
    r.key(&PanelKey::Up);
    assert_eq!(
        sheet(&r).typing,
        Some(Typing::Value(2)),
        "from the own label to the line above"
    );
    r.click(at(3, FieldButton::Label));
    assert_eq!(sheet(&r).typing, Some(Typing::Label(3)));
    let table = r.click(at(0, FieldButton::Label));
    assert_eq!(table, Do::Stay, "a table label is not typed");
    r.click(button(FillButton::Name));
    r.text("Locker 2");
    assert_eq!(r.click(button(FillButton::Save)), Do::Saved);
    let fields = saved(&r, "Locker 2").fields();
    assert_eq!(fields.len(), 1, "the empty ones go");
    assert!(matches!(&fields[0].label, Label::Own(own) if own.expose() == "Code"));
    assert_eq!(fields[0].value.expose().as_str(), "42");
}

#[test]
fn add_a_field_backs_out_to_the_page() {
    let mut r = Rig::sample();
    new(&mut r, EntryKind::Other);
    r.click(button(FillButton::AddField));
    assert_eq!(r.click(button(FillButton::Back)), Do::Show);
    assert!(matches!(r.page.sheet, Sheet::Edit(_)));
    r.click(button(FillButton::AddField));
    assert_eq!(
        r.click(button(FillButton::OwnLabel)),
        Do::Show,
        "from the table too"
    );
    assert_eq!(sheet(&r).typing, Some(Typing::Label(0)));
}

#[test]
fn add_a_field_stops_at_max_fields_with_the_full_note() {
    let mut r = limited(Limits {
        max_fields: 3,
        ..LIMITS
    });
    new(&mut r, EntryKind::Login);
    for b in [FillButton::AddField, FillButton::OwnLabel] {
        assert_eq!(r.click(button(b)), Do::Show, "{b:?}");
        assert_eq!(r.page.note.map(|n| n.note), Some(FillNote::Full));
        assert_eq!(sheet(&r).fields.len(), 3);
    }
}

#[test]
fn make_one_fills_a_password_field_and_shows_it() {
    let mut r = Rig::sample();
    new(&mut r, EntryKind::Login);
    assert_eq!(r.click(at(1, FieldButton::Make)), Do::Show);
    let made = values(&r)[1].clone().unwrap();
    assert_eq!(made.chars().count(), CONFIG.make_chars);
    assert!(
        made.chars()
            .all(|c| groups().iter().any(|g| g.contains(&c)))
    );
    assert!(
        sheet(&r).fields[1].value.shown(),
        "shown so the user sees it"
    );
    assert_eq!(sheet(&r).typing, Some(Typing::Value(1)));
    assert_eq!(
        r.click(at(0, FieldButton::Make)),
        Do::Stay,
        "only a Password field"
    );
    r.random.fail_next();
    assert_eq!(
        r.click(at(1, FieldButton::Make)),
        Do::Stay,
        "no random bytes, nothing changes"
    );
    assert_eq!(values(&r)[1].as_deref(), Some(made.as_str()));
}

#[test]
fn the_hidden_flag_switches_and_a_switched_value_hides_at_once() {
    let mut r = Rig::sample();
    new(&mut r, EntryKind::Login);
    r.click(at(0, FieldButton::Value));
    r.text("sam");
    assert_eq!(r.click(at(0, FieldButton::Hidden)), Do::Show);
    assert!(sheet(&r).fields[0].value.hidden());
    assert_eq!(values(&r)[0], None);
    r.click(at(0, FieldButton::Show));
    r.click(at(0, FieldButton::Hidden));
    r.click(at(0, FieldButton::Hidden));
    assert_eq!(values(&r)[0], None, "hidden again, not shown");
    assert_eq!(r.click(at(2, FieldButton::Show)), Do::Stay, "not hidden");
    r.key(&PanelKey::Up);
    r.text("S");
    assert_eq!(r.click(button(FillButton::Save)), Do::Saved);
    assert!(saved(&r, "S").fields()[0].hidden);
}

#[test]
fn typing_into_a_hidden_line_tells_only_its_count_until_it_is_shown() {
    let mut r = Rig::sample();
    new(&mut r, EntryKind::Login);
    r.click(at(1, FieldButton::Value));
    let dots = |r: &Rig| sheet(r).fields[1].value.count();
    assert_eq!((values(&r)[1].as_deref(), dots(&r)), (Some(""), None));
    r.text("ب");
    assert_eq!((values(&r)[1].as_deref(), dots(&r)), (None, Some(1)));
    r.text("x");
    assert_eq!((values(&r)[1].as_deref(), dots(&r)), (None, Some(2)));
    r.click(at(1, FieldButton::Show));
    assert_eq!((values(&r)[1].as_deref(), dots(&r)), (Some("بx"), None));
    r.click(at(1, FieldButton::Show));
    r.key(&PanelKey::Erase);
    assert_eq!((values(&r)[1].as_deref(), dots(&r)), (None, Some(1)));
    r.key(&PanelKey::Erase);
    assert_eq!((values(&r)[1].as_deref(), dots(&r)), (Some(""), None));
}

#[test]
fn paste_asks_the_clipboard_for_that_field() {
    let mut r = Rig::sample();
    new(&mut r, EntryKind::Login);
    assert_eq!(r.click(at(1, FieldButton::Paste)), Do::Paste);
    assert_eq!(sheet(&r).typing, Some(Typing::Value(1)));
    assert_eq!(
        r.click(at(1, FieldButton::Insert)),
        Do::Stay,
        "the entry page's"
    );
}

#[test]
fn leaving_the_form_for_the_labels_table_hides_the_shown_values() {
    let mut r = Rig::sample();
    new(&mut r, EntryKind::Login);
    r.click(at(1, FieldButton::Value));
    r.text("pw");
    r.click(at(1, FieldButton::Show));
    assert_eq!(values(&r)[1].as_deref(), Some("pw"));
    r.click(button(FillButton::AddField));
    r.click(button(FillButton::Back));
    assert_eq!(values(&r)[1], None, "hidden again once the form closed");
}

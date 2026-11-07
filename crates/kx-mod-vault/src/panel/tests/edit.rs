//! New and Edit: the kind chips and starter fields, the highlighted line taking letters, Save, Cancel and Delete entry
//! asking once; `fields` tests the fields.

use kx_module_api::ports::{
    EditSheet, EntryKind, FieldButton, FillButton, FillClick, FillNote, Label, PanelKey, Sheet,
    Typing,
};

use super::super::rig::{CONFIG, Rig};
use super::super::{Do, PanelConfig};
use super::{button, open};
use crate::book::Limits;
use crate::entry::Entry;
use crate::fixture::LIMITS;

/// Field `field`'s `button`.
pub(super) fn at(field: u8, button: FieldButton) -> FillClick {
    FillClick::Field { field, button }
}

/// The New or Edit page shown.
pub(super) fn sheet(r: &Rig) -> &EditSheet {
    match &r.page.sheet {
        Sheet::Edit(e) => e,
        other => panic!("not New or Edit: {other:?}"),
    }
}

/// The values the page shows, `None` for the mask.
pub(super) fn values(r: &Rig) -> Vec<Option<String>> {
    let shown = sheet(r)
        .fields
        .iter()
        .map(|f| f.value.text().map(|t| t.expose().to_string()));
    shown.collect()
}

/// A New page of `kind`.
pub(super) fn new(r: &mut Rig, kind: EntryKind) {
    r.click(button(FillButton::New));
    assert_eq!(r.click(FillClick::Kind(kind)), Do::Show);
}

/// The saved entry named `name`.
pub(super) fn saved<'r>(r: &'r Rig, name: &str) -> &'r Entry {
    r.book.get(r.id(name)).unwrap()
}

/// A rig whose limits are `limits`.
pub(super) fn limited(limits: Limits) -> Rig {
    Rig::with(
        PanelConfig { limits, ..CONFIG },
        super::super::rig::sample(),
    )
}

#[test]
fn new_starts_with_the_kind_chips_then_the_kinds_starter_fields() {
    let mut r = Rig::sample();
    assert_eq!(r.click(button(FillButton::New)), Do::Show);
    let e = sheet(&r);
    assert!(e.new && e.kind.is_none() && e.fields.is_empty() && e.typing.is_none());
    assert_eq!(r.text("x"), Do::Stay, "nothing takes letters before a kind");
    assert_eq!(
        r.click(button(FillButton::Save)),
        Do::Show,
        "the blank note"
    );
    assert_eq!(r.page.note.map(|n| n.note), Some(FillNote::Refused));
    assert_eq!(r.click(FillClick::Kind(EntryKind::Login)), Do::Show);
    let e = sheet(&r);
    assert_eq!(e.kind, Some(EntryKind::Login));
    let ids = ["user", "password", "website"].map(|id| Label::Table(id.to_owned()));
    let labels: Vec<Label> = e.fields.iter().map(|f| f.label.clone()).collect();
    assert_eq!(labels, ids);
    let hidden: Vec<bool> = e.fields.iter().map(|f| f.value.hidden()).collect();
    assert_eq!(hidden, [false, true, false]);
    assert!(e.fields.iter().all(|f| f.value.switches()));
    let make: Vec<bool> = e.fields.iter().map(|f| f.make).collect();
    assert_eq!(make, [false, true, false], "Make one on the Password field");
    assert_eq!(e.typing, Some(Typing::Name));
    assert_eq!(
        r.click(FillClick::Kind(EntryKind::Note)),
        Do::Stay,
        "picked once"
    );
    assert_eq!(r.click(button(FillButton::New)), Do::Stay, "already New");
}

#[test]
fn the_highlighted_line_takes_letters_space_and_backspace() {
    let mut r = Rig::sample();
    new(&mut r, EntryKind::Login);
    r.text("My");
    assert_eq!(r.key(&PanelKey::Tick), Do::Show);
    r.text("Banks");
    assert_eq!(r.key(&PanelKey::Erase), Do::Show);
    assert_eq!(sheet(&r).name.expose(), "My Bank");
    assert_eq!(r.page.search.expose(), "", "never the search");
    assert_eq!(r.key(&PanelKey::Down), Do::Show);
    assert_eq!(sheet(&r).typing, Some(Typing::Value(0)));
    assert_eq!(r.page.highlight, Some(0));
    r.text("sam");
    r.key(&PanelKey::Down);
    r.text("pw");
    assert_eq!(values(&r), [Some("sam".into()), None, Some(String::new())]);
    assert_eq!(r.click(at(1, FieldButton::Show)), Do::Show);
    assert_eq!(values(&r)[1].as_deref(), Some("pw"));
    r.key(&PanelKey::Down);
    assert_eq!(r.key(&PanelKey::Down), Do::Stay, "the last line");
    for _ in 0..3 {
        r.key(&PanelKey::Up);
    }
    assert_eq!(sheet(&r).typing, Some(Typing::Name));
    assert_eq!(r.key(&PanelKey::Up), Do::Stay);
    assert_eq!(r.click(at(2, FieldButton::Value)), Do::Show);
    assert_eq!(sheet(&r).typing, Some(Typing::Value(2)));
    assert_eq!(r.click(button(FillButton::Name)), Do::Show);
    assert_eq!(sheet(&r).typing, Some(Typing::Name));
    for key in [PanelKey::Enter, PanelKey::Right] {
        assert_eq!(r.key(&key), Do::Stay, "{key:?}, the caret at the end");
    }
}

#[test]
fn a_line_refuses_what_no_field_holds_with_the_refused_note() {
    let mut r = limited(Limits {
        field_chars: 8,
        ..LIMITS
    });
    new(&mut r, EntryKind::Note);
    assert_eq!(r.text("123456789"), Do::Show);
    assert_eq!(sheet(&r).name.expose(), "12345678");
    assert_eq!(r.page.note.map(|n| n.note), Some(FillNote::Refused));
    r.key(&PanelKey::Down);
    assert_eq!(r.page.note, None);
    assert_eq!(r.text("a\u{202e}b"), Do::Show);
    assert_eq!(values(&r)[0].as_deref(), Some("ab"));
    assert_eq!(r.page.note.map(|n| n.note), Some(FillNote::Refused));
}

#[test]
fn save_keeps_the_new_entry_and_shows_its_page() {
    let mut r = Rig::sample();
    new(&mut r, EntryKind::Login);
    r.text("Shop");
    r.key(&PanelKey::Down);
    r.text("sam");
    r.key(&PanelKey::Down);
    r.text("pw");
    assert_eq!(r.click(button(FillButton::Save)), Do::Saved);
    assert!(matches!(&r.page.sheet, Sheet::Entry(e) if e.name.expose() == "Shop"));
    let shop = saved(&r, "Shop");
    assert_eq!(shop.kind(), EntryKind::Login);
    assert_eq!(shop.fields().len(), 2, "the empty website is not kept");
    assert!(shop.fields()[1].hidden && shop.fields()[1].value.expose().as_str() == "pw");
    r.key(&PanelKey::Esc);
    assert!(r.names().contains(&"Shop".to_owned()));
}

#[test]
fn a_blank_new_entry_saves_nothing_and_says_so() {
    let mut r = Rig::sample();
    new(&mut r, EntryKind::Person);
    assert_eq!(r.click(button(FillButton::Save)), Do::Show);
    assert_eq!(r.book.len(), 6);
    assert_eq!(r.page.note.map(|n| n.note), Some(FillNote::Refused));
    assert!(matches!(r.page.sheet, Sheet::Edit(_)), "the form stays");
}

#[test]
fn a_refused_save_keeps_the_page_and_its_typing_with_the_note() {
    let mut r = limited(Limits {
        max_entries: 6,
        ..LIMITS
    });
    new(&mut r, EntryKind::Note);
    r.text("Seventh");
    assert_eq!(r.click(button(FillButton::Save)), Do::Show);
    assert_eq!(r.page.note.map(|n| n.note), Some(FillNote::Full));
    assert_eq!(sheet(&r).name.expose(), "Seventh");
    assert_eq!(r.book.len(), 6);
}

#[test]
fn edit_loads_the_entry_and_save_puts_it_back_under_its_id() {
    let mut r = Rig::sample();
    let bank = r.id("Bank");
    r.click(open(1));
    assert_eq!(r.click(button(FillButton::Edit)), Do::Show);
    let e = sheet(&r);
    assert!(!e.new && e.kind == Some(EntryKind::Login) && e.name.expose() == "Bank");
    assert_eq!(
        values(&r),
        [
            Some("alex@example.com".into()),
            None,
            Some("bank.example".into())
        ]
    );
    r.click(at(2, FieldButton::Value));
    for _ in 0.."example".len() {
        r.key(&PanelKey::Erase);
    }
    r.text("fr");
    assert_eq!(r.click(button(FillButton::Save)), Do::Saved);
    assert!(matches!(r.page.sheet, Sheet::Entry(_)));
    let entry = r.book.get(bank).unwrap();
    assert_eq!(entry.fields()[2].value.expose().as_str(), "bank.fr");
    assert_eq!(
        entry.fields()[1].value.expose().as_str(),
        "hunter2",
        "a masked value is kept"
    );
}

#[test]
fn cancel_leaves_without_saving() {
    let mut r = Rig::sample();
    new(&mut r, EntryKind::Note);
    r.text("Draft");
    assert_eq!(r.click(button(FillButton::Cancel)), Do::Show);
    assert!(matches!(r.page.sheet, Sheet::List(_)));
    assert_eq!(r.book.len(), 6);
    r.click(open(1));
    r.click(button(FillButton::Edit));
    r.text("!");
    assert_eq!(r.click(button(FillButton::Cancel)), Do::Show);
    assert!(matches!(&r.page.sheet, Sheet::Entry(e) if e.name.expose() == "Bank"));
    assert_eq!(saved(&r, "Bank").name().expose(), "Bank");
}

#[test]
fn delete_asks_once_and_esc_or_no_keeps_the_entry() {
    let mut r = Rig::sample();
    let bank = r.id("Bank");
    r.click(open(1));
    r.click(button(FillButton::Edit));
    assert_eq!(r.click(button(FillButton::Delete)), Do::Show);
    assert!(sheet(&r).confirm);
    assert_eq!(r.text("x"), Do::Stay, "the question takes only its answer");
    assert_eq!(r.click(button(FillButton::Save)), Do::Stay);
    assert_eq!(r.click(button(FillButton::Delete)), Do::Stay, "asked once");
    assert_eq!(r.key(&PanelKey::Enter), Do::Stay);
    assert_eq!(r.key(&PanelKey::Esc), Do::Show);
    assert!(!sheet(&r).confirm && r.book.get(bank).is_some());
    r.click(button(FillButton::Delete));
    assert_eq!(r.click(FillClick::Answer(false)), Do::Show);
    assert!(!sheet(&r).confirm && r.book.get(bank).is_some());
    r.click(button(FillButton::Delete));
    assert_eq!(r.click(FillClick::Answer(true)), Do::Saved);
    assert!(r.book.get(bank).is_none());
    assert!(matches!(r.page.sheet, Sheet::List(_)));
    assert_eq!(r.click(FillClick::Answer(true)), Do::Stay, "no question");
}

#[test]
fn the_question_takes_only_its_answer_and_close() {
    let mut r = Rig::sample();
    r.click(open(1));
    r.click(button(FillButton::Edit));
    r.click(button(FillButton::Delete));
    let others = [
        at(1, FieldButton::Show),
        at(0, FieldButton::Value),
        FillClick::Page { back: false },
        button(FillButton::Cancel),
        button(FillButton::Back),
        button(FillButton::AddField),
        button(FillButton::OwnLabel),
        button(FillButton::New),
    ];
    for click in others {
        assert_eq!(r.click(click), Do::Stay, "{click:?}");
        assert!(sheet(&r).confirm);
    }
    assert_eq!(r.click(button(FillButton::Close)), Do::Close);
    assert_eq!(r.book.len(), 6, "a close is never a yes");
}

#[test]
fn new_offers_no_delete() {
    let mut r = Rig::sample();
    new(&mut r, EntryKind::Note);
    assert_eq!(r.click(button(FillButton::Delete)), Do::Stay);
    assert!(!sheet(&r).confirm);
}

#[test]
fn an_edit_whose_entry_left_saves_nothing_and_shows_the_list() {
    let mut r = Rig::sample();
    let bank = r.id("Bank");
    r.click(open(1));
    r.click(button(FillButton::Edit));
    r.book.remove(bank);
    assert_eq!(r.click(button(FillButton::Save)), Do::Show);
    assert!(matches!(r.page.sheet, Sheet::List(_)));
    assert_eq!(r.book.len(), 5);
}

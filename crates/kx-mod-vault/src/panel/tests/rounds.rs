//! Clicks from an older view: those that name a place on the page, save, delete, answer or type are dropped; those
//! that only move between pages are kept, as they act the same on any view.

use kx_module_api::ports::{
    EntryKind, FieldButton, FillButton, FillClick, FillFilter, RowButton, Sheet,
};

use super::super::rig::{NOW, Rig};
use super::super::{Do, Fill, Part};
use super::{button, open};

/// Row `row`'s `button`.
fn row(row: u8, button: RowButton) -> FillClick {
    FillClick::Row { row, button }
}

#[test]
fn an_older_views_row_or_field_click_is_dropped() {
    let mut r = Rig::sample();
    assert_eq!(r.stale(row(1, RowButton::User)), Do::Stay);
    assert_eq!(r.stale(open(1)), Do::Stay);
    assert!(matches!(r.page.sheet, Sheet::List(_)));
    r.click(open(1));
    let show = FillClick::Field {
        field: 1,
        button: FieldButton::Show,
    };
    assert_eq!(r.stale(show), Do::Stay);
    let insert = FillClick::Field {
        field: 0,
        button: FieldButton::Insert,
    };
    assert_eq!(r.stale(insert), Do::Stay);
    assert_eq!(r.stale(button(FillButton::Edit)), Do::Stay);
    assert!(matches!(r.page.sheet, Sheet::Entry(_)));
}

#[test]
fn an_older_views_save_cancel_delete_or_answer_is_dropped() {
    let mut r = Rig::sample();
    r.click(open(1));
    r.click(button(FillButton::Edit));
    r.text("!");
    for b in [FillButton::Save, FillButton::Cancel, FillButton::Delete] {
        assert_eq!(r.stale(button(b)), Do::Stay, "{b:?}");
    }
    r.click(button(FillButton::Delete));
    assert_eq!(r.stale(FillClick::Answer(true)), Do::Stay);
    assert_eq!(r.book.len(), 6);
    assert!(matches!(&r.page.sheet, Sheet::Edit(e) if e.confirm));
    let make = FillClick::Field {
        field: 1,
        button: FieldButton::Make,
    };
    r.click(FillClick::Answer(false));
    assert_eq!(r.stale(make), Do::Stay);
}

#[test]
fn an_older_views_moves_between_pages_are_kept() {
    let mut r = Rig::sample();
    assert_eq!(r.stale(FillClick::Filter(FillFilter::Logins)), Do::Show);
    assert_eq!(r.stale(FillClick::Filter(FillFilter::All)), Do::Show);
    assert_eq!(r.stale(FillClick::Page { back: false }), Do::Show);
    assert_eq!(r.stale(FillClick::Rows(3)), Do::Show);
    assert_eq!(r.stale(button(FillButton::New)), Do::Show);
    assert_eq!(r.stale(FillClick::Kind(EntryKind::Note)), Do::Show);
    assert_eq!(r.stale(button(FillButton::AddField)), Do::Show);
    assert_eq!(r.stale(button(FillButton::Back)), Do::Show);
    assert_eq!(r.stale(button(FillButton::Name)), Do::Show);
    assert_eq!(r.stale(button(FillButton::Close)), Do::Close);
}

#[test]
fn a_row_click_names_the_row_the_user_saw_even_as_the_list_moves() {
    let mut r = Rig::sample();
    let (bank, gate) = (r.id("Bank"), r.id("Gate"));
    r.book.use_at(gate, NOW);
    let want = Do::Fill(Fill {
        id: bank,
        parts: vec![Part::Value(0)],
    });
    assert_eq!(r.click(row(1, RowButton::User)), want, "row 1 showed Bank");
    assert_eq!(r.names()[0], "Gate");
    r.book.remove(bank);
    let before = r.page.round;
    assert_eq!(
        r.click(row(1, RowButton::User)),
        Do::Stay,
        "Alex now shows there, with no User"
    );
    assert!(r.page.round > before);
}

#[test]
fn a_row_click_for_an_entry_gone_since_its_view_does_nothing() {
    let mut r = Rig::sample();
    let bank = r.id("Bank");
    r.book.remove(bank);
    assert_eq!(
        r.click(row(1, RowButton::Both)),
        Do::Stay,
        "nothing to fill"
    );
    assert!(matches!(r.page.sheet, Sheet::List(_)));
}

#[test]
fn a_shaky_second_your_own_label_from_the_older_view_adds_no_second_field() {
    let mut r = Rig::sample();
    r.click(button(FillButton::New));
    r.click(FillClick::Kind(EntryKind::Other));
    assert_eq!(r.click(button(FillButton::OwnLabel)), Do::Show);
    assert_eq!(r.stale(button(FillButton::OwnLabel)), Do::Stay);
    assert!(matches!(&r.page.sheet, Sheet::Edit(e) if e.fields.len() == 1));
}

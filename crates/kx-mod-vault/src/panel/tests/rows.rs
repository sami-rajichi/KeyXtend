//! The entry list's rows: the pages of 5 with their count, the row buttons by kind and their fills, Enter, and the
//! highlight following its entry.

use kx_module_api::ports::{EntryKind, FillClick, Hint, PanelKey, RowButton, Sheet};

use super::super::Do;
use super::super::Part;
use super::super::rig::{NOW, Rig};
use super::list::{fill, row};
use super::open;
use crate::fixture::{draft, field};

#[test]
fn the_list_shows_five_rows_a_page_with_the_count() {
    let notes = (1..=12).map(|n| {
        draft(
            &format!("n{n:02}"),
            EntryKind::Note,
            vec![field("text", "x", false)],
        )
    });
    let mut r = Rig::new(notes.collect());
    assert_eq!(r.rows().len(), 5);
    assert_eq!((r.page.pages.at, r.page.pages.count), (0, 3));
    assert_eq!(r.click(FillClick::Page { back: true }), Do::Stay);
    for _ in 0..5 {
        assert_eq!(r.key(&PanelKey::Down), Do::Show);
    }
    assert_eq!(
        (r.page.pages.at, r.page.highlight),
        (1, Some(0)),
        "the page follows"
    );
    assert_eq!(r.names()[0], "n06");
    assert_eq!(r.click(FillClick::Page { back: false }), Do::Show);
    assert_eq!(r.names(), ["n11", "n12"]);
    assert_eq!(r.page.highlight, Some(0));
    assert_eq!(r.click(FillClick::Page { back: false }), Do::Stay);
    r.key(&PanelKey::Down);
    assert_eq!(r.key(&PanelKey::Down), Do::Stay, "the last row");
    assert_eq!(r.click(FillClick::Page { back: true }), Do::Show);
    assert_eq!((r.page.pages.at, r.page.highlight), (1, Some(0)));
}

#[test]
fn each_row_offers_its_kinds_buttons_then_open_with_its_hint() {
    use RowButton::{Both, Insert, Open, Password, User};
    let r = Rig::sample();
    let want: [&[RowButton]; 5] = [
        &[Open],
        &[User, Password, Both, Open],
        &[Insert, Open],
        &[Open],
        &[Insert, Open],
    ];
    for (row, buttons) in r.rows().iter().zip(want) {
        assert_eq!(row.buttons, buttons, "{}", row.name.expose());
    }
    let hints: Vec<&Hint> = r.rows().iter().map(|row| &row.hint).collect();
    assert_eq!(*hints[0], Hint::Fields(3));
    let Hint::Values(bank) = hints[1] else {
        panic!()
    };
    assert_eq!(
        bank[0].text().map(|t| t.expose().as_str()),
        Some("alex@example.com")
    );
    assert!(bank[1].hidden() && bank[1].text().is_none(), "the mask");
    assert!(matches!(hints[2], Hint::Values(v) if v[0].text().is_some()));
    assert_eq!(*hints[3], Hint::Fields(2));
    assert!(matches!(hints[4], Hint::Values(v) if v[0].text().is_none()));
    assert_eq!(r.rows()[1].kind, EntryKind::Login);
}

#[test]
fn a_login_missing_half_of_its_pair_follows_the_general_rule() {
    let r = Rig::new(vec![
        draft(
            "A",
            EntryKind::Login,
            vec![field("user", "u", false), field("website", "w", false)],
        ),
        draft("B", EntryKind::Login, vec![field("password", "p", true)]),
    ]);
    assert_eq!(r.rows()[0].buttons, [RowButton::Open]);
    assert_eq!(r.rows()[1].buttons, [RowButton::Insert, RowButton::Open]);
}

#[test]
fn row_buttons_fill_the_right_entry_and_fields() {
    let mut r = Rig::sample();
    let bank = r.id("Bank");
    assert_eq!(
        r.click(row(1, RowButton::User)),
        fill(bank, vec![Part::Value(0)])
    );
    assert_eq!(
        r.click(row(1, RowButton::Password)),
        fill(bank, vec![Part::Value(1)])
    );
    let both = vec![Part::Value(0), Part::Tab, Part::Value(1)];
    assert_eq!(r.click(row(1, RowButton::Both)), fill(bank, both));
    assert_eq!(r.page.highlight, Some(1), "a click highlights its row");
    let gate = r.id("Gate");
    assert_eq!(
        r.click(row(2, RowButton::Insert)),
        fill(gate, vec![Part::Value(0)])
    );
    assert!(
        matches!(r.page.sheet, Sheet::List(_)),
        "a fill keeps the list"
    );
    assert_eq!(r.click(open(1)), Do::Show);
    assert!(matches!(&r.page.sheet, Sheet::Entry(e) if e.name.expose() == "Bank"));
}

#[test]
fn only_a_rows_own_buttons_act_so_a_slip_never_types() {
    let mut r = Rig::sample();
    let others = [
        RowButton::User,
        RowButton::Password,
        RowButton::Both,
        RowButton::Insert,
        RowButton::Remove,
    ];
    for button in others {
        assert_eq!(r.click(row(0, button)), Do::Stay, "{button:?}");
    }
    assert_eq!(
        r.click(row(1, RowButton::Insert)),
        Do::Stay,
        "a login has no Insert"
    );
    assert_eq!(r.click(row(7, RowButton::Open)), Do::Stay, "past the page");
    assert!(matches!(r.page.sheet, Sheet::List(_)));
    assert_eq!(r.page.highlight, Some(0));
}

#[test]
fn enter_presses_the_highlighted_rows_first_button() {
    let mut r = Rig::sample();
    r.key(&PanelKey::Down);
    assert_eq!(
        r.key(&PanelKey::Enter),
        fill(r.id("Bank"), vec![Part::Value(0)])
    );
    r.key(&PanelKey::Up);
    assert_eq!(r.key(&PanelKey::Enter), Do::Show);
    assert!(matches!(&r.page.sheet, Sheet::Entry(e) if e.name.expose() == "Alex Martin"));
}

#[test]
fn the_highlight_stays_on_its_entry_as_the_list_moves() {
    let mut r = Rig::sample();
    r.key(&PanelKey::Down);
    r.key(&PanelKey::Down);
    let gate = r.id("Gate");
    r.book.use_at(gate, NOW);
    r.draw();
    assert_eq!(r.names()[0], "Gate", "the last used first");
    assert_eq!(r.page.highlight, Some(0));
    r.book.remove(gate);
    r.draw();
    assert_eq!(r.page.highlight, Some(0), "kept inside the list");
}

//! The list pages: the filters with Other under All, By label and its counts, and the search; `rows` tests the rows.

use kx_module_api::ports::{
    EntryKind, FillClick, FillFilter, Label, LabelCount, PanelKey, RowButton, Sheet,
};

use super::super::rig::{NOW, Rig};
use super::super::{Do, Fill, Part};
use super::open;
use crate::find::LabelNames;
use crate::fixture::{draft, field};

/// Row `row`'s `button`.
pub(super) fn row(row: u8, button: RowButton) -> FillClick {
    FillClick::Row { row, button }
}

/// The table label `id`.
fn table(id: &str) -> Label {
    Label::Table(id.to_owned())
}

/// The fill of entry `id` with `parts`.
pub(super) fn fill(id: u64, parts: Vec<Part>) -> Do {
    Do::Fill(Fill { id, parts })
}

/// The labels By label shows, with their counts.
fn labels(r: &Rig) -> Vec<(Label, u32)> {
    match &r.page.sheet {
        Sheet::Labels(rows) => rows
            .iter()
            .map(|c: &LabelCount| (c.label.clone(), c.count))
            .collect(),
        other => panic!("not By label: {other:?}"),
    }
}

#[test]
fn all_lists_every_entry_a_to_z_other_ones_among_them() {
    let mut r = Rig::sample();
    assert_eq!(r.page.filter, FillFilter::All);
    assert_eq!(
        r.names(),
        ["Alex Martin", "Bank", "Gate", "Home", "Library"]
    );
    assert_eq!((r.page.pages.at, r.page.pages.count), (0, 2));
    r.click(FillClick::Page { back: false });
    assert_eq!(r.names(), ["Locker"], "Other shows under All");
}

#[test]
fn each_filter_lists_its_own_kinds() {
    let mut r = Rig::sample();
    let want = [
        (FillFilter::People, "Alex Martin"),
        (FillFilter::Logins, "Bank"),
        (FillFilter::Wifi, "Home"),
        (FillFilter::Cards, "Library"),
        (FillFilter::Notes, "Gate"),
    ];
    for (filter, name) in want {
        assert_eq!(r.click(FillClick::Filter(filter)), Do::Show);
        assert_eq!(r.names(), [name], "{filter:?}");
        assert_eq!(r.page.filter, filter);
    }
    assert_eq!(r.click(FillClick::Filter(FillFilter::Notes)), Do::Stay);
}

#[test]
fn left_and_right_step_through_the_filters_and_stop_at_the_ends() {
    let mut r = Rig::sample();
    assert_eq!(r.key(&PanelKey::Left), Do::Stay);
    for f in &FillFilter::ALL[1..] {
        assert_eq!(r.key(&PanelKey::Right), Do::Show);
        assert_eq!(r.page.filter, *f);
    }
    assert_eq!(r.key(&PanelKey::Right), Do::Stay);
    assert_eq!(r.key(&PanelKey::Left), Do::Show);
    assert_eq!(r.page.filter, FillFilter::Notes);
}

#[test]
fn by_label_counts_the_values_under_each_label_in_use() {
    let mut r = Rig::sample();
    r.click(FillClick::Filter(FillFilter::Labels));
    let first = ["full-name", "email", "phone", "user", "password"];
    let want: Vec<(Label, u32)> = first
        .iter()
        .map(|id| (table(id), if *id == "password" { 2 } else { 1 }))
        .collect();
    assert_eq!(labels(&r), want);
    assert_eq!(r.page.pages.count, 2);
    r.click(FillClick::Page { back: false });
    let shown = labels(&r);
    let ids: Vec<Label> = ["number", "website", "network", "text"].map(table).into();
    assert_eq!(
        shown[..4]
            .iter()
            .map(|(l, _)| l.clone())
            .collect::<Vec<_>>(),
        ids
    );
    assert!(matches!(&shown[4], (Label::Own(own), 1) if own.expose() == "Code"));
}

#[test]
fn a_label_lists_its_values_masked_when_hidden_with_insert() {
    let mut r = Rig::sample();
    r.click(FillClick::Filter(FillFilter::Labels));
    assert_eq!(r.click(open(4)), Do::Show, "password");
    let Sheet::Values { label, rows } = &r.page.sheet else {
        panic!("{:?}", r.page.sheet)
    };
    assert_eq!(*label, table("password"));
    let names: Vec<&str> = rows.iter().map(|v| v.name.expose().as_str()).collect();
    assert_eq!(names, ["Bank", "Home"]);
    assert!(
        rows.iter()
            .all(|v| v.value.hidden() && v.value.text().is_none())
    );
    let home = r.id("Home");
    assert_eq!(
        r.click(row(1, RowButton::Insert)),
        fill(home, vec![Part::Value(1)])
    );
    assert_eq!(r.key(&PanelKey::Up), Do::Show);
    assert_eq!(
        r.key(&PanelKey::Enter),
        fill(r.id("Bank"), vec![Part::Value(1)])
    );
    assert_eq!(
        r.click(row(0, RowButton::Open)),
        Do::Stay,
        "values offer Insert alone"
    );
}

#[test]
fn enter_on_by_label_opens_the_highlighted_label() {
    let mut r = Rig::sample();
    r.click(FillClick::Filter(FillFilter::Labels));
    r.key(&PanelKey::Down);
    assert_eq!(r.key(&PanelKey::Enter), Do::Show);
    assert!(matches!(&r.page.sheet, Sheet::Values { label, .. } if *label == table("email")));
}

#[test]
fn the_search_folds_and_never_matches_a_hidden_value() {
    let mut r = Rig::new(vec![
        draft(
            "École du soir",
            EntryKind::Note,
            vec![field("text", "salle 4", false)],
        ),
        draft(
            "Site",
            EntryKind::Login,
            vec![
                field("user", "sam", false),
                field("password", "ecole-secret", true),
            ],
        ),
    ]);
    assert_eq!(r.text("ECOLE"), Do::Show);
    assert_eq!(r.names(), ["École du soir"], "never the hidden value");
    r.key(&PanelKey::Esc);
    r.text("SALLE");
    assert_eq!(r.names(), ["École du soir"], "a shown value");
    r.key(&PanelKey::Esc);
    r.text("secret");
    assert!(r.names().is_empty());
}

#[test]
fn table_labels_match_by_the_names_the_ui_gives() {
    let mut r = Rig::sample();
    r.text("courriel");
    assert!(r.names().is_empty(), "no names given yet");
    let names = LabelNames::from([("email".to_owned(), "Courriel".to_owned())]);
    r.panel.set_names(names);
    r.draw();
    assert_eq!(r.names(), ["Alex Martin"]);
    r.click(FillClick::Filter(FillFilter::Labels));
    let ids: Vec<Label> = labels(&r).into_iter().map(|(l, _)| l).collect();
    assert_eq!(
        ids,
        ["full-name", "email", "phone"].map(table),
        "the search narrows By label"
    );
}

#[test]
fn letters_space_and_backspace_go_into_the_search() {
    let mut r = Rig::sample();
    r.key(&PanelKey::Down);
    assert_eq!(r.text("x"), Do::Show);
    assert_eq!(r.key(&PanelKey::Tick), Do::Show);
    r.text("m");
    assert_eq!(r.page.search.expose(), "x m");
    assert_eq!(r.names(), ["Alex Martin"]);
    assert_eq!(r.page.highlight, Some(0), "a new search starts at the top");
    assert_eq!(r.key(&PanelKey::Erase), Do::Show);
    assert_eq!(r.page.search.expose(), "x ");
    r.key(&PanelKey::Erase);
    r.key(&PanelKey::Erase);
    assert_eq!(r.key(&PanelKey::Erase), Do::Stay);
    assert_eq!(
        r.text("\n"),
        Do::Stay,
        "a line break never joins the search"
    );
}

#[test]
fn a_fill_moves_no_row_while_open_and_its_order_shows_on_the_next_open() {
    let mut r = Rig::sample();
    let before = r.names();
    let gate = r.id("Gate");
    assert!(r.panel.filled(&mut r.book, gate, NOW));
    r.draw();
    assert_eq!(r.names(), before, "rows stay still under the finger");
    r.panel.close();
    r.panel.open();
    r.draw();
    assert_eq!(r.names()[0], "Gate", "the last used first on the next open");
    assert!(
        r.book.get(gate).is_some_and(|e| e.used() >= Some(NOW)),
        "used now"
    );
}

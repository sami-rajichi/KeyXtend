//! The panel over a small notebook with the plan's page of 5 rows: `list` tests the list pages, `entry` the entry
//! page, `edit` New and Edit, `paste` Paste, `rounds` the clicks of older views, `caret` the lines' carets; here are
//! Esc's steps and the panel's life.

mod caret;
mod edit;
mod entry;
mod fields;
mod line;
mod list;
mod paste;
mod rounds;
mod rows;

use kx_module_api::REDACTED_MARKER;
use kx_module_api::ports::{
    EntryKind, FieldButton, FillAct, FillAsk, FillButton, FillClick, FillClose, FillFilter,
    FillNote, FillOpen, FillPanel, PanelKey, RowButton, Sheet,
};

use super::rig::{CONFIG, Rig};
use super::*;

/// The open button of row `row`.
pub(super) fn open(row: u8) -> FillClick {
    FillClick::Row {
        row,
        button: RowButton::Open,
    }
}

/// Panel button `button`.
pub(super) const fn button(button: FillButton) -> FillClick {
    FillClick::Button(button)
}

#[test]
fn esc_backs_out_step_by_step_search_then_page_then_panel() {
    let mut r = Rig::sample();
    r.text("a");
    assert_eq!(r.click(open(1)), Do::Show, "Bank");
    r.click(button(FillButton::Edit));
    r.click(button(FillButton::AddField));
    assert!(matches!(r.page.sheet, Sheet::AddField(_)));
    assert_eq!(r.key(&PanelKey::Esc), Do::Show);
    assert!(matches!(&r.page.sheet, Sheet::Edit(e) if !e.confirm));
    r.click(button(FillButton::Delete));
    assert_eq!(r.key(&PanelKey::Esc), Do::Show, "No");
    assert!(matches!(&r.page.sheet, Sheet::Edit(e) if !e.confirm));
    assert_eq!(r.key(&PanelKey::Esc), Do::Show, "Cancel");
    assert!(matches!(r.page.sheet, Sheet::Entry(_)));
    assert_eq!(r.key(&PanelKey::Esc), Do::Show, "back to the list");
    assert_eq!(r.page.search.expose(), "a", "the list keeps its search");
    assert_eq!(r.key(&PanelKey::Esc), Do::Show);
    assert_eq!(r.page.search.expose(), "");
    assert_eq!(r.key(&PanelKey::Esc), Do::Close);
    assert!(!r.panel.is_open());
}

#[test]
fn the_values_page_steps_back_with_one_esc_and_keeps_the_search_it_does_not_draw() {
    let mut r = Rig::sample();
    r.click(FillClick::Filter(FillFilter::Labels));
    r.text("e");
    r.click(open(0));
    assert!(matches!(r.page.sheet, Sheet::Values { .. }));
    assert_eq!(r.key(&PanelKey::Esc), Do::Show);
    assert!(matches!(r.page.sheet, Sheet::Labels(_)), "one press");
    assert_eq!(r.page.search.expose(), "e", "the search is kept");
    assert_eq!(r.key(&PanelKey::Esc), Do::Show);
    assert_eq!(r.page.search.expose(), "", "only the list clears it");
    assert_eq!(r.key(&PanelKey::Esc), Do::Close);
}

#[test]
fn back_moves_between_pages_but_never_cancels_or_closes() {
    let mut r = Rig::sample();
    assert_eq!(r.click(button(FillButton::Back)), Do::Stay);
    r.click(open(1));
    assert_eq!(r.click(button(FillButton::Back)), Do::Show);
    assert!(matches!(r.page.sheet, Sheet::List(_)));
    r.click(button(FillButton::New));
    r.click(FillClick::Kind(EntryKind::Note));
    r.text("kept");
    assert_eq!(
        r.click(button(FillButton::Back)),
        Do::Stay,
        "only Cancel or Esc leaves New"
    );
    assert!(matches!(&r.page.sheet, Sheet::Edit(e) if e.name.expose() == "kept"));
}

#[test]
fn a_closed_panel_takes_nothing_and_opens_fresh() {
    let mut r = Rig::sample();
    r.click(FillClick::Filter(FillFilter::Logins));
    r.text("ban");
    assert_eq!(r.click(button(FillButton::Close)), Do::Close);
    assert!(!r.panel.is_open());
    assert_eq!(r.key(&PanelKey::Down), Do::Stay);
    assert_eq!(r.click(open(0)), Do::Stay);
    r.panel.open();
    r.draw();
    assert_eq!(r.page.filter, FillFilter::All);
    assert_eq!(r.page.search.expose(), "");
    assert_eq!(r.page.highlight, Some(0));
}

#[test]
fn the_rounds_go_on_counting_across_a_close() {
    let mut r = Rig::sample();
    let before = r.page.round;
    r.panel.close();
    r.panel.open();
    assert!(r.draw().round > before);
}

#[test]
fn a_note_shows_until_the_next_change() {
    let mut r = Rig::sample();
    r.panel.say(FillNote::Typed);
    r.draw();
    assert_eq!(r.page.note.map(|n| n.note), Some(FillNote::Typed));
    assert_eq!(r.key(&PanelKey::Up), Do::Stay);
    assert!(r.page.note.is_some(), "nothing changed");
    r.key(&PanelKey::Down);
    assert_eq!(r.page.note, None);
    r.panel.say(FillNote::Typed);
    r.panel.close();
    r.panel.open();
    assert_eq!(r.draw().note, None, "a close forgets it");
}

#[test]
fn an_empty_notebook_invites_a_first_entry() {
    let mut r = Rig::new(Vec::new());
    assert!(r.page.empty);
    assert!(matches!(&r.page.sheet, Sheet::List(rows) if rows.is_empty()));
    assert_eq!(r.page.highlight, None);
    assert_eq!((r.page.pages.at, r.page.pages.count), (0, 1));
    for key in [PanelKey::Down, PanelKey::Up, PanelKey::Enter] {
        assert_eq!(r.key(&key), Do::Stay);
    }
    assert!(!Rig::sample().page.empty);
}

#[test]
fn the_ui_rows_page_the_lists_and_zero_counts_as_one() {
    let mut r = Rig::sample();
    assert_eq!(r.click(FillClick::Rows(3)), Do::Show);
    assert_eq!((r.rows().len(), r.page.pages.count), (3, 2));
    assert_eq!(r.click(FillClick::Rows(3)), Do::Stay);
    r.click(FillClick::Rows(0));
    assert_eq!((r.rows().len(), r.page.pages.count), (1, 6));
    r.panel.close();
    let mut desk = Desk {
        book: &mut r.book,
        random: &r.random,
        now: super::rig::NOW,
    };
    let rows = FillAct {
        round: 0,
        click: FillClick::Rows(5),
    };
    assert_eq!(r.panel.act(rows, &mut desk), Do::Stay, "kept while closed");
    r.panel.open();
    assert_eq!(r.draw().pages.count, 2);
}

#[test]
fn debug_prints_no_value_name_or_search() {
    let mut r = Rig::sample();
    r.text("Ban");
    r.click(open(0));
    r.click(FillClick::Field {
        field: 1,
        button: FieldButton::Show,
    });
    r.click(button(FillButton::Edit));
    let shown = format!("{:?}", r.panel);
    for secret in ["hunter2", "Bank", "alex@", "Ban"] {
        assert!(!shown.contains(secret), "{secret}: {shown}");
    }
    assert!(shown.contains(REDACTED_MARKER));
}

#[test]
fn the_config_comes_from_the_settings() {
    let settings = crate::settings::VaultSettings::from_defaults().unwrap();
    let cfg = PanelConfig::of(&settings);
    assert_eq!(cfg.limits, settings.limits());
    assert_eq!(cfg.make_chars, settings.make_chars);
    assert_eq!(cfg.rows, 0, "until the ui tells its rows");
    assert_eq!(CONFIG.rows, 5);
}

#[test]
fn fill_open_toggles_the_panel_and_tells_its_state() {
    let mut p = Panel::new(CONFIG);
    assert_eq!(p.toggle(FillOpen), FillPanel { open: true });
    assert!(p.is_open());
    assert_eq!(p.toggle(FillOpen), FillPanel { open: false });
    assert!(!p.is_open());
}

#[test]
fn fill_close_when_shut_does_nothing_and_never_opens() {
    let mut p = Panel::new(CONFIG);
    assert_eq!(p.close_for(FillClose), Do::Stay);
    assert!(!p.is_open());
}

#[test]
fn fill_close_when_open_closes_and_hides_any_shown_value() {
    let mut r = Rig::sample();
    r.click(open(1));
    let show = FillClick::Field {
        field: 1,
        button: FieldButton::Show,
    };
    r.click(show);
    assert!(matches!(&r.page.sheet, Sheet::Entry(e) if e.fields[1].value.shown()));
    assert_eq!(r.panel.close_for(FillClose), Do::Close);
    assert!(!r.panel.is_open());
    assert_eq!(r.panel.toggle(FillOpen), FillPanel { open: true });
    r.draw();
    assert!(
        matches!(r.page.sheet, Sheet::List(_)),
        "opens fresh on the list"
    );
    r.click(open(1));
    assert!(matches!(&r.page.sheet, Sheet::Entry(e) if e.fields[1].value.text().is_none()));
}

#[test]
fn fill_ask_answers_the_current_state() {
    let mut p = Panel::new(CONFIG);
    assert_eq!(p.state(FillAsk), FillPanel { open: false });
    p.open();
    assert_eq!(p.state(FillAsk), FillPanel { open: true });
    p.close_for(FillClose);
    assert_eq!(p.state(FillAsk), FillPanel { open: false });
}

#[test]
fn fill_close_drops_new_or_edit_unsaved_as_the_close_button_does() {
    for clipboard in [false, true] {
        let mut r = Rig::sample();
        let close = |r: &mut Rig| {
            if clipboard {
                r.panel.close_for(FillClose)
            } else {
                r.click(button(FillButton::Close))
            }
        };
        r.click(open(1));
        r.click(button(FillButton::Edit));
        r.text("!");
        assert_eq!(close(&mut r), Do::Close, "clipboard: {clipboard}");
        assert_eq!(r.panel.state(FillAsk), FillPanel { open: false });
        let bank = r.book.get(r.id("Bank")).unwrap();
        assert_eq!(bank.name().expose(), "Bank", "nothing saved");
        r.panel.toggle(FillOpen);
        r.click(button(FillButton::New));
        r.click(FillClick::Kind(EntryKind::Note));
        r.text("Draft");
        assert_eq!(close(&mut r), Do::Close);
        assert_eq!(r.book.len(), 6, "nothing saved");
        r.panel.toggle(FillOpen);
        r.draw();
        assert!(
            matches!(r.page.sheet, Sheet::List(_)),
            "opens fresh on the list"
        );
    }
}

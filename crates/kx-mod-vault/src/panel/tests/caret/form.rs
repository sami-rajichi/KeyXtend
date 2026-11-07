//! The caret of the form's lines: each line keeps its own, keys act at it, a click places it, a masked line counts it
//! in dots, and no text leaves the form.

use kx_module_api::ports::{ClipText, PanelLine};

use super::super::edit::{sheet, values};
use super::*;
use crate::fixture::{LIMITS, draft, field, own, secret};

/// A family of three joined by ZWJ: one grapheme of five characters.
const FAMILY: &str = "\u{1f468}\u{200d}\u{1f469}\u{200d}\u{1f467}";
/// The click that shows the first field's hidden value.
const SHOW: FillClick = FillClick::Field {
    field: 0,
    button: FieldButton::Show,
};

/// The form's caret on the last view.
fn typed_at(r: &Rig) -> u32 {
    *sheet(r).caret.expose()
}

/// The Edit form of a sample's login, its name line taking the letters.
fn edit_bank() -> Rig {
    let mut r = Rig::sample();
    r.text("bank");
    r.click(open(0));
    r.click(button(FillButton::Edit));
    r
}

/// The Edit form of an entry whose one hidden value reads `BEH_FATHA_TEH`, its name line taking the letters.
fn edit_locker() -> Rig {
    let fields = vec![field("password", BEH_FATHA_TEH, true)];
    let mut r = Rig::new(vec![draft("Locker", EntryKind::Login, fields)]);
    r.click(open(0));
    r.click(button(FillButton::Edit));
    r
}

/// The click that gives field `field`'s value the letters.
const fn value(field: u8) -> FillClick {
    FillClick::Field {
        field,
        button: FieldButton::Value,
    }
}

#[test]
fn a_caret_click_from_an_older_view_still_counts_as_a_line_clicked_just_now() {
    let mut r = edit_bank();
    r.click(value(0));
    r.stale(place(1));
    assert_eq!(typed_at(&r), 1, "the line's click drew a new view first");
}

#[test]
fn a_form_line_still_sends_no_text_wherever_its_caret_is() {
    let mut r = edit_bank();
    r.click(value(0));
    r.key(&PanelKey::Left);
    assert_eq!(r.panel.line(false), PanelLine::Private);
    assert!(!format!("{:?}", r.panel.line(false)).contains("alex"));
}

#[test]
fn a_form_line_types_erases_and_deletes_at_its_caret() {
    let mut r = edit_bank();
    assert_eq!(typed_at(&r), 4, "the saved name, caret at its end");
    r.click(place(1));
    r.text("x");
    assert_eq!(sheet(&r).name.expose(), "Bxank");
    assert_eq!(typed_at(&r), 2);
    r.key(&PanelKey::Erase);
    assert_eq!(
        (sheet(&r).name.expose().as_str(), typed_at(&r)),
        ("Bank", 1)
    );
    assert_eq!(r.key(&PanelKey::Delete), Do::Show);
    assert_eq!((sheet(&r).name.expose().as_str(), typed_at(&r)), ("Bnk", 1));
    assert_eq!(r.key(&PanelKey::Tick), Do::Show);
    assert_eq!(sheet(&r).name.expose(), "B nk");
    assert_eq!(r.key(&PanelKey::Right), Do::Show);
    assert_eq!(typed_at(&r), 3);
}

#[test]
fn a_form_line_keeps_its_caret_while_another_takes_the_letters() {
    let mut r = edit_bank();
    r.key(&PanelKey::Left);
    r.click(value(0));
    r.click(place(2));
    assert_eq!(typed_at(&r), 2);
    r.key(&PanelKey::Up);
    assert_eq!(typed_at(&r), 3, "the name kept its own");
    r.key(&PanelKey::Down);
    assert_eq!(typed_at(&r), 2);
}

#[test]
fn the_left_and_right_of_a_form_never_leave_its_ends_or_its_page() {
    let mut r = edit_bank();
    assert_eq!(r.key(&PanelKey::Right), Do::Stay);
    r.click(place(0));
    assert_eq!(r.key(&PanelKey::Left), Do::Stay);
    assert!(matches!(r.page.sheet, Sheet::Edit(_)));
    assert_eq!(r.key(&PanelKey::Delete), Do::Show);
}

#[test]
fn the_cap_in_the_middle_of_a_form_line_refuses_and_keeps_the_caret() {
    let limits = Limits {
        field_chars: 3,
        ..LIMITS
    };
    let drafts = vec![draft(
        "abc",
        EntryKind::Note,
        vec![field("text", "x", false)],
    )];
    let cfg = PanelConfig { limits, ..CONFIG };
    let mut r = Rig::with(cfg, drafts);
    r.click(open(0));
    r.click(button(FillButton::Edit));
    r.click(place(1));
    assert_eq!(r.text("z"), Do::Show);
    assert_eq!(r.page.note.map(|n| n.note), Some(FillNote::Refused));
    assert_eq!((sheet(&r).name.expose().as_str(), typed_at(&r)), ("abc", 1));
}

#[test]
fn a_masked_value_counts_its_caret_in_dots_and_steps_by_them_whatever_it_reads() {
    let mut r = edit_locker();
    r.click(value(0));
    assert_eq!(sheet(&r).fields[0].value.count(), Some(3), "three dots");
    assert_eq!(typed_at(&r), 3);
    assert_eq!(
        r.key(&PanelKey::Left),
        Do::Show,
        "left goes back along the dots"
    );
    assert_eq!(typed_at(&r), 2);
    assert_eq!(r.key(&PanelKey::Left), Do::Show, "onto the fatha's own dot");
    assert_eq!(typed_at(&r), 1);
    assert_eq!(r.key(&PanelKey::Right), Do::Show);
    assert_eq!(typed_at(&r), 2);
    assert_eq!(r.click(place(1)), Do::Show, "its own dot, by the fatha");
    assert_eq!(typed_at(&r), 1);
    assert_eq!(r.click(place(9)), Do::Show);
    assert_eq!(typed_at(&r), 3);
}

#[test]
fn one_press_on_a_mask_moves_or_deletes_one_dot_whatever_joins_them() {
    let mut r = Rig::sample();
    r.click(button(FillButton::New));
    r.click(FillClick::Kind(EntryKind::Login));
    r.click(value(1));
    let (text, dots) = (format!("\u{628}\u{64e}{FAMILY}\u{200f}"), 8);
    r.text(&text);
    assert_eq!(sheet(&r).fields[1].value.count(), Some(8));
    assert_eq!(typed_at(&r), dots);
    for dot in (0..dots).rev() {
        assert_eq!(r.key(&PanelKey::Left), Do::Show);
        assert_eq!(typed_at(&r), dot);
    }
    assert_eq!(r.key(&PanelKey::Left), Do::Stay);
    for dot in 1..=dots {
        assert_eq!(r.key(&PanelKey::Right), Do::Show);
        assert_eq!(typed_at(&r), dot);
    }
    assert_eq!(r.key(&PanelKey::Right), Do::Stay);
    for dot in 0..=dots {
        r.click(place(dot));
        assert_eq!(typed_at(&r), dot, "a click lands on its own dot");
    }
    r.click(place(0));
    for left in (1..dots).rev() {
        assert_eq!(r.key(&PanelKey::Delete), Do::Show);
        let count = sheet(&r).fields[1].value.count();
        assert_eq!(count, Some(usize::try_from(left).unwrap()));
        assert_eq!(typed_at(&r), 0);
    }
    assert_eq!(r.key(&PanelKey::Delete), Do::Show);
    assert_eq!(values(&r)[1].as_deref(), Some(""), "no dot is left");
    assert_eq!(r.key(&PanelKey::Delete), Do::Stay);
}

#[test]
fn backspace_on_a_mask_takes_one_dot_as_it_always_did() {
    let mut r = edit_locker();
    r.click(value(0));
    r.click(place(2));
    assert_eq!(r.key(&PanelKey::Erase), Do::Show);
    assert_eq!(sheet(&r).fields[0].value.count(), Some(2));
    assert_eq!(typed_at(&r), 1);
}

#[test]
fn a_shown_hidden_value_counts_its_caret_in_units_and_reads_its_own_way() {
    let mut r = edit_locker();
    r.click(SHOW);
    assert_eq!(r.key(&PanelKey::Left), Do::Stay, "the end is on the left");
    assert_eq!(r.key(&PanelKey::Right), Do::Show);
    assert_eq!(typed_at(&r), 2);
    assert_eq!(r.key(&PanelKey::Right), Do::Show, "the beh and its fatha");
    assert_eq!(typed_at(&r), 0);
    assert_eq!(r.key(&PanelKey::Delete), Do::Show, "take them as one");
    assert_eq!(values(&r)[0].as_deref(), Some("\u{62a}"));
}

#[test]
fn a_mask_shown_with_its_caret_inside_a_grapheme_moves_to_its_edge_so_a_letter_never_splits_it() {
    let mut r = edit_locker();
    r.click(value(0));
    r.click(place(1));
    r.click(SHOW);
    assert_eq!(typed_at(&r), 0, "before the beh and its fatha");
    r.text("\u{627}");
    assert_eq!(
        values(&r)[0].as_deref(),
        Some("\u{627}\u{628}\u{64e}\u{62a}")
    );
}

#[test]
fn a_mask_switched_to_shown_with_its_caret_inside_a_grapheme_moves_to_its_edge() {
    let mut r = edit_locker();
    r.click(value(0));
    r.click(place(1));
    r.click(FillClick::Field {
        field: 0,
        button: FieldButton::Hidden,
    });
    assert_eq!(typed_at(&r), 0, "before the beh and its fatha");
}

#[test]
fn a_mask_shown_and_hidden_again_keeps_a_caret_already_on_an_edge() {
    let mut r = edit_locker();
    r.click(value(0));
    r.click(place(2));
    r.click(SHOW);
    assert_eq!(typed_at(&r), 2, "after the beh and its fatha");
    r.click(SHOW);
    assert_eq!(typed_at(&r), 2, "the same dot");
}

#[test]
fn an_own_label_has_its_own_caret() {
    let fields = vec![own("Code", "42", false)];
    let mut r = Rig::new(vec![draft("Locker", EntryKind::Other, fields)]);
    r.click(open(0));
    r.click(button(FillButton::Edit));
    r.click(FillClick::Field {
        field: 0,
        button: FieldButton::Label,
    });
    assert_eq!(typed_at(&r), 4);
    r.click(place(1));
    r.text("x");
    assert_eq!(typed_at(&r), 2);
    let Label::Own(label) = &sheet(&r).fields[0].label else {
        panic!("its own label");
    };
    assert_eq!(label.expose(), "Cxode");
}

#[test]
fn a_question_over_the_form_takes_neither_the_keys_nor_the_click() {
    let mut r = edit_bank();
    r.click(button(FillButton::Delete));
    let before = typed_at(&r);
    for key in [PanelKey::Left, PanelKey::Right, PanelKey::Delete] {
        assert_eq!(r.key(&key), Do::Stay, "{key:?}");
    }
    assert_eq!(r.click(place(0)), Do::Stay);
    assert_eq!(typed_at(&r), before);
    assert_eq!(sheet(&r).name.expose(), "Bank");
}

#[test]
fn the_labels_table_over_the_form_takes_no_caret() {
    let mut r = edit_bank();
    r.click(button(FillButton::AddField));
    assert_eq!(r.key(&PanelKey::Delete), Do::Stay);
    assert_eq!(r.click(place(0)), Do::Stay);
}

#[test]
fn make_one_and_paste_leave_the_caret_at_the_end_of_the_new_value() {
    let mut r = Rig::sample();
    r.click(button(FillButton::New));
    r.click(FillClick::Kind(EntryKind::Login));
    r.click(FillClick::Field {
        field: 1,
        button: FieldButton::Make,
    });
    let made = u32::try_from(CONFIG.make_chars).unwrap();
    assert_eq!(typed_at(&r), made);
    r.click(FillClick::Field {
        field: 0,
        button: FieldButton::Paste,
    });
    r.panel.pasted(&ClipText::Text(secret("pasted")));
    r.draw();
    assert_eq!(typed_at(&r), 6);
}

#[test]
fn a_view_never_shows_a_caret_in_its_debug() {
    let mut r = edit_bank();
    r.click(place(3));
    let shown = format!("{:?}", r.page);
    assert!(
        shown.contains(&format!("caret: {REDACTED_MARKER}")),
        "{shown}"
    );
}

//! The window over a list: the page that holds the highlight, its count, and the highlight's place on it; and what a
//! hidden value puts in the page: its empty text, nothing, or on the form its count.

use kx_module_api::ports::EntryKind;

use super::*;
use crate::entry::{Line, Stamp};
use crate::fixture::{draft, field};

/// The most characters a line holds here.
const MAX: usize = 40;
/// A password of 7 bytes but 3 characters: a Latin letter, an accented one and an emoji.
const WIDE: &str = "pä😀";

/// What the page holds of `value`, `None` for the mask.
fn held(value: &FieldValue) -> Option<&str> {
    value.text().map(|t| t.expose().as_str())
}

/// New's login form with `text` in its hidden password, `shown` or not.
fn login(text: &str, shown: bool) -> EditSheet {
    let mut e = Editor::new(MAX);
    assert!(e.pick(EntryKind::Login));
    e.slots[1].value = Line::with(text, MAX).unwrap();
    if shown {
        e.shown.insert(1);
    }
    form(&e, &Window::of(Some(0), e.slots.len(), 5, true))
}

/// A saved login of `fields`, as the store reads one back, empty fields and all.
fn saved(fields: Vec<Field>) -> Entry {
    let d = draft("Bank", EntryKind::Login, fields);
    Entry::restored(1, d, Stamp::from_unix_ms(0), None)
}

#[test]
fn a_read_page_gives_an_empty_hidden_value_its_empty_text() {
    let v = read(&field("password", "", true), false);
    assert_eq!(held(&v), Some(""));
    assert!(v.hidden() && !v.shown(), "it still offers Show");
    assert_eq!(v.count(), None);
}

#[test]
fn a_read_page_gives_a_hidden_value_no_text_and_no_count() {
    let v = read(&field("password", WIDE, true), false);
    assert_eq!((held(&v), v.count()), (None, None));
    assert!(v.hidden() && !v.shown());
    let shown = read(&field("password", WIDE, true), true);
    assert_eq!((held(&shown), shown.count()), (Some(WIDE), None));
    assert!(shown.shown());
}

#[test]
fn the_form_gives_a_hidden_value_only_its_count() {
    let sheet = login(WIDE, false);
    let v = &sheet.fields[1].value;
    assert_eq!((held(v), v.count()), (None, Some(3)));
    assert!(v.hidden() && !v.shown() && v.switches());
}

#[test]
fn the_form_gives_an_empty_hidden_value_its_empty_text() {
    let sheet = login("", false);
    let v = &sheet.fields[1].value;
    assert_eq!((held(v), v.count()), (Some(""), None));
    assert!(v.hidden() && !v.shown() && v.switches());
}

#[test]
fn the_form_gives_a_shown_or_plain_value_its_text_and_no_count() {
    let sheet = login(WIDE, true);
    let v = &sheet.fields[1].value;
    assert_eq!((held(v), v.count()), (Some(WIDE), None));
    assert!(v.shown());
    let plain = &sheet.fields[0].value;
    assert_eq!((held(plain), plain.count()), (Some(""), None));
}

#[test]
fn a_row_glimpses_no_empty_value() {
    let half = saved(vec![
        field("user", "alex", false),
        field("password", "", true),
    ]);
    let Hint::Values(values) = row(&half).hint else {
        panic!("the user shows")
    };
    assert_eq!(values.iter().map(held).collect::<Vec<_>>(), [Some("alex")]);
    let blank = saved(vec![field("user", "", false), field("password", "", true)]);
    assert_eq!(row(&blank).hint, Hint::Fields(2), "nothing to glimpse");
}

#[test]
fn a_row_still_masks_a_hidden_value_with_no_count() {
    let full = saved(vec![
        field("user", "alex", false),
        field("password", WIDE, true),
    ]);
    let Hint::Values(values) = row(&full).hint else {
        panic!("both show")
    };
    assert_eq!((held(&values[1]), values[1].count()), (None, None));
}

#[test]
fn a_window_shows_the_page_that_holds_the_highlight() {
    let w = Window::of(Some(7), 12, 5, true);
    assert_eq!((w.start, w.end, w.highlight), (5, 10, Some(2)));
    assert_eq!((w.pages.at, w.pages.count), (1, 3));
    let last = Window::of(Some(11), 12, 5, true);
    assert_eq!((last.start, last.end), (10, 12), "a short last page");
}

#[test]
fn an_empty_list_has_one_page_and_no_highlight() {
    let w = Window::of(None, 0, 5, true);
    assert_eq!((w.start, w.end, w.highlight), (0, 0, None));
    assert_eq!((w.pages.at, w.pages.count), (0, 1));
}

#[test]
fn an_unlit_window_follows_its_place_without_a_highlight() {
    let w = Window::of(Some(6), 8, 5, false);
    assert_eq!((w.start, w.highlight), (5, None));
}

#[test]
fn a_window_names_each_row_it_shows_from_its_start() {
    let w = Window::of(Some(3), 8, 3, true);
    let seen = w.seen(Target::Field);
    assert_eq!(seen.start, 3);
    assert_eq!(seen.rows, [3, 4, 5].map(Target::Field));
}

#[test]
fn counts_past_a_page_read_as_the_largest() {
    assert_eq!(small(300), u8::MAX);
    assert_eq!(short(70_000), u16::MAX);
}

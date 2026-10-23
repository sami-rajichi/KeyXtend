//! The shortcut pages: the owner's three pages by default, and no empty, over-full or extra page, nor an id twice.

use super::*;
use crate::spec::BoardSpec;
use kx_module_api::parse_as;
use kx_module_api::ports::ShortcutId;
use proptest::prelude::*;
use std::collections::HashSet;

/// The owner's starting set (spec §4.2, 2026-10-23), page by page.
const PAGE_1: [&str; 13] = [
    "copy",
    "paste",
    "cut",
    "select-all",
    "undo",
    "redo",
    "copy-all",
    "cut-all",
    "delete-all",
    "find",
    "save",
    "print",
    "emoji",
];
const PAGE_2: [&str; 12] = [
    "switch-app",
    "show-desktop",
    "file-explorer",
    "lock-pc",
    "task-manager",
    "close-window",
    "new-tab",
    "close-tab",
    "reopen-tab",
    "refresh",
    "zoom-in",
    "zoom-out",
];
/// The function keys on page 3.
const F_KEYS: usize = 12;

/// The defaults with `shortcut_pages` set to `pages`.
fn pages(pages: Vec<Vec<String>>) -> toml::Table {
    with("shortcut_pages", pages)
}

/// `n` distinct well-formed ids, from `first` on.
fn ids(first: usize, n: usize) -> Vec<String> {
    (first..first + n).map(|i| format!("s{i}")).collect()
}

/// How many shortcuts a page holds on the shipped board when the layer has `count` pages.
fn room(count: usize) -> usize {
    BoardSpec::shipped().unwrap().layer_grid().capacity(count)
}

/// The refusal `why`, as the validator gives it.
fn refused(why: &str) -> Result<(), SettingsError> {
    Err(SettingsError::Invalid(why.to_owned()))
}

#[test]
fn the_defaults_hold_the_owners_three_pages() {
    let s = KeyboardSettings::from_defaults().unwrap();
    let text = |p: &Vec<ShortcutId>| p.iter().map(|i| i.as_str().to_owned()).collect::<Vec<_>>();
    let got: Vec<Vec<String>> = s.shortcut_pages.iter().map(text).collect();
    let f_keys: Vec<String> = (1..=F_KEYS).map(|n| format!("f{n}")).collect();
    assert_eq!(got.len(), 3);
    assert_eq!(got[0], PAGE_1);
    assert_eq!(got[1], PAGE_2);
    assert_eq!(got[2], f_keys);
}

#[test]
fn no_pages_at_all_are_allowed() {
    let none = pages(Vec::new());
    assert_eq!(validate_as::<KeyboardSettings>(&none), Ok(()));
    let s: KeyboardSettings = parse_as(&none).unwrap();
    assert!(s.shortcut_pages.is_empty());
}

#[test]
fn an_empty_page_is_refused() {
    let got = validate_as::<KeyboardSettings>(&pages(vec![ids(0, 1), Vec::new()]));
    assert_eq!(got, refused(BAD_EMPTY_PAGE));
}

#[test]
fn a_page_holds_no_more_than_the_side_block_has_room_for() {
    let alone = |n| validate_as::<KeyboardSettings>(&pages(vec![ids(0, n)]));
    assert_eq!(alone(room(1)), Ok(()));
    assert_eq!(alone(room(1) + 1), refused(BAD_FULL_PAGE));
    let two = |n| validate_as::<KeyboardSettings>(&pages(vec![ids(100, 1), ids(0, n)]));
    assert_eq!(two(room(2)), Ok(()));
    assert_eq!(
        two(room(2) + 1),
        refused(BAD_FULL_PAGE),
        "the Next key takes a cell"
    );
}

#[test]
fn a_shortcut_listed_twice_is_refused() {
    let one = |p: Vec<&str>| p.into_iter().map(str::to_owned).collect::<Vec<_>>();
    let same_page = pages(vec![one(vec!["copy", "paste", "copy"])]);
    assert_eq!(
        validate_as::<KeyboardSettings>(&same_page),
        refused(BAD_TWICE)
    );
    let two_pages = pages(vec![one(vec!["copy"]), one(vec!["paste", "copy"])]);
    assert_eq!(
        validate_as::<KeyboardSettings>(&two_pages),
        refused(BAD_TWICE)
    );
}

#[test]
fn no_more_pages_than_the_next_keys_dots_show() {
    let many = |n: usize| pages((0..n).map(|p| ids(p, 1)).collect());
    assert_eq!(validate_as::<KeyboardSettings>(&many(MAX_PAGES)), Ok(()));
    assert_eq!(
        validate_as::<KeyboardSettings>(&many(MAX_PAGES + 1)),
        refused(BAD_PAGE_COUNT)
    );
}

#[test]
fn an_id_of_a_bad_form_does_not_parse() {
    for bad in ["Copy", "-copy", "", "copy all"] {
        assert!(unparsed(&pages(vec![vec![bad.to_owned()]])), "{bad:?}");
    }
    assert!(
        unparsed(&with("shortcut_pages", vec!["copy"])),
        "a page is a list"
    );
}

/// A short id, often repeated, or any text at all.
fn any_id() -> impl Strategy<Value = String> {
    prop_oneof!["[a-c]{1,2}", "s[0-9]{1,2}", ".{0,6}"]
}

proptest! {
    #[test]
    fn any_pages_pass_only_without_a_repeat_or_an_over_full_page(
        list in prop::collection::vec(prop::collection::vec(any_id(), 0..18), 0..11),
    ) {
        let t = pages(list.clone());
        if validate_as::<KeyboardSettings>(&t).is_ok() {
            let all: Vec<&String> = list.iter().flatten().collect();
            let unique: HashSet<&String> = all.iter().copied().collect();
            prop_assert_eq!(unique.len(), all.len(), "a repeat passed");
            let room = room(list.len());
            prop_assert!(list.iter().all(|p| !p.is_empty() && p.len() <= room));
            prop_assert!(list.len() <= MAX_PAGES);
        }
    }
}

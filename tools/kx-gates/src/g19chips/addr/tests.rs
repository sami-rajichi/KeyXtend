//! Tests of the address-bar walk: which control is the bar, which address is the long line's, and which window.

use super::*;

/// The shipped table.
fn bars() -> Vec<Bar> {
    crate::config::load().expect("loads").g19_product.bars
}

#[test]
fn only_an_address_whose_text_starts_as_the_long_line_counts() {
    let long = "حدد هذا";
    let ours_ = |v: &str| ours(v, "text", long);
    assert_eq!(
        ours_("kx.test/?text=%D8%AD%D8%AF%D8%AF&op=t").as_deref(),
        Some("حدد"),
        "cut short"
    );
    assert_eq!(ours_("kx.test/?q=1"), None);
    assert_eq!(ours("kx.test/?text=%D8%AD", "text", ""), None, "no line");
}

#[test]
fn the_window_read_is_the_new_one_or_the_one_newly_in_front_and_never_a_guess() {
    let before: HashSet<isize> = [1, 2].into_iter().collect();
    let pick = |front_before, now: &[isize], front| pick(&before, front_before, now, front, false);
    assert_eq!(pick(9, &[1, 2, 3], 9), Some(3), "a new window");
    assert_eq!(pick(9, &[1, 2, 3, 4], 4), Some(4), "the new one in front");
    assert_eq!(pick(9, &[1, 2, 3, 4], 9), None, "two new, neither in front");
    assert_eq!(pick(9, &[1, 2], 2), Some(2), "a new tab brought it forward");
    assert_eq!(pick(2, &[1, 2], 2), None, "it was in front already");
    assert_eq!(pick(9, &[1, 2], 9), None, "nothing changed");
}

#[test]
fn no_window_is_picked_while_the_question_still_asks() {
    let before: HashSet<isize> = [1].into_iter().collect();
    assert_eq!(
        pick(&before, 9, &[1, 3], 3, true),
        None,
        "a new one in front"
    );
    assert_eq!(pick(&before, 9, &[1], 1, true), None, "one newly in front");
    assert_eq!(
        pick(&before, 9, &[1, 3], 3, false),
        Some(3),
        "once it has gone"
    );
}

/// A name read that fails the test when it is called.
fn unread() -> String {
    panic!("the name was read")
}

#[test]
fn only_the_control_a_row_names_is_the_address_bar() {
    let table = bars();
    let chrome = row(&table, "Chrome.EXE").expect("a row for Chrome");
    let class = chrome.classes.first().expect("a class");
    assert!(fits(chrome, "", class, unread), "the class matched first");
    let name = chrome.names.first().expect("a name").clone();
    assert!(fits(chrome, "", "", || name.clone()));
    assert!(!fits(chrome, "", "Edit", || "Find".into()), "another edit");
    assert!(!fits(chrome, "", "", String::new), "an empty value");
    let firefox = row(&table, "firefox.exe").expect("a row for Firefox");
    let id = firefox.ids.first().expect("an id");
    assert!(fits(firefox, id, "", unread));
    assert!(
        !fits(firefox, "", class, unread),
        "a row with no names reads none"
    );
    assert!(row(&table, "notepad.exe").is_none());
}

#[test]
fn the_walk_matches_only_edits_and_never_enters_the_page_or_the_tabs() {
    use windows::Win32::UI::Accessibility::{UIA_ComboBoxControlTypeId, UIA_PaneControlTypeId};
    for kind in [
        UIA_DocumentControlTypeId,
        UIA_TabControlTypeId,
        UIA_TabItemControlTypeId,
    ] {
        assert_eq!(step(kind), Step::Skip, "{kind:?}");
    }
    assert_eq!(step(UIA_EditControlTypeId), Step::Edit);
    let (pane, combo) = (step(UIA_PaneControlTypeId), step(UIA_ComboBoxControlTypeId));
    assert_eq!((pane, combo), (Step::Enter, Step::Enter));
}

#[test]
fn hover_cards_tooltips_and_popups_are_no_browser_window() {
    let (app, tool) = (WS_EX_APPWINDOW.0, WS_EX_TOOLWINDOW.0);
    assert!(on_taskbar(0, false), "a plain window");
    assert!(!on_taskbar(tool, false), "a tool window");
    assert!(!on_taskbar(0, true), "an owned popup");
    assert!(on_taskbar(app, true), "marked an app window");
}

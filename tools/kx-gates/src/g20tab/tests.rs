use super::*;
use crate::config::{self, GatesConfig};
use crate::ownkeys::fake::{Fake, OUR_TAB, OWNER_TAB, hwnd, notepad};

/// The shipped settings.
fn cfg() -> GatesConfig {
    config::load().expect("settings load")
}

/// A click that must never run: the fake desk only counts clicks.
fn no_click() -> Result<(), String> {
    Err("a real click ran".to_string())
}

#[test]
fn ctrl_a_and_c_never_reach_the_owners_tab() {
    let (cfg, app) = (cfg(), notepad(None));
    let desk = Fake::new(app.hwnd, &[OWNER_TAB]);
    let e = copy_in_tab(&desk, &app, &cfg).expect_err("the owner's tab");
    assert!(e.contains("our tab is not shown"), "{e}");
    assert!(desk.quiet());
}

#[test]
fn ctrl_c_is_held_back_when_the_tab_changes_after_ctrl_a() {
    let (cfg, app) = (cfg(), notepad(None));
    let desk = Fake::new(app.hwnd, &[OUR_TAB, OWNER_TAB]);
    assert!(copy_in_tab(&desk, &app, &cfg).is_err());
    assert_eq!(
        *desk.sent.borrow(),
        std::slice::from_ref(&cfg.keys.select_all),
        "Ctrl+A only"
    );
}

#[test]
fn ctrl_a_and_c_go_to_our_tab_in_front() {
    let (cfg, app) = (cfg(), notepad(None));
    let desk = Fake::new(app.hwnd, &[OUR_TAB]);
    assert_eq!(copy_in_tab(&desk, &app, &cfg), Ok(()));
    let both = [cfg.keys.select_all.clone(), cfg.keys.copy.clone()];
    assert_eq!(*desk.sent.borrow(), both);
    let away = Fake::new(hwnd(2), &[OUR_TAB]);
    assert!(copy_in_tab(&away, &app, &cfg).is_err());
    assert!(away.quiet(), "another window in front");
}

#[test]
fn insert_pastes_nothing_into_the_owners_tab() {
    let (cfg, app) = (cfg(), notepad(None));
    let desk = Fake::new(app.hwnd, &[OWNER_TAB]);
    let e = paste(&desk, &app, &cfg, &no_click).expect_err("the owner's tab");
    assert!(e.contains("our tab is not shown"), "{e}");
    assert!(desk.quiet(), "no Ctrl+A and no Insert click");
}

#[test]
fn insert_is_not_clicked_when_the_tab_changes_after_ctrl_a() {
    let (cfg, app) = (cfg(), notepad(None));
    let desk = Fake::new(app.hwnd, &[OUR_TAB, OWNER_TAB]);
    let e = paste(&desk, &app, &cfg, &no_click).expect_err("the tab changed");
    assert!(e.contains("our tab is not shown"), "{e}");
    assert_eq!(desk.clicks.get(), 0, "Insert held back");
    let ours = Fake::new(app.hwnd, &[OUR_TAB]);
    assert_eq!(paste(&ours, &app, &cfg, &no_click), Ok(()));
    assert_eq!(ours.clicks.get(), 1);
}

#[test]
fn the_skipped_copy_counts_only_when_the_clipboard_changed_to_its_text() {
    let want = "Copied in a skipped app";
    assert!(skipped_ok(false, None, want).is_err(), "no change seen");
    assert!(skipped_ok(false, Some(want), want).is_err(), "an old copy");
    assert!(
        skipped_ok(true, Some("Morning coffee notes"), want).is_err(),
        "the wrong text"
    );
    assert!(skipped_ok(true, None, want).is_err(), "no text");
    assert_eq!(skipped_ok(true, Some(want), want), Ok(()));
    assert_eq!(
        skipped_ok(true, Some("Copied in a skipped app\r\n"), want),
        Ok(()),
        "Notepad's line end"
    );
}

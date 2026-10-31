use super::*;
use crate::ownkeys::fake::{Fake, OUR_TAB, OWNER_TAB, hwnd, notepad};

#[test]
fn the_panel_passes_only_once_win_and_its_spare_shift_both_read_let_go() {
    assert!(both_let_go((Some(false), Some(false))));
    assert!(
        !both_let_go((Some(false), Some(true))),
        "Shift still latched"
    );
    assert!(!both_let_go((Some(true), Some(false))), "Win still latched");
    assert!(!both_let_go((Some(false), None)), "Shift did not read");
    assert!(!both_let_go((None, Some(false))), "Win did not read");
}

#[test]
fn alt_goes_out_after_power_only_while_no_window_of_the_dialogs_class_shows() {
    let (dialog, other) = (1, 2);
    assert!(alt_ok::<i32>(&[], &[]), "all closed");
    assert!(!alt_ok(&[dialog], &[dialog]), "the dialog still shows");
    assert!(
        !alt_ok(&[], &[other]),
        "one whose program did not read may be the dialog"
    );
}

#[test]
fn notepad_takes_esc_for_the_panel_and_for_the_switcher_only_once_its_wait_ran_out() {
    assert!(
        !EscTo::Watched.app_takes(false) && !EscTo::Watched.app_takes(true),
        "the power dialog"
    );
    assert!(
        EscTo::WatchedOrApp.app_takes(false),
        "the panel never takes the front"
    );
    assert!(
        !EscTo::WatchedFirst.app_takes(false),
        "the switcher came to the front"
    );
    assert!(
        EscTo::WatchedFirst.app_takes(true),
        "it never did: today's rule"
    );
}

#[test]
fn g25_fetches_notepad_back_after_the_switcher_and_waits_for_it_after_the_panel() {
    let first = EscTo::WatchedFirst;
    assert!(
        first.fetches_app() && !first.app_back(),
        "Windows leaves the switcher's hidden window in front"
    );
    let panel = EscTo::WatchedOrApp;
    assert!(
        panel.app_back() && !panel.fetches_app(),
        "the panel gives the front back"
    );
    let power = EscTo::Watched;
    assert!(
        !power.app_back() && !power.fetches_app(),
        "Power fetches it after its class check"
    );
}

#[test]
fn esc_goes_out_only_while_a_watched_window_or_an_allowed_app_is_in_front() {
    let (app, dialog, other) = (1, 2, 3);
    assert!(esc_ok(&dialog, &[dialog], None));
    assert!(
        !esc_ok(&app, &[dialog], None),
        "only the power dialog itself takes Esc"
    );
    assert!(
        esc_ok(&app, &[dialog], Some(&app)),
        "Notepad, under the switcher or the panel"
    );
    assert!(
        !esc_ok(&other, &[dialog], Some(&app)),
        "another app never gets Esc"
    );
    assert!(!esc_ok(&other, &[], None), "nothing shown");
}

#[test]
fn esc_never_reaches_the_owners_notepad_tab() {
    let (app, esc) = (notepad(None), crate::config::load().expect("loads").g25.esc);
    let panel = [hwnd(5)];
    let owners = Fake::new(app.hwnd, &[OWNER_TAB]);
    let e = send_esc(&owners, &app, &panel, true, &esc).expect_err("the owner's tab");
    assert!(e.contains("our tab is not shown"), "{e}");
    assert!(owners.quiet());
    let ours = Fake::new(app.hwnd, &[OUR_TAB]);
    assert_eq!(send_esc(&ours, &app, &panel, true, &esc), Ok(true));
    assert_eq!(*ours.sent.borrow(), std::slice::from_ref(&esc));
}

#[test]
fn esc_goes_to_a_watched_window_and_to_notepad_only_where_allowed() {
    let (app, esc) = (notepad(None), crate::config::load().expect("loads").g25.esc);
    let panel = [hwnd(5)];
    let held = Fake::new(app.hwnd, &[OUR_TAB]);
    assert_eq!(
        send_esc(&held, &app, &panel, false, &esc),
        Ok(false),
        "Notepad may not take it"
    );
    let other = Fake::new(hwnd(9), &[OUR_TAB]);
    assert_eq!(
        send_esc(&other, &app, &panel, true, &esc),
        Ok(false),
        "another app"
    );
    assert!(held.quiet() && other.quiet());
    let watched = Fake::new(hwnd(5), &[OWNER_TAB]);
    assert_eq!(
        send_esc(&watched, &app, &panel, false, &esc),
        Ok(true),
        "the watched window"
    );
}

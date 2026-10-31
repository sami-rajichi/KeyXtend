use super::fake::{Fake, OUR_TAB, OURS, OWNER_TAB, hwnd, notepad};
use super::*;

#[test]
fn only_our_tab_takes_keys_when_the_app_has_a_file_name() {
    assert!(tab_ours(OUR_TAB, Some(OURS)));
    assert!(!tab_ours(OWNER_TAB, Some(OURS)), "the owner's tab");
    assert!(!tab_ours("Untitled - Notepad", Some(OURS)));
    assert!(!tab_ours("", Some(OURS)), "a gone window");
    assert!(tab_ours("Document1 - Word", None), "no file to check");
}

#[test]
fn keys_may_go_only_to_our_window_in_front_with_our_tab() {
    let (app, other) = (hwnd(1), hwnd(2));
    assert_eq!(may_send(app, app, OUR_TAB, Some(OURS)), Ok(()));
    assert_eq!(may_send(other, app, OUR_TAB, Some(OURS)), Err(Held::Front));
    assert_eq!(may_send(app, app, OWNER_TAB, Some(OURS)), Err(Held::Tab));
    assert_eq!(
        may_send(app, app, "Anything", None),
        Ok(()),
        "no tab to check"
    );
}

#[test]
fn a_key_or_a_click_on_the_owners_tab_never_goes_out() {
    let app = notepad(None);
    let desk = Fake::new(app.hwnd, &[OWNER_TAB]);
    let e = send(&desk, &app, &[0x41]).expect_err("the owner's tab");
    assert!(e.contains("our tab is not shown"), "{e}");
    let e = click(&desk, &app, &|| Ok(())).expect_err("the owner's tab");
    assert!(e.contains("our tab is not shown"), "{e}");
    assert!(desk.quiet());
}

#[test]
fn a_key_or_a_click_with_another_window_in_front_never_goes_out() {
    let app = notepad(None);
    let desk = Fake::new(hwnd(2), &[OUR_TAB]);
    let e = send(&desk, &app, &[0x41]).expect_err("another window");
    assert!(e.contains("keys held back"), "{e}");
    assert!(click(&desk, &app, &|| Ok(())).is_err());
    assert!(desk.quiet());
}

#[test]
fn our_tab_in_front_takes_the_key_and_the_click() {
    let app = notepad(None);
    let desk = Fake::new(app.hwnd, &[OUR_TAB]);
    assert_eq!(send(&desk, &app, &[0x41]), Ok(()));
    assert_eq!(click(&desk, &app, &|| Ok(())), Ok(()));
    assert_eq!((desk.sent.borrow().len(), desk.clicks.get()), (1, 1));
}

#[test]
fn each_key_of_a_run_is_checked_so_a_tab_that_changes_stops_the_rest() {
    let app = notepad(None);
    let (a, b, c): (&[u16], &[u16], &[u16]) = (&[0x41], &[0x42], &[0x43]);
    let desk = Fake::new(app.hwnd, &[OUR_TAB, OUR_TAB, OWNER_TAB]);
    let e = each(&desk, &app, &[a, b, c]).expect_err("the tab changed");
    assert!(e.contains("our tab is not shown"), "{e}");
    assert_eq!(
        *desk.sent.borrow(),
        [a.to_vec(), b.to_vec()],
        "the third held back"
    );
    let ours = Fake::new(app.hwnd, &[OUR_TAB]);
    assert_eq!(each(&ours, &app, &[a, b, c]), Ok(()));
    assert_eq!(ours.sent.borrow().len(), 3);
}

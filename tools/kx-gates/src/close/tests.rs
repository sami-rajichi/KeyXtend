//! Tests of `close`.

use kx_platform_fake::FakeInstance;

use super::*;

#[test]
fn quit_is_posted_to_the_running_copy_and_starts_nothing() {
    let session = FakeInstance::new();
    let mut claim = session.claim().expect("the first copy runs");
    claim.listen(Box::new(|_| {})).expect("it listens");
    assert_eq!(ask_quit(&session.clone()), Ok(()));
    assert_eq!(session.sent(), [Command::Quit]);
}

#[test]
fn a_copy_just_started_gets_quit_once_it_listens() {
    let session = FakeInstance::new();
    let late = std::cell::RefCell::new(Vec::new());
    let waits = std::cell::Cell::new(0);
    let wait = || {
        waits.set(waits.get() + 1);
        if waits.get() == 2 {
            let mut claim = session.claim().expect("the started copy runs");
            claim.listen(Box::new(|_| {})).expect("it listens");
            late.borrow_mut().push(claim);
        }
    };
    assert_eq!(
        ask_quit_tries(&session.clone(), 5, || false, wait),
        Ok(Asked::Sent(3))
    );
    assert_eq!(session.sent(), [Command::Quit]);
    let never = ask_quit_tries(&FakeInstance::new(), 3, || false, || {});
    assert!(
        never.is_err_and(|e| e.contains("command window")),
        "it never listened"
    );
}

#[test]
fn a_copy_that_ended_by_itself_gets_no_more_tries() {
    let session = FakeInstance::new();
    let gone = std::cell::Cell::new(false);
    let tries = ask_quit_tries(&session.clone(), 5, || gone.get(), || gone.set(true));
    assert_eq!(tries, Ok(Asked::Gone(1)), "one try, then it was gone");
    assert!(session.sent().is_empty());
}

#[test]
fn a_copy_that_cannot_hear_quit_says_why() {
    let none = ask_quit(&FakeInstance::new()).expect_err("no copy listens");
    assert!(
        none.why.contains("command window") && !none.refused,
        "{none:?}"
    );
    let refused = Unasked::from(UiError::Refused(5));
    assert!(refused.refused, "{refused:?}");
    let why = refused.why;
    assert!(why.contains("code 5") && why.contains("uiAccess"), "{why}");
}

#[test]
fn the_result_names_the_product_as_its_keyboard() {
    let quit = (r"C:\x\keyxtend.exe".to_string(), true);
    let v = result(Some(&quit), &Ok(false));
    assert_eq!(v["keyboard"], product::NAME);
    assert_eq!(
        (v["gate"].as_str(), v["pass"].as_bool()),
        (Some("close"), Some(true))
    );
}

#[test]
fn close_passes_only_when_the_copy_ended_and_any_backup_came_back() {
    let back = result(None, &Ok(true));
    assert_eq!(back["settings_back"].as_bool(), Some(true));
    assert_eq!(back["pass"].as_bool(), Some(true), "no copy ran");
    let hung = (r"C:\x\keyxtend.exe".to_string(), false);
    assert_eq!(
        result(Some(&hung), &Ok(false))["pass"].as_bool(),
        Some(false)
    );
    let refused = result(None, &Err("the backup stays".into()));
    assert_eq!(refused["pass"].as_bool(), Some(false));
    assert_eq!(refused["settings_error"].as_str(), Some("the backup stays"));
}

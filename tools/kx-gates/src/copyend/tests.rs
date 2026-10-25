//! Tests of `copyend`, with the fake command window.

use std::cell::Cell;

use kx_platform::ui::{Claim, Instance};
use kx_platform_fake::FakeInstance;

use super::*;

/// An exit code other than a clean one, as a crash leaves.
const CRASH: u32 = 3;

/// A copy that listens for Quit.
fn listening() -> (FakeInstance, Box<dyn Claim>) {
    let session = FakeInstance::new();
    let mut claim = session.claim().expect("the copy runs");
    claim.listen(Box::new(|_| {})).expect("it listens");
    (session, claim)
}

/// The Quit tries on `session`, which ended by itself when `gone`.
fn asked(session: &FakeInstance, gone: bool) -> Result<Asked, String> {
    ask_quit_tries(&session.clone(), 3, || gone, || {})
}

/// An end that marks `forced` and works.
fn forcing(forced: &Cell<bool>) -> impl FnOnce() -> Result<(), String> + '_ {
    || {
        forced.set(true);
        Ok(())
    }
}

#[test]
fn a_copy_that_takes_quit_and_exits_cleanly_is_never_ended_by_force() {
    let (session, _claim) = listening();
    let forced = Cell::new(false);
    let got = finish(
        &asked(&session, false),
        || true,
        forcing(&forced),
        || Some(CLEAN),
    );
    assert_eq!(got, (Gone::Quit, None));
    assert!(got.0.ok() && !forced.get());
}

#[test]
fn a_copy_that_crashed_reads_as_crashed_not_as_quit() {
    let forced = Cell::new(false);
    let got = finish(
        &asked(&FakeInstance::new(), true),
        || true,
        forcing(&forced),
        || Some(CRASH),
    );
    assert_eq!(got.0, Gone::Crashed(Some(CRASH)));
    assert!(!got.0.ok() && got.0.over() && !forced.get());
}

#[test]
fn a_copy_that_never_hears_quit_is_ended_and_still_fails() {
    let forced = Cell::new(false);
    let ended_code = || forced.get().then_some(1);
    let got = finish(
        &asked(&FakeInstance::new(), false),
        || forced.get(),
        forcing(&forced),
        ended_code,
    );
    assert_eq!(got, (Gone::Ended, None));
    assert!(!got.0.ok() && got.0.over());
}

#[test]
fn a_copy_that_took_quit_but_hangs_is_ended() {
    let (session, _claim) = listening();
    let forced = Cell::new(false);
    let got = finish(
        &asked(&session, false),
        || forced.get(),
        forcing(&forced),
        || None,
    );
    assert_eq!(got.0, Gone::Ended);
    assert!(forced.get());
}

#[test]
fn a_copy_that_cannot_be_ended_still_runs() {
    let refused = || Err("access denied".to_string());
    let got = finish(
        &asked(&FakeInstance::new(), false),
        || false,
        refused,
        || None,
    );
    assert_eq!(got, (Gone::Running, Some("access denied".to_string())));
    assert!(!got.0.over());
}

#[test]
fn an_end_that_fails_on_a_copy_already_gone_is_a_note_not_a_failure() {
    let (session, _claim) = listening();
    let late = Cell::new(false);
    let exited = || {
        late.set(true);
        Err("access denied".to_string())
    };
    let got = finish(
        &asked(&session, false),
        || late.get(),
        exited,
        || Some(CLEAN),
    );
    assert_eq!(got, (Gone::Quit, Some("access denied".to_string())));
}

#[test]
fn a_copy_gone_before_any_quit_reads_as_crashed_even_with_a_clean_code() {
    let forced = Cell::new(false);
    let gone = asked(&FakeInstance::new(), true);
    assert_eq!(gone, Ok(Asked::Gone(0)), "no Quit was sent");
    let got = finish(&gone, || true, forcing(&forced), || Some(CLEAN));
    assert_eq!(got, (Gone::Crashed(Some(CLEAN)), None));
    assert!(!forced.get());
}

#[test]
fn a_copy_that_left_by_itself_before_quit_reached_it_reads_as_crashed() {
    let late = Cell::new(false);
    let exited = || {
        late.set(true);
        Err("access denied".to_string())
    };
    let unheard = asked(&FakeInstance::new(), false);
    let got = finish(&unheard, || late.get(), exited, || Some(CLEAN));
    assert_eq!(got.0, Gone::Crashed(Some(CLEAN)), "Quit never reached it");
}

#[test]
fn a_next_copy_follows_any_copy_that_no_longer_runs() {
    assert!(may_follow(None), "no copy started");
    for gone in [Gone::Quit, Gone::Crashed(None), Gone::Ended] {
        assert!(may_follow(Some(gone)), "{gone:?}");
    }
    assert!(!may_follow(Some(Gone::Running)));
}

#[test]
fn a_line_says_how_the_copy_went_and_what_was_let_go() {
    let ended = Ended {
        gone: Gone::Crashed(Some(CRASH)),
        note: None,
        released: vec!["released a button or key left down".into()],
    };
    let line = ended.line();
    assert_eq!(line["ok"], false);
    assert!(
        line["gone"]
            .as_str()
            .is_some_and(|g| g.contains("crashed") && g.contains('3'))
    );
    assert_eq!(line["released"].as_array().map(Vec::len), Some(1));
    assert!(
        Ended {
            gone: Gone::Quit,
            note: None,
            released: Vec::new()
        }
        .ok()
    );
}

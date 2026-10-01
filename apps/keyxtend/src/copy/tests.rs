use super::*;
use kx_platform_fake::FakeInstance;

/// Three tries, 7 ms apart.
const REACH: Reach = Reach {
    tries: 3,
    wait: Duration::from_millis(7),
};

/// Starts a copy in `session` that never waits; it records each pause instead.
fn run(session: &FakeInstance, command: Command, pauses: &mut Vec<Duration>) -> Start {
    start(session, command, REACH, |d| pauses.push(d))
}

#[test]
fn the_first_copy_runs_and_holds_the_session() {
    let session = FakeInstance::new();
    let mut pauses = Vec::new();
    let first = run(&session, Command::Show, &mut pauses);
    assert!(matches!(first, Start::Run(_)));
    assert!(session.claim().is_err(), "the session is held");
    assert!(pauses.is_empty());
}

#[test]
fn quit_with_no_copy_running_starts_nothing() {
    let session = FakeInstance::new();
    let mut pauses = Vec::new();
    assert!(matches!(
        run(&session, Command::Quit, &mut pauses),
        Start::Done
    ));
    assert!(session.claim().is_ok(), "nothing kept the session");
    assert!(session.sent().is_empty() && pauses.is_empty());
}

#[test]
fn a_later_copy_hands_its_command_over_and_ends() {
    let session = FakeInstance::new();
    let mut claim = session.claim().unwrap();
    claim.listen(Box::new(|_| {})).unwrap();
    for command in [Command::Show, Command::Quit] {
        let mut pauses = Vec::new();
        let later = run(&session.clone(), command, &mut pauses);
        assert!(matches!(later, Start::Done), "{command:?}");
        assert!(pauses.is_empty(), "the first try reached it");
    }
    assert_eq!(session.sent(), [Command::Show, Command::Quit]);
}

#[test]
fn a_copy_that_never_listens_is_tried_then_given_up() {
    let session = FakeInstance::new();
    let _claim = session.claim().unwrap();
    let mut pauses = Vec::new();
    let later = run(&session.clone(), Command::Show, &mut pauses);
    assert!(matches!(later, Start::Unanswered(UiError::NoWindow)));
    assert_eq!(pauses, [REACH.wait; 2], "a pause between each two tries");
    assert!(session.sent().is_empty());
}

#[test]
fn a_copy_that_listens_during_the_tries_gets_the_command() {
    let session = FakeInstance::new();
    let mut claim = session.claim().unwrap();
    let mut pauses = Vec::new();
    let later = start(&session.clone(), Command::Show, REACH, |d| {
        pauses.push(d);
        claim.listen(Box::new(|_| {})).unwrap();
    });
    assert!(matches!(later, Start::Done));
    assert_eq!(pauses, [REACH.wait], "the second try reached it");
    assert_eq!(session.sent(), [Command::Show]);
}

#[test]
fn the_reach_follows_the_ui_defaults() {
    let settings = UiSettings::from_defaults().unwrap();
    let reach = Reach::of(&settings);
    assert_eq!(reach.tries, settings.send_tries);
    assert_eq!(reach.wait, Duration::from_millis(settings.send_wait_ms));
}

//! The fake's watch itself: a refused watch, a stop, a second watch, clones on another thread, a `Debug` that hides the
//! text, and errors the port never returns.

use super::*;

#[test]
fn a_failed_watch_fails_once_then_watches_with_its_config() {
    let fake = FakeSelectionWatch::new();
    fake.fail_watch(SelError::Refused);
    let failed = fake.watch(CONFIG, Box::new(|| {}));
    assert_eq!(failed.err(), Some(SelError::Refused));
    assert!(!fake.watching());
    assert_eq!((fake.calls().watches, fake.config()), (0, None));
    let (_run, _) = watched(&fake);
    assert!(fake.watching());
    assert_eq!((fake.calls().watches, fake.config()), (1, Some(CONFIG)));
}

#[test]
fn dropping_the_run_stops_the_watch_and_its_tells() {
    let fake = FakeSelectionWatch::new();
    let (run, heard) = watched(&fake);
    drop(run);
    assert!(!fake.watching());
    fake.focus(selected());
    assert!(
        heard.lock().unwrap().is_empty(),
        "a stopped watch tells nothing"
    );
    assert_eq!(fake.config(), Some(CONFIG), "kept after the stop");
    assert_eq!((fake.calls().watches, fake.calls().stops), (1, 1));
}

#[test]
fn a_second_watch_replaces_the_first_whose_drop_keeps_it_watching() {
    let fake = FakeSelectionWatch::new();
    let (first, one) = watched(&fake);
    let (second, two) = watched(&fake);
    fake.focus(selected());
    drop(first);
    assert!(fake.watching(), "the second still runs");
    fake.change(caret());
    assert!(one.lock().unwrap().is_empty(), "replaced");
    assert_eq!(two.lock().unwrap().len(), 2);
    drop(second);
    assert!(!fake.watching());
}

#[test]
fn clones_share_the_script_and_serve_as_the_port_from_another_thread() {
    let fake = FakeSelectionWatch::new();
    let port: Arc<dyn SelectionWatch> = Arc::new(fake.clone());
    let run = thread::spawn(move || port.watch(CONFIG, Box::new(|| {})).ok());
    let mut run = run.join().unwrap().unwrap();
    assert!(fake.watching());
    fake.focus(selected());
    fake.set_text(TEXT);
    assert_eq!(run.look().map(|s| s.kind), Ok(SelKind::Editable));
    assert_eq!(read(&fake, run.as_mut(), 5), Ok("Hello".to_owned()));
}

#[test]
fn debug_never_prints_the_scripted_text() {
    let fake = FakeSelectionWatch::new();
    fake.set_text(TEXT);
    fake.focus(selected());
    let text = format!("{fake:?}");
    assert!(!text.contains(TEXT) && text.contains("Editable"), "{text}");
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "never returns")]
fn a_watch_error_the_port_never_returns_is_a_test_mistake() {
    FakeSelectionWatch::new().fail_watch(SelError::TimedOut);
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "never returns")]
fn a_closed_look_is_a_test_mistake() {
    FakeSelectionWatch::new().fail_looks(SelError::Closed);
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "never returns")]
fn a_closed_read_failure_is_a_test_mistake_as_only_a_closed_box_gives_it() {
    FakeSelectionWatch::new().fail_reads(SelError::Closed);
}

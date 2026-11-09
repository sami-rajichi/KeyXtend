//! The shared rules over a scripted probe: a look comes from the focused box, our own focus is never looked at or read,
//! a late job makes no OS call, a read is refused in a closed box and empty at a bare caret with no text call, a
//! job's token lives until `serve` returns, and a look answers before the adapter's own step.

use super::rig::{Calls, Script, Stop, own, spot, theirs};
use super::*;
use crate::deadline;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// A minute from now, which no test reaches.
fn far() -> Instant {
    deadline(Duration::from_secs(60))
}

/// A moment ago.
fn past() -> Instant {
    let ago = Instant::now().checked_sub(Duration::from_millis(1));
    ago.unwrap_or_else(Instant::now)
}

/// What `serve` answers to a look by `by`.
fn look(probe: &Script, by: Instant) -> Result<Option<SelSpot>, SelError> {
    let (reply, answer) = mpsc::channel();
    serve(probe, Job::new(Ask::Look { by, reply }));
    answer.recv().unwrap_or(Err(SelError::Unavailable))
}

/// What `serve` answers to whether box `box_id` still holds a selection, by `by`.
fn still(probe: &Script, box_id: u64, by: Instant) -> bool {
    let (reply, answer) = mpsc::channel();
    serve(probe, Job::new(Ask::Still { box_id, by, reply }));
    answer.recv().unwrap_or(false)
}

/// What `serve` answers to a read of at most `max_chars` characters of box `box_id`, by `by`, unwrapped.
fn read_of(probe: &Script, box_id: u64, max_chars: u16, by: Instant) -> Result<String, SelError> {
    let (reply, answer) = mpsc::channel();
    let ask = Ask::Read {
        box_id,
        max_chars,
        by,
        reply,
    };
    serve(probe, Job::new(ask));
    let text = answer.recv().unwrap_or(Err(SelError::Unavailable));
    text.map(Redacted::into_inner)
}

/// What `serve` answers to whether the clipboard holds anything, by `by`.
fn clip(probe: &Script, by: Instant) -> bool {
    let (reply, answer) = mpsc::channel();
    serve(probe, Job::new(Ask::Clip { by, reply }));
    answer.recv().unwrap_or(false)
}

#[test]
fn a_look_comes_back_from_the_focused_box_with_no_text_call() {
    let probe = Script::on(Ok(theirs(SelKind::Editable, 7)));
    assert_eq!(
        look(&probe, far()),
        Ok(Some(theirs(SelKind::Editable, 7).1))
    );
    assert_eq!(probe.calls().reads, 0);
}

#[test]
fn still_asks_the_probe_about_the_focused_box() {
    let probe = Script::on(Ok(theirs(SelKind::Terminal, 7)));
    assert!(still(&probe, 7, far()));
    assert!(!still(&probe, 8, far()), "another box");
    assert_eq!(probe.calls().stills, 2);
}

#[test]
fn a_read_gives_the_selection_cut_at_its_cap() {
    let probe = Script::on(Ok(theirs(SelKind::ReadOnly, 3)));
    probe.text("مرحبا بالعالم");
    assert_eq!(read_of(&probe, 3, 5, far()), Ok("مرحبا".to_owned()));
    assert_eq!(probe.calls().reads, 1);
}

#[test]
fn a_read_for_a_box_focus_left_is_closed_with_no_text_call_in_the_same_job() {
    let probe = Script::on(Ok(theirs(SelKind::Editable, 9)));
    probe.text("مرحبا بالعالم");
    assert_eq!(read_of(&probe, 3, 5, far()), Err(SelError::Closed));
    assert_eq!(probe.calls().reads, 0);
}

/// A box with no id (0) is never the box `still` confirmed, as two such boxes cannot be told apart.
#[test]
fn a_read_for_a_box_with_no_id_is_closed_with_no_text_call() {
    let probe = Script::on(Ok(theirs(SelKind::Editable, 0)));
    probe.text("مرحبا بالعالم");
    assert_eq!(read_of(&probe, 0, 5, far()), Err(SelError::Closed));
    assert_eq!(probe.calls().reads, 0);
}

#[test]
fn a_closed_box_is_refused_with_no_text_call() {
    let probe = Script::on(Ok(theirs(SelKind::Closed, 3)));
    probe.text("secret");
    assert_eq!(read_of(&probe, 3, 5, far()), Err(SelError::Closed));
    assert_eq!(probe.calls().reads, 0);
}

#[test]
fn a_bare_caret_reads_as_empty_with_no_text_call() {
    let probe = Script::on(Ok((false, spot(SelKind::Editable, 3, false))));
    probe.text("words");
    assert_eq!(read_of(&probe, 3, 5, far()), Ok(String::new()));
    assert_eq!(probe.calls().reads, 0);
}

#[test]
fn focus_in_our_own_process_is_never_looked_at_or_read() {
    let probe = Script::on(Ok(own(4)));
    probe.text("ours");
    assert_eq!(look(&probe, far()), Ok(None), "the run keeps the last box");
    assert!(!still(&probe, 4, far()));
    assert_eq!(read_of(&probe, 4, 5, far()), Err(SelError::Closed));
    let calls = probe.calls();
    assert_eq!((calls.spots, calls.stills, calls.reads), (0, 0, 0));
}

#[test]
fn a_late_job_makes_no_os_call() {
    let probe = Script::on(Ok(theirs(SelKind::Editable, 1)));
    probe.clip(true);
    assert_eq!(look(&probe, past()), Err(SelError::TimedOut));
    assert!(!still(&probe, 1, past()));
    assert_eq!(read_of(&probe, 1, 5, past()), Err(SelError::TimedOut));
    assert!(!clip(&probe, past()));
    assert_eq!(probe.calls(), Calls::default());
}

#[test]
fn a_refused_focus_passes_its_error_on() {
    let probe = Script::on(Err(SelError::Refused));
    assert_eq!(look(&probe, far()), Err(SelError::Refused));
    assert!(!still(&probe, 1, far()));
    assert_eq!(read_of(&probe, 1, 5, far()), Err(SelError::Refused));
    assert_eq!(probe.calls().spots, 0);
}

#[test]
fn the_clipboard_answer_is_the_probes() {
    let probe = Script::on(Ok(own(1)));
    assert!(!clip(&probe, far()));
    probe.clip(true);
    assert!(clip(&probe, far()), "even with our own focus");
}

#[test]
fn a_jobs_token_lives_while_the_app_hangs_and_ends_when_serve_returns() {
    let probe = Script::on(Ok(theirs(SelKind::Editable, 1)));
    probe.shut(Stop::Spot);
    let (reply, answer) = mpsc::channel();
    let job = Job::new(Ask::Look { by: far(), reply });
    let alive = job.alive();
    let serving = probe.clone();
    let worker = thread::spawn(move || serve(&serving, job));
    assert!(probe.hung(), "serve is inside the app");
    assert_eq!(alive.strong_count(), 1, "the job lives while the app hangs");
    probe.open();
    assert!(worker.join().is_ok());
    assert_eq!(alive.strong_count(), 0, "it ends once serve returns");
    assert!(answer.recv().is_ok_and(|a| a.is_ok()), "after its answer");
}

#[test]
fn a_deadline_is_late_only_once_it_passed() {
    assert_eq!(late(far()), Ok(()));
    assert_eq!(late(past()), Err(SelError::TimedOut));
}

#[test]
fn a_looks_answer_goes_before_the_adapters_step_which_our_own_focus_never_gets() {
    let probe = Script::on(Ok(theirs(SelKind::Editable, 7)));
    let (reply, answer) = mpsc::channel();
    probe.hear(answer);
    serve(&probe, Job::new(Ask::Look { by: far(), reply }));
    assert_eq!(probe.stepped(), [(7, false, true)], "the answer went first");
    probe.focus(Ok(own(8)));
    assert_eq!(look(&probe, far()), Ok(None));
    probe.focus(Err(SelError::Refused));
    assert!(look(&probe, far()).is_err());
    assert_eq!(probe.calls().steps, 1, "no step without a spot");
}

//! A hung app: a job past `read_ms` times out, the thread gets no job while it still holds that one, so every ask
//! answers at once, a change meanwhile is told once the app answers, and a timed-out read's text never reaches a later
//! ask. The rig's `Take` gate holds a job whatever its deadline, so no test races the thread.

use super::*;

/// A `read_ms` an answered look meets even beside other test runs, short enough to wait out for a timeout.
const HANG_MS: u32 = 1000;

#[test]
fn a_job_past_read_ms_times_out_and_the_thread_is_skipped_until_it_returns() {
    let probe = Script::on(Ok(theirs(SelKind::Editable, 3)));
    let mut rig = rig(&probe, HANG_MS);
    assert!(rig.run.look().is_ok());
    assert_eq!(rig.served.recv_timeout(WAIT), Ok(()));
    probe.shut(Stop::Take);
    assert_eq!(rig.run.look(), Err(SelError::TimedOut));
    assert!(probe.hung(), "the thread holds the job");
    assert_eq!(rig.run.look(), Err(SelError::TimedOut), "busy");
    assert!(!rig.run.still(3));
    assert_eq!(rig.run.read(3, 4), Err(SelError::TimedOut));
    assert!(!rig.run.clip_holds());
    assert_eq!(rig.taken(), 2, "nothing queued behind the hung look");
    probe.open();
    assert_eq!(rig.served.recv_timeout(WAIT), Ok(()), "the hung look ended");
    assert_eq!(rig.run.look().map(|s| s.box_id), Ok(3), "looks resume");
    assert_eq!(rig.taken(), 3);
}

#[test]
fn a_change_during_a_hang_is_told_once_the_app_answers() {
    let probe = Script::on(Ok(theirs(SelKind::Editable, 3)));
    let mut rig = rig(&probe, HANG_MS);
    probe.shut(Stop::Take);
    assert_eq!(rig.run.look(), Err(SelError::TimedOut));
    assert!(probe.hung(), "the thread holds the job");
    rig.shared.moved(&probe, &theirs(SelKind::Editable, 3));
    rig.clock.advance(Duration::from_secs(1));
    assert_eq!(rig.tells(), 0, "a stuck thread tells nothing");
    probe.open();
    assert_eq!(rig.heard.recv_timeout(WAIT), Ok(()), "told once it returns");
    assert_eq!(rig.run.look().map(|s| s.box_id), Ok(3));
    assert_eq!(rig.tells(), 1);
}

#[test]
fn a_timed_out_reads_text_never_reaches_a_later_ask() {
    let probe = Script::on(Ok(theirs(SelKind::Editable, 3)));
    probe.text("old");
    let mut rig = rig(&probe, HANG_MS);
    assert!(rig.run.look().is_ok());
    assert_eq!(rig.served.recv_timeout(WAIT), Ok(()));
    probe.shut(Stop::Read);
    assert_eq!(rig.run.read(3, 9), Err(SelError::TimedOut));
    probe.text("new");
    probe.open();
    assert_eq!(
        rig.served.recv_timeout(WAIT),
        Ok(()),
        "the stuck read ended"
    );
    let fresh = rig.run.read(3, 9).map(Redacted::into_inner);
    assert_eq!(fresh, Ok("new".to_owned()), "never the late answer");
}

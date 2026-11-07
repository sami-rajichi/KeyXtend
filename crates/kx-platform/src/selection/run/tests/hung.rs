//! A hung app: a job past `read_ms` times out, the thread gets no job while it still owes that answer, so every ask
//! answers at once, a change meanwhile is told once the app answers, and jobs go through again.

use super::*;

/// A `read_ms` an answered job meets even on a busy PC, short enough to wait out for a timeout.
const HANG_MS: u32 = 400;

#[test]
fn a_job_past_read_ms_times_out_and_the_thread_is_skipped_until_it_returns() {
    let probe = Script::on(Ok(theirs(SelKind::Editable, 3)));
    let mut rig = rig(&probe, HANG_MS);
    assert!(rig.run.look().is_ok());
    assert_eq!(rig.served.recv_timeout(WAIT), Ok(()));
    probe.shut();
    assert_eq!(rig.run.look(), Err(SelError::TimedOut));
    assert!(probe.hung(), "the thread is stuck in the app");
    assert_eq!(rig.run.look(), Err(SelError::TimedOut), "busy");
    assert!(!rig.run.still(3));
    assert_eq!(rig.run.read(4), Err(SelError::TimedOut));
    assert!(!rig.run.clip_holds());
    assert_eq!(rig.taken(), 2, "nothing queued behind the hung look");
    probe.open();
    assert_eq!(
        rig.served.recv_timeout(WAIT),
        Ok(()),
        "the hung look returned"
    );
    assert_eq!(rig.run.look().map(|s| s.box_id), Ok(3), "looks resume");
    assert_eq!(rig.taken(), 3);
}

#[test]
fn a_change_during_a_hang_is_told_once_the_app_answers() {
    let probe = Script::on(Ok(theirs(SelKind::Editable, 3)));
    let mut rig = rig(&probe, HANG_MS);
    probe.shut();
    assert_eq!(rig.run.look(), Err(SelError::TimedOut));
    assert!(probe.hung(), "the thread is stuck in the app");
    rig.shared.moved(&probe, &theirs(SelKind::Editable, 3));
    rig.clock.advance(Duration::from_secs(1));
    assert_eq!(rig.tells(), 0, "a stuck thread tells nothing");
    probe.open();
    assert_eq!(rig.heard.recv_timeout(WAIT), Ok(()), "told once it returns");
    assert_eq!(rig.run.look().map(|s| s.box_id), Ok(3));
    assert_eq!(rig.tells(), 1);
}

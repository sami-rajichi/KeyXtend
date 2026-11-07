//! The thread's pacing on a clock the test moves: a burst is told once after it settles, changes past the cap wait for
//! the window's end, and an idle thread tells nothing and takes no timed wait. A look's answer proves the thread told
//! every change due before it took the look.

use super::*;
use kx_module_api::MS_PER_S;

/// Moves the clock on by `ms`.
fn advance(rig: &Rig, ms: u32) {
    rig.clock.advance(Duration::from_millis(u64::from(ms)));
}

/// A change in the focused box.
fn change(rig: &Rig, probe: &Script) {
    rig.shared.moved(probe, &theirs(SelKind::Editable, 1));
}

#[test]
fn a_burst_of_changes_is_told_once_after_it_settles() {
    let probe = Script::on(Ok(theirs(SelKind::Editable, 1)));
    let mut rig = rig(&probe, LONG_MS);
    for _ in 0..3 {
        change(&rig, &probe);
        advance(&rig, SETTLE_MS / 2);
    }
    assert!(rig.run.look().is_ok());
    assert_eq!(rig.tells(), 0, "the last change has not settled");
    advance(&rig, SETTLE_MS / 2);
    assert!(rig.run.look().is_ok());
    assert_eq!(rig.tells(), 1);
    advance(&rig, MS_PER_S);
    assert!(rig.run.look().is_ok());
    assert_eq!(rig.tells(), 1, "told once");
    assert!(rig.shared.timed() > 0, "a change owed is waited for");
}

#[test]
fn changes_past_the_cap_wait_for_the_windows_end() {
    let probe = Script::on(Ok(theirs(SelKind::Editable, 1)));
    let mut rig = rig(&probe, LONG_MS);
    for told in 1..=usize::from(LOOKS) {
        change(&rig, &probe);
        advance(&rig, SETTLE_MS);
        assert!(rig.run.look().is_ok());
        assert_eq!(rig.tells(), told, "inside the cap");
    }
    change(&rig, &probe);
    advance(&rig, SETTLE_MS);
    assert!(rig.run.look().is_ok());
    assert_eq!(rig.tells(), usize::from(LOOKS), "past the cap");
    let first = SETTLE_MS;
    let now = (u32::from(LOOKS) + 1) * SETTLE_MS;
    advance(&rig, first + MS_PER_S - now - 1);
    assert!(rig.run.look().is_ok());
    assert_eq!(
        rig.tells(),
        usize::from(LOOKS),
        "a moment before the window's end"
    );
    advance(&rig, 1);
    assert!(rig.run.look().is_ok());
    assert_eq!(rig.tells(), usize::from(LOOKS) + 1, "at the window's end");
}

#[test]
fn an_idle_thread_tells_nothing_and_takes_no_timed_wait() {
    let probe = Script::on(Ok(theirs(SelKind::Editable, 1)));
    let mut rig = rig(&probe, LONG_MS);
    assert!(rig.run.look().is_ok());
    advance(&rig, 60 * MS_PER_S);
    assert!(rig.run.look().is_ok());
    assert_eq!(rig.tells(), 0);
    assert_eq!(rig.shared.timed(), 0);
    assert_eq!(probe.calls().spots, 2, "only the looks asked");
}

#[test]
fn our_own_focus_changes_tell_nothing_and_theirs_do() {
    let probe = Script::on(Ok(theirs(SelKind::Editable, 1)));
    let mut rig = rig(&probe, LONG_MS);
    rig.shared.moved(&probe, &own(2));
    advance(&rig, MS_PER_S);
    assert!(rig.run.look().is_ok());
    assert_eq!(rig.tells(), 0, "our own focus keeps the last box");
    change(&rig, &probe);
    advance(&rig, MS_PER_S);
    assert!(rig.run.look().is_ok());
    assert_eq!(rig.tells(), 1);
}

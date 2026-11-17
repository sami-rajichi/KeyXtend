//! A look at the box in front: it comes back from the thread, keeps and tells nothing, passes our own focus on as
//! `None`, and fails as a read fails.

use super::*;

#[test]
fn a_look_finds_the_box_in_front_and_keeps_and_tells_nothing() {
    let (mut run, shared, heard) = started(LONG_MS, answers);
    shared.moved(&Script, &other(FieldKind::None, 1));
    assert_eq!(run.look(), Ok(Some(change(FieldKind::Text, LOOKED))));
    assert_eq!(run.kind(), FieldKind::None, "the box told last stays");
    assert_eq!(
        *lock(&heard),
        [change(FieldKind::None, 1)],
        "nothing more told"
    );
}

#[test]
fn a_look_at_our_own_focus_is_none() {
    let (mut run, _, _) = started(LONG_MS, |inbox| {
        for job in inbox {
            if let Ask::Look { reply, .. } = job.0 {
                let _ = reply.send(Ok(None));
            }
        }
    });
    assert_eq!(run.look(), Ok(None));
}

#[test]
fn a_look_times_out_past_its_cap_and_an_ended_thread_is_unavailable() {
    let (mut run, _, _) = started(SHORT_MS, |inbox| {
        let held: Vec<Job> = inbox.iter().collect();
        drop(held);
    });
    assert_eq!(run.look(), Err(FieldError::TimedOut));
    let (mut run, _, _) = started(LONG_MS, |_| {});
    assert_eq!(run.look(), Err(FieldError::Unavailable));
}

#[test]
fn a_dead_watch_looks_at_nothing() {
    let (failed, done) = mpsc::channel();
    let (mut run, _, _) = with_shared(LONG_MS, move |inbox, shared| {
        shared.fail("the OS client did not start");
        let _ = failed.send(());
        answers(inbox);
    });
    assert_eq!(done.recv_timeout(WAIT), Ok(()));
    assert_eq!(run.look(), Err(FieldError::Unavailable));
}

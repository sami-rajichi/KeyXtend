//! The run's end checks: an end past `read_chars` sends no read, and a check hands back the kind the reader saw.

use super::*;

#[test]
fn an_end_longer_than_read_chars_answers_none_and_sends_no_read() {
    let jobs = Arc::new(AtomicUsize::new(0));
    let seen = Arc::clone(&jobs);
    let (mut run, shared, _) = started(LONG_MS, move |inbox| {
        for job in inbox {
            seen.fetch_add(1, Ordering::SeqCst);
            if let Ask::Ends { reply, .. } = job.0 {
                let _ = reply.send(Ok(Some(ended(FieldKind::Text, Some(true)))));
            }
        }
    });
    shared.moved(&Script, &other(FieldKind::Text, 1));
    let end = |n: u16| Redacted::new("ب".repeat(usize::from(n)));
    let (fits, over) = (
        ended(FieldKind::Text, Some(true)),
        ended(FieldKind::Text, None),
    );
    assert_eq!(run.ends_with(&end(CHARS)), fits, "at the cap");
    assert_eq!(run.ends_with(&end(CHARS + 1)), over, "past the cap");
    assert_eq!(jobs.load(Ordering::SeqCst), 1);
}

#[test]
fn a_check_reports_the_kind_the_reader_saw_and_our_own_focus_cannot_tell() {
    let (mut run, shared, _) = started(LONG_MS, |inbox| {
        for (job, n) in inbox.iter().zip(0_u8..) {
            let Ask::Ends { reply, .. } = job.0 else {
                continue;
            };
            let seen = ended(FieldKind::Password, Some(false));
            let _ = reply.send(Ok((n == 0).then_some(seen)));
        }
    });
    shared.moved(&Script, &other(FieldKind::Text, 1));
    let seen = ended(FieldKind::Password, Some(false));
    assert_eq!(ends(run.as_mut()), seen, "seen at the check");
    let ours = ended(FieldKind::Text, None);
    assert_eq!(ends(run.as_mut()), ours, "our own focus");
}

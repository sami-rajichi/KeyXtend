//! The run with scripted thread bodies: closed boxes send no read, reads come back or time out, an ended thread is
//! unavailable, and a dropped run tells nothing more.

use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

/// How long a test waits for a thread; a wait, never a sleep.
const WAIT: Duration = Duration::from_secs(5);
/// A short `read_ms`, for the read that is never answered.
const SHORT_MS: u32 = 30;
/// A `read_ms` no answered read reaches.
const LONG_MS: u32 = 5000;

/// What a test heard from its run's `tell`.
type Heard = Arc<Mutex<Vec<FieldChange>>>;

/// A run over `body` with `read_ms`, the shared box its thread got, and what its `tell` heard.
fn started<B>(read_ms: u32, body: B) -> (Box<dyn FieldRun>, Arc<Shared>, Heard)
where
    B: FnOnce(&Receiver<Job>) + Send + 'static,
{
    let heard = Heard::default();
    let log = Arc::clone(&heard);
    let tell: Tell = Box::new(move |change| lock(&log).push(change));
    let (give, got) = mpsc::channel();
    let body = move |inbox: &Receiver<Job>, shared: &Arc<Shared>, _: u32| {
        let _ = give.send(Arc::clone(shared));
        body(inbox);
    };
    let run = spawn(FieldConfig { read_ms }, tell, body).unwrap();
    (run, got.recv_timeout(WAIT).unwrap(), heard)
}

/// A body that answers every read: the words asked as text, and whether the text is "ap".
fn answers(inbox: &Receiver<Job>) {
    for job in inbox {
        match job {
            Job::Before {
                max_words, reply, ..
            } => {
                let _ = reply.send(Ok(Some(format!("{max_words} words"))));
            }
            Job::Ends { text, reply, .. } => {
                let _ = reply.send(Ok(Some(text.expose() == "ap")));
            }
        }
    }
}

/// A box of `kind` with id `box_id`.
fn change(kind: FieldKind, box_id: u64) -> FieldChange {
    FieldChange { kind, box_id }
}

/// The text a run reads before the cursor, unwrapped.
fn read(run: &mut dyn FieldRun, max_words: u8) -> Result<Option<String>, FieldError> {
    run.before_cursor(max_words)
        .map(|t| t.map(Redacted::into_inner))
}

/// Asks the run whether the text ends with `ap`.
fn ends(run: &mut dyn FieldRun) -> Option<bool> {
    run.ends_with(&Redacted::new(String::from("ap")))
}

#[test]
fn a_password_unknown_or_empty_box_sends_no_read() {
    let jobs = Arc::new(AtomicUsize::new(0));
    let seen = Arc::clone(&jobs);
    let (mut run, shared, _) = started(LONG_MS, move |inbox| {
        for job in inbox {
            seen.fetch_add(1, Ordering::SeqCst);
            drop(job);
        }
    });
    assert_eq!(run.kind(), FieldKind::None, "no box before the first focus");
    for (kind, id) in [FieldKind::Password, FieldKind::Unknown, FieldKind::None]
        .into_iter()
        .zip(1..)
    {
        shared.told(change(kind, id));
        assert_eq!(read(run.as_mut(), 2), Ok(None), "{kind:?}");
        assert_eq!(ends(run.as_mut()), None, "{kind:?}");
    }
    assert_eq!(jobs.load(Ordering::SeqCst), 0);
}

#[test]
fn a_text_box_read_comes_back_from_the_thread() {
    let (mut run, shared, heard) = started(LONG_MS, answers);
    shared.told(change(FieldKind::Text, 7));
    assert_eq!(run.kind(), FieldKind::Text);
    assert_eq!(read(run.as_mut(), 3), Ok(Some("3 words".to_owned())));
    assert_eq!(ends(run.as_mut()), Some(true));
    assert_eq!(*lock(&heard), [change(FieldKind::Text, 7)]);
}

#[test]
fn a_read_past_its_cap_times_out() {
    let (mut run, shared, _) = started(SHORT_MS, |inbox| {
        let held: Vec<Job> = inbox.iter().collect();
        drop(held);
    });
    shared.told(change(FieldKind::Text, 1));
    assert_eq!(read(run.as_mut(), 2), Err(FieldError::TimedOut));
    assert_eq!(ends(run.as_mut()), None);
}

#[test]
fn a_thread_that_ended_is_unavailable() {
    let (mut run, shared, _) = started(LONG_MS, |_| {});
    shared.told(change(FieldKind::Text, 1));
    assert_eq!(read(run.as_mut(), 2), Err(FieldError::Unavailable));
    assert_eq!(ends(run.as_mut()), None);
}

#[test]
fn dropping_the_run_stops_its_tells_and_ends_the_thread() {
    let (ended, done) = mpsc::channel();
    let (run, shared, heard) = started(LONG_MS, move |inbox| {
        answers(inbox);
        let _ = ended.send(());
    });
    shared.told(change(FieldKind::Text, 1));
    drop(run);
    shared.told(change(FieldKind::Password, 2));
    assert_eq!(*lock(&heard), [change(FieldKind::Text, 1)]);
    assert_eq!(
        shared.kind(),
        FieldKind::Text,
        "nothing is kept after the drop"
    );
    assert_eq!(done.recv_timeout(WAIT), Ok(()));
}

#[test]
fn the_watch_thread_has_its_name() {
    let (name, got) = mpsc::channel();
    let (_run, _, _) = started(LONG_MS, move |_| {
        let _ = name.send(thread::current().name().map(str::to_owned));
    });
    assert_eq!(got.recv_timeout(WAIT), Ok(Some(THREAD.to_owned())));
}

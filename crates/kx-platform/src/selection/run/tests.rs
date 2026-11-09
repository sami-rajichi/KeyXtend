//! The run over scripted thread bodies: answers come back from the thread, a closed box or our own focus sends no read,
//! a dropped run tells nothing more, an ended watch is unavailable, and a panicking tell is caught.

use super::*;
use crate::selection::rig::{Script, Stop, WAIT, own, theirs};
use crate::selection::serve;
use crate::worker::fake::Hands;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Receiver;

mod hung;
mod paced;

/// A `read_ms` no answered job reaches.
const LONG_MS: u32 = 5000;
/// The test pacing: a change settles in `SETTLE_MS`, and `LOOKS` tells fit in a second.
const SETTLE_MS: u32 = 100;
const LOOKS: u16 = 3;

/// A run whose thread serves each job in turn on a clock the test moves, and what it told and took. Its thread passes
/// the probe's `Take` gate before each job, so a test can hold a job whatever its deadline.
struct Rig {
    run: Box<dyn SelRun>,
    shared: Arc<Shared>,
    clock: Hands,
    /// How many tells were heard.
    tells: Arc<AtomicUsize>,
    /// One message per tell, to wait for one.
    heard: Receiver<()>,
    /// How many jobs the thread took.
    taken: Arc<AtomicUsize>,
    /// One message per job served, to wait for one.
    served: Receiver<()>,
}

/// The test config with `read_ms`.
fn cfg(read_ms: u32) -> SelConfig {
    SelConfig {
        settle_ms: SETTLE_MS,
        read_ms,
        looks_per_s: LOOKS,
    }
}

/// A tell that counts and signals each change it hears, then runs `then`.
fn counted(then: fn()) -> (Tell, Arc<AtomicUsize>, Receiver<()>) {
    let (say, heard) = mpsc::channel();
    let tells = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&tells);
    let tell: Tell = Box::new(move || {
        count.fetch_add(1, Ordering::SeqCst);
        let _ = say.send(());
        then();
    });
    (tell, tells, heard)
}

/// A run over `probe` with `read_ms`.
fn rig(probe: &Script, read_ms: u32) -> Rig {
    rig_then(probe, read_ms, || {})
}

/// Like `rig`, with a tell that runs `then` after it counts.
fn rig_then(probe: &Script, read_ms: u32, then: fn()) -> Rig {
    let (tell, tells, heard) = counted(then);
    let (done, served) = mpsc::channel();
    let (give, got) = mpsc::channel();
    let taken = Arc::new(AtomicUsize::new(0));
    let (probe, count) = (probe.clone(), Arc::clone(&taken));
    let body = move |inbox: Inbox, shared: &Arc<Shared>, _: SelConfig| {
        let _ = give.send(Arc::clone(shared));
        for job in inbox {
            count.fetch_add(1, Ordering::SeqCst);
            probe.pass(Stop::Take);
            serve(&probe, job);
            let _ = done.send(());
        }
    };
    let clock = Hands::default();
    let run = start(cfg(read_ms), Arc::new(clock.clone()), tell, body).unwrap();
    let shared = got.recv_timeout(WAIT).unwrap();
    Rig {
        run,
        shared,
        clock,
        tells,
        heard,
        taken,
        served,
    }
}

impl Rig {
    /// How many tells were heard so far.
    fn tells(&self) -> usize {
        self.tells.load(Ordering::SeqCst)
    }

    /// How many jobs the thread took so far.
    fn taken(&self) -> usize {
        self.taken.load(Ordering::SeqCst)
    }
}

#[test]
fn the_watch_thread_has_its_name_and_the_config() {
    let (seen, got) = mpsc::channel();
    let body = move |_: Inbox, _: &Arc<Shared>, cfg: SelConfig| {
        let _ = seen.send((thread::current().name().map(str::to_owned), cfg));
    };
    let _run = spawn(cfg(LONG_MS), Box::new(|| {}), body).unwrap();
    assert_eq!(
        got.recv_timeout(WAIT),
        Ok((Some(THREAD.to_owned()), cfg(LONG_MS)))
    );
}

#[test]
fn each_answer_comes_back_from_the_thread() {
    let probe = Script::on(Ok(theirs(SelKind::Editable, 7)));
    probe.text("hello");
    probe.clip(true);
    let mut rig = rig(&probe, LONG_MS);
    assert_eq!(rig.run.look(), Ok(theirs(SelKind::Editable, 7).1));
    assert!(rig.run.still(7));
    assert_eq!(
        rig.run.read(7, 4).map(Redacted::into_inner),
        Ok("hell".to_owned())
    );
    assert!(rig.run.clip_holds());
    assert_eq!(rig.taken(), 4);
}

#[test]
fn a_closed_box_sends_no_read() {
    let probe = Script::on(Ok(theirs(SelKind::Closed, 2)));
    probe.text("secret");
    let mut rig = rig(&probe, LONG_MS);
    assert_eq!(
        rig.run.read(2, 4),
        Err(SelError::Closed),
        "no box before a look"
    );
    assert_eq!(rig.run.look().map(|s| s.kind), Ok(SelKind::Closed));
    assert_eq!(rig.run.read(2, 4), Err(SelError::Closed));
    assert_eq!(rig.taken(), 1, "only the look");
    assert_eq!(probe.calls().reads, 0);
}

#[test]
fn our_own_focus_keeps_the_last_box_and_is_never_read() {
    let probe = Script::on(Ok(theirs(SelKind::Editable, 5)));
    let mut rig = rig(&probe, LONG_MS);
    let theirs = rig.run.look().unwrap();
    assert!(!rig.run.ours(), "another app's box");
    probe.focus(Ok(own(6)));
    assert_eq!(rig.run.look(), Ok(theirs), "the last box");
    assert!(rig.run.ours(), "a look that kept it says so");
    assert!(!rig.run.still(5));
    assert_eq!(
        rig.run.read(5, 4),
        Err(SelError::Closed),
        "our own text is never read"
    );
    assert_eq!(rig.taken(), 3, "the read sent no job");
    assert_eq!(probe.calls().reads, 0);
}

#[test]
fn the_real_clock_tells_a_change_once_it_settles() {
    let (tell, tells, heard) = counted(|| {});
    let probe = Script::on(Ok(theirs(SelKind::Editable, 1)));
    let serving = probe.clone();
    let (give, got) = mpsc::channel();
    let body = move |inbox: Inbox, shared: &Arc<Shared>, _: SelConfig| {
        let _ = give.send(Arc::clone(shared));
        inbox.for_each(|job| serve(&serving, job));
    };
    let mut run = spawn(cfg(LONG_MS), tell, body).unwrap();
    let shared: Arc<Shared> = got.recv_timeout(WAIT).unwrap();
    shared.moved(&probe, &theirs(SelKind::Editable, 1));
    assert_eq!(heard.recv_timeout(WAIT), Ok(()));
    assert!(run.look().is_ok());
    assert_eq!(tells.load(Ordering::SeqCst), 1);
}

#[test]
fn dropping_the_run_stops_its_tells_and_ends_the_thread() {
    let (ended, done) = mpsc::channel();
    let (tell, tells, _heard) = counted(|| {});
    let (give, got) = mpsc::channel();
    let body = move |inbox: Inbox, shared: &Arc<Shared>, _: SelConfig| {
        let _ = give.send(Arc::clone(shared));
        inbox.for_each(drop);
        let _ = ended.send(());
    };
    let run = start(cfg(LONG_MS), Arc::new(Hands::default()), tell, body).unwrap();
    let shared: Arc<Shared> = got.recv_timeout(WAIT).unwrap();
    drop(run);
    assert_eq!(done.recv_timeout(WAIT), Ok(()));
    shared.moved(
        &Script::on(Err(SelError::Refused)),
        &theirs(SelKind::Editable, 1),
    );
    assert_eq!(tells.load(Ordering::SeqCst), 0);
}

#[test]
fn a_watch_that_failed_to_start_or_whose_thread_ended_is_unavailable() {
    let (failed, done) = mpsc::channel();
    let body = move |_: Inbox, shared: &Arc<Shared>, _: SelConfig| {
        shared.fail("the OS client did not start");
        shared.fail("again");
        let _ = failed.send(());
    };
    let mut failed = spawn(cfg(LONG_MS), Box::new(|| {}), body).unwrap();
    assert_eq!(done.recv_timeout(WAIT), Ok(()));
    let mut ended = spawn(cfg(LONG_MS), Box::new(|| {}), |_, _, _| {}).unwrap();
    for run in [&mut failed, &mut ended] {
        assert_eq!(run.look(), Err(SelError::Unavailable));
        assert!(!run.still(1));
        assert_eq!(
            run.read(1, 4),
            Err(SelError::Closed),
            "no box was looked at"
        );
        assert!(!run.clip_holds());
    }
}

#[test]
fn a_tell_that_panics_never_ends_the_watch() {
    let probe = Script::on(Ok(theirs(SelKind::Editable, 1)));
    let mut rig = rig_then(&probe, LONG_MS, || panic!("a module bug"));
    for _ in 0..2 {
        rig.shared.moved(&probe, &theirs(SelKind::Editable, 1));
        rig.clock.advance(Duration::from_secs(1));
        assert_eq!(rig.run.look().map(|s| s.box_id), Ok(1), "still watching");
    }
    assert_eq!(rig.tells(), 2);
}

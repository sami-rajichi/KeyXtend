//! Downloads: the ask checked and recorded, progress, a stall, a pause and a resume, a bad fingerprint and a cancel
//! on cue, each with its part file, the named file once done, and no note after a drop.

use super::*;
use kx_module_api::ports::Link;
use std::sync::mpsc;

/// What a test fetch heard, with the thread it heard it on.
type Heard = Arc<Mutex<Vec<(FetchNote, ThreadId)>>>;

fn ask() -> FetchAsk {
    let link = Link::new("https://huggingface.co/m/resolve/0a1b/x.gguf".into()).unwrap();
    FetchAsk {
        job: kx_module_api::ports::JobLimits {
            job_mb: 512,
            idle_min: 5,
            ask_ms: 3_000,
            load_s: 60,
            warm_s: 180,
            piece_wait_s: 120,
            restart_ms: 1_000,
            restart_max_ms: 30_000,
        },
        links: vec![link],
        hosts: vec!["huggingface.co".into()],
        file: Redacted::new(PathBuf::from(FILE)),
        bytes: BYTES,
        print: print(),
        min_free_mb: 2_048,
        stall_s: 30,
    }
}

/// A started download of `ask` whose notes are recorded.
fn fetching(fake: &FakeModelFiles) -> (Box<dyn FetchRun>, Heard) {
    let heard = Heard::default();
    let log = Arc::clone(&heard);
    let tell = move |note| log.lock().unwrap().push((note, thread::current().id()));
    (fake.fetch(ask(), Box::new(tell)).ok().unwrap(), heard)
}

fn notes(heard: &Heard) -> Vec<FetchNote> {
    heard.lock().unwrap().iter().map(|(n, _)| *n).collect()
}

fn progress(done: u64) -> FetchNote {
    FetchNote::Progress { done, total: BYTES }
}

#[test]
fn a_fetch_checks_its_ask_and_its_folder_and_records_it() {
    let fake = FakeModelFiles::new();
    let odd = FetchAsk {
        stall_s: 0,
        ..ask()
    };
    let refused = fake.fetch(odd, Box::new(|_| {})).err();
    assert_eq!(refused, Some(ModelError::Range("stall_s")));
    let homeless = fake.fetch(ask(), Box::new(|_| {})).err();
    assert_eq!(homeless, Some(ModelError::Missing), "no models folder");
    assert!(fake.fetches().is_empty());
    let fake = with_dir();
    let (_run, _heard) = fetching(&fake);
    let fetches = fake.fetches();
    assert_eq!(fetches.len(), 1);
    assert_eq!(fetches[0].ask, ask());
    assert_eq!(fake.calls(FileCall::Fetch), 1);
}

#[test]
fn a_fetch_progresses_on_cue_then_its_file_takes_its_name() {
    let fake = with_dir();
    let (_run, heard) = fetching(&fake);
    assert!(fake.progress(BYTES / 2));
    assert_eq!(fake.part(Path::new(FILE)), Some(BYTES / 2));
    assert!(
        !fake.has_file(Path::new(FILE)),
        "no named file before the check"
    );
    assert!(fake.tell(FetchNote::Done));
    assert!(!fake.progress(BYTES), "an ended download tells no more");
    assert_eq!(notes(&heard), [progress(BYTES / 2), FetchNote::Done]);
    let me = thread::current().id();
    assert!(heard.lock().unwrap().iter().all(|(_, t)| *t == me));
    assert_eq!(fake.part(Path::new(FILE)), None);
    assert_eq!(fake.fingerprint(Path::new(FILE)), Ok(print()));
    let listed = names(&fake.scan(Path::new(DIR), &exts()).unwrap()).1;
    assert_eq!(listed, [("parakeet-5521.gguf".into(), BYTES)]);
    assert!(fake.fetches()[0].ended);
}

#[test]
fn a_stall_keeps_the_part_and_a_bad_fingerprint_deletes_it() {
    let fake = with_dir();
    let (_first, heard) = fetching(&fake);
    assert!(fake.progress(50));
    let stalled = FetchNote::Fault(ModelError::Stalled);
    assert!(fake.tell(stalled));
    assert_eq!(notes(&heard), [progress(50), stalled]);
    assert_eq!(fake.part(Path::new(FILE)), Some(50), "kept to resume");
    let (_second, again) = fetching(&fake);
    assert!(fake.progress(BYTES));
    let bad = FetchNote::Fault(ModelError::Print);
    assert!(fake.tell(bad));
    assert_eq!(notes(&again), [progress(BYTES), bad]);
    assert_eq!(fake.part(Path::new(FILE)), None);
    assert!(!fake.has_file(Path::new(FILE)), "a bad file is never named");
}

#[test]
fn pause_and_resume_are_recorded_and_cancel_tells_its_fault_and_deletes_the_part() {
    let fake = with_dir();
    let (mut run, heard) = fetching(&fake);
    assert!(fake.progress(10));
    run.pause();
    assert!(fake.fetches()[0].paused);
    run.resume();
    assert!(fake.progress(20));
    run.cancel();
    let cancelled = FetchNote::Fault(ModelError::Cancelled);
    assert_eq!(notes(&heard), [progress(10), progress(20), cancelled]);
    assert_eq!(fake.part(Path::new(FILE)), None);
    let life = &fake.fetches()[0];
    let counts = (life.pauses, life.resumes, life.cancels);
    assert_eq!(counts, (1, 1, 1));
    assert!(life.ended && !life.paused);
}

#[test]
fn a_paused_fetch_moves_no_bytes_until_resumed_but_still_hears_a_fault() {
    let fake = with_dir();
    let (mut run, heard) = fetching(&fake);
    assert!(fake.progress(10));
    run.pause();
    assert!(!fake.progress(20), "a paused download reads nothing");
    assert!(!fake.tell(FetchNote::Done));
    assert_eq!(fake.part(Path::new(FILE)), Some(10));
    run.resume();
    assert!(fake.progress(20));
    run.pause();
    let gone = FetchNote::Fault(ModelError::Gone);
    assert!(fake.tell(gone));
    assert_eq!(notes(&heard), [progress(10), progress(20), gone]);
    assert_eq!(fake.part(Path::new(FILE)), Some(20), "kept to resume");
}

#[test]
fn a_part_vanishes_by_its_own_path_and_not_with_its_file() {
    let fake = with_dir();
    let (_run, _heard) = fetching(&fake);
    assert!(fake.progress(10));
    fake.vanish(Path::new(FILE));
    assert_eq!(
        fake.part(Path::new(FILE)),
        Some(10),
        "the part is another file"
    );
    fake.vanish(&ask().part().unwrap());
    assert_eq!(fake.part(Path::new(FILE)), None);
}

#[test]
fn a_drop_waits_out_a_note_being_told_and_none_comes_after() {
    let (inside, entered) = mpsc::channel();
    let (go, held) = mpsc::channel::<()>();
    let order = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&order);
    let tell = move |_: FetchNote| {
        inside.send(()).unwrap();
        held.recv().unwrap();
        seen.lock().unwrap().push("heard");
    };
    let fake = with_dir();
    let run = fake.fetch(ask(), Box::new(tell)).ok().unwrap();
    let teller = fake.clone();
    let teller = thread::spawn(move || teller.progress(1));
    entered.recv_timeout(WITHIN).unwrap();
    let after = Arc::clone(&order);
    let dropper = thread::spawn(move || {
        drop(run);
        after.lock().unwrap().push("dropped");
    });
    assert!(fake.wait(WITHIN, |fetches| fetches[0].dropped));
    go.send(()).unwrap();
    assert!(teller.join().unwrap());
    dropper.join().unwrap();
    assert_eq!(*order.lock().unwrap(), ["heard", "dropped"]);
    assert!(!fake.progress(2));
}

#[test]
fn dropping_a_fetch_pauses_it_and_keeps_the_part_until_removed() {
    let fake = with_dir();
    let (run, _heard) = fetching(&fake);
    assert!(fake.progress(10));
    drop(run);
    let life = &fake.fetches()[0];
    assert!(life.dropped && life.paused && !life.ended);
    assert!(!fake.progress(20), "a dropped download tells no more");
    assert_eq!(fake.part(Path::new(FILE)), Some(10));
    let part = ask().part().unwrap();
    assert_eq!(fake.remove(&part), Ok(()));
    assert_eq!(fake.part(Path::new(FILE)), None);
}

#[test]
fn a_failure_set_for_fetch_refuses_it_once() {
    let fake = with_dir();
    for error in [ModelError::Full, ModelError::ReadOnly] {
        fake.fail(FileCall::Fetch, error);
        assert_eq!(fake.fetch(ask(), Box::new(|_| {})).err(), Some(error));
    }
    assert!(fake.fetches().is_empty());
    let (_run, _heard) = fetching(&fake);
    assert_eq!(fake.fetches().len(), 1);
}

#[test]
fn clones_share_and_a_waiter_wakes_for_a_fetch_on_another_thread() {
    let fake = with_dir();
    let port: Arc<dyn ModelFiles> = Arc::new(fake.clone());
    let heard = Heard::default();
    let log = Arc::clone(&heard);
    let tell = move |note| log.lock().unwrap().push((note, thread::current().id()));
    let other = thread::spawn(move || port.fetch(ask(), Box::new(tell)).ok());
    assert!(fake.wait(WITHIN, |fetches| fetches.len() == 1));
    let run = other.join().unwrap();
    assert!(run.is_some());
    assert!(fake.progress(1));
    assert_eq!(notes(&heard), [progress(1)]);
}

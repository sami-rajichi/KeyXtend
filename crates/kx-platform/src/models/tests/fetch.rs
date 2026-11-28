//! A download in a job of its own: its cap and Get, the room checked first, pause, resume and cancel unwatched, a drop
//! that pauses and goes quiet, and a lost job told once.

use super::*;
use crate::worker::Launch;
use kx_ipc::{FETCH_JOB, FetchFault};

/// The part holds nothing yet.
fn started(id: u32) -> kx_ipc::FetchNote {
    kx_ipc::FetchNote::Progress {
        id,
        got: 0,
        total: BYTES,
    }
}

/// The file was checked and named.
fn named(id: u32) -> kx_ipc::FetchNote {
    kx_ipc::FetchNote::Done { id }
}

/// The download was cancelled and its part deleted.
fn cancelled(id: u32) -> kx_ipc::FetchNote {
    let fault = FetchFault::Cancelled;
    kx_ipc::FetchNote::Fault { id, fault }
}

/// The next note heard.
fn next(heard: &Receiver<FetchNote>) -> FetchNote {
    heard.recv_timeout(LONG).expect("told in time")
}

#[test]
fn a_download_runs_in_a_job_of_its_own_capped_at_job_mb() {
    let script = Script::default();
    let models = host(&script, roomy(need()));
    script.act(says(vec![started]));
    let (tell, heard) = ear();
    let _run = models.fetch(ask(&file()), tell).unwrap();
    assert_eq!(
        next(&heard),
        FetchNote::Progress {
            done: 0,
            total: BYTES
        }
    );
    let job = Launch {
        args: vec![FETCH_JOB.to_owned()],
        memory_mb: limits().job_mb,
    };
    assert_eq!(script.launches(), [job]);
    let [
        kx_ipc::FetchAsk::Get {
            urls,
            hosts,
            path,
            size,
            sha256,
            ..
        },
    ] = &gets(&script)[..]
    else {
        panic!("one Get");
    };
    assert_eq!(
        (&urls[..], &hosts[..], &**path),
        (&[URL.into()][..], &[HOST.into()][..], &*file())
    );
    assert_eq!((*size, *sha256), (BYTES, PRINT));
    script.say(Ok(named(1)));
    assert_eq!(next(&heard), FetchNote::Done);
}

#[test]
fn a_drive_without_room_for_the_rest_and_min_free_mb_is_full_before_any_job() {
    let script = Script::default();
    let short = host(&script, roomy(need() - 1));
    assert_eq!(
        short.fetch(ask(&file()), ear().0).err(),
        Some(ModelError::Full)
    );
    let gone = host(&script, FakeDisk::default());
    assert_eq!(
        gone.fetch(ask(&file()), ear().0).err(),
        Some(ModelError::Missing)
    );
    assert_eq!(script.tries(), 0);
    let mut odd = ask(&file());
    odd.stall_s = 0;
    let range = host(&script, roomy(need())).fetch(odd, ear().0).err();
    assert_eq!(range, Some(ModelError::Range("stall_s")));
}

#[test]
fn a_job_that_cannot_start_is_gone() {
    let script = Script::default();
    script.refuse(1);
    let models = host(&script, roomy(need()));
    assert_eq!(
        models.fetch(ask(&file()), ear().0).err(),
        Some(ModelError::Gone)
    );
}

#[test]
fn pause_resume_and_cancel_go_to_its_job_and_a_cancel_tells_cancelled() {
    let script = Script::default();
    let models = host(&script, roomy(need()));
    script.act(says(vec![started]));
    let (tell, heard) = ear();
    let mut run = models.fetch(ask(&file()), tell).unwrap();
    next(&heard);
    run.pause();
    run.resume();
    script.act(says(vec![cancelled]));
    run.cancel();
    assert_eq!(next(&heard), FetchNote::Fault(ModelError::Cancelled));
    run.pause();
    let ids: Vec<&str> = gets(&script).iter().map(kind).collect();
    assert_eq!(
        ids,
        ["get", "pause", "resume", "cancel"],
        "nothing after its end"
    );
}

#[test]
fn progress_still_on_its_way_after_a_pause_is_not_told_until_resume() {
    use crate::ear::Ear;
    use crate::worker::Told;
    use std::sync::atomic::AtomicBool;
    let (tell, heard) = ear();
    let (ear, ended) = (Ear::new(tell), AtomicBool::new(false));
    let progress = Told::Note(started(1));
    super::super::fetch::heard(&ear, (&ended, &AtomicBool::new(true)), &progress);
    assert!(heard.try_recv().is_err(), "none while paused");
    super::super::fetch::heard(&ear, (&ended, &AtomicBool::new(false)), &progress);
    assert_eq!(
        heard.try_recv().ok(),
        Some(FetchNote::Progress {
            done: 0,
            total: BYTES
        })
    );
}

#[test]
fn a_dropped_download_pauses_says_goodbye_and_tells_nothing_more() {
    let script = Script::default();
    let models = host(&script, roomy(need()));
    script.act(says(vec![started]));
    let (tell, heard) = ear();
    let run = models.fetch(ask(&file()), tell).unwrap();
    next(&heard);
    let (to, seen) = mpsc::channel();
    script.tell(to);
    drop(run);
    let calls: Vec<Call> = std::iter::from_fn(|| seen.recv_timeout(LONG).ok())
        .take_while(|c| *c != Call::Bye)
        .filter(|c| matches!(c, Call::Fetch(_)))
        .collect();
    assert!(
        matches!(&calls[..], [Call::Fetch(kx_ipc::FetchAsk::Pause { id: 1 })]),
        "{calls:?}"
    );
    assert!(
        heard.recv_timeout(LONG).is_err(),
        "its callback went with it"
    );
}

#[test]
fn a_job_lost_mid_download_is_gone_once_and_one_lost_after_the_end_tells_nothing() {
    let script = Script::default();
    let models = host(&script, roomy(need()));
    script.act(says(vec![started]));
    let (tell, heard) = ear();
    let mut run = models.fetch(ask(&file()), tell).unwrap();
    next(&heard);
    script.quit(3);
    assert_eq!(next(&heard), FetchNote::Fault(ModelError::Gone));
    run.cancel();
    assert_eq!(
        gets(&script).len(),
        1,
        "a cancel its job cannot take is done here"
    );
    script.act(says(vec![started, named]));
    let (tell, heard) = ear();
    let _done = models.fetch(ask(&file()), tell).unwrap();
    assert_eq!(
        next(&heard),
        FetchNote::Progress {
            done: 0,
            total: BYTES
        }
    );
    assert_eq!(next(&heard), FetchNote::Done);
    script.quit(3);
    let late = heard.recv_timeout(Duration::from_millis(1));
    assert!(late.is_err(), "nothing after its end: {late:?}");
}

#[test]
fn a_part_counts_toward_the_room_and_a_cancel_after_the_job_deletes_it_here() {
    // A temp folder that goes even when the test fails.
    let dir = kx_test_support::tempdir::TempDir::new("fetch").unwrap();
    let file = dir.path().join("a.gguf");
    let ask = ask(file.to_str().unwrap());
    let part = ask.part().unwrap();
    let held: u16 = 1_000;
    std::fs::write(&part, vec![0; usize::from(held)]).unwrap();
    let script = Script::default();
    let models = host(&script, roomy(need() - u64::from(held)));
    script.act(says(vec![started]));
    let (tell, heard) = ear();
    let mut run = models.fetch(ask, tell).unwrap();
    next(&heard);
    script.quit(3);
    next(&heard);
    run.cancel();
    assert!(!part.exists(), "the part went");
}

/// The kind of fetch ask `ask` is.
fn kind(ask: &kx_ipc::FetchAsk<'static>) -> &'static str {
    match ask {
        kx_ipc::FetchAsk::Get { .. } => "get",
        kx_ipc::FetchAsk::Pause { .. } => "pause",
        kx_ipc::FetchAsk::Resume { .. } => "resume",
        kx_ipc::FetchAsk::Cancel { .. } => "cancel",
        kx_ipc::FetchAsk::Bye => "bye",
    }
}

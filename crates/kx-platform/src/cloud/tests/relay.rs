//! A run's relay: no job before `send`, one job per run sized by its sound, the row, the key and each piece's size
//! first, then the sound in wiped parts in order, the texts back in order, and a drop that cancels at once.

use super::*;
use kx_ipc::{MAX_SOUND_BYTES, NET_JOB, NetHeader, NetShape, Wiped};
use std::sync::{Mutex, PoisonError};

#[test]
fn nothing_starts_before_send_and_each_run_has_a_job_of_its_own() {
    let (host, script, _) = host();
    assert_eq!(script.spawns(), 0, "nothing leaves before Stop (edge 1)");
    let sizes = [PER_S, 2 * PER_S];
    let _first = sent(&host, ask(&sizes));
    let _second = sent(&host, ask(&sizes));
    let launches = script.launches();
    assert_eq!(launches.len(), 2, "a job per run");
    let room = size::run_mb(3 * PER_S, limits().at_once, 2 * PER_S);
    assert_eq!(launches[0].args, [NET_JOB]);
    assert_eq!(launches[0].memory_mb, job().job_mb + room);
}

#[test]
fn the_row_and_the_key_go_first_then_the_sound_in_wiped_parts_in_order() {
    let (host, script, _) = host();
    let sizes = [PER_S, MAX_SOUND_BYTES + PER_S, 2 * SAMPLE_BYTES];
    let (_run, _) = sent(&host, ask(&sizes));
    let asks = nets(&script);
    let NetAsk::Send { id, call, run } = &asks[0] else {
        panic!("a send first, not {:?}", asks[0]);
    };
    assert_eq!(*id, ID);
    assert!(matches!(call.shape, NetShape::Form(_)) && call.header == NetHeader::Bearer);
    assert_eq!((call.url.as_ref(), call.wait_s), (GROQ, WAIT_S));
    assert!(matches!(&call.key, Wiped::Kept(k) if k.as_slice() == KEY.as_bytes()));
    let wanted: Vec<u32> = sizes.iter().map(|&n| u32::try_from(n).unwrap()).collect();
    assert_eq!(
        (run.sizes.clone(), run.tries, run.lang.as_ref()),
        (wanted, TRIES, LANG)
    );
    assert_eq!(run.model.as_deref(), Some(MODEL));
    let parts: Vec<(u16, usize)> = asks[1..]
        .iter()
        .map(|ask| match ask {
            NetAsk::Sound {
                id: ID,
                piece,
                bytes: Wiped::Kept(b),
            } => (*piece, b.len()),
            other => panic!("a wiped part of run 1, not {other:?}"),
        })
        .collect();
    let split = [
        (0, PER_S),
        (1, MAX_SOUND_BYTES),
        (1, PER_S),
        (2, 2 * SAMPLE_BYTES),
    ];
    assert_eq!(parts, split, "each piece in parts of one frame at most");
}

#[test]
fn texts_come_back_in_order_then_the_end_and_the_job_is_told_goodbye() {
    let (host, script, _) = host();
    let (calls, called) = mpsc::channel();
    script.tell(calls);
    let (_run, heard) = sent(&host, ask(&[PER_S, PER_S]));
    script.say(Ok(text(0, SAID)));
    script.say(Ok(text(1, "")));
    script.say(Ok(NetNote::Done { id: ID }));
    let told = next(&heard, 3);
    assert_eq!(told, [piece(0, SAID), piece(1, ""), CloudNote::Done]);
    assert!(said_bye(&called), "an ended run keeps no job");
}

#[test]
fn a_ten_minute_dictation_goes_in_several_pieces_within_the_row() {
    let (host, script, _) = host();
    let piece_s = 30;
    let sizes = vec![piece_s * PER_S; 600 / piece_s];
    let (_run, _) = sent(&host, ask(&sizes));
    let asks = nets(&script);
    let NetAsk::Send { run, .. } = &asks[0] else {
        panic!("a send first");
    };
    assert_eq!(run.sizes.len(), 20, "several pieces (edge 8)");
    let most = limits().piece_bytes();
    assert!(
        run.sizes.iter().all(|&n| n as usize <= most),
        "each within the row"
    );
    assert_eq!(run.limits.at_once, limits().at_once, "several at a time");
    let sound: usize = asks[1..]
        .iter()
        .map(|a| match a {
            NetAsk::Sound { bytes, .. } => bytes.len(),
            _ => 0,
        })
        .sum();
    assert_eq!(sound, 600 * PER_S, "every sample handed over once");
}

#[test]
fn a_fault_after_its_tries_is_told_once_then_the_job_is_told_goodbye() {
    let (host, script, _) = host();
    let (calls, called) = mpsc::channel();
    script.tell(calls);
    let (_run, heard) = sent(&host, ask(&[PER_S, PER_S]));
    let NetAsk::Send { run, .. } = &nets(&script)[0] else {
        panic!("a send first");
    };
    assert_eq!(run.tries, TRIES, "the job tries each piece net_tries times");
    script.say(Ok(text(0, SAID)));
    script.say(Ok(fault(NetFault::Network)));
    let fault = CloudNote::Fault(CloudFault::Network);
    assert_eq!(next(&heard, 2), [piece(0, SAID), fault]);
    assert!(said_bye(&called), "an ended run keeps no job");
    script.say(Ok(text(1, SAID)));
    assert!(heard.try_recv().is_err(), "nothing after");
}

#[test]
fn a_job_that_ends_tells_gone() {
    let (host, script, _) = host();
    let (_run, heard) = sent(&host, ask(&[PER_S]));
    script.quit(1);
    assert_eq!(next(&heard, 1), [CloudNote::Fault(CloudFault::Gone)]);
}

#[test]
fn a_bad_ask_is_refused_before_any_job_starts() {
    let (host, script, _) = host();
    let tell = || Box::new(|_| ()) as Box<dyn Fn(CloudNote) + Send>;
    let mut long = ask(&[limits().piece_bytes() + SAMPLE_BYTES]);
    assert_eq!(
        host.send(long, tell()).err(),
        Some(CloudFault::Range("pieces"))
    );
    long = ask(&[]);
    assert_eq!(
        host.send(long, tell()).err(),
        Some(CloudFault::Range("pieces"))
    );
    assert_eq!(script.spawns(), 0, "refused before any job (carry 4)");
}

#[test]
fn dropping_a_run_cancels_its_requests_and_says_goodbye() {
    let (host, script, _) = host();
    let (calls, called) = mpsc::channel();
    script.tell(calls);
    let (run, heard) = sent(&host, ask(&[PER_S, PER_S]));
    drop(run);
    took(&called, &NetAsk::Cancel { id: ID });
    assert!(said_bye(&called), "a goodbye");
    script.say(Ok(text(0, SAID)));
    assert!(heard.try_recv().is_err(), "a closed run tells nothing");
}

/// Waits for a goodbye among the calls `called` hears.
fn said_bye(called: &Receiver<Call>) -> bool {
    std::iter::from_fn(|| called.recv_timeout(LONG).ok()).any(|call| call == Call::Bye)
}

#[test]
fn a_run_dropped_inside_its_own_callback_cancels_without_waiting() {
    let (host, script, _) = host();
    let (calls, called) = mpsc::channel();
    script.tell(calls);
    let held: Arc<Mutex<Option<Box<dyn CloudRun>>>> = Arc::default();
    let own = Arc::clone(&held);
    let tell = Box::new(move |_| {
        let run = own.lock().unwrap_or_else(PoisonError::into_inner).take();
        drop(run);
    });
    let run = host
        .send(ask(&[PER_S, PER_S]), tell)
        .expect("the run starts");
    *held.lock().unwrap_or_else(PoisonError::into_inner) = Some(run);
    script.say(Ok(text(0, SAID)));
    took(&called, &NetAsk::Cancel { id: ID });
}

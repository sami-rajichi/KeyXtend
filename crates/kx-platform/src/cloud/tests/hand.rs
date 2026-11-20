//! A run's hand-over: it stops once the run ended, and a part the job cannot take ends the run as `Gone` and cancels
//! it. A run that told its end, or was closed, tells nothing more.

use super::board::Board;
use super::job::NetJob;
use super::live::{Live, Tell};
use super::*;
use crate::worker::Told;

/// A job over `script`, unbegun, for a run of `sizes`, and what it tells.
fn opened(script: &Script<NetFrames>, sizes: &[usize]) -> (NetJob<Script<NetFrames>>, Heard) {
    let (said, heard) = mpsc::channel();
    let tell: Tell = Box::new(move |note| drop(said.send(note)));
    let board = Board::new(sizes.to_vec(), WAIT_S, TRIES, kx_ipc::NET_PACE_S);
    let shape = (size::launch(&job(), 0), size::times(&job()));
    let spawner = Arc::new(script.clone());
    let opened = NetJob::open(spawner, Arc::new(Hands::default()), shape, board, tell);
    (opened.expect("the job opens"), heard)
}

/// Begins `job`'s run with a key test's call, as the scripted worker takes any ask.
fn begun(job: &NetJob<Script<NetFrames>>) -> u32 {
    let test = key_test();
    let call = frame::call(test.shape, &test.url, test.header, test.key, WAIT_S);
    job.begin(move |id| NetAsk::Test { id, call })
        .expect("the run begins")
}

/// A live run of `n` one-second pieces with no job, and what it tells.
fn alone(n: usize) -> (Live, Heard) {
    let (said, heard) = mpsc::channel();
    let board = Board::new(vec![PER_S; n], WAIT_S, TRIES, kx_ipc::NET_PACE_S);
    let tell: Tell = Box::new(move |note| drop(said.send(note)));
    (Live::new(board, tell, Arc::new(Hands::default())), heard)
}

#[test]
fn the_hand_over_stops_once_the_run_ended() {
    let script = Script::default();
    let (job, heard) = opened(&script, &[PER_S]);
    let id = begun(&job);
    job.live().hear(Told::Note(NetNote::Fault {
        id,
        fault: NetFault::Network,
    }));
    job.hand_over(id, vec![sound(PER_S)]);
    assert_eq!(heard.try_recv(), Ok(CloudNote::Fault(CloudFault::Network)));
    let parts = nets(&script)
        .into_iter()
        .filter(|a| matches!(a, NetAsk::Sound { .. }));
    assert_eq!(parts.count(), 0, "no part after the end");
}

#[test]
fn a_part_the_job_cannot_take_ends_the_run_as_gone_and_cancels_it() {
    let script = Script::default();
    let (calls, called) = mpsc::channel();
    script.tell(calls);
    let (job, heard) = opened(&script, &[3]);
    let id = begun(&job);
    let part_sample = Redacted::new(Zeroizing::new(vec![1; 3]));
    job.hand_over(id, vec![part_sample]);
    let told = heard.recv_timeout(LONG);
    assert_eq!(told, Ok(CloudNote::Fault(CloudFault::Gone)), "told at once");
    took(&called, &NetAsk::Cancel { id });
}

#[test]
fn a_run_that_told_its_end_or_was_closed_tells_nothing_more() {
    let (ended, heard) = alone(2);
    ended.hear(Told::Note(fault(NetFault::Network)));
    ended.hear(Told::Note(text(0, SAID)));
    let told: Vec<CloudNote> = heard.try_iter().collect();
    assert_eq!(told, [CloudNote::Fault(CloudFault::Network)]);
    let (closed, heard) = alone(2);
    closed.close();
    closed.hear(Told::Note(text(0, SAID)));
    assert!(heard.try_recv().is_err(), "a closed run tells nothing");
}

//! A run's job is sized by its sound, its bodies in flight and their replies, and each piece has a time bound scaled
//! by its size from the moment the one before it is told; past it the run times out and the job's run is cancelled,
//! on a clock the test moves. A key's test waits for no turn in the pace.

use super::*;
use kx_ipc::{MAX_RECORD_BYTES, NET_AT_ONCE, NET_MAX_S, NET_PACE_S, piece_bytes};

/// The longest piece a row may name, in bytes: ten minutes.
fn longest() -> usize {
    piece_bytes(u32::MAX, *NET_MAX_S.end())
}

#[test]
fn a_job_holds_the_sound_the_bodies_in_flight_and_their_replies() {
    let worst = size::run_mb(MAX_RECORD_BYTES, *NET_AT_ONCE.end(), longest());
    assert_eq!(worst, 467, "about half a GB at worst");
    assert_eq!(size::run_mb(0, 1, 0), 4, "a key test holds one reply");
    let minute = 60 * PER_S;
    assert!(
        size::run_mb(0, 2, minute) < size::run_mb(minute, 2, minute),
        "the sound counts"
    );
    assert!(
        size::run_mb(0, 2, minute) < size::run_mb(0, 3, minute),
        "each request in flight"
    );
}

#[test]
fn a_pieces_bound_grows_with_its_size_its_tries_and_the_wait() {
    let (s, pace) = (Duration::from_secs, NET_PACE_S);
    let half = size::bound(30 * PER_S, 60, 2, pace);
    assert_eq!(half, s(660), "half a minute's piece");
    assert_eq!(size::bound(longest(), 60, 2, pace), s(2_940), "ten minutes");
    let test = size::bound(0, 5, 1, TEST_PACE_S);
    assert_eq!(test, s(20), "a key test at the shortest wait, with no turn");
    assert!(size::bound(PER_S, 60, 3, pace) > size::bound(PER_S, 60, 2, pace));
}

#[test]
fn a_piece_past_its_bound_times_out_and_cancels_the_jobs_run() {
    let (host, script, clock) = host();
    let (calls, called) = mpsc::channel();
    script.tell(calls);
    let (_run, heard) = sent(&host, ask(&[PER_S, 2 * PER_S]));
    let first = size::bound(PER_S, WAIT_S, TRIES, NET_PACE_S);
    clock.advance(first.saturating_sub(Duration::from_millis(1)));
    host.poke();
    assert!(heard.try_recv().is_err(), "not yet");
    clock.advance(Duration::from_millis(1));
    host.poke();
    assert_eq!(next(&heard, 1), [CloudNote::Fault(CloudFault::Timeout)]);
    took(&called, &NetAsk::Cancel { id: ID });
}

#[test]
fn each_text_starts_the_next_pieces_time() {
    let (host, script, clock) = host();
    let (_run, heard) = sent(&host, ask(&[PER_S, 20 * PER_S]));
    let first = size::bound(PER_S, WAIT_S, TRIES, NET_PACE_S);
    clock.advance(first.saturating_sub(Duration::from_secs(1)));
    script.say(Ok(text(0, SAID)));
    assert_eq!(next(&heard, 1), [piece(0, SAID)]);
    let second = size::bound(20 * PER_S, WAIT_S, TRIES, NET_PACE_S);
    clock.advance(second.saturating_sub(Duration::from_millis(1)));
    host.poke();
    assert!(heard.try_recv().is_err(), "the larger piece's own time");
    clock.advance(Duration::from_millis(1));
    host.poke();
    assert_eq!(next(&heard, 1), [CloudNote::Fault(CloudFault::Timeout)]);
}

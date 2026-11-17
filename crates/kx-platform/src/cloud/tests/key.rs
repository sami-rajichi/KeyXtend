//! A key's test: a job of its own that sends no sound, answers on the job's end or fault, times out past its bound,
//! and refuses a bad test before any job starts.

use super::*;
use std::thread;

/// Runs `host.test` on a thread of its own while `then` plays the worker, and gives its answer.
fn tested(
    host: &Host,
    script: &Script<NetFrames>,
    then: impl FnOnce(&Receiver<Call>),
) -> Result<(), CloudFault> {
    let (calls, called) = mpsc::channel();
    script.tell(calls);
    thread::scope(|s| {
        let asked = s.spawn(|| host.test(key_test()));
        then(&called);
        asked.join().expect("the test returns")
    })
}

/// Waits for the test's ask, then gives it `note`.
fn answer(script: &Script<NetFrames>, note: NetNote) -> impl FnOnce(&Receiver<Call>) {
    move |called| {
        while !matches!(
            called.recv_timeout(LONG),
            Ok(Call::Net(NetAsk::Test { .. }))
        ) {}
        script.say(Ok(note));
    }
}

#[test]
fn a_key_test_sends_no_sound_and_a_done_says_the_key_works() {
    let (host, script, _) = host();
    let done = tested(&host, &script, answer(&script, NetNote::Done { id: ID }));
    assert_eq!(done, Ok(()));
    let asks = nets(&script);
    assert!(matches!(&asks[0], NetAsk::Test { id: ID, call } if call.url == MODELS));
    assert!(
        !asks.iter().any(|a| matches!(a, NetAsk::Sound { .. })),
        "no sound"
    );
    let room = size::run_mb(0, 1, 0);
    assert_eq!(script.launches()[0].memory_mb, job().job_mb + room);
}

#[test]
fn a_refused_key_is_told_as_the_key() {
    let (host, script, _) = host();
    let refused = tested(&host, &script, answer(&script, fault(NetFault::Key)));
    assert_eq!(refused, Err(CloudFault::Key));
}

#[test]
fn a_key_test_past_its_bound_is_a_timeout() {
    let (host, script, clock) = host();
    let late = tested(&host, &script, |called| {
        while !matches!(
            called.recv_timeout(LONG),
            Ok(Call::Net(NetAsk::Test { .. }))
        ) {}
        clock.advance(size::bound(0, WAIT_S, kx_ipc::NET_TEST_TRIES, TEST_PACE_S));
        host.poke();
    });
    assert_eq!(late, Err(CloudFault::Timeout));
}

#[test]
fn a_bad_key_test_is_refused_before_any_job() {
    let (host, script, _) = host();
    let mut bad = key_test();
    bad.key = Redacted::new(Zeroizing::new(String::new()));
    assert_eq!(host.test(bad), Err(CloudFault::Range("key")));
    assert_eq!(script.spawns(), 0);
}

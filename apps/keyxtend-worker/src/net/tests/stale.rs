//! Outcomes the job never tells: an older run's under a reused id, and a key test's after its Cancel. A note that fails
//! its own check goes down the pipe as `Gone`.

use super::bench::{Bench, gate, part, send, text};
use super::*;
use crate::net::Notes;
use crate::stream::Pipe;
use kx_ipc::{MAX_SPEECH_PIECES, NetAsk, NetFault, NetNote};
use std::sync::mpsc::{Receiver, Sender};

/// A bench whose requests wait at a gate, with the gate's ends: calls started, and the release.
fn gated() -> (Bench, Receiver<()>, Sender<()>) {
    let (http, started, free) = gate();
    (Bench::new(http, &Arc::new(Still::default())), started, free)
}

#[test]
fn an_older_runs_outcome_under_a_reused_id_is_never_told() {
    let (mut b, started, free) = gated();
    b.ask(send(20, call(), run_of(&[2], 1, 1, 20)));
    b.ask(part(20, 0, sound(2, 1)));
    started.recv_timeout(bench::WAIT).unwrap();
    b.ask(NetAsk::Cancel { id: 20 });
    b.ask(send(20, call(), run_of(&[2], 1, 1, 20)));
    b.ask(part(20, 0, sound(2, 1)));
    free.send(()).unwrap();
    b.one();
    assert!(b.out.is_empty(), "the cancelled run's outcome is dropped");
    started.recv_timeout(bench::WAIT).unwrap();
    free.send(()).unwrap();
    assert_eq!(b.until(2), [text(20, 0, "x"), NetNote::Done { id: 20 }]);
}

#[test]
fn a_key_test_cancelled_while_it_waits_tells_nothing() {
    let (mut b, started, free) = gated();
    let call = call_to(MODELS, NetShape::Form(form()));
    b.ask(NetAsk::Test { id: 21, call });
    started.recv_timeout(bench::WAIT).unwrap();
    b.ask(NetAsk::Cancel { id: 21 });
    free.send(()).unwrap();
    b.one();
    assert!(b.out.is_empty());
}

#[test]
fn a_note_that_fails_its_own_check_goes_as_gone() {
    let mut bytes = Vec::new();
    let index = u16::try_from(MAX_SPEECH_PIECES).unwrap();
    let text = Zeroizing::new(String::new());
    Pipe(&mut bytes)
        .tell(NetNote::Text { id: 4, index, text })
        .unwrap();
    let gone = NetNote::Fault {
        id: 4,
        fault: NetFault::Gone,
    };
    assert_eq!(NetNote::read(&mut &bytes[..]), Ok(gone));
}

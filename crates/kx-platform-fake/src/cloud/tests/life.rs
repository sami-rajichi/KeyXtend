//! A run's life: a slow piece held until the test answers it, a cut mid-way, a drop that cancels the unanswered
//! pieces and waits out a note being told, and a run sent from another thread.

use super::*;
use std::sync::mpsc;
use std::thread;

#[test]
fn a_slow_piece_holds_until_the_test_answers_it() {
    let fake = FakeCloud::new();
    fake.script(vec![text("un")]);
    let (_run, heard) = sent(&fake, &[2, 2, 2]);
    assert_eq!(notes(&heard), [piece(0, "un")]);
    assert_eq!(fake.calls().runs[0].state, CloudState::Sending);
    assert!(fake.answer(0, vec![text("deux")]));
    assert_eq!(notes(&heard), [piece(0, "un"), piece(1, "deux")]);
    assert!(fake.answer(0, vec![text("trois"), text("de trop")]));
    let end = [piece(2, "trois"), CloudNote::Done];
    assert_eq!(
        notes(&heard)[2..],
        end,
        "an answer past the pieces is dropped"
    );
    assert!(
        !fake.answer(0, vec![text(SAID)]),
        "a finished run tells nothing"
    );
    assert!(!fake.answer(1, vec![text(SAID)]), "no such run");
    assert!(!fake.answer(0, Vec::new()));
}

#[test]
fn a_cut_mid_way_tells_its_fault_and_nothing_after() {
    for fault in [CloudFault::Network, CloudFault::Timeout, CloudFault::Gone] {
        let fake = FakeCloud::new();
        fake.script(vec![text("un")]);
        let (_run, heard) = sent(&fake, &[2, 2, 2]);
        assert!(fake.cut(0, fault), "{fault:?}");
        assert!(!fake.answer(0, vec![text("deux")]));
        assert!(!fake.cut(0, fault), "it already ended");
        assert_eq!(notes(&heard), [piece(0, "un"), CloudNote::Fault(fault)]);
        let run = &fake.calls().runs[0];
        let end = (run.told, run.state, run.cancelled());
        assert_eq!(end, (1, CloudState::Fault(fault), 0));
    }
    assert!(!FakeCloud::new().cut(0, CloudFault::Network), "no such run");
}

#[test]
fn dropping_a_run_cancels_its_unanswered_pieces_and_it_hears_no_more() {
    let fake = FakeCloud::new();
    fake.script(vec![text("un")]);
    let (run, heard) = sent(&fake, &[2, 2, 2]);
    drop(run);
    assert!(!fake.answer(0, vec![text("deux"), text("trois")]));
    assert!(!fake.cut(0, CloudFault::Network));
    assert_eq!(notes(&heard), [piece(0, "un")]);
    let log = &fake.calls().runs[0];
    assert_eq!(
        (log.state, log.told, log.cancelled()),
        (CloudState::Dropped, 1, 2)
    );
}

#[test]
fn dropping_a_finished_run_keeps_its_end() {
    let empty = CloudFault::Empty;
    let ends = [
        (text("a"), CloudState::Done),
        (CloudAnswer::Fault(empty), CloudState::Fault(empty)),
    ];
    for (answer, end) in ends {
        let fake = FakeCloud::new();
        fake.script(vec![answer]);
        drop(sent(&fake, &[2]).0);
        let log = &fake.calls().runs[0];
        assert_eq!((log.state, log.cancelled()), (end, 0));
    }
}

#[test]
fn a_run_sent_on_another_thread_is_answered_from_the_test() {
    let fake = FakeCloud::new();
    let port: Arc<dyn CloudSpeech> = Arc::new(fake.clone());
    let (heard, tell) = hearer();
    let other = thread::spawn(move || port.send(ask(&[2]), tell).ok());
    assert!(fake.wait(WITHIN, |calls| calls.runs.len() == 1));
    let run = other.join().unwrap();
    assert!(run.is_some());
    assert!(fake.answer(0, vec![text(SAID)]));
    assert_eq!(notes(&heard), [piece(0, SAID), CloudNote::Done]);
    assert!(!fake.wait(Duration::ZERO, |calls| calls.runs.len() == 2));
}

#[test]
fn a_drop_waits_out_a_note_being_told_on_another_thread() {
    let fake = FakeCloud::new();
    let (inside, entered) = mpsc::channel();
    let (go, gate) = mpsc::channel::<()>();
    let tell: Tell = Box::new(move |_| {
        inside.send(()).unwrap();
        gate.recv().unwrap();
    });
    let run = fake.send(ask(&[2, 2]), tell).unwrap();
    let teller = fake.clone();
    let told = thread::spawn(move || teller.answer(0, vec![text("un")]));
    entered.recv_timeout(WITHIN).unwrap();
    let (gone, returned) = mpsc::channel();
    let dropper = thread::spawn(move || {
        drop(run);
        gone.send(()).unwrap();
    });
    assert!(fake.wait(WITHIN, |c| c.runs[0].state == CloudState::Dropped));
    assert!(returned.try_recv().is_err(), "the drop waits for the note");
    go.send(()).unwrap();
    returned.recv_timeout(WITHIN).unwrap();
    assert!(told.join().unwrap() && dropper.join().is_ok());
    assert!(
        !fake.answer(0, vec![text("deux")]),
        "nothing after the drop"
    );
    assert_eq!(fake.calls().runs[0].cancelled(), 1);
}

//! A run's pieces and texts: parts filling each piece in order, texts told in the pieces' order whatever order the
//! replies come in, a fault ending the run before its later pieces go, `Cancel` before and during an upload, `at_once`
//! and `per_min` over every try, a blank run, a text past its cap, a shape not built yet, a repeated id and the key's
//! test.

use super::bench::{Bench, Swap, WAIT, faked, fault, gate, part, send, text};
use super::*;
use kx_ipc::{MAX_NET_TEXT_BYTES, NetAsk, NetFault, NetNote};
use kx_platform_net::{NetError, SEND_CHUNK};
use std::sync::mpsc;

#[test]
fn a_run_tells_each_pieces_text_in_order_then_done() {
    let (mut b, http, _) = faked([said("Bonjour"), said(" le monde")]);
    b.ask(send(1, call(), run_of(&[4, 2], 1, 1, 20)));
    b.ask(part(1, 0, sound(2, 7)));
    b.ask(part(1, 0, sound(2, 8)));
    b.ask(part(1, 1, sound(2, 9)));
    let want = [
        text(1, 0, "Bonjour"),
        text(1, 1, " le monde"),
        NetNote::Done { id: 1 },
    ];
    assert_eq!(b.until(3), want);
    let called = http.called();
    assert_eq!(called.len(), 2);
    let data = [7u8, 7, 8, 8];
    assert!(
        called[0].body.windows(4).any(|w| w == data),
        "piece 0's samples in order"
    );
}

#[test]
fn texts_are_told_in_order_when_replies_come_out_of_order() {
    let (free, freed) = mpsc::channel();
    let mut b = Bench::new(
        Arc::new(Swap(Mutex::new(freed))),
        &Arc::new(Still::default()),
    );
    b.ask(send(2, call(), run_of(&[4, 4], 1, 2, 20)));
    b.ask(part(2, 0, sound(4, 0xA0)));
    b.ask(part(2, 1, sound(4, 0xA1)));
    b.one();
    assert!(b.out.is_empty(), "piece 1's text waits for piece 0's");
    free.send(()).unwrap();
    let want = [
        text(2, 0, "zero"),
        text(2, 1, "one"),
        NetNote::Done { id: 2 },
    ];
    assert_eq!(b.until(3), want);
}

#[test]
fn a_part_out_of_order_or_past_its_piece_ends_the_run_as_gone() {
    let cases: [&[(u16, usize)]; 5] = [
        &[(1, 2)],
        &[(0, 6)],
        &[(0, 4), (0, 2)],
        &[(0, 4), (1, 4), (1, 2)],
        &[(0, 4), (2, 2)],
    ];
    for parts in cases {
        let (mut b, _, _) = faked([said("a"), said("b")]);
        b.ask(send(3, call(), run_of(&[4, 4], 1, 1, 20)));
        for (piece, n) in parts {
            b.ask(part(3, *piece, sound(*n, 1)));
        }
        b.ask(part(3, 1, sound(2, 1)));
        assert_eq!(b.out, [fault(3, NetFault::Gone)], "{parts:?}");
    }
}

#[test]
fn parts_for_an_unknown_or_cancelled_run_are_ignored() {
    let (mut b, http, _) = faked([said("a")]);
    b.ask(part(9, 0, sound(2, 1)));
    b.ask(send(4, call(), run_of(&[2], 1, 1, 20)));
    b.ask(NetAsk::Cancel { id: 4 });
    b.ask(part(4, 0, sound(2, 1)));
    assert!(b.out.is_empty() && http.called().is_empty());
}

#[test]
fn a_fault_ends_the_run_and_its_later_pieces_are_never_sent() {
    let (mut b, http, _) = faked([reply(401, KEY_BAD), said("b"), said("c")]);
    b.ask(send(5, call(), run_of(&[2, 2, 2], 1, 1, 20)));
    for piece in 0..3 {
        b.ask(part(5, piece, sound(2, 1)));
    }
    assert_eq!(b.until(1), [fault(5, NetFault::Key)]);
    assert_eq!(http.called().len(), 1);
}

#[test]
fn cancel_drops_a_run_in_flight_and_tells_nothing_more() {
    let (gate, started, free) = gate();
    let mut b = Bench::new(gate, &Arc::new(Still::default()));
    b.ask(send(6, call(), run_of(&[2, 2], 1, 1, 20)));
    b.ask(part(6, 0, sound(2, 1)));
    b.ask(part(6, 1, sound(2, 1)));
    started.recv_timeout(WAIT).unwrap();
    b.ask(NetAsk::Cancel { id: 6 });
    free.send(()).unwrap();
    b.one();
    assert!(b.out.is_empty());
    assert!(started.try_recv().is_err(), "piece 1 never started");
}

#[test]
fn at_once_caps_the_requests_in_flight() {
    let (gate, started, free) = gate();
    let mut b = Bench::new(gate, &Arc::new(Still::default()));
    b.ask(send(7, call(), run_of(&[2, 2, 2], 1, 2, 20)));
    for piece in 0..3 {
        b.ask(part(7, piece, sound(2, 1)));
    }
    for _ in 0..2 {
        started.recv_timeout(WAIT).unwrap();
    }
    assert!(started.try_recv().is_err(), "a third waits for a free slot");
    free.send(()).unwrap();
    b.one();
    started.recv_timeout(WAIT).unwrap();
    for _ in 0..2 {
        free.send(()).unwrap();
    }
    let want = [
        text(7, 0, "x"),
        text(7, 1, "x"),
        text(7, 2, "x"),
        NetNote::Done { id: 7 },
    ];
    assert_eq!(b.until(4), want);
}

#[test]
fn per_min_spaces_the_starts_a_minute_apart() {
    let (mut b, _, clock) = faked((0..4).map(|_| said("x")));
    b.ask(send(8, call(), run_of(&[2; 4], 1, 4, 2)));
    for piece in 0..4 {
        b.ask(part(8, piece, sound(2, 1)));
    }
    assert_eq!(b.until(5).last(), Some(&NetNote::Done { id: 8 }));
    assert_eq!(
        clock.waits(),
        [60_000, 60_000],
        "the third and fourth wait a minute"
    );
}

#[test]
fn a_run_of_blank_texts_ends_empty_and_one_word_is_done() {
    let (mut b, _, _) = faked([said(" "), said("")]);
    b.ask(send(9, call(), run_of(&[2, 2], 1, 1, 20)));
    b.ask(part(9, 0, sound(2, 1)));
    b.ask(part(9, 1, sound(2, 1)));
    let want = [text(9, 0, " "), text(9, 1, ""), fault(9, NetFault::Empty)];
    assert_eq!(b.until(3), want);
    let (mut b, _, _) = faked([said(" "), said("oui")]);
    b.ask(send(9, call(), run_of(&[2, 2], 1, 1, 20)));
    b.ask(part(9, 0, sound(2, 1)));
    b.ask(part(9, 1, sound(2, 1)));
    assert_eq!(b.until(3).last(), Some(&NetNote::Done { id: 9 }));
}

#[test]
fn a_text_past_its_cap_is_strange() {
    let (mut b, _, _) = faked([said(&"a".repeat(MAX_NET_TEXT_BYTES + 1))]);
    b.ask(send(10, call(), run_of(&[2], 1, 1, 20)));
    b.ask(part(10, 0, sound(2, 1)));
    assert_eq!(b.until(1), [fault(10, NetFault::Strange)]);
}

#[test]
fn a_shape_this_job_does_not_build_yet_is_out_of_range() {
    let (mut b, http, _) = faked([said("a")]);
    b.ask(send(
        11,
        call_to(URL, NetShape::Gemini),
        run_of(&[2], 1, 1, 20),
    ));
    b.ask(part(11, 0, sound(2, 1)));
    assert_eq!(b.out, [fault(11, NetFault::Range)]);
    assert!(http.called().is_empty());
}

#[test]
fn a_repeated_id_is_gone_and_drops_the_live_one() {
    let (mut b, http, _) = faked([said("a")]);
    b.ask(send(12, call(), run_of(&[2], 1, 1, 20)));
    b.ask(send(12, call(), run_of(&[2], 1, 1, 20)));
    b.ask(part(12, 0, sound(2, 1)));
    assert_eq!(b.out, [fault(12, NetFault::Gone)]);
    assert!(http.called().is_empty());
}

#[test]
fn a_key_test_tells_done_or_its_fault() {
    let cases = [
        (reply(OK, LIST), NetNote::Done { id: 13 }),
        (reply(401, KEY_BAD), fault(13, NetFault::Key)),
    ];
    for (act, want) in cases {
        let (mut b, _, _) = faked([act]);
        let call = call_to(MODELS, NetShape::Form(form()));
        b.ask(NetAsk::Test { id: 13, call });
        assert_eq!(b.until(1), [want]);
    }
}

#[test]
fn cancel_stops_an_upload_between_its_chunks_and_tells_nothing() {
    let (mut b, http, _) = faked([said("x")]);
    let ((started, seen), (free, freed)) = (mpsc::channel(), mpsc::channel::<()>());
    let (started, freed) = (Mutex::new(started), Mutex::new(freed));
    http.on_chunk(move |n| {
        if n == 1 {
            lock(&started).send(()).unwrap();
            lock(&freed).recv_timeout(WAIT).unwrap();
        }
    });
    let size = SEND_CHUNK * 3;
    b.ask(send(
        14,
        call(),
        run_of(&[u32::try_from(size).unwrap()], 1, 1, 20),
    ));
    b.ask(part(14, 0, sound(size, 1)));
    seen.recv_timeout(WAIT).unwrap();
    b.ask(NetAsk::Cancel { id: 14 });
    free.send(()).unwrap();
    b.one();
    assert!(b.out.is_empty(), "nothing is told after Cancel");
    assert_eq!(
        http.called()[0].body.len(),
        SEND_CHUNK,
        "no chunk after the first"
    );
}

#[test]
fn a_retry_and_a_429_resend_wait_their_turn_within_per_min() {
    let (mut b, http, clock) = faked([Act::Fail(NetError::Net), said("x")]);
    b.ask(send(15, call(), run_of(&[2], 2, 1, 1)));
    b.ask(part(15, 0, sound(2, 1)));
    assert_eq!(b.until(2).last(), Some(&NetNote::Done { id: 15 }));
    assert_eq!((clock.waits(), http.called().len()), (vec![60_000], 2));
    let (mut b, _, clock) = faked([busy(&[("retry-after", "3")]), said("x")]);
    b.ask(send(16, call(), run_of(&[2], 1, 1, 1)));
    b.ask(part(16, 0, sound(2, 1)));
    assert_eq!(b.until(2).last(), Some(&NetNote::Done { id: 16 }));
    assert_eq!(clock.waits(), [3000, 60_000]);
}

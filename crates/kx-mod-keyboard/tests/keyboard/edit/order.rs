//! The edit's answer keeps the pieces' order, which the selection module's one swallowed break needs: the
//! bus hears every earlier piece, then `EditDone`, then the edit's own `Break`, whichever thread sends them, and a
//! no for an edit not sent comes after the earlier pieces too.

use super::{ask, done};
use crate::assist::{Bench, ear};
use crate::held::A;
use crate::told::text;
use kx_module_api::ports::{Edit, EditDone, FaceAction, KeyId, Piece, Typed};
use kx_module_api::{Bus, Module, Subscription};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::Duration;

/// How long a thread waits for the other side before the test fails.
const WAIT: Duration = Duration::from_secs(10);

/// What a reader of both events hears, in order.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Heard {
    Piece(Piece),
    Done(EditDone),
}

/// The keyboard with a reader of `Typed` and `EditDone` after `first`'s handlers, which go before the module starts.
fn bench(first: impl FnOnce(&Bus) -> Vec<Subscription>) -> (Bench, Arc<Mutex<Vec<Heard>>>) {
    let heard = Arc::<Mutex<Vec<Heard>>>::default();
    let h = Arc::clone(&heard);
    let b = Bench::after(None, move |bus| {
        let mut ears = first(bus);
        ears.push(ear(bus, &h, |e: &Typed| Heard::Piece(e.piece.clone())));
        ears.push(ear(bus, &h, |e: &EditDone| Heard::Done(*e)));
        ears
    });
    b.said();
    (b, heard)
}

/// What was heard so far.
fn heard(heard: &Mutex<Vec<Heard>>) -> Vec<Heard> {
    heard.lock().unwrap_or_else(PoisonError::into_inner).clone()
}

/// A first handler on `Typed` that, once `armed`, holds the thread sending that piece, and so the outbox, until the
/// test lets it go; it tells `inside` when it holds.
fn gate(
    armed: Arc<AtomicBool>,
    inside: Sender<()>,
    go: Receiver<()>,
) -> impl Fn(&Bus) -> Vec<Subscription> {
    let go = Arc::new(Mutex::new(go));
    move |bus: &Bus| {
        let (armed, inside, go) = (Arc::clone(&armed), inside.clone(), Arc::clone(&go));
        let hold = move |_: &Typed| {
            if armed.swap(false, SeqCst) {
                inside.send(()).unwrap();
                let go = go.lock().unwrap_or_else(PoisonError::into_inner);
                go.recv_timeout(WAIT).unwrap();
            }
        };
        vec![bus.subscribe(hold).unwrap()]
    }
}

#[test]
fn the_edits_answer_comes_after_the_keys_before_it_and_before_its_own_break() {
    let (b, seen) = bench(|_| Vec::new());
    b.rig.tap("esc").unwrap();
    b.send(&ask(3, Edit::Copy, false));
    let want = [
        Heard::Piece(Piece::Break),
        Heard::Done(done(3, true)),
        Heard::Piece(Piece::Break),
    ];
    assert_eq!(heard(&seen), want);
}

/// Holds another thread's key in the outbox, taps Esc, runs `edit` on the bench, finds nothing heard yet, then lets
/// the key go; everything heard, in order.
fn behind_a_held_key(edit: impl FnOnce(&mut Bench)) -> Vec<Heard> {
    let armed = Arc::new(AtomicBool::new(false));
    let ((inside, entered), (go, gated)) = (mpsc::channel(), mpsc::channel());
    let (mut b, seen) = bench(gate(Arc::clone(&armed), inside, gated));
    let face = Arc::clone(&b.rig.face);
    thread::scope(|s| {
        armed.store(true, SeqCst);
        let letter = FaceAction::Tap {
            id: KeyId::new(A),
            back: false,
        };
        let typer = s.spawn(move || face.act(letter));
        entered.recv_timeout(WAIT).unwrap();
        // The typer keeps the outbox, so Esc's break waits there while the edit runs.
        b.rig.tap("esc").unwrap();
        edit(&mut b);
        assert!(
            heard(&seen).is_empty(),
            "the answer waits behind Esc's break"
        );
        go.send(()).unwrap();
        assert!(typer.join().unwrap().is_ok());
    });
    heard(&seen)
}

#[test]
fn a_key_waiting_in_another_threads_outbox_still_goes_out_before_the_edits_answer() {
    let got = behind_a_held_key(|b| b.send(&ask(4, Edit::Copy, false)));
    let want = [
        Heard::Piece(text("a")),
        Heard::Piece(Piece::Break),
        Heard::Done(done(4, true)),
        Heard::Piece(Piece::Break),
    ];
    assert_eq!(got, want, "the selection module's order");
}

/// A stopped module, like a waiting fill, sends nothing, and its no still comes after the keys typed before it.
#[test]
fn an_edit_not_sent_still_answers_after_the_keys_waiting_before_it() {
    let got = behind_a_held_key(|b| {
        b.rig.module.stop();
        b.send(&ask(5, Edit::Copy, false));
    });
    let want = [
        Heard::Piece(text("a")),
        Heard::Piece(Piece::Break),
        Heard::Done(done(5, false)),
    ];
    assert_eq!(got, want, "the selection module's order");
}

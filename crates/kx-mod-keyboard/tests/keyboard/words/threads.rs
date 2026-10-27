//! Pieces from two threads at once: a click on the UI's thread and a picked word on the prediction module's, which the
//! click may stop; every piece is told once, in the order it reached the app, and a module that answers a piece at once
//! deadlocks nothing.

use super::{BKSP, Desk, text};
use crate::held::A;
use kx_module_api::Redacted;
use kx_module_api::ports::{FaceAction, KeyId, Piece, TypeWord, Typed};
use kx_platform_fake::Stroke;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::thread;

/// Clicks and words each thread sends.
const ROUNDS: usize = 200;

/// The piece `stroke` tells: its text, or an erase for Backspace.
fn told(stroke: &Stroke) -> Piece {
    match stroke {
        Stroke::Text(t, _) => Piece::Text(t.clone()),
        Stroke::Tap(k) if k.code() == BKSP => Piece::Erase,
        other => panic!("not sent here: {other:?}"),
    }
}

/// A picked word: one Backspace, then `t`.
fn word(t: &str) -> TypeWord {
    TypeWord {
        round: 0,
        erase: 1,
        text: Redacted::new(t.into()),
    }
}

#[test]
fn a_click_and_a_picked_word_at_once_tell_every_piece_once_in_the_order_sent() {
    let d = Desk::new(None);
    let face = Arc::clone(&d.rig.face);
    let bus = d.bus.clone();
    thread::scope(|s| {
        s.spawn(move || {
            for _ in 0..ROUNDS {
                let id = KeyId::new(A);
                face.act(FaceAction::Tap { id, back: false }).unwrap();
            }
        });
        s.spawn(move || {
            for _ in 0..ROUNDS {
                bus.publish(&word("ab")).unwrap();
            }
        });
    });
    let strokes = d.rig.fakes.injector.strokes();
    let clicks = strokes.iter().filter(|s| told(s) == text("a")).count();
    assert_eq!(clicks, ROUNDS, "every click went");
    assert!(
        strokes.len() <= ROUNDS * 3,
        "a word sends at most its two parts"
    );
    let want: Vec<Piece> = strokes.iter().map(told).collect();
    assert_eq!(d.pieces(), want);
}

#[test]
fn a_module_that_answers_a_piece_with_a_word_at_once_deadlocks_nothing() {
    let d = Desk::new(None);
    let bus = d.bus.clone();
    let asked = AtomicBool::new(false);
    let answer = move |e: &Typed| {
        if matches!(e.piece, Piece::Text(_)) && !asked.swap(true, SeqCst) {
            bus.publish(&word("I ")).unwrap();
        }
    };
    let _predict = d.bus.subscribe(answer).unwrap();
    d.rig.tap(A).unwrap();
    assert_eq!(d.pieces(), [text("a"), Piece::Erase, text("I ")]);
}

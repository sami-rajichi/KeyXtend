//! A picked word's pauses (`[keyboard] erase_gap_ms`): each waits on the thread that sent the word, with the face
//! free, so a click from the UI's thread goes through meanwhile and stops the rest; a restart brings a new gap.

use super::{BKSP, Desk, text};
use crate::held::A;
use crate::typing::{tap, unicode};
use kx_mod_keyboard::KeyboardSettings;
use kx_module_api::ports::{FaceAction, KeyId, Piece};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// The longest a wait looks for the UI thread's click; it never sleeps.
const WAIT: Duration = Duration::from_secs(10);

#[test]
fn a_picked_word_waits_on_the_senders_thread_and_leaves_the_face_free() {
    let d = Desk::new(None);
    let face = Arc::clone(&d.rig.face);
    let clicked = Arc::new(Mutex::new(Vec::new()));
    let heard = Arc::clone(&clicked);
    d.rig.fakes.pause.during(move || {
        let (tx, rx) = mpsc::channel();
        let face = Arc::clone(&face);
        thread::spawn(move || {
            let id = KeyId::new(A);
            let _ = tx.send(face.act(FaceAction::Tap { id, back: false }).is_ok());
        });
        heard.lock().unwrap().push(rx.recv_timeout(WAIT).ok());
    });
    d.word(1, "ab");
    assert_eq!(
        *clicked.lock().unwrap(),
        [Some(true)],
        "the click went through"
    );
    let gap = KeyboardSettings::from_defaults().unwrap().erase_gap_ms;
    let waits = d.rig.fakes.pause.waits();
    assert_eq!(
        waits,
        [(thread::current().id(), Duration::from_millis(gap))]
    );
    let strokes = [tap(BKSP), unicode("a")];
    assert_eq!(
        d.rig.fakes.injector.strokes(),
        strokes,
        "the key stops the rest"
    );
    assert_eq!(d.pieces(), [Piece::Erase, text("a")], "as sent");
}

#[test]
fn a_new_gap_reaches_the_keyboard_through_a_restart() {
    let mut d = Desk::new(None);
    d.rig.restart(Some(("erase_gap_ms", 0.into())));
    d.word(2, "ab");
    assert!(d.rig.fakes.pause.waits().is_empty(), "a zero gap");
    d.rig.restart(Some(("erase_gap_ms", 20.into())));
    d.word(1, "ab");
    let waits = d.rig.fakes.pause.waits();
    assert_eq!(waits.len(), 1);
    assert_eq!(waits.first().map(|w| w.1), Some(Duration::from_millis(20)));
}

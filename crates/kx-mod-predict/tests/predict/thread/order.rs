//! The typing mode in any start order: a keyboard that started first answers the module's ask at each start, so its
//! shortcut latch holds the lane empty from the first word.

use super::{Live, WAIT, hear, lang};
use crate::rig::{DAY, blob, learner};
use kx_module_api::ports::{
    FieldKind, Lane, Piece, Suggested, Typed, TypingMode, TypingModeAsk, WordsAsk,
};
use kx_module_api::{Bus, Module, ModuleCx, Redacted, Subscription};
use kx_platform_fake::{FakeFieldWatch, FakeSealedStore, FakeWordPacks};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};

/// A keyboard that started first with Ctrl latched: it answers each ask with its mode, then says it did.
fn keyboard(bus: &Bus) -> (Subscription, Receiver<()>) {
    let (said, answered) = mpsc::channel();
    let own = bus.clone();
    let ctrl = TypingMode {
        shortcut: true,
        ..TypingMode::default()
    };
    let sub = bus.subscribe(move |_: &TypingModeAsk| {
        own.publish(&ctrl).unwrap();
        let _ = said.send(());
    });
    (sub.unwrap(), answered)
}

/// The lane the module told for the last message before an `Again`, whose echo repeats its round.
fn before_again(lanes: &Receiver<Suggested>) -> Suggested {
    let mut last = lanes.recv_timeout(WAIT).unwrap();
    loop {
        let lane = lanes.recv_timeout(WAIT).unwrap();
        if lane.round == last.round {
            return last;
        }
        last = lane;
    }
}

#[test]
fn a_keyboard_that_started_first_answers_each_start_and_its_latch_holds_the_lane_empty() {
    let (field, store) = (FakeFieldWatch::new(), FakeSealedStore::new());
    let known = learner("en", &["hello", "help"], DAY);
    store.plant(&blob("en"), known.to_bytes());
    field.focus(FieldKind::Text);
    let listen = |bus: &Bus| (keyboard(bus), hear::<Suggested>(bus));
    let (mut run, ((_k, answered), (_l, lanes))) =
        Live::start(&field, Arc::new(FakeWordPacks::new()), &store, listen);
    answered.recv_timeout(WAIT).unwrap();
    run.send(&lang("en"));
    run.send(&Typed {
        piece: Piece::Text(Redacted::new("hel".to_owned())),
    });
    run.send(&WordsAsk::Again);
    assert_eq!(before_again(&lanes).lane, Lane::Empty, "Ctrl is latched");
    run.stop();
    run.module.start(&mut ModuleCx::new(&mut run.host)).unwrap();
    answered.recv_timeout(WAIT).unwrap();
    run.stop();
}

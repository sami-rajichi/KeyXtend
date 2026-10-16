//! The pointer on the pad, as the scroll module hears it: each change once, in order, newest last, from any thread,
//! and never while the module is off.

use super::{Heard, Pad};
use kx_module_api::ports::{ScrollChanged, ScrollPeek};
use proptest::collection::vec;
use proptest::prelude::*;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// How long a thread waits for the other side before the test fails.
const WAIT: Duration = Duration::from_secs(10);

/// The scroll module running and still.
const LIVE: ScrollChanged = ScrollChanged {
    live: true,
    dir: None,
    fast: false,
};

/// The scroll module comes live on its own thread while the pointer rests on the pad, and its `Peek(true)` is held
/// on the bus while the UI thread hears the pointer leave; the leave must still be heard last.
#[test]
fn a_peek_held_on_another_thread_never_lands_after_a_later_leave() {
    let (inside, entered) = mpsc::channel();
    let (go, gate) = mpsc::channel::<()>();
    let gate = Mutex::new(gate);
    let order = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::clone(&order);
    let first = AtomicBool::new(true);
    let p = Pad::after(None, move |bus| {
        let held = move |e: &ScrollPeek| {
            if first.swap(false, SeqCst) {
                inside.send(()).unwrap();
                gate.lock().unwrap().recv_timeout(WAIT).unwrap();
            }
            log.lock().unwrap().push(e.on);
        };
        vec![bus.subscribe(held).unwrap()]
    });
    p.hover(true);
    let bus = p.b.bus.clone();
    let scroll = thread::spawn(move || bus.publish(&LIVE).unwrap());
    entered.recv_timeout(WAIT).unwrap();
    p.hover(false);
    go.send(()).unwrap();
    scroll.join().unwrap();
    assert_eq!(
        *order.lock().unwrap(),
        [true, false],
        "the leave comes last"
    );
}

/// One thing that happens to the pad.
#[derive(Copy, Clone, Debug)]
enum Step {
    /// The pointer goes onto the pad or off it.
    Hover(bool),
    /// The scroll module says it runs, or stops.
    Live(bool),
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        any::<bool>().prop_map(Step::Hover),
        any::<bool>().prop_map(Step::Live)
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// A module that stops forgets the pointer, so in each run it hears `true` first, then each change once.
    #[test]
    fn while_live_the_module_last_heard_where_the_pointer_is(steps in vec(step(), 0..40)) {
        let p = Pad::quiet(None);
        let (mut hover, mut live, mut known) = (false, false, false);
        for s in steps {
            match s {
                Step::Hover(on) => {
                    p.hover(on);
                    hover = on;
                }
                Step::Live(on) => {
                    p.scroll(on, None, false);
                    live = on;
                    known &= on;
                }
            }
            for h in p.heard() {
                prop_assert!(matches!(h, Heard::Peek(_)), "{h:?}");
                prop_assert!(live, "a peek while the module is off");
                let on = h == Heard::Peek(true);
                prop_assert_ne!(on, known, "the same peek twice in a row");
                known = on;
            }
            if live {
                prop_assert_eq!(known, hover, "the module knows where the pointer is");
            }
        }
    }
}

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

/// A pad whose first `ScrollPeek` waits inside its bus handler until `go`, with each peek logged as it is heard.
struct Held {
    p: Pad,
    /// Says the first peek is waiting.
    entered: mpsc::Receiver<()>,
    /// Lets it go on.
    go: mpsc::Sender<()>,
    /// Each peek's `on`, in the order the handler finished.
    order: Arc<Mutex<Vec<bool>>>,
}

impl Held {
    /// The module with the pointer on the pad and the scroll module off.
    fn new() -> Self {
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
        Self {
            p,
            entered,
            go,
            order,
        }
    }

    /// The scroll module comes live on its own thread, whose `Peek(true)` waits; `then` runs meanwhile on this
    /// thread, and the peeks heard come back once both are done.
    fn while_held(&self, then: impl FnOnce(&Pad)) -> Vec<bool> {
        let bus = self.p.b.bus.clone();
        let scroll = thread::spawn(move || bus.publish(&LIVE).unwrap());
        self.entered.recv_timeout(WAIT).unwrap();
        then(&self.p);
        self.go.send(()).unwrap();
        scroll.join().unwrap();
        self.order.lock().unwrap().clone()
    }
}

/// The pointer leaves while the scroll thread's `Peek(true)` is held on the bus; the leave must still be heard last.
#[test]
fn a_peek_held_on_another_thread_never_lands_after_a_later_leave() {
    let order = Held::new().while_held(|p| {
        p.hover(false);
    });
    assert_eq!(order, [true, false], "the leave comes last");
}

/// The scroll module restarts while its `Peek(true)` is held: the stop and the new start merge into one send, and the
/// new run must still hear that the pointer is on the pad.
#[test]
fn a_quick_restart_while_a_peek_is_held_still_tells_the_new_run() {
    let order = Held::new().while_held(|p| {
        p.b.send(&ScrollChanged::OFF);
        p.b.send(&LIVE);
    });
    assert_eq!(order, [true, true], "the new run hears it");
}

/// The scroll module may have restarted unheard while the keyboard was down, so the keyboard's start tells its answer
/// that the pointer rests on the pad.
#[test]
fn a_keyboard_restart_tells_the_scroll_module_the_pointer_again() {
    let mut p = Pad::quiet(None);
    p.scroll(true, None, false);
    p.hover(true);
    assert_eq!(p.heard(), [Heard::Peek(true)]);
    p.b.rig.restart(Some(("lang_key", "cycle".into())));
    p.b.send(&LIVE);
    assert_eq!(p.heard(), [Heard::Ask, Heard::Peek(true)]);
}

/// A scroll module that kept running heard the pointer come onto the pad, so it must hear it leave while the keyboard
/// was down, or the outline stays.
#[test]
fn a_pointer_that_left_while_the_keyboard_restarted_is_told() {
    let mut p = Pad::quiet(None);
    p.scroll(true, None, false);
    p.hover(true);
    assert_eq!(p.heard(), [Heard::Peek(true)]);
    p.b.rig.restart(Some(("lang_key", "cycle".into())));
    p.hover(false);
    p.b.send(&LIVE);
    assert_eq!(p.heard(), [Heard::Ask, Heard::Peek(false)]);
}

/// One thing that happens to the pad.
#[derive(Copy, Clone, Debug)]
enum Step {
    /// The pointer goes onto the pad or off it.
    Hover(bool),
    /// The scroll module says it runs, or stops.
    Live(bool),
    /// The keyboard module starts again, and the scroll module keeps running.
    Restart,
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        3 => any::<bool>().prop_map(Step::Hover),
        3 => any::<bool>().prop_map(Step::Live),
        1 => Just(Step::Restart)
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// A module that stops forgets the pointer, so in each run it hears `true` first, then each change once; after a
    /// keyboard restart, the first peek the answer brings may repeat the last.
    #[test]
    fn while_live_the_module_last_heard_where_the_pointer_is(steps in vec(step(), 0..40)) {
        let mut p = Pad::quiet(None);
        let (mut hover, mut live, mut known, mut again) = (false, false, false, false);
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
                Step::Restart => {
                    p.b.rig.restart(None);
                    (live, again) = (false, true);
                }
            }
            for h in p.heard() {
                if h == Heard::Ask {
                    prop_assert!(matches!(s, Step::Restart), "an ask only at a start");
                    continue;
                }
                prop_assert!(matches!(h, Heard::Peek(_)), "{h:?}");
                prop_assert!(live, "a peek while the module is off");
                let on = h == Heard::Peek(true);
                prop_assert!(again || on != known, "the same peek twice in a row");
                known = on;
            }
            if live {
                prop_assert_eq!(known, hover, "the module knows where the pointer is");
                again = false;
            }
        }
    }
}

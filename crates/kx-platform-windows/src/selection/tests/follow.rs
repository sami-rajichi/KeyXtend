//! Where the selection handler sits, over a fake hook: added on an open box and kept there, moved to another box,
//! taken off a closed one, tried again after a failed add, left alone by a failed or late look, and never on more than
//! one answering element (property); a hung box is in `hung`.

use super::super::follow::{Heard, Hook, Seen};
use kx_module_api::ports::{Screen, SelError, SelKind, SelSpot};
use proptest::prelude::*;
use std::cell::{Cell, RefCell};

mod hung;

/// A hook call: `true` for an add, with the element it named.
type Call = (bool, u8);

/// A scripted hook: it logs each call, keeps the elements that hold the handler, refuses adds while `refuse`, and
/// calls the elements in `hung` hung.
#[derive(Default)]
struct Fake {
    calls: RefCell<Vec<Call>>,
    held: RefCell<Vec<u8>>,
    refuse: Cell<bool>,
    hung: RefCell<Vec<u8>>,
}

impl Fake {
    /// The calls since the last ask.
    fn take(&self) -> Vec<Call> {
        self.calls.take()
    }

    /// Marks the elements in `els` hung, and every other one answering.
    fn hang(&self, els: &[u8]) {
        *self.hung.borrow_mut() = els.to_vec();
    }
}

impl Hook for Fake {
    type El = u8;
    type On = u8;

    fn at(&self, el: &u8) -> u8 {
        *el
    }

    fn add(&self, el: &u8) -> bool {
        self.calls.borrow_mut().push((true, *el));
        let took = !self.refuse.get();
        if took {
            self.held.borrow_mut().push(*el);
        }
        took
    }

    fn remove(&self, el: &u8) {
        self.calls.borrow_mut().push((false, *el));
        let mut held = self.held.borrow_mut();
        let at = held.iter().position(|h| h == el);
        assert!(at.is_some(), "removed from {el}, where it was never added");
        if let Some(at) = at {
            held.remove(at);
        }
    }

    fn hung(&self, el: &u8) -> bool {
        self.hung.borrow().contains(el)
    }
}

/// An add on element `el`.
fn add(el: u8) -> Call {
    (true, el)
}

/// A removal from element `el`.
fn remove(el: u8) -> Call {
    (false, el)
}

#[test]
fn an_open_box_gets_the_handler_once_and_keeps_it() {
    let (hook, mut heard) = (Fake::default(), Heard::default());
    heard.look(&hook, &1, 10, Seen::Open);
    assert_eq!(hook.take(), [add(1)]);
    heard.look(&hook, &1, 10, Seen::Open);
    assert_eq!(hook.take(), [], "the same box keeps it");
}

#[test]
fn another_box_takes_it_from_the_last() {
    let (hook, mut heard) = (Fake::default(), Heard::default());
    heard.look(&hook, &1, 10, Seen::Open);
    heard.look(&hook, &2, 20, Seen::Open);
    assert_eq!(hook.take(), [add(1), remove(1), add(2)]);
}

#[test]
fn a_closed_box_only_loses_the_last_one() {
    let (hook, mut heard) = (Fake::default(), Heard::default());
    heard.look(&hook, &1, 10, Seen::Open);
    heard.look(&hook, &2, 20, Seen::Closed);
    assert_eq!(
        hook.take(),
        [add(1), remove(1)],
        "no handler on a closed box"
    );
    heard.look(&hook, &3, 30, Seen::Closed);
    assert_eq!(hook.take(), []);
    heard.look(&hook, &1, 10, Seen::Closed);
    heard.look(&hook, &1, 10, Seen::Open);
    assert_eq!(hook.take(), [add(1)], "the box opens again");
}

#[test]
fn a_box_turning_closed_loses_it_and_gets_it_back_when_open() {
    let (hook, mut heard) = (Fake::default(), Heard::default());
    heard.look(&hook, &1, 10, Seen::Open);
    heard.look(&hook, &1, 10, Seen::Closed);
    heard.look(&hook, &1, 10, Seen::Open);
    assert_eq!(hook.take(), [add(1), remove(1), add(1)]);
}

#[test]
fn a_failed_add_is_tried_again_at_the_next_look() {
    let (hook, mut heard) = (Fake::default(), Heard::default());
    hook.refuse.set(true);
    heard.look(&hook, &1, 10, Seen::Open);
    hook.refuse.set(false);
    heard.look(&hook, &1, 10, Seen::Open);
    heard.look(&hook, &1, 10, Seen::Open);
    assert_eq!(
        hook.take(),
        [add(1), add(1)],
        "no removal of what was never added"
    );
    heard.leave(&hook);
    assert_eq!(hook.take(), [remove(1)]);
}

#[test]
fn a_box_with_no_id_gets_it_afresh_at_each_look() {
    let (hook, mut heard) = (Fake::default(), Heard::default());
    heard.look(&hook, &1, 0, Seen::Open);
    heard.look(&hook, &1, 0, Seen::Open);
    assert_eq!(hook.take(), [add(1), remove(1), add(1)]);
}

#[test]
fn leaving_removes_it_once() {
    let (hook, mut heard) = (Fake::default(), Heard::default());
    heard.leave(&hook);
    heard.look(&hook, &1, 10, Seen::Open);
    heard.leave(&hook);
    heard.leave(&hook);
    assert_eq!(hook.take(), [add(1), remove(1)]);
}

#[test]
fn an_unsure_look_touches_nothing() {
    let (hook, mut heard) = (Fake::default(), Heard::default());
    heard.look(&hook, &1, 10, Seen::Unsure);
    assert_eq!(hook.take(), [], "nothing is added after a failed look");
    heard.look(&hook, &1, 10, Seen::Open);
    heard.look(&hook, &2, 20, Seen::Unsure);
    assert_eq!(
        hook.take(),
        [add(1)],
        "nothing is removed from a hung app past the deadline"
    );
    heard.look(&hook, &1, 10, Seen::Open);
    assert_eq!(hook.take(), [], "it is still on the last box");
    heard.look(&hook, &2, 20, Seen::Open);
    assert_eq!(hook.take(), [remove(1), add(2)]);
}

/// The spot of a box of `kind`.
fn spot(kind: SelKind) -> SelSpot {
    SelSpot {
        box_id: 10,
        kind,
        selected: true,
        first: None,
        last: None,
        caret: None,
        screen: Screen::default(),
    }
}

#[test]
fn a_look_is_unsure_when_it_failed_or_came_late() {
    let seen = |kind, late| Seen::of(&Ok(spot(kind)), late);
    assert_eq!(seen(SelKind::Editable, false), Seen::Open);
    assert_eq!(seen(SelKind::Terminal, false), Seen::Open);
    assert_eq!(seen(SelKind::Closed, false), Seen::Closed);
    for kind in [SelKind::ReadOnly, SelKind::Closed] {
        assert_eq!(seen(kind, true), Seen::Unsure, "late: {kind:?}");
    }
    for error in [SelError::TimedOut, SelError::Refused, SelError::Unavailable] {
        assert_eq!(Seen::of(&Err(error), false), Seen::Unsure, "{error:?}");
    }
}

/// Elements tried; the box id follows the element, 0 for the first, which has none.
const ELS: u8 = 4;

/// One look: an element, what it saw, whether its add fails, and which elements hang then.
fn any_look() -> impl Strategy<Value = (u8, Seen, bool, Vec<u8>)> {
    let seen = prop::sample::select(vec![Seen::Open, Seen::Closed, Seen::Unsure]);
    let hung = prop::collection::vec(0..ELS, 0..=2);
    (0..ELS, seen, any::<bool>(), hung)
}

proptest! {
    /// After a sure look, every answering element that holds the handler is the open box just looked at, so only hung
    /// windows keep an old one; no add goes to a hung app, no element holds two, and the stop takes every one off.
    #[test]
    fn the_handler_is_on_at_most_the_last_open_box(looks in prop::collection::vec(any_look(), 0..24)) {
        let (hook, mut heard) = (Fake::default(), Heard::default());
        for (el, seen, refuse, hung) in looks {
            hook.refuse.set(refuse);
            hook.hang(&hung);
            let before = hook.held.borrow().clone();
            heard.look(&hook, &el, u64::from(el), seen);
            let (held, calls) = (hook.held.borrow().clone(), hook.take());
            if seen == Seen::Unsure {
                prop_assert_eq!(held, before, "an unsure look touches nothing");
                continue;
            }
            let hung_add = calls.iter().any(|&(add, at)| add && hung.contains(&at));
            prop_assert!(!hung_add, "an add on a hung app: {calls:?}");
            let mut once = held.clone();
            once.sort_unstable();
            once.dedup();
            prop_assert_eq!(once.len(), held.len(), "one handler per element");
            let awake: Vec<_> = held.iter().filter(|h| !hung.contains(h)).collect();
            prop_assert!(awake.len() <= 1, "{held:?}");
            prop_assert!(awake.iter().all(|h| seen == Seen::Open && **h == el), "{held:?}");
            if seen == Seen::Open && !refuse && !hung.contains(&el) {
                prop_assert!(held.contains(&el), "{held:?}");
            }
        }
        heard.leave(&hook);
        prop_assert!(hook.held.borrow().is_empty(), "leaving takes it off");
    }
}

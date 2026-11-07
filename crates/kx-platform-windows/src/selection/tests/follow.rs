//! Where the selection handler sits, over a fake hook: added on an open box and kept there, moved to another box,
//! taken off a closed one, tried again after a failed add, and never on more than one element (property).

use super::super::follow::{Heard, Hook, same};
use proptest::prelude::*;
use std::cell::{Cell, RefCell};

/// A hook call: `true` for an add, with the element it named.
type Call = (bool, u8);

/// A scripted hook: it logs each call, keeps the elements that hold the handler, and refuses adds while `refuse`.
#[derive(Default)]
struct Fake {
    calls: RefCell<Vec<Call>>,
    held: RefCell<Vec<u8>>,
    refuse: Cell<bool>,
}

impl Fake {
    /// The calls since the last ask.
    fn take(&self) -> Vec<Call> {
        self.calls.take()
    }
}

impl Hook for Fake {
    type El = u8;

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
        held.retain(|h| h != el);
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
    heard.look(&hook, &1, 10, true);
    assert_eq!(hook.take(), [add(1)]);
    heard.look(&hook, &1, 10, true);
    assert_eq!(hook.take(), [], "the same box keeps it");
}

#[test]
fn another_box_takes_it_from_the_last() {
    let (hook, mut heard) = (Fake::default(), Heard::default());
    heard.look(&hook, &1, 10, true);
    heard.look(&hook, &2, 20, true);
    assert_eq!(hook.take(), [add(1), remove(1), add(2)]);
}

#[test]
fn a_closed_box_only_loses_the_last_one() {
    let (hook, mut heard) = (Fake::default(), Heard::default());
    heard.look(&hook, &1, 10, true);
    heard.look(&hook, &2, 20, false);
    assert_eq!(
        hook.take(),
        [add(1), remove(1)],
        "no handler on a closed box"
    );
    heard.look(&hook, &3, 30, false);
    assert_eq!(hook.take(), []);
    heard.look(&hook, &1, 10, false);
    heard.look(&hook, &1, 10, true);
    assert_eq!(hook.take(), [add(1)], "the box opens again");
}

#[test]
fn a_box_turning_closed_loses_it_and_gets_it_back_when_open() {
    let (hook, mut heard) = (Fake::default(), Heard::default());
    heard.look(&hook, &1, 10, true);
    heard.look(&hook, &1, 10, false);
    heard.look(&hook, &1, 10, true);
    assert_eq!(hook.take(), [add(1), remove(1), add(1)]);
}

#[test]
fn a_failed_add_is_tried_again_at_the_next_look() {
    let (hook, mut heard) = (Fake::default(), Heard::default());
    hook.refuse.set(true);
    heard.look(&hook, &1, 10, true);
    hook.refuse.set(false);
    heard.look(&hook, &1, 10, true);
    heard.look(&hook, &1, 10, true);
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
    heard.look(&hook, &1, 0, true);
    heard.look(&hook, &1, 0, true);
    assert_eq!(hook.take(), [add(1), remove(1), add(1)]);
}

#[test]
fn leaving_removes_it_once() {
    let (hook, mut heard) = (Fake::default(), Heard::default());
    heard.leave(&hook);
    heard.look(&hook, &1, 10, true);
    heard.leave(&hook);
    heard.leave(&hook);
    assert_eq!(hook.take(), [add(1), remove(1)]);
}

#[test]
fn only_two_equal_ids_name_the_same_box() {
    assert!(same(7, 7));
    assert!(!same(7, 8), "another box");
    assert!(!same(0, 0), "a box with no id is never the same");
    assert!(!same(7, 0) && !same(0, 7));
}

/// One look: an element (its box id follows it, 0 for the last), whether it is open, and whether its add fails.
fn any_look() -> impl Strategy<Value = (u8, bool, bool)> {
    (0..4u8, any::<bool>(), any::<bool>())
}

proptest! {
    #[test]
    fn the_handler_is_on_at_most_the_last_open_box(looks in prop::collection::vec(any_look(), 0..24)) {
        let (hook, mut heard) = (Fake::default(), Heard::default());
        for (el, open, refuse) in looks {
            hook.refuse.set(refuse);
            heard.look(&hook, &el, u64::from(el), open);
            let held = hook.held.borrow().clone();
            prop_assert!(held.len() <= 1, "{held:?}");
            if !open {
                prop_assert!(held.is_empty());
            }
            if open && !refuse {
                prop_assert_eq!(held, vec![el]);
            }
        }
        heard.leave(&hook);
        prop_assert!(hook.held.borrow().is_empty(), "leaving takes it off");
    }
}

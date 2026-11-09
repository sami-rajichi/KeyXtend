//! A box whose window Windows reports hung, over the fake hook: its handler waits to leave until a sure look finds the
//! window answering, it takes that handler back when looked at again, the stop removes it, and it gets no add while
//! hung, with no id too.

use super::*;

#[test]
fn a_hung_boxs_handler_waits_until_its_window_answers() {
    let (hook, mut heard) = (Fake::default(), Heard::default());
    heard.look(&hook, &1, 10, Seen::Open);
    hook.hang(&[1]);
    heard.look(&hook, &2, 20, Seen::Open);
    assert_eq!(hook.take(), [add(1), add(2)], "no removal from a hung app");
    heard.look(&hook, &2, 20, Seen::Open);
    assert_eq!(hook.take(), [], "still hung");
    hook.hang(&[]);
    heard.look(&hook, &2, 20, Seen::Open);
    assert_eq!(hook.take(), [remove(1)], "it answers again");
}

#[test]
fn only_a_sure_look_takes_a_waiting_handler_off() {
    let (hook, mut heard) = (Fake::default(), Heard::default());
    heard.look(&hook, &1, 10, Seen::Open);
    hook.hang(&[1]);
    heard.look(&hook, &2, 20, Seen::Closed);
    assert_eq!(hook.take(), [add(1)]);
    hook.hang(&[]);
    heard.look(&hook, &3, 30, Seen::Unsure);
    assert_eq!(hook.take(), [], "no OS call after an unsure look");
    heard.look(&hook, &2, 20, Seen::Closed);
    assert_eq!(hook.take(), [remove(1)]);
}

#[test]
fn a_hung_box_looked_at_again_takes_its_handler_back() {
    let (hook, mut heard) = (Fake::default(), Heard::default());
    heard.look(&hook, &1, 10, Seen::Open);
    hook.hang(&[1]);
    heard.look(&hook, &2, 20, Seen::Open);
    heard.look(&hook, &1, 10, Seen::Open);
    assert_eq!(hook.take(), [add(1), add(2), remove(2)], "no second add");
    hook.hang(&[]);
    heard.leave(&hook);
    assert_eq!(hook.take(), [remove(1)]);
}

#[test]
fn the_stop_removes_a_handler_left_on_a_hung_window() {
    let (hook, mut heard) = (Fake::default(), Heard::default());
    heard.look(&hook, &1, 10, Seen::Open);
    hook.hang(&[1]);
    heard.look(&hook, &2, 20, Seen::Open);
    heard.leave(&hook);
    assert_eq!(hook.take(), [add(1), add(2), remove(2), remove(1)]);
    assert!(hook.held.borrow().is_empty(), "never left for good");
}

#[test]
fn a_hung_box_gets_no_handler_until_its_window_answers() {
    let (hook, mut heard) = (Fake::default(), Heard::default());
    hook.hang(&[1]);
    heard.look(&hook, &1, 10, Seen::Open);
    heard.look(&hook, &1, 10, Seen::Open);
    assert_eq!(hook.take(), [], "no add on a hung app");
    hook.hang(&[]);
    heard.look(&hook, &1, 10, Seen::Open);
    assert_eq!(hook.take(), [add(1)], "it answers again");
}

#[test]
fn a_hung_box_with_no_id_gets_no_extra_handler_at_each_look() {
    let (hook, mut heard) = (Fake::default(), Heard::default());
    heard.look(&hook, &1, 0, Seen::Open);
    hook.hang(&[1]);
    for _ in 0..3 {
        heard.look(&hook, &1, 0, Seen::Open);
    }
    assert_eq!(
        hook.take(),
        [add(1)],
        "its handler waits, and none is added"
    );
    hook.hang(&[]);
    heard.look(&hook, &1, 0, Seen::Open);
    assert_eq!(hook.take(), [remove(1), add(1)]);
    assert_eq!(*hook.held.borrow(), [1]);
}

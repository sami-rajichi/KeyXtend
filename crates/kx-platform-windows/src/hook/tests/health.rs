//! Tests of the health check: current hooks stay, input on another desktop is not lag, and lagging hooks go back after
//! two lagging checks in a row, once a period, after letting go.

use kx_module_api::ports::{Button, PointerAct};

use super::fake::Call;
use super::run::{holding, sends};
use super::{A, B, CFG, LEFT_UP, MOVE, OTHER, Rig, ms, rig};
use crate::hook::health::Health;
use crate::hook::{Mark, Source};

/// How far ahead another program dates its input: about a week.
const FORWARD_MS: u32 = 600_000_000;
/// Whether the input desktop is another one, which the hooks cannot see.
const AWAY: bool = true;
const HOME: bool = false;

/// The sample check, counting the hooks current up to tick `seen`.
fn health(seen: u32) -> Health {
    Health::new(&CFG, seen)
}

/// Input the hooks never saw, after two seconds.
fn missed(rig: &Rig) {
    rig.world.pass(2_000);
    rig.world.last_input.set(Some(rig.world.tick.get()));
}

/// Two health checks a period apart, as the timer runs them.
fn check_twice(rig: &mut Rig) {
    rig.host.check();
    rig.world.pass(CFG.health_ms);
    rig.host.check();
}

#[test]
fn hooks_that_see_the_input_stay() {
    let mut h = health(100);
    h.saw(2_000);
    assert!(!h.due(ms(3_000), Some(2_400), HOME), "inside the slack");
    assert!(!h.due(ms(4_000), Some(2_000), HOME));
    assert!(!h.due(ms(5_000), None, HOME), "Windows would not say");
}

#[test]
fn hooks_that_miss_input_go_back_after_two_lagging_checks_once_per_period() {
    let mut h = health(100);
    h.saw(1_000);
    assert!(
        !h.due(ms(2_000), Some(1_600), HOME),
        "one lagging check alone"
    );
    assert!(h.due(ms(3_000), Some(1_600), HOME), "two in a row");
    assert!(
        !h.due(ms(3_000), Some(1_600), HOME),
        "that input is handled"
    );
    assert!(!h.due(ms(3_500), Some(9_000), HOME));
    assert!(
        !h.due(ms(3_800), Some(9_000), HOME),
        "not twice in a period"
    );
    assert!(h.due(ms(4_000), Some(9_000), HOME), "the next period");
}

#[test]
fn a_check_that_sees_no_lag_resets_the_count() {
    let mut h = health(100);
    h.saw(1_000);
    assert!(
        !h.due(ms(2_000), Some(1_600), HOME),
        "lag, as the hooks' callback is late"
    );
    h.saw(1_600);
    assert!(!h.due(ms(3_000), Some(1_600), HOME), "no lag");
    assert!(
        !h.due(ms(4_000), Some(2_200), HOME),
        "the first lagging check in a row"
    );
    assert!(h.due(ms(5_000), Some(2_200), HOME));
}

#[test]
fn input_on_another_desktop_is_not_lag_there_nor_at_the_first_check_back() {
    let mut h = health(100);
    h.saw(1_000);
    assert!(!h.due(ms(2_000), Some(1_600), AWAY), "the secure desktop");
    assert!(!h.due(ms(3_000), Some(2_600), AWAY));
    assert!(
        !h.due(ms(4_000), Some(3_600), HOME),
        "input there just before the switch"
    );
    assert!(
        !h.due(ms(5_000), Some(4_600), HOME),
        "home again, a first lagging check"
    );
    assert!(h.due(ms(6_000), Some(4_600), HOME));
}

#[test]
fn a_hook_stamp_across_the_ticks_wrap_is_current() {
    let mut h = health(u32::MAX - 10);
    h.saw(5);
    assert!(!h.due(ms(2_000), Some(u32::MAX - 10), HOME), "older input");
    assert!(!h.due(ms(3_000), Some(300), HOME));
    assert!(!h.due(ms(4_000), Some(300), HOME));
}

#[test]
fn current_hooks_stay() {
    let mut rig = rig(Source::Physical);
    rig.world.pass(400);
    rig.hand(MOVE, A);
    rig.world.last_input.set(Some(rig.world.tick.get()));
    rig.world.pass(2_000);
    check_twice(&mut rig);
    assert!(!rig.world.take().contains(&Call::Unhook));
    assert_eq!(rig.stats().reinstalls, 0);
}

#[test]
fn one_lagging_check_does_nothing_a_clean_one_resets_and_two_in_a_row_reinstall() {
    let mut rig = rig(Source::Physical);
    missed(&rig);
    rig.host.check();
    assert_eq!(rig.stats().reinstalls, 0, "one alone");
    rig.hand(MOVE, A);
    rig.host.check();
    missed(&rig);
    rig.host.check();
    assert_eq!(rig.stats().reinstalls, 0, "the clean check reset the count");
    rig.host.check();
    assert_eq!(rig.stats().reinstalls, 1, "two in a row");
}

#[test]
fn our_own_input_counts_as_seen_but_later_input_the_hooks_miss_does_not() {
    let mut rig = rig(Source::Physical);
    rig.world.pass(2_000);
    rig.plan().settle_acts = vec![PointerAct::Down(Button::Left, A)];
    check_twice(&mut rig);
    assert_eq!(rig.world.last_input.get(), Some(rig.world.tick.get()));
    assert_eq!(rig.stats().reinstalls, 0, "a hold that came due is not lag");
    rig.plan().settle_acts.clear();
    missed(&rig);
    check_twice(&mut rig);
    assert_eq!(rig.stats().reinstalls, 1);
}

#[test]
fn input_on_another_desktop_which_the_hooks_cannot_see_is_not_lag() {
    let mut rig = rig(Source::Physical);
    rig.world.away.set(true);
    missed(&rig);
    check_twice(&mut rig);
    rig.world.pass(1_000);
    rig.world.last_input.set(Some(rig.world.tick.get()));
    rig.world.away.set(false);
    rig.host.check();
    assert_eq!(rig.stats().reinstalls, 0, "away, then the first check back");
    missed(&rig);
    check_twice(&mut rig);
    assert_eq!(rig.stats().reinstalls, 1, "home again");
}

#[test]
fn lagging_hooks_go_back_once_after_letting_go() {
    let mut rig = holding();
    missed(&rig);
    check_twice(&mut rig);
    let calls = rig.world.take();
    let at = calls.iter().position(|c| *c == Call::Unhook).unwrap();
    let want = [
        Call::Unhook,
        Call::Send(vec![PointerAct::Up(Button::Left, A)]),
        Call::Hook,
        Call::Wake,
    ];
    assert_eq!(calls[at..], want, "let go, hooks back, then a turn");
    assert_eq!(rig.stats().reinstalls, 1);
    check_twice(&mut rig);
    assert!(
        !rig.world.take().contains(&Call::Unhook),
        "that input is handled"
    );
}

#[test]
fn a_forward_dated_injected_event_cannot_hide_lagging_hooks() {
    let mut rig = rig(Source::Physical);
    let future = Mark {
        time: rig.world.tick.get().wrapping_add(FORWARD_MS),
        ..rig.mark(true, OTHER)
    };
    assert!(!rig.host.mouse(MOVE, A, future));
    missed(&rig);
    check_twice(&mut rig);
    assert_eq!(rig.stats().reinstalls, 1);
}

#[test]
fn a_reinstalls_batch_sent_with_the_hooks_out_is_settled_at_once() {
    let mut rig = rig(Source::Physical);
    rig.world.inside.set(true);
    rig.plan().release_acts = vec![PointerAct::Down(Button::Left, A)];
    missed(&rig);
    check_twice(&mut rig);
    assert_eq!(rig.stats().reinstalls, 1);
    assert!(rig.host.is_clear(), "no hook of ours could echo it");
    rig.world.pass(CFG.echo_ms + 1);
    assert!(!rig.hand(LEFT_UP, B), "the hand's release passes");
    rig.world.take();
    rig.host.turn();
    let sent = sends(&rig.world.take()).concat();
    let ups = sent.iter().filter(|a| matches!(a, PointerAct::Up(..)));
    assert_eq!(ups.count(), 0, "no Up is owed for our press");
    assert_eq!(rig.stats().lost_echoes, 0);
}

#[test]
fn a_refused_reinstall_keeps_the_thread_running() {
    let mut rig = rig(Source::Physical);
    missed(&rig);
    rig.world.refuse.set(Some(5));
    check_twice(&mut rig);
    assert_eq!(rig.stats().reinstalls, 0);
    rig.host.turn();
}

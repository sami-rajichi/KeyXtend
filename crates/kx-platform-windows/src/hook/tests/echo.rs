//! Tests of the wait for our own input: each echo names its act and settles it with every older act still out, so a
//! swallowed echo is a gap and a late one matches only its own act; only an echo of the newest batch ends the wait.

use kx_module_api::Mono;
use kx_module_api::ports::{Button, PointerAct};

use super::fake::Call;
use super::{A, B, CFG, CTRL, KEY_DOWN, LEFT_DOWN, MOVE, SHIFT, ms, rig};
use crate::hook::Source;
use crate::hook::echo::{Echo, Wait};
use crate::keys::{REPLAY, TAG, replay_tag};

/// Room for a few acts.
const ROOM: usize = 8;
/// Room for two acts, so a third makes room.
const TIGHT: usize = 2;
const LEFT_A: PointerAct = PointerAct::Down(Button::Left, A);
const LEFT_B: PointerAct = PointerAct::Down(Button::Left, B);

/// Sends `acts` at `at`, as the host does, and gives their first act's id.
fn send(echo: &mut Echo, acts: &[PointerAct], at: Mono) -> u32 {
    let first = echo.next();
    echo.sent(acts, at);
    first
}

#[test]
fn the_wait_ends_when_our_input_comes_back_or_after_echo_ms() {
    let mut echo = Echo::new(CFG.echo_ms, ROOM);
    assert_eq!(echo.check(ms(1)), Wait::Clear);
    let id = send(&mut echo, &[PointerAct::Move(A)], ms(10));
    assert_eq!(echo.check(ms(260)), Wait::Waiting);
    echo.back(id);
    assert_eq!(echo.check(ms(261)), Wait::Clear);
    send(&mut echo, &[PointerAct::Move(A)], ms(300));
    assert_eq!(echo.check(ms(551)), Wait::Lost);
    assert_eq!(echo.check(ms(552)), Wait::Clear, "counted once");
}

#[test]
fn each_act_taken_gets_the_next_id_and_nothing_taken_starts_no_wait() {
    let mut echo = Echo::new(CFG.echo_ms, ROOM);
    let first = send(&mut echo, &[], ms(10));
    assert!(echo.is_clear());
    assert_eq!(echo.next(), first, "no id used");
    send(&mut echo, &[LEFT_A, PointerAct::Move(B)], ms(11));
    assert_eq!(echo.next(), first.wrapping_add(2));
}

#[test]
fn an_older_batchs_echo_never_ends_the_newest_batchs_wait() {
    let mut echo = Echo::new(CFG.echo_ms, ROOM);
    let keys = send(
        &mut echo,
        &[PointerAct::KeyDown(CTRL), PointerAct::KeyUp(CTRL)],
        ms(10),
    );
    echo.back(keys);
    assert!(
        echo.is_clear(),
        "the first echo proves the whole batch passed"
    );
    let press = send(&mut echo, &[LEFT_A], ms(11));
    echo.back(keys.wrapping_add(1));
    assert!(!echo.is_clear(), "the older batch's tail");
    echo.back(press);
    assert!(echo.is_clear());
}

#[test]
fn a_press_and_its_move_both_name_one_act_so_the_second_changes_nothing() {
    let mut echo = Echo::new(CFG.echo_ms, ROOM);
    let press = send(&mut echo, &[LEFT_A], ms(10));
    send(&mut echo, &[PointerAct::Move(B)], ms(11));
    echo.back(press);
    echo.back(press);
    assert!(!echo.out(LEFT_A));
    assert!(!echo.is_clear(), "the newer move is still out");
}

#[test]
fn a_swallowed_echo_is_a_gap_whose_press_stays_in_doubt_and_shifts_no_later_match() {
    let mut echo = Echo::new(CFG.echo_ms, ROOM);
    let click = send(
        &mut echo,
        &[LEFT_A, PointerAct::Up(Button::Left, A)],
        ms(10),
    );
    echo.back(click.wrapping_add(1));
    assert!(echo.is_clear(), "an echo of the newest batch ends its wait");
    assert!(echo.out(LEFT_A), "its echo never came");
    let newer = send(
        &mut echo,
        &[LEFT_B, PointerAct::Up(Button::Left, B)],
        ms(11),
    );
    echo.back(newer);
    assert!(echo.is_clear(), "the newer press's own echo");
    echo.back(newer.wrapping_add(1));
    echo.owed(LEFT_A);
    assert!(!echo.out(LEFT_A), "an Up owed after it");
}

#[test]
fn a_given_up_batchs_late_echo_settles_only_its_own_acts() {
    let mut echo = Echo::new(CFG.echo_ms, ROOM);
    let click = send(
        &mut echo,
        &[LEFT_A, PointerAct::Up(Button::Left, A)],
        ms(10),
    );
    assert_eq!(echo.check(ms(300)), Wait::Lost);
    assert!(echo.out(LEFT_A), "given up, still on its way");
    let press = send(&mut echo, &[LEFT_B, PointerAct::Move(B)], ms(310));
    echo.back(click);
    echo.back(click.wrapping_add(1));
    assert!(!echo.is_clear(), "the older click came back, not ours");
    assert!(echo.out(LEFT_B));
    echo.back(press);
    assert!(echo.is_clear());
    assert!(!echo.out(LEFT_B));
}

#[test]
fn the_oldest_acts_make_room_and_a_late_echo_of_one_ends_no_newer_wait() {
    let mut echo = Echo::new(CFG.echo_ms, TIGHT);
    let old = send(&mut echo, &[LEFT_A, PointerAct::Move(A)], ms(10));
    send(&mut echo, &[PointerAct::KeyDown(CTRL)], ms(11));
    assert_eq!(echo.room(), TIGHT, "the oldest act made room");
    assert!(echo.out(LEFT_A), "in doubt");
    let right = PointerAct::Down(Button::Right, B);
    send(&mut echo, &[right], ms(12));
    echo.back(old);
    assert!(!echo.out(LEFT_A), "it came back late");
    echo.back(old.wrapping_add(1));
    assert!(!echo.is_clear(), "a made-room move names only itself");
    assert!(echo.out(right));
}

#[test]
fn a_press_made_room_for_stays_in_doubt_until_an_up_is_owed() {
    let mut echo = Echo::new(CFG.echo_ms, TIGHT);
    send(&mut echo, &[LEFT_A, PointerAct::Move(A)], ms(10));
    send(
        &mut echo,
        &[PointerAct::Move(B), PointerAct::Move(B)],
        ms(11),
    );
    assert!(echo.out(LEFT_A));
    echo.owed(LEFT_A);
    assert!(!echo.out(LEFT_A));
}

#[test]
fn an_echo_naming_no_act_out_changes_nothing() {
    let mut echo = Echo::new(CFG.echo_ms, ROOM);
    let id = send(&mut echo, &[LEFT_A], ms(10));
    for other in [id.wrapping_add(1), id.wrapping_sub(1), u32::MAX / 2] {
        echo.back(other);
    }
    assert!(!echo.is_clear());
    assert!(echo.out(LEFT_A));
}

#[test]
fn ids_wrap_past_the_largest() {
    let mut echo = Echo::new(CFG.echo_ms, ROOM);
    echo.skip_to(u32::MAX);
    let first = send(&mut echo, &[PointerAct::Move(A), LEFT_A], ms(10));
    assert_eq!(first.wrapping_add(1), 0);
    echo.back(0);
    assert!(echo.is_clear());
    assert!(!echo.out(LEFT_A));
}

#[test]
fn many_batches_out_keep_the_room() {
    let mut echo = Echo::new(CFG.echo_ms, ROOM);
    for n in 0..100 {
        let keys = [PointerAct::KeyDown(CTRL), PointerAct::KeyUp(CTRL)];
        let id = send(&mut echo, &keys, ms(n));
        if n % 2 == 0 {
            echo.back(id.wrapping_add(1));
        }
    }
    assert_eq!(echo.room(), ROOM, "acts settled or made room for");
}

#[test]
fn our_input_passes_and_ends_the_wait_for_it() {
    let mut rig = rig(Source::Physical);
    rig.send_ours();
    assert!(!rig.injected(MOVE, B, REPLAY));
    assert!(rig.heard().is_empty(), "never reaches the engine");
    assert!(!rig.hand(MOVE, A), "nothing to wait for");
    rig.host.turn();
    assert!(!rig.world.take().iter().any(|c| matches!(c, Call::Send(_))));
}

#[test]
fn an_echo_through_the_hooks_ends_the_wait_only_when_it_names_the_newest_batch() {
    let mut rig = rig(Source::Physical);
    rig.send_ours();
    rig.send_ours();
    assert!(!rig.injected(MOVE, B, replay_tag(0)));
    assert!(rig.hand(MOVE, A), "the older move came back, not the newer");
    let mut rig = super::rig(Source::Physical);
    rig.send_ours();
    rig.send_ours();
    let mark = rig.mark(true, replay_tag(1));
    rig.host.key(KEY_DOWN, 0, CTRL, mark);
    assert!(
        !rig.hand(MOVE, A),
        "the newest act came back, by the key hook"
    );
}

#[test]
fn our_keyboards_input_passes_and_leaves_the_wait_for_the_hosts() {
    let mut rig = rig(Source::Physical);
    rig.send_ours();
    assert!(!rig.injected(MOVE, B, TAG));
    let mark = rig.mark(true, TAG);
    rig.host.key(KEY_DOWN, 0, CTRL, mark);
    assert!(rig.heard().is_empty(), "never reaches the engine");
    assert!(rig.hand(MOVE, A), "still behind our move");
}

#[test]
fn let_through_input_waits_behind_our_unechoed_input_in_order() {
    let mut rig = rig(Source::Physical);
    rig.send_ours();
    assert!(rig.hand(MOVE, A), "held back");
    rig.plan().on_acts = vec![PointerAct::KeyUp(SHIFT)];
    assert!(rig.hand(LEFT_DOWN, B), "held back");
    assert_eq!(rig.world.take(), [Call::Wake], "one wake");
    rig.host.turn();
    let want = vec![
        PointerAct::Move(A),
        PointerAct::Down(Button::Left, B),
        PointerAct::KeyUp(SHIFT),
    ];
    assert_eq!(rig.world.take().first(), Some(&Call::Send(want)));
}

#[test]
fn after_echo_ms_the_wait_ends_and_counts_a_lost_echo() {
    let mut rig = rig(Source::Physical);
    rig.send_ours();
    rig.world.pass(CFG.echo_ms + 1);
    assert!(!rig.hand(MOVE, A), "the wait is over");
    assert!(!rig.hand(MOVE, A));
    assert_eq!(rig.stats().lost_echoes, 1);
}

//! The spike's cases on the new API, then Grab, modifiers and notes in their own files.

use super::*;
use PointerAct as A;
use kx_module_api::ports::Button::{self, Left, Right};
use kx_module_api::ports::Toggle;

mod grab;
mod mods;
mod notes;
mod presses;
mod scroll;
mod timing;

/// The default hold, 1.5 s, in µs.
const HOLD: u64 = 1_500_000;
/// The default still radius, in logical pixels.
const STILL: u32 = 7;
/// 100 % scale.
const DPI: u32 = 96;
/// 125 % scale, where the 7 px radius is 9 px.
const DPI_125: u32 = 120;
/// Left Shift.
const SHIFT: Scan = Scan::new(0x2A);
/// Left Ctrl.
const CTRL: Scan = Scan::new(0x1D);

/// What the engine did with one event: dropped or not, then the input it sent.
type Out = (bool, Vec<PointerAct>);

fn p(x: i32, y: i32) -> Point {
    Point { x, y }
}

fn eng(rclick: bool) -> Engine {
    Engine::new(EngineCfg {
        hold_us: HOLD,
        still_px: STILL,
        rclick,
    })
}

/// A left press outside our windows at 100 %.
fn down(at: Point) -> PointerEvent {
    press(Left, at, DPI)
}

fn press(button: Button, at: Point, dpi: u32) -> PointerEvent {
    PointerEvent::Down {
        button,
        at,
        ours: false,
        dpi,
    }
}

/// A left press on one of our windows.
fn ours(at: Point) -> PointerEvent {
    PointerEvent::Down {
        button: Left,
        at,
        ours: true,
        dpi: DPI,
    }
}

fn up(at: Point) -> PointerEvent {
    lift(Left, at)
}

fn lift(button: Button, at: Point) -> PointerEvent {
    PointerEvent::Up { button, at }
}

fn mv(at: Point) -> PointerEvent {
    PointerEvent::Move { at }
}

fn on(e: &mut Engine, event: PointerEvent, us: u64) -> Out {
    let mut acts = Vec::new();
    let dropped = e.on(event, Mono::from_us(us), &mut acts);
    (dropped, acts)
}

fn pass(acts: Vec<PointerAct>) -> Out {
    (false, acts)
}

fn eat(acts: Vec<PointerAct>) -> Out {
    (true, acts)
}

fn tick(e: &mut Engine, us: u64) -> Vec<PointerAct> {
    let mut acts = Vec::new();
    e.tick(Mono::from_us(us), &mut acts);
    acts
}

fn toggle(e: &mut Engine, which: Toggle) -> Vec<PointerAct> {
    let mut acts = Vec::new();
    e.toggle(which, &mut acts);
    acts
}

fn release(e: &mut Engine) -> Vec<PointerAct> {
    let mut acts = Vec::new();
    e.release(&mut acts);
    acts
}

fn notes(e: &mut Engine) -> Vec<Note> {
    let mut out = Vec::new();
    e.take_notes(&mut out);
    out
}

fn right_click(at: Point) -> Vec<PointerAct> {
    vec![A::Down(Right, at), A::Up(Right, at)]
}

fn mono(us: u64) -> Mono {
    Mono::from_us(us)
}

#[test]
fn with_no_mode_and_no_latches_everything_passes() {
    let mut e = eng(false);
    assert_eq!(on(&mut e, down(p(1, 1)), 0), pass(vec![]));
    assert_eq!(e.deadline(), None);
    assert_eq!(on(&mut e, up(p(1, 1)), HOLD * 2), pass(vec![]));
    assert!(!e.needs_hook());
}

#[test]
fn a_short_click_is_replayed_at_the_press_point() {
    let mut e = eng(true);
    let p0 = p(10, 10);
    assert_eq!(on(&mut e, down(p0), 0), eat(vec![]));
    assert_eq!(e.deadline(), Some(mono(HOLD)));
    let want = vec![A::Down(Left, p0), A::Up(Left, p0)];
    assert_eq!(on(&mut e, up(p(12, 11)), 200_000), eat(want));
    assert_eq!(e.deadline(), None);
}

#[test]
fn a_still_hold_becomes_a_right_click_and_its_release_is_dropped() {
    let mut e = eng(true);
    let p0 = p(10, 10);
    on(&mut e, down(p0), 0);
    assert_eq!(tick(&mut e, HOLD - 1), vec![]);
    assert_eq!(tick(&mut e, HOLD), right_click(p0));
    assert_eq!(on(&mut e, up(p0), HOLD + 9), eat(vec![]));
    assert_eq!(on(&mut e, down(p0), HOLD * 2), eat(vec![]));
}

#[test]
fn six_px_is_still_but_eight_px_starts_a_normal_drag() {
    let p0 = p(100, 100);
    let mut e = eng(true);
    on(&mut e, down(p0), 0);
    assert_eq!(on(&mut e, mv(p(106, 100)), 10), pass(vec![]));
    assert_eq!(tick(&mut e, HOLD), right_click(p0));

    let mut e = eng(true);
    on(&mut e, down(p0), 0);
    let to = p(108, 100);
    let want = vec![A::Down(Left, p0), A::Move(to)];
    assert_eq!(on(&mut e, mv(to), 10), eat(want));
    assert_eq!(e.deadline(), None);
    assert_eq!(on(&mut e, up(to), 20), pass(vec![]));
    assert_eq!(tick(&mut e, HOLD), vec![]);
}

#[test]
fn at_125_percent_the_radius_scales_to_9_px() {
    let p0 = p(100, 100);
    let mut e = eng(true);
    on(&mut e, press(Left, p0, DPI_125), 0);
    assert_eq!(on(&mut e, mv(p(108, 100)), 10), pass(vec![]));
    assert_eq!(on(&mut e, mv(p(109, 100)), 20), pass(vec![]));
    assert_eq!(tick(&mut e, HOLD), right_click(p0));

    let mut e = eng(true);
    on(&mut e, press(Left, p0, DPI_125), 0);
    let to = p(110, 100);
    assert_eq!(
        on(&mut e, mv(to), 10),
        eat(vec![A::Down(Left, p0), A::Move(to)])
    );
}

#[test]
fn a_screen_with_no_dpi_counts_as_100_percent() {
    let p0 = p(100, 100);
    let mut e = eng(true);
    on(&mut e, press(Left, p0, 0), 0);
    assert_eq!(on(&mut e, mv(p(107, 100)), 10), pass(vec![]));
    let to = p(108, 100);
    let want = vec![A::Down(Left, p0), A::Move(to)];
    assert_eq!(on(&mut e, mv(to), 20), eat(want));
}

#[test]
fn a_second_button_replays_the_held_press_first() {
    let mut e = eng(true);
    let (p0, p1) = (p(1, 1), p(2, 2));
    on(&mut e, down(p0), 0);
    let want = vec![A::Down(Left, p0), A::Down(Right, p1)];
    assert_eq!(on(&mut e, press(Right, p1, DPI), 10), eat(want));
    assert_eq!(on(&mut e, lift(Right, p1), 20), pass(vec![]));
    assert_eq!(on(&mut e, up(p1), 30), pass(vec![]));
}

#[test]
fn a_mode_switch_replays_the_held_press() {
    let mut e = eng(true);
    let p0 = p(3, 3);
    on(&mut e, down(p0), 0);
    assert_eq!(toggle(&mut e, Toggle::RightClick), vec![A::Down(Left, p0)]);
    assert_eq!(on(&mut e, up(p0), 10), pass(vec![]));
}

#[test]
fn a_press_on_our_own_window_is_never_held() {
    let mut e = eng(true);
    let p0 = p(4, 4);
    assert_eq!(on(&mut e, ours(p0), 0), pass(vec![]));
    assert_eq!(tick(&mut e, HOLD), vec![]);
    assert_eq!(on(&mut e, up(p0), HOLD + 1), pass(vec![]));
}

#[test]
fn esc_and_other_buttons_pass_outside_a_carry() {
    let mut e = eng(true);
    let p0 = p(6, 6);
    assert_eq!(on(&mut e, PointerEvent::Esc, 0), pass(vec![]));
    assert_eq!(on(&mut e, press(Right, p0, DPI), 1), pass(vec![]));
    assert_eq!(on(&mut e, lift(Right, p0), 2), pass(vec![]));
    on(&mut e, down(p0), 3);
    assert_eq!(on(&mut e, PointerEvent::Esc, 4), pass(vec![]));
    assert_eq!(e.deadline(), Some(mono(HOLD + 3)));
}

#[test]
fn the_config_comes_from_the_settings() {
    let s = MouseSettings::from_defaults().unwrap();
    let cfg = EngineCfg::from(&s);
    let hold_us = u64::from(s.hold_ms) * u64::from(US_PER_MS);
    assert_eq!(
        (cfg.hold_us, cfg.still_px, cfg.rclick),
        (hold_us, s.still_px, s.rclick)
    );
}

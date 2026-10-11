//! Tests of the mouse guard's rules: its settings, which events drop, and when it lets go.

use std::collections::BTreeSet;
use std::time::Duration;

use kx_module_api::{Mono, US_PER_MS};
use windows::Win32::UI::WindowsAndMessaging::{
    WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_XBUTTONDOWN,
    WM_XBUTTONUP, XBUTTON1,
};

use super::*;
use crate::rng::Rng;

fn shipped() -> MouseGuard {
    crate::config::load()
        .expect("kx-gates.toml loads")
        .mouse_guard
}

/// The point `ms` milliseconds after the guard started.
fn at(ms: u64) -> Mono {
    Mono::from_us(ms * u64::from(US_PER_MS))
}

fn filter() -> Filter {
    Filter::new(&shipped(), at(0), Duration::ZERO)
}

/// A physical `kind` event at `ms`.
fn physical(f: &mut Filter, kind: Kind, ms: u64) -> Verdict {
    f.event(false, kind, at(ms))
}

const LEFT_DOWN: Kind = Kind::Down(LEFT);
const LEFT_UP: Kind = Kind::Up(LEFT);
/// Seeds and events per seed of the property test.
const RUNS: u64 = 200;
const STEPS: usize = 200;

/// A random event, injected or not: a release or other event of a random button, or a press when `presses`.
fn random_event(rng: &mut Rng, presses: bool) -> (bool, Kind) {
    let bit = [LEFT, RIGHT, MIDDLE, X1, X2][rng.below(5)];
    let kinds = [Kind::Up(bit), Kind::Other, Kind::Down(bit)];
    let kind = kinds[rng.below(kinds.len() - usize::from(!presses))];
    (rng.below(2) == 0, kind)
}

#[test]
fn over_random_input_our_events_pass_a_held_mouse_drops_the_hands_and_a_guard_that_let_go_stays_let_go()
 {
    let g = shipped();
    // Steps average one escape time; odd seeds press nothing, so both the limit and a hold end some runs.
    let step = usize::try_from(g.escape_ms * 2).expect("a short hold");
    let mut ends = BTreeSet::new();
    for seed in 0..RUNS {
        let (mut rng, mut f, mut now) = (Rng::new(seed), filter(), 0);
        let (mut dropped, mut out) = (0u8, None);
        for _ in 0..STEPS {
            now += u64::try_from(rng.below(step)).expect("fits");
            let (mine, kind) = random_event(&mut rng, seed % 2 == 0);
            let before = f.clone().tick(at(now));
            let verdict = f.event(mine, kind, at(now));
            let held = before.is_none();
            let pressed = matches!(kind, Kind::Up(b) if dropped & b != 0);
            let hand = !mine && held && (pressed || !matches!(kind, Kind::Up(_)));
            let want = if hand { Verdict::Drop } else { Verdict::Pass };
            assert_eq!(verdict, want, "seed {seed} at {now} ms: {kind:?}");
            if hand {
                dropped = match kind {
                    Kind::Down(b) => dropped | b,
                    Kind::Up(b) => dropped & !b,
                    Kind::Other => dropped,
                };
            }
            let end = f.tick(at(now));
            assert!(out.is_none() || end == out, "seed {seed}: it stays let go");
            out = end;
        }
        ends.extend(out.map(End::name));
    }
    assert!(ends.contains("owner") && ends.contains("limit"), "{ends:?}");
}

#[test]
fn the_shipped_guard_is_on_with_the_owners_limit_and_escape_hold() {
    let g = shipped();
    assert!(g.on);
    assert_eq!(
        (g.limit_ms, g.escape_ms),
        (360_000, 2_000),
        "owner, 2026-10-11"
    );
    assert_eq!(g.check(), Ok(()));
}

#[test]
fn a_zero_limit_or_hold_or_a_hold_the_limit_always_cuts_short_is_refused() {
    let bad: [fn(&mut MouseGuard); 3] = [
        |g| g.limit_ms = 0,
        |g| g.escape_ms = 0,
        |g| g.escape_ms = g.limit_ms,
    ];
    for (i, f) in bad.iter().enumerate() {
        let mut g = shipped();
        f(&mut g);
        assert!(g.check().is_err(), "case {i}");
    }
}

#[test]
fn only_input_that_kx_gates_injected_and_tagged_counts_as_its_own() {
    let injected = LLMHF_INJECTED;
    assert!(ours(injected, TAG));
    assert!(ours(injected, SIM_TAG));
    assert!(!ours(injected, 0), "another program's injected input");
    assert!(!ours(0, TAG), "a tag without the injected flag");
    assert!(!ours(0, 0), "the hand");
}

#[test]
fn hook_messages_become_presses_releases_or_other_events() {
    let x2 = u32::from(XBUTTON1 + 1) << 16;
    let x1 = u32::from(XBUTTON1) << 16;
    assert_eq!(kind_of(WM_LBUTTONDOWN, 0), Kind::Down(LEFT));
    assert_eq!(kind_of(WM_LBUTTONUP, 0), Kind::Up(LEFT));
    assert_eq!(kind_of(WM_RBUTTONDOWN, 0), Kind::Down(RIGHT));
    assert_eq!(kind_of(WM_XBUTTONDOWN, x1), Kind::Down(X1));
    assert_eq!(kind_of(WM_XBUTTONUP, x2), Kind::Up(X2));
    assert_eq!(kind_of(WM_MOUSEMOVE, 0), Kind::Other);
    assert_eq!(kind_of(WM_MOUSEWHEEL, x1), Kind::Other);
}

#[test]
fn injected_events_pass_and_physical_moves_clicks_and_wheels_drop() {
    let mut f = filter();
    for kind in [Kind::Other, LEFT_DOWN, LEFT_UP] {
        assert_eq!(f.event(true, kind, at(1)), Verdict::Pass, "{kind:?}");
    }
    for kind in [Kind::Other, LEFT_DOWN, LEFT_UP] {
        assert_eq!(physical(&mut f, kind, 2), Verdict::Drop, "{kind:?}");
    }
    assert_eq!(f.tick(at(3)), None, "a click is no escape");
}

#[test]
fn a_physical_button_held_past_the_escape_time_lets_go_and_a_shorter_hold_does_not() {
    let escape = shipped().escape_ms;
    let mut f = filter();
    physical(&mut f, LEFT_DOWN, 10);
    assert_eq!(f.tick(at(10 + escape - 1)), None);
    assert_eq!(physical(&mut f, LEFT_UP, 10 + escape - 1), Verdict::Drop);
    assert_eq!(
        f.tick(at(10 + 3 * escape)),
        None,
        "the button went up in time"
    );
    physical(&mut f, LEFT_DOWN, 100_000);
    assert_eq!(
        physical(&mut f, Kind::Other, 100_000 + escape / 2),
        Verdict::Drop
    );
    assert_eq!(f.tick(at(100_000 + escape)), Some(End::Owner));
    assert_eq!(physical(&mut f, LEFT_UP, 100_001 + escape), Verdict::Pass);
    assert_eq!(
        physical(&mut f, LEFT_DOWN, 100_002 + escape),
        Verdict::Pass,
        "it let go"
    );
}

#[test]
fn a_held_button_lets_go_through_the_next_event_too_and_moves_do_not_reset_the_hold() {
    let escape = shipped().escape_ms;
    let mut f = filter();
    physical(&mut f, LEFT_DOWN, 0);
    physical(&mut f, Kind::Down(RIGHT), escape / 2);
    physical(&mut f, Kind::Up(RIGHT), escape / 2 + 1);
    assert_eq!(physical(&mut f, Kind::Other, escape), Verdict::Pass);
    assert_eq!(
        f.tick(at(escape)),
        Some(End::Owner),
        "left was held throughout"
    );
}

#[test]
fn a_release_whose_press_came_before_the_guard_passes_so_no_button_sticks() {
    let mut f = filter();
    assert_eq!(physical(&mut f, LEFT_UP, 5), Verdict::Pass);
    physical(&mut f, Kind::Down(RIGHT), 6);
    assert_eq!(
        physical(&mut f, LEFT_UP, 7),
        Verdict::Pass,
        "only right was dropped"
    );
}

#[test]
fn the_guard_lets_go_at_its_time_limit_counted_from_its_start() {
    let limit = shipped().limit_ms;
    let mut f = Filter::new(&shipped(), at(500), Duration::ZERO);
    assert_eq!(f.tick(at(500 + limit - 1)), None);
    assert_eq!(
        physical(&mut f, Kind::Other, 500 + limit - 1),
        Verdict::Drop
    );
    assert_eq!(f.tick(at(500 + limit)), Some(End::Limit));
    assert_eq!(physical(&mut f, Kind::Other, 500 + limit), Verdict::Pass);
}

#[test]
fn a_guard_that_comes_back_holds_only_for_what_is_left_of_the_limit() {
    let limit = shipped().limit_ms;
    let used = Duration::from_millis(limit / 4);
    let mut f = Filter::new(&shipped(), at(500), used);
    assert_eq!(f.tick(at(500 + limit - limit / 4 - 1)), None);
    assert_eq!(f.tick(at(500 + limit - limit / 4)), Some(End::Limit));
    let spent = Duration::from_millis(limit * 2);
    let mut over = Filter::new(&shipped(), at(500), spent);
    assert_eq!(over.tick(at(500)), Some(End::Limit), "no time left");
}

#[test]
fn the_next_check_is_due_at_the_limit_or_a_held_buttons_escape_whichever_is_first() {
    let g = shipped();
    let mut f = filter();
    assert_eq!(
        f.due_in(at(1_000)),
        Duration::from_millis(g.limit_ms - 1_000)
    );
    physical(&mut f, LEFT_DOWN, 1_000);
    assert_eq!(
        f.due_in(at(1_500)),
        Duration::from_millis(g.escape_ms - 500)
    );
    assert_eq!(f.due_in(at(g.limit_ms + 1)), Duration::ZERO);
}

#[test]
fn a_wait_rounds_up_to_whole_ms_and_stops_at_the_largest_wait() {
    assert_eq!(wait_ms(Duration::from_micros(1)), 1);
    assert_eq!(wait_ms(Duration::from_millis(7)), 7);
    assert_eq!(wait_ms(Duration::ZERO), 0);
    assert_eq!(wait_ms(Duration::from_secs(u64::MAX)), INFINITE - 1);
    let endless = Duration::from_millis(u64::from(INFINITE));
    assert_eq!(wait_ms(endless), INFINITE - 1, "never a wait without end");
}

#[test]
fn only_the_owner_the_limit_or_a_lost_guard_stops_the_gate_and_each_end_has_its_own_name() {
    let all = [
        End::Done,
        End::Error,
        End::Panic,
        End::Prompt,
        End::Idle,
        End::Owner,
        End::Limit,
        End::Lost,
    ];
    let stops: Vec<_> = all.iter().filter(|e| e.stops()).collect();
    assert_eq!(stops, [&End::Owner, &End::Limit, &End::Lost]);
    let names: BTreeSet<_> = all.iter().map(|e| e.name()).collect();
    assert_eq!(names.len(), all.len());
    assert!(all.iter().all(|e| !e.why().is_empty()));
}

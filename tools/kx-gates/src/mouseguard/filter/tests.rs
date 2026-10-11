//! Tests of the guard's filter: which events drop, and when it lets go.

use std::collections::BTreeSet;

use kx_module_api::US_PER_MS;

use super::*;
use crate::mouseguard::{LEFT, MIDDLE, RIGHT, X1, X2};
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

/// A filter that lets go after two physical presses within the shipped window.
fn two_clicks() -> Filter {
    let cfg = MouseGuard {
        escape_clicks: 2,
        ..shipped()
    };
    Filter::new(&cfg, at(0), Duration::ZERO)
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
    // Steps add up to twice the limit on average, so odd seeds, which press nothing, reach it.
    let step = usize::try_from(g.limit_ms).expect("fits") * 4 / STEPS;
    let mut ends = BTreeSet::new();
    for seed in 0..RUNS {
        let f = if seed % 4 == 0 {
            two_clicks()
        } else {
            filter()
        };
        let (mut rng, mut f, mut now) = (Rng::new(seed), f, 0);
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
fn injected_events_pass_without_letting_go_and_physical_moves_drop() {
    let mut f = filter();
    for kind in [Kind::Other, LEFT_DOWN, LEFT_UP] {
        assert_eq!(f.event(true, kind, at(1)), Verdict::Pass, "{kind:?}");
    }
    assert_eq!(f.ended(), None, "kx-gates' own clicks are no escape");
    assert_eq!(physical(&mut f, Kind::Other, 2), Verdict::Drop);
    assert_eq!(f.tick(at(3)), None, "a move is no escape");
}

#[test]
fn the_filter_counts_the_hands_events_it_dropped_and_its_own_it_passed_but_nothing_else() {
    let mut f = filter();
    f.event(true, Kind::Other, at(1));
    f.event(true, LEFT_DOWN, at(1));
    physical(&mut f, Kind::Other, 2);
    physical(&mut f, LEFT_UP, 3);
    physical(&mut f, LEFT_DOWN, 4);
    assert_eq!(
        f.counts(),
        Counts {
            dropped: 2,
            passed: 2
        },
        "a release whose press came before the guard is neither"
    );
    physical(&mut f, Kind::Other, 5);
    assert_eq!(f.counts().dropped, 2, "nothing drops once it let go");
}

#[test]
fn one_physical_press_gives_the_mouse_back_at_once_and_its_release_then_passes() {
    let mut f = filter();
    assert_eq!(physical(&mut f, LEFT_DOWN, 10), Verdict::Drop, "the press");
    assert_eq!(f.ended(), Some(End::Owner), "at once, with no wait");
    assert_eq!(physical(&mut f, LEFT_UP, 11), Verdict::Pass);
    assert_eq!(
        physical(&mut f, Kind::Other, 12),
        Verdict::Pass,
        "it let go"
    );
    assert_eq!(f.event(true, Kind::Other, at(13)), Verdict::Pass);
}

#[test]
fn with_two_clicks_set_one_press_does_not_let_go_and_two_within_the_window_do() {
    let window = shipped().escape_window_ms;
    let mut f = two_clicks();
    assert_eq!(physical(&mut f, LEFT_DOWN, 100), Verdict::Drop);
    assert_eq!(
        physical(&mut f, LEFT_UP, 101),
        Verdict::Drop,
        "its press dropped"
    );
    assert_eq!(
        f.tick(at(100 + window - 1)),
        None,
        "one press is not enough"
    );
    assert_eq!(
        physical(&mut f, Kind::Down(RIGHT), 100 + window - 1),
        Verdict::Drop
    );
    assert_eq!(f.ended(), Some(End::Owner));
    assert_eq!(
        physical(&mut f, Kind::Up(RIGHT), 100 + window),
        Verdict::Pass
    );
}

#[test]
fn with_two_clicks_set_two_presses_a_window_apart_do_not_let_go() {
    let window = shipped().escape_window_ms;
    let mut f = two_clicks();
    physical(&mut f, LEFT_DOWN, 100);
    physical(&mut f, LEFT_UP, 101);
    physical(&mut f, LEFT_DOWN, 100 + window);
    assert_eq!(f.ended(), None, "the first press fell out of the window");
    physical(&mut f, LEFT_UP, 101 + window);
    physical(&mut f, LEFT_DOWN, 99 + 2 * window);
    assert_eq!(f.ended(), Some(End::Owner), "the last two came within it");
}

#[test]
fn a_release_whose_press_came_before_the_guard_passes_so_no_button_sticks() {
    let mut f = two_clicks();
    assert_eq!(physical(&mut f, LEFT_UP, 5), Verdict::Pass);
    physical(&mut f, Kind::Down(RIGHT), 6);
    assert_eq!(f.ended(), None);
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
fn the_next_check_is_due_at_the_time_limit_whatever_the_presses() {
    let g = shipped();
    let mut f = two_clicks();
    assert_eq!(
        f.due_in(at(1_000)),
        Duration::from_millis(g.limit_ms - 1_000)
    );
    physical(&mut f, LEFT_DOWN, 1_000);
    assert_eq!(
        f.due_in(at(1_500)),
        Duration::from_millis(g.limit_ms - 1_500)
    );
    assert_eq!(f.due_in(at(g.limit_ms + 1)), Duration::ZERO);
}

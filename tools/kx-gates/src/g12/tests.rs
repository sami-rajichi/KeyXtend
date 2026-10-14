//! Tests of `g12`: its own capture, and the verdict.

use super::*;

fn cfg() -> GatesConfig {
    crate::config::load().expect("kx-gates.toml loads")
}

#[test]
fn g12_records_its_own_csv_with_g8s_presentmon_and_waits() {
    let c = cfg();
    let cap = capture(&c);
    assert_eq!(
        (cap.csv.as_str(), cap.secs),
        (c.g12.presentmon.as_str(), c.g12.presentmon_secs)
    );
    let g8 = &c.g8;
    assert_eq!(cap.exe, g8.presentmon_exe);
    assert_eq!(
        (cap.wait_ms, cap.stop_ms),
        (g8.presentmon_wait_ms, g8.presentmon_stop_ms)
    );
    assert_ne!(cap.csv, g8.presentmon, "a CSV of its own");
}

/// Clicks in the verdict tests.
const N: usize = 200;

/// A run of `n` clicks that all reached target-window under the overlay, each with its ring span.
fn clean(n: usize) -> Driven {
    let tally = Tally {
        clicked: vec![POINT::default(); n],
        spans: vec![
            Span {
                from_us: 0,
                to_us: 0
            };
            n
        ],
        covered: n,
        blocked: 0,
    };
    Driven {
        tally,
        reached: n,
        left_out: 0,
    }
}

/// A ring drawn in all `N` spans that just meets `g`'s frame rate and gap budgets, every frame shown.
fn smooth(g: &G12) -> Ring {
    Ring {
        chain: "0x1".into(),
        frames: 2 * N,
        gaps: N,
        drawn: N,
        fps: Some(g.min_fps),
        quiet_ms: 0.0,
        p99_ms: Some(g.p99_ms),
        lost: 0,
    }
}

/// Whether `d` with `ring` passes under the shipped `[g12]`, the test build having quit or not.
fn passes(d: &Driven, ring: &Result<Ring, String>, quit: bool) -> bool {
    let closed = Closed {
        quit,
        owner_back: None,
        lines: Ok(Vec::new()),
    };
    result(&cfg().g12, d, ring, &closed)["pass"] == json!(true)
}

#[test]
fn the_verdict_needs_every_click_under_the_overlay_and_the_test_build_quit() {
    let ring = Ok(smooth(&cfg().g12));
    assert!(passes(&clean(N), &ring, true));
    let spoilt: [fn(&mut Driven); 5] = [
        |d| d.tally.blocked = 1,
        |d| d.tally.covered -= 1,
        |d| d.reached -= 1,
        |d| d.left_out = 1,
        |d| *d = clean(0),
    ];
    for (i, spoil) in spoilt.iter().enumerate() {
        let mut d = clean(N);
        spoil(&mut d);
        assert!(!passes(&d, &ring, true), "case {i}");
    }
    assert!(
        !passes(&clean(N), &ring, false),
        "the test build did not quit"
    );
}

#[test]
fn the_verdict_needs_a_ring_drawn_in_every_span_smooth_and_shown() {
    let g = cfg().g12;
    let spoilt: [fn(&mut Ring, &G12); 4] = [
        |r, g| r.fps = Some(g.min_fps / 2.0),
        |r, g| r.p99_ms = Some(g.p99_ms * 2.0),
        |r, _| r.drawn -= 1,
        |r, _| r.lost = 1,
    ];
    for (i, spoil) in spoilt.iter().enumerate() {
        let mut r = smooth(&g);
        spoil(&mut r, &g);
        assert!(!passes(&clean(N), &Ok(r), true), "case {i}");
    }
    assert!(!passes(&clean(N), &Err("cut".into()), true), "no frames");
}

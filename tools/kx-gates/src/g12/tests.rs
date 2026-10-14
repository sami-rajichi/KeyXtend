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

/// A run of `n` clicks that all reached target-window under the overlay.
fn clean(n: usize) -> Driven {
    let tally = Tally {
        clicked: vec![POINT::default(); n],
        covered: n,
        ..Tally::default()
    };
    Driven {
        tally,
        reached: n,
        left_out: 0,
    }
}

/// A ring that just meets `g`'s frame rate and gap budgets.
fn smooth(g: &G12) -> Ring {
    Ring {
        chain: "0x1".into(),
        frames: 2,
        gaps: 1,
        fps: Some(g.min_fps),
        p99_ms: Some(g.p99_ms),
        lost: 0,
    }
}

/// A test build that quit in time, or not.
fn ended(quit: bool) -> Closed {
    Closed {
        quit,
        owner_back: None,
        lines: Ok(Vec::new()),
    }
}

#[test]
fn the_verdict_needs_every_click_a_smooth_ring_and_the_test_build_quit() {
    let g = cfg().g12;
    let pass = |d: &Driven, r: &Result<Ring, String>, quit: bool| {
        result(&g, d, r, &ended(quit))["pass"] == json!(true)
    };
    let ring = Ok(smooth(&g));
    assert!(pass(&clean(200), &ring, true));
    let spoilt: [fn(&mut Driven); 5] = [
        |d| d.tally.blocked = 1,
        |d| d.tally.covered -= 1,
        |d| d.reached -= 1,
        |d| d.left_out = 1,
        |d| *d = clean(0),
    ];
    for (i, spoil) in spoilt.iter().enumerate() {
        let mut d = clean(200);
        spoil(&mut d);
        assert!(!pass(&d, &ring, true), "case {i}");
    }
    let slow = Ring {
        fps: Some(g.min_fps / 2.0),
        ..smooth(&g)
    };
    assert!(!pass(&clean(200), &Ok(slow), true), "too few frames");
    let uneven = Ring {
        p99_ms: Some(g.p99_ms * 2.0),
        ..smooth(&g)
    };
    assert!(!pass(&clean(200), &Ok(uneven), true), "gaps too long");
    assert!(!pass(&clean(200), &Err("cut".into()), true), "no frames");
    assert!(
        !pass(&clean(200), &ring, false),
        "the test build did not quit"
    );
}

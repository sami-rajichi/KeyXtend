//! Tests of G8's tip hovers: the polls through a rest, what one hover saw and each key's counts.

use std::cell::Cell;

use serde_json::json;

use super::*;
use crate::g8churn::{Churn, whole_over};
use crate::proc::Memory;

/// Polls at the given ms, each with whether a new window showed.
fn polls(at: &[(u64, bool)]) -> Vec<Poll> {
    at.iter().map(|&(ms, shown)| Poll { ms, shown }).collect()
}

fn ids() -> Vec<String> {
    ["bksp", "enter", "shift"].map(String::from).to_vec()
}

#[test]
fn a_rest_polls_every_poll_ms_and_its_last_poll_comes_at_its_end() {
    // A stand-in clock that moves only when the rest waits, so nothing sleeps.
    let now = Cell::new(0);
    let shows_at = 520;
    let got = rest(
        800,
        50,
        || now.get(),
        |ms| now.set(now.get() + ms),
        || now.get() >= shows_at,
    );
    let times: Vec<u64> = got.iter().map(|p| p.ms).collect();
    assert_eq!(times, (1..=16).map(|i| i * 50).collect::<Vec<_>>());
    assert_eq!(got.iter().filter(|p| p.shown).count(), 6, "from 550 on");
    let then = Cell::new(0);
    let short = rest(
        120,
        50,
        || then.get(),
        |ms| then.set(then.get() + ms),
        || false,
    );
    let ends: Vec<u64> = short.iter().map(|p| p.ms).collect();
    assert_eq!(
        ends,
        [50, 100, 120],
        "the last wait stops at the rest's end"
    );
}

#[test]
fn a_rest_whose_waits_overshoot_and_whose_looks_take_time_still_ends_one_look_past_its_end() {
    // Each wait sleeps a little long and each look takes time, as on a busy PC; the clock moves only in them.
    let (over, look) = (3, 2);
    let now = Cell::new(0);
    let got = rest(
        800,
        50,
        || now.get(),
        |ms| now.set(now.get() + ms + over),
        || {
            now.set(now.get() + look);
            false
        },
    );
    let (last, earlier) = got.split_last().expect("at least one poll");
    assert!(
        earlier.iter().all(|p| p.ms < 800),
        "only the last poll is past the end"
    );
    assert!((800..=800 + over + look).contains(&last.ms), "{}", last.ms);
    assert_eq!(now.get(), last.ms, "nothing runs after the last poll");
}

#[test]
fn a_hover_hits_when_a_new_window_shows_at_the_rests_end_as_g8_always_counted() {
    let h = Hover::of(&polls(&[(450, false), (500, true), (800, true)]));
    assert_eq!(
        h,
        Hover {
            hit: true,
            shown_ms: Some(500)
        }
    );
    assert!(!h.hid());
    let never = Hover::of(&polls(&[(400, false), (800, false)]));
    assert_eq!(
        never,
        Hover {
            hit: false,
            shown_ms: None
        }
    );
    assert_eq!(Hover::of(&[]), never, "no poll sees nothing");
}

#[test]
fn a_tip_still_showing_from_the_last_key_is_not_this_keys_show() {
    let late = Hover::of(&polls(&[
        (50, true),
        (100, false),
        (550, true),
        (800, true),
    ]));
    assert_eq!(
        late,
        Hover {
            hit: true,
            shown_ms: Some(550)
        }
    );
    let stuck = Hover::of(&polls(&[(50, true), (800, true)]));
    assert_eq!(
        stuck,
        Hover {
            hit: true,
            shown_ms: None
        },
        "it never left"
    );
    let gone = Hover::of(&polls(&[(50, true), (800, false)]));
    assert!(
        !gone.hit && !gone.hid(),
        "the last key's tip hid, and this one never showed"
    );
}

#[test]
fn a_tip_that_showed_then_hid_before_the_rests_end_is_a_miss_that_hid() {
    let h = Hover::of(&polls(&[
        (450, false),
        (500, true),
        (650, false),
        (800, false),
    ]));
    assert_eq!(
        h,
        Hover {
            hit: false,
            shown_ms: Some(500)
        }
    );
    assert!(h.hid());
    let again = Hover::of(&polls(&[
        (450, false),
        (500, true),
        (650, false),
        (800, true),
    ]));
    assert!(again.hit && !again.hid(), "it showed again by the end");
}

#[test]
fn each_key_reports_its_hits_hidden_tips_and_the_median_and_slowest_show() {
    let mut keys = Keys::default();
    let hit = |ms| Hover {
        hit: true,
        shown_ms: Some(ms),
    };
    for ms in [500, 540, 520, 600] {
        keys.add(0, hit(ms));
    }
    keys.add(
        1,
        Hover {
            hit: false,
            shown_ms: Some(510),
        },
    );
    keys.add(
        1,
        Hover {
            hit: true,
            shown_ms: None,
        },
    );
    let shown = serde_json::to_value(keys.report(&ids())).expect("serialises");
    assert_eq!(
        shown,
        json!([
            { "id": "bksp", "hits": 4, "hid": 0, "show_p50_ms": 530.0, "show_max_ms": 600 },
            { "id": "enter", "hits": 1, "hid": 1, "show_p50_ms": null, "show_max_ms": null },
            { "id": "shift", "hits": 0, "hid": 0, "show_p50_ms": null, "show_max_ms": null }
        ])
    );
}

#[test]
fn a_churns_result_lists_each_tip_key_for_the_measured_the_first_and_the_later_warm_up_rounds() {
    let g = crate::config::load().expect("kx-gates.toml loads").g8;
    let (mut first, mut warm) = (Keys::default(), Keys::default());
    let late = Hover {
        hit: false,
        shown_ms: Some(g.churn_tip_ms),
    };
    warm.add(1, late);
    first.add(2, late);
    let flat = Memory {
        working_mb: 98.0,
        commit_mb: 138.0,
    };
    let c = Churn {
        first_tip_keys: first.report(&g.churn_tips),
        warm_tip_keys: warm.report(&g.churn_tips),
        ..whole_over(&g, flat, flat)
    };
    let shown = serde_json::to_value(&c).expect("serialises");
    let lists = [
        ("tip_keys", g.churn_rounds),
        ("first_tip_keys", 0),
        ("warm_tip_keys", 0),
    ];
    for (list, hits) in lists {
        let keys = shown[list].as_array().expect("a list");
        let ids: Vec<&str> = keys.iter().filter_map(|k| k["id"].as_str()).collect();
        assert_eq!(ids, g.churn_tips, "{list}");
        assert!(keys.iter().all(|k| k["hits"] == json!(hits)), "{list}");
    }
    assert_eq!(shown["first_tip_keys"][2]["hid"], json!(1), "round 1 apart");
    assert_eq!(shown["warm_tip_keys"][1]["hid"], json!(1));
    assert_eq!(shown["tips"], json!(c.hovers), "the total stays");
}

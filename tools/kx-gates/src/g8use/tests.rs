//! Tests of G8's reading after use: its settings, its two parts, its sweep and how a broken step ends.

use kx_mod_keyboard::Assist;
use serde_json::json;

use super::*;

/// kx-ui's defaults, whose tip time the sweep's rest stays below.
// Workspace path from `tools/kx-gates/src/g8use`; a moved file breaks the build here, never silently.
const UI_DEFAULTS: &str = include_str!("../../../../crates/kx-ui/defaults.toml");

fn shipped() -> G8 {
    crate::config::load().expect("kx-gates.toml loads").g8
}

fn used(working_mb: f64, commit_mb: f64) -> Used {
    read_as(Memory {
        working_mb,
        commit_mb,
    })
}

#[test]
fn the_shipped_use_step_lights_right_click_and_rests_below_the_tip_time() {
    let g = shipped();
    assert_eq!(check(&g), Ok(()));
    let rclick = board::id_of(Role::Assist(Assist::RClick));
    assert_eq!(rclick, Ok(g.use_mode.clone()));
    let ui: toml::Table = UI_DEFAULTS.parse().expect("is TOML");
    let tip_ms = ui["tip_ms"]
        .as_integer()
        .and_then(|t| u64::try_from(t).ok());
    assert!(
        tip_ms.is_some_and(|t| g.use_hover_ms < t),
        "no tip shows: {tip_ms:?}"
    );
}

#[test]
fn a_use_step_that_could_use_nothing_or_light_no_mode_is_refused() {
    let bad: [fn(&mut G8); 5] = [
        |g| g.use_hover_ms = 0,
        |g| g.use_lit_ms = 0,
        |g| g.use_mode = "c-0x001E".into(),
        |g| g.use_mode = "shortcuts".into(),
        |g| g.use_mode = "c-0xFFFF".into(),
    ];
    for (i, f) in bad.iter().enumerate() {
        let mut g = shipped();
        f(&mut g);
        let e = check(&g).expect_err("refused");
        assert!(e.contains("g8.use_"), "case {i}: {e}");
        assert!(g.check().is_err(), "case {i}: G8's own check");
    }
}

#[test]
fn a_reading_within_the_cold_budgets_passes_and_one_over_fails_its_own_part() {
    let g = shipped();
    let within = parts(&g, Some(&used(g.memory_mb, g.commit_mb)));
    assert!(within.iter().all(|p| p.pass), "{within:?}");
    assert_eq!(within.map(|p| p.name), [MEMORY, COMMIT]);
    let commit = parts(&g, Some(&used(g.memory_mb, g.commit_mb + 1.0)));
    assert_eq!(commit.map(|p| p.pass), [true, false]);
    let working = parts(&g, Some(&used(g.memory_mb + 1.0, g.commit_mb)));
    assert_eq!(working.map(|p| p.pass), [false, true]);
}

#[test]
fn no_reading_fails_both_parts_as_not_measured() {
    let missing = parts(&shipped(), None);
    assert!(
        missing.iter().all(|p| !p.pass && p.got.is_none()),
        "{missing:?}"
    );
}

#[test]
fn the_sweep_rests_on_each_key_in_the_order_given() {
    let ids = ["rclick", "esc", "lang"].map(String::from);
    let p = |x, y| POINT { x, y };
    let at = |id: &str| match id {
        "esc" => Some(p(5, 9)),
        "lang" => Some(p(300, 200)),
        "rclick" => Some(p(900, 40)),
        _ => None,
    };
    assert_eq!(sweep(&ids, at), Ok(vec![p(900, 40), p(5, 9), p(300, 200)]));
}

#[test]
fn a_key_with_no_box_fails_the_sweep_rather_than_read_too_low() {
    let ids = ["esc", "c-0x0010", "lang"].map(String::from);
    let at = |id: &str| (id != "c-0x0010").then_some(POINT { x: 1, y: 1 });
    let e = sweep(&ids, at).expect_err("a part sweep");
    assert!(e.starts_with("1 of 3 board keys"), "{e}");
    assert!(!e.contains("c-0x0010"), "names no key: {e}");
}

#[test]
fn the_sweep_takes_every_key_of_the_board_once_in_board_order() {
    let ids = board::ids().expect("the board reads");
    let place = |id: &str| ids.iter().position(|k| k == id);
    let at = |id: &str| {
        place(id)
            .and_then(|i| i32::try_from(i).ok())
            .map(|x| POINT { x, y: 0 })
    };
    let points = sweep(&ids, at).expect("every key has a box");
    assert_eq!(points.len(), ids.len());
    assert!(points.windows(2).all(|w| w[0].x < w[1].x), "board order");
}

#[test]
fn a_step_that_broke_says_whether_the_mode_went_back() {
    assert_eq!(joined(Ok(7), Ok(())), Ok(7));
    let e = joined::<u8>(Err("the pointer stopped".into()), Ok(())).expect_err("broke");
    assert!(
        e.starts_with("the pointer stopped") && e.contains(BACK),
        "{e}"
    );
    let e = joined(Ok(7), Err("no state".into())).expect_err("the mode stayed");
    assert!(e.contains(NOT_BACK) && e.contains("no state"), "{e}");
    let e = joined::<u8>(Err("covered".into()), Err("no state".into())).expect_err("both");
    assert!(e.starts_with("covered") && e.contains(NOT_BACK), "{e}");
}

#[test]
fn the_layout_at_the_reading_is_read_back_from_the_result_line() {
    let layout = "0x00000000040C040C".to_string();
    let mut v = json!({ "gate": "G8" });
    v[FIELD] = json!(Used {
        layout: Some(layout.clone()),
        ..used(90.0, 130.0)
    });
    assert_eq!(layout_in(&v), Some(layout));
    v[FIELD] = json!({ "error": "no keyboard" });
    assert_eq!(layout_in(&v), None, "not read");
}

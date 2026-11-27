//! Tests of `p12cfg`.

use super::*;
use crate::config::load;

/// The shipped file, whose every key the tests remove in turn.
const SHIPPED: &str = include_str!("../../kx-gates-p12.toml");

/// A setting spoiled, and the key its error must name.
type Spoil = (fn(&mut G24), &'static str);

/// Runs each spoil on a fresh copy of the shipped settings and checks its error names its key.
fn refused(spoils: &[Spoil]) {
    let good = load().expect("loads");
    for (i, (spoil, key)) in spoils.iter().enumerate() {
        let mut c = good.clone();
        spoil(&mut c.g24);
        let e = check(&c).expect_err(&format!("case {i} passed"));
        assert!(e.contains(key), "case {i}: {e}");
    }
}

/// The snip module's defaults, which the gate's copy runs on.
fn snip() -> kx_mod_snip::SnipSettings {
    kx_mod_snip::SnipSettings::from_defaults().expect("the snip defaults")
}

#[test]
fn the_shipped_settings_load_and_pass() {
    let c = load().expect("loads");
    assert_eq!(check(&c), Ok(()));
}

#[test]
fn every_key_in_the_file_is_read_and_a_missing_one_is_an_error_naming_it() {
    let file: toml::Table = SHIPPED.parse().expect("parses");
    let read = |v: toml::Value| v.try_into::<G24>().map(drop).map_err(|e| e.to_string());
    let section = file["g24"].as_table().expect("a section").clone();
    assert_eq!(file.len(), 1, "only [g24]");
    assert_eq!(read(toml::Value::Table(section.clone())), Ok(()));
    for key in section.keys() {
        let mut less = section.clone();
        less.remove(key);
        let e = read(toml::Value::Table(less)).expect_err(key);
        assert!(e.contains(key.as_str()), "{key}: {e}");
    }
    let mut more = section;
    more.insert("stray".into(), toml::Value::Integer(1));
    let e = read(toml::Value::Table(more)).expect_err("unknown");
    assert!(e.contains("stray"), "{e}");
}

#[test]
fn the_waits_outlast_the_countdown_and_the_step_aside() {
    let (g, s) = (load().expect("loads").g24, snip());
    let count_ms = u64::from(s.countdown_s) * u64::from(MS_PER_S);
    let aside_ms = u64::from(s.aside_ms);
    assert!(
        g.wait_ms > count_ms,
        "the Stop key shows after the countdown"
    );
    assert!(g.cover_ms > aside_ms, "the freeze follows the step aside");
}

#[test]
fn the_speed_clicks_step_the_strips_own_speeds_from_as_recorded_to_the_speed_asked() {
    let mut g = load().expect("loads").g24;
    for &times in &SpeedAsk::SPEEDS[1..] {
        g.speed = times;
        let reached = (0..g.speed_steps()).fold(Speed::PLAIN, |at, _| at.next());
        assert_eq!(Some(reached), Speed::new(times), "{times}×");
    }
}

#[test]
fn a_value_that_could_judge_nothing_is_refused() {
    refused(&[
        (|g| g.text.clear(), "g24.text"),
        (|g| g.region_px[2] = 0.0, "g24.region_px"),
        (|g| g.dpi = 0, "g24.dpi"),
        (|g| g.frame_share = 1.0, "g24.frame_share"),
        (|g| g.same_most = 0.0, "g24.same_most"),
        (|g| g.back_mb = 0.0, "g24.back_mb"),
        (|g| g.speed = 1, "g24.speed"),
        (|g| g.speed = 3, "g24.speed"),
        (|g| g.record_s = 0, "g24.record_s"),
        (
            |g| g.slack_ms = g.record_s * u64::from(MS_PER_S),
            "g24.slack_ms",
        ),
        (|g| g.long_s = 1, "g24.long_s"),
        (|g| g.edit_px[3] = 0.0, "g24.edit_px"),
        (|g| g.edit_px[2] = 160.0, "g24.edit_px"),
        (|g| g.edit_px[3] = 28.0, "g24.edit_px"),
        (|g| g.reset_passes = 0, "g24.reset_passes"),
        (|g| g.warm_s = 0, "g24.warm_s"),
        (|g| g.history_item_mb = 0, "g24.history_item_mb"),
        (|g| g.history_pad_ms = 0, "g24.history_pad_ms"),
        (|g| g.history_pad_ms = g.wait_ms + 1, "g24.history_pad_ms"),
        (|g| g.close_tries = 0, "g24.close_tries"),
    ]);
}

#[test]
fn each_close_of_a_kept_video_may_wait_its_own_time() {
    let c = load().expect("loads");
    let mut longer = c.clone();
    longer.g24.close_ms += 1;
    let closes = KEPT_CLOSES * c.g24.close_tries;
    assert_eq!(g24_ms(&longer) - g24_ms(&c), times(closes, 1));
}

#[test]
fn the_editors_part_lands_between_logical_pixels_so_the_hub_must_trim_it() {
    let g = load().expect("loads").g24;
    let [.., w, h] = g.edit_px.map(|v| physical(f64::from(v), g.dpi));
    for side in [w, h] {
        let side = u32::try_from(side).expect("a size");
        assert!(
            !(side * BASE_DPI).is_multiple_of(g.dpi),
            "{side} px is whole logical pixels"
        );
    }
}

/// kx-ui's Snip QML that names the buttons' gate ids, and the ids its hub gives each key.
const QML: [&str; 8] = [
    include_str!("../../../../crates/kx-ui/qml/SnipKey.qml"),
    include_str!("../../../../crates/kx-ui/qml/SnipBar.qml"),
    include_str!("../../../../crates/kx-ui/qml/SnipStrip.qml"),
    include_str!("../../../../crates/kx-ui/qml/SnipPill.qml"),
    include_str!("../../../../crates/kx-ui/qml/SnipKeeps.qml"),
    include_str!("../../../../crates/kx-ui/qml/SnipAsk.qml"),
    include_str!("../../../../crates/kx-ui/qml/SnipHead.qml"),
    include_str!("../../../../crates/kx-ui/qml/SnipNames.qml"),
];
const KEY_IDS: &str = include_str!("../../../../crates/kx-ui/src/i18n/keys/snip.rs");
/// `SnipKey`'s id, from its group's name in `SnipNames`.
const GROUP_ID: &str = "res.uid(\"snip.\" + sk.group";

/// The ids kx-ui's `snip_<group>_id` table gives one group's buttons; none when it has no such table.
fn table(group: &str) -> Vec<&'static str> {
    let body = KEY_IDS
        .split_once(&format!("snip_{group}_id("))
        .and_then(|(_, rest)| rest.split_once('}'))
        .map_or("", |(body, _)| body);
    let ids = body.split("=> \"").skip(1);
    ids.filter_map(|s| s.split_once('"').map(|(id, _)| id))
        .collect()
}

/// True when a Snip QML file builds the ids of `group`'s buttons: by its literal, or as `SnipKey`s of that group.
fn builds(group: &str) -> bool {
    let has = |s: &str| QML.iter().any(|q| q.contains(s));
    let literal = has(&format!("\"snip.{group}.\"")) || has(&format!("res.uid(\"snip.{group}\""));
    let keyed = has(GROUP_ID)
        && has(&format!("group: glow.n.acts.{group}"))
        && has(&format!("{group}: \"{group}\""));
    literal || keyed
}

#[test]
fn every_id_is_one_kx_uis_snip_buttons_carry() {
    let g = load().expect("loads").g24;
    for (key, id) in g.ids.all() {
        let (group, part) = id
            .strip_prefix("snip.")
            .and_then(|r| r.split_once('.'))
            .filter(|(group, part)| !group.is_empty() && !part.is_empty())
            .unwrap_or_else(|| panic!("g24.ids.{key}: {id} is not snip.<group>.<key>"));
        assert!(builds(group), "g24.ids.{key}: no {group} buttons");
        assert!(
            table(group).contains(&part),
            "g24.ids.{key}: kx-ui's {group} buttons have no {part}"
        );
    }
}

#[test]
fn a_gate_id_no_snip_button_carries_is_found() {
    assert!(table("strip").contains(&"speed") && builds("strip"));
    assert!(
        table("speed").is_empty() && !builds("speed"),
        "no speed buttons"
    );
    assert!(!table("strip").contains(&"speedy"));
    assert!(table("bar").contains(&"video-region") && builds("bar"));
}

#[test]
fn a_wait_past_the_mouse_guard_or_of_nothing_is_refused() {
    refused(&[
        (|g| g.wait_ms = 0, "g24.wait_ms"),
        (|g| g.ready_ms = 0, "g24.ready_ms"),
        (|g| g.settle_ms = u64::MAX, "g24.settle_ms"),
        (|g| g.idle_ms = 0, "g24.idle_ms"),
        (|g| g.cover_ms = 0, "g24.cover_ms"),
        (|g| g.speed_wait_ms = 0, "g24.speed_wait_ms"),
        (|g| g.gone_ms = 0, "g24.gone_ms"),
        (|g| g.order_ms = 0, "g24.order_ms"),
        (|g| g.close_ms = 0, "g24.close_ms"),
        (|g| g.long_s = 1_000_000, "g24: G24 on the product"),
    ]);
}

#[test]
fn the_trap_part_fits_the_guard_in_a_fraction_of_the_full_run() {
    let c = load().expect("loads");
    let (trap, full) = (trap_ms(&c), g24_ms(&c));
    assert!(trap * 3 < full, "{trap} ms of {full} ms");
    let mut longer = c.clone();
    longer.g24.order_ms += 1;
    assert_eq!(trap_ms(&longer) - trap, times(TRAP_ORDERS, 1));
    let mut long = c;
    long.g24.wait_ms = long.mouse_guard.limit_ms / 8;
    let e = check(&long).expect_err("past the guard");
    assert!(e.contains("g24: the trap part"), "{e}");
}

#[test]
fn each_order_check_may_wait_its_own_time() {
    let c = load().expect("loads");
    let mut longer = c.clone();
    longer.g24.order_ms += 1;
    assert_eq!(g24_ms(&longer) - g24_ms(&c), times(ORDERS, 1));
}

#[test]
fn each_recording_may_wait_for_its_clock_and_the_shipped_run_still_fits_the_guard() {
    let c = load().expect("loads");
    let mut longer = c.clone();
    longer.g24.wait_ms += 1;
    let waits = SNIPS * (SNIP_WAITS + c.g24.reset_passes) + RECORDINGS;
    assert_eq!(g24_ms(&longer) - g24_ms(&c), times(waits, 1));
    let need = g24_ms(&c) + c.fit.guard_headroom_ms;
    assert!(need <= c.mouse_guard.limit_ms, "{need} ms");
}

#[test]
fn an_empty_or_shared_id_or_a_title_kx_ui_lacks_is_refused() {
    refused(&[
        (|g| g.ids.copy.clear(), "g24.ids.copy"),
        (|g| g.ids.stop.clone_from(&g.ids.copy), "g24.ids.stop"),
        (|g| g.ids.speed.clear(), "g24.ids.speed"),
        (
            |g| g.titles.frozen = "window.none".into(),
            "g24.titles.frozen",
        ),
        (|g| g.titles.capture.clear(), "g24.titles.capture"),
        (
            |g| g.titles.capture.push("window.none".into()),
            "g24.titles.capture",
        ),
        (|g| g.titles.bubble.clear(), "g24.titles.bubble"),
        (|g| g.clock.time = "snip.none".into(), "g24.clock.time"),
        (|g| g.clock.word.clear(), "g24.clock.word"),
        (|g| g.dialog_class.clear(), "g24.dialog_class"),
        (|g| g.cancel_id.clear(), "g24.cancel_id"),
        (|g| g.taskbar_class.clear(), "g24.taskbar_class"),
    ]);
}

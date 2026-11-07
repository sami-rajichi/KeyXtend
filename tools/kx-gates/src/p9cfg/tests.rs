//! Tests of `p9cfg`.

use super::*;
use crate::config::load;

/// The shipped file, whose every key the tests remove in turn.
const SHIPPED: &str = include_str!("../../kx-gates-p9.toml");

/// A setting spoiled, and the key its error must name.
type Spoil = (fn(&mut G23), &'static str);

/// Runs each spoil on a fresh copy of the shipped settings and checks its error names its key.
fn refused(spoils: &[Spoil]) {
    let good = load().expect("loads");
    for (i, (spoil, key)) in spoils.iter().enumerate() {
        let mut c = good.clone();
        spoil(&mut c.g23);
        let e = check(&c).expect_err(&format!("case {i} passed"));
        assert!(e.contains(key), "case {i}: {e}");
    }
}

#[test]
fn the_shipped_settings_load_and_pass() {
    let c = load().expect("loads");
    assert_eq!(check(&c), Ok(()));
    assert_ne!(c.g23.user, c.g23.password);
}

#[test]
fn every_key_in_the_file_is_read_and_a_missing_one_is_an_error_naming_it() {
    let file: toml::Table = SHIPPED.parse().expect("parses");
    let read = |v: toml::Value| v.try_into::<G23>().map(drop).map_err(|e| e.to_string());
    let section = file["g23"].as_table().expect("a section").clone();
    assert_eq!(file.len(), 1, "only [g23]");
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
fn the_panel_ids_follow_kx_uis_group_and_parts() {
    let g = load().expect("loads").g23;
    assert_eq!(g.id(&g.new), "fill.new");
    assert_eq!(g.id(&g.row), "fill.row-", "a row's id before its place");
    assert_eq!(g.row_button(0, &g.both), "fill.row-0-both");
    assert_eq!(g.value_id(1), "fill.field-1-value");
    assert_eq!(g.kind_id(), "fill.kind-login");
}

#[test]
fn a_test_value_no_key_types_at_once_is_refused() {
    refused(&[
        (|g| g.name.clear(), "g23.name"),
        (|g| g.user = "Kx".into(), "g23.user"),
        (|g| g.password = "kx pass".into(), "g23.password"),
        (|g| g.password = "قهوة".into(), "g23.password"),
        (|g| g.password.clone_from(&g.user), "g23.password"),
        (|g| g.lang = "english".into(), "g23.lang"),
    ]);
}

#[test]
fn ids_that_could_name_no_part_or_one_field_twice_are_refused() {
    refused(&[
        (|g| g.group.clear(), "g23.group"),
        (|g| g.group = "fill.x".into(), "g23.group"),
        (|g| g.save.clear(), "g23.save"),
        (|g| g.both.clear(), "g23.both"),
        (|g| g.kind.clear(), "g23.kind"),
        (|g| g.password_field = g.user_field, "g23.password_field"),
    ]);
}

#[test]
fn marks_the_title_could_not_tell_apart_are_refused() {
    refused(&[
        (|g| g.length_mark.clear(), "g23.length_mark"),
        (
            |g| g.enter_mark.clone_from(&g.length_mark),
            "g23.enter_mark",
        ),
        (|g| g.submit_mark = " #1".into(), "g23.submit_mark"),
        (|g| g.enter_mark = " '".into(), "g23.enter_mark"),
        (
            |g| g.submit_mark = format!("{}x", g.length_mark),
            "g23.submit_mark",
        ),
        (|g| g.enter_key = "Enter'".into(), "g23.enter_key"),
    ]);
}

#[test]
fn a_text_kx_ui_lacks_and_a_wait_past_the_guard_are_refused() {
    refused(&[
        (
            |g| g.notice_title = "window.no-such".into(),
            "g23.notice_title",
        ),
        (|g| g.ready_ms = 0, "g23.ready_ms"),
        (|g| g.fill_wait_ms = u64::MAX, "g23.fill_wait_ms"),
        (|g| g.late_ms = u64::MAX, "g23.late_ms"),
        (|g| g.notice_wait_ms = 0, "g23.notice_wait_ms"),
    ]);
}

#[test]
fn values_that_share_a_length_are_refused_as_the_password_shows_only_its_length() {
    refused(&[
        (|g| g.name = "a".repeat(g.user.len()), "g23.name"),
        (|g| g.user = "b".repeat(g.password.len()), "g23.user"),
        (|g| g.password = "c".repeat(g.name.len()), "g23.password"),
    ]);
}

#[test]
fn a_late_pause_no_longer_than_the_longest_gap_and_the_keyboards_lateness_is_refused() {
    let gap = u64::from(*TAB_GAP_MS.end());
    let late = KeyboardSettings::from_defaults()
        .expect("defaults")
        .fill_late_ms;
    let mut c = load().expect("loads");
    c.g23.late_ms = gap + late;
    let e = check(&c).expect_err("as long as the latest part");
    assert!(e.contains("g23.late_ms"), "{e}");
    c.g23.late_ms = gap + late + 1;
    assert_eq!(check(&c), Ok(()));
}

#[test]
fn todays_g23_fits_the_mouse_guard_and_each_wait_counts() {
    let c = load().expect("loads");
    assert_eq!(held_fits(&c, "g23", g23_ms(&c)), Ok(()));
    let more = |spoil: fn(&mut GatesConfig)| {
        let mut s = c.clone();
        spoil(&mut s);
        g23_ms(&s) - g23_ms(&c)
    };
    assert_eq!(more(|s| s.g23.name.push('x')), c.g27.key_gap_ms);
    assert_eq!(more(|s| s.g23.fill_wait_ms += 1), times(FILLS, 1));
    assert_eq!(more(|s| s.g23.late_ms += 1), times(FILLS, 1));
    let clicks = SEED_CLICKS + OPEN_CLOSE + FILLS;
    assert_eq!(more(|s| s.g23.settle_ms += 1), times(clicks + LINES, 1));
    let panel = clicks + SEED_WAITS + SHUT_WAITS + FILLS;
    assert_eq!(more(|s| s.g23.panel_wait_ms += 1), times(panel, 1));
    let switches = SWITCHES + LINES;
    assert_eq!(more(|s| s.g27.switch_wait_ms += 1), times(switches, 1));
    assert_eq!(more(|s| s.g23.notice_wait_ms += 1), times(NOTICE_WAITS, 1));
    assert_eq!(more(|s| s.timing.read_wait_ms += 1), times(FILLS, 1));
    assert_eq!(more(|s| s.probes.settle_ms += 1), times(FILLS, 1));
}

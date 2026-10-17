//! Tests of `p5cfg`.

use super::*;
use crate::config::load;

// Workspace paths from `tools/kx-gates/src/p5cfg`; a moved file breaks the build here, never silently.
/// The pad's arrows and their ids.
const DPAD_QML: &str = include_str!("../../../../crates/kx-ui/qml/DPad.qml");
/// Stop and its id.
const STOP_QML: &str = include_str!("../../../../crates/kx-ui/qml/Stop.qml");
/// The keyboard window's kit, whose `uid` joins a group and an id.
const KIT_QML: &str = include_str!("../../../../crates/kx-ui/qml/KbRes.qml");
/// The outline and its edge windows.
const OUTLINE_QML: &str = include_str!("../../../../crates/kx-ui/qml/Outline.qml");

#[test]
fn the_shipped_settings_pass() {
    assert_eq!(check(&load().expect("loads")), Ok(()));
}

#[test]
fn g6_settings_that_check_nothing_are_refused() {
    let bad: [fn(&mut G6); 24] = [
        |g| g.inset_px = 0,
        |g| g.spot_px = 0,
        |g| g.max_per_row = 1,
        |g| g.grow_wait_ms = 0,
        |g| g.grow_wait_ms = g.sample_ms - 1,
        |g| g.slope_reads = 1,
        |g| g.phase_ms = g.sample_ms * (g.slope_reads - 1),
        |g| g.min_changed = 0.0,
        |g| g.min_changed = 1.0,
        |g| g.min_changed = f64::NAN,
        |g| g.sample_ms = 0,
        |g| g.phase_ms = g.sample_ms,
        |g| g.ratio_slack = 0.0,
        |g| g.ratio_slack = 1.0,
        |g| g.ratio_slack = f64::NAN,
        |g| g.move_ms = 0,
        |g| g.still_ms = 0,
        |g| g.still_ms = g.sample_ms - 1,
        |g| g.back_ms = 0,
        |g| g.end_wait_ms = 0,
        |g| g.lag_wait_ms = 0,
        |g| g.hang_ms = 0,
        |g| g.burst_ms = 0,
        |g| g.burst_ms = g.hang_lead_ms + 1,
    ];
    for (i, f) in bad.iter().enumerate() {
        let mut g = load().expect("loads").g6;
        f(&mut g);
        assert!(g.check().is_err(), "case {i}");
    }
}

#[test]
fn a_g6_hang_past_target_windows_cap_is_refused() {
    let cap = kx_target_window::config::load()
        .expect("target config loads")
        .hang_max_ms;
    let mut c = load().expect("loads");
    c.g6.hang_ms = cap;
    assert_eq!(check(&c), Ok(()), "at the cap");
    c.g6.hang_ms = cap + 1;
    let e = check(&c).expect_err("past the cap");
    assert!(e.contains("hang_max_ms"), "{e}");
}

#[test]
fn pad_ids_join_the_group_and_the_part() {
    let pad = load().expect("loads").pad;
    assert_eq!(pad.arrow(Dir::Down), "dpad.down");
    assert_eq!(pad.arrow(Dir::Left), "dpad.left");
    assert_eq!(pad.stop_id(), "dpad.stop");
    let reach = pad.reach();
    assert_eq!((reach.under_px, reach.lead_px), (pad.under_px, pad.lead_px));
    let run = 1_000;
    assert_eq!(pad.undone(run) * u64::from(pad.undo_div), run);
    assert!(
        pad.undone(run) < run,
        "a run back never undoes the run before it"
    );
}

#[test]
fn pad_settings_that_name_no_part_are_refused() {
    let bad: [fn(&mut Pad); 11] = [
        |p| p.settle_ms = 1,
        |p| p.moves.clear(),
        |p| p.moves.push("nowhere".into()),
        |p| p.undo_div = 0,
        |p| p.undo_div = 1,
        |p| p.group.clear(),
        |p| p.group = "d.pad".into(),
        |p| p.stop = "halt".into(),
        |p| p.settle_ms = 0,
        |p| p.under_px = 0,
        |p| p.lead_px = -1,
    ];
    for (i, f) in bad.iter().enumerate() {
        let mut p = load().expect("loads").pad;
        f(&mut p);
        assert!(p.check().is_err(), "case {i}");
    }
}

#[test]
fn the_pads_settle_outlasts_each_move_it_names_and_the_shipped_ones_name_stop_and_the_outline() {
    let motion = kx_theme::Themes::shipped().expect("tokens").motion.full;
    let mut p = load().expect("loads").pad;
    assert_eq!(
        p.moves,
        ["stop", "outline"],
        "Stop's pop and the outline's fade"
    );
    let longest = p.moves.iter().map(|m| motion.moves[m].ms).max();
    p.settle_ms = u64::from(longest.expect("a move"));
    assert_eq!(p.check(), Ok(()), "as long as the longest");
    p.settle_ms -= 1;
    let e = p.check().expect_err("shorter");
    assert!(e.contains("motion.toml"), "{e}");
}

#[test]
fn the_pad_ids_are_the_ones_kx_ui_gives_its_arrows_and_stop() {
    let pad = load().expect("loads").pad;
    let group = format!("res.uid(\"{}\", ", pad.group);
    let arrows = DPAD_QML
        .lines()
        .any(|l| l.contains(&group) && l.trim_end().ends_with("modelData.id)"));
    assert!(arrows, "arrows");
    assert!(STOP_QML.contains(&format!("{group}pad.stopId)")), "Stop");
    assert!(DPAD_QML.contains(&format!("stopId: \"{}\"", pad.stop)));
    assert!(KIT_QML.contains(&format!("group + \"{SEP}\" + id")), "uid");
    for dir in Dir::ALL {
        let id = pad.arrow(dir);
        let part = id.strip_prefix(&format!("{}{SEP}", pad.group));
        let line = format!("{{ id: \"{}\",", part.expect("grouped"));
        assert!(DPAD_QML.contains(&line), "{id}");
    }
}

#[test]
fn outline_settings_that_find_nothing_are_refused() {
    let bad: [fn(&mut Outline); 4] = [
        |o| o.title = "window.nowhere".into(),
        |o| o.title.clear(),
        |o| o.rest = "c-0xFFFF".into(),
        |o| o.edges = 0,
    ];
    for (i, f) in bad.iter().enumerate() {
        let mut o = load().expect("loads").outline;
        f(&mut o);
        assert!(o.check().is_err(), "case {i}");
    }
}

#[test]
fn the_outline_has_as_many_edges_as_kx_ui_makes() {
    let edges = load().expect("loads").outline.edges;
    let model = OUTLINE_QML
        .lines()
        .find_map(|l| l.trim().strip_prefix("model: ["))
        .expect("the edges' model");
    assert_eq!(model.matches('"').count() / 2, edges, "{model}");
}

#[test]
fn the_outline_title_is_the_text_kx_ui_titles_its_edges_with() {
    let title = load().expect("loads").outline.title;
    let tr = format!("tr(\"{title}\")");
    assert!(OUTLINE_QML.contains(&tr), "{tr}");
}

#[test]
fn g8_scroll_and_host_settings_that_do_nothing_are_refused() {
    let mut c = load().expect("loads");
    c.g8_scroll.scroll_ms = 0;
    assert!(check(&c).is_err(), "no scroll");
    c.g8_scroll.scroll_ms = c.pad.settle_ms;
    let e = check(&c).expect_err("Stop would go out before the pad's settle");
    assert!(e.contains("settle_ms"), "{e}");
    let names = [
        "",
        r"C:\vmmem",
        "vmmem.exe",
        "vmmem.EXE",
        "vmmem#1",
        "vm(1)",
        "vm)",
    ];
    let mut vms: Vec<Vec<String>> = names.iter().map(|n| vec![(*n).to_string()]).collect();
    vms.push(vec![]);
    for vm in vms {
        let mut c = load().expect("loads");
        c.host.vm = vm;
        assert!(check(&c).is_err(), "{:?}", c.host.vm);
    }
}

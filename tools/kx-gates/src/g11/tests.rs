//! Tests of G11's verdict, its series and its records on samples recorded on this PC.

use super::*;

/// The keyboard as recorded on this PC on 2026-10-01: 125 %, 850 × 257 px, per-monitor v2.
fn at(scale: u32, size: [f64; 2]) -> Record {
    let dpi = scale * USER_DEFAULT_SCREEN_DPI / PCT;
    Record {
        scale,
        window_dpi: dpi,
        screen_dpi: dpi,
        pmv2: true,
        size,
        build: 0,
    }
}

#[test]
fn a_dpi_reads_as_its_scale_and_pixels_as_logical_pixels() {
    assert_eq!([96, 120, 144, 192].map(scale_of), [100, 125, 150, 200]);
    assert!(
        (logical(1063, 120) - 850.4).abs() < 1e-9,
        "this PC's keyboard at 125 %"
    );
    assert!(
        (logical(10, 0) - 960.0).abs() < 1e-9,
        "a DPI of 0 reads as 1, never a division by 0"
    );
}

#[test]
fn every_wanted_scale_sharp_at_one_size_passes() {
    let all = [
        at(100, [680.0, 205.6]),
        at(150, [680.0, 205.3]),
        at(200, [679.5, 205.5]),
    ];
    let v = verdict(&all, &[100, 150, 200], 2.0);
    assert!(v.pass, "{v:?}");
    assert!((v.size_gap_px - 0.5).abs() < 1e-9);
}

#[test]
fn a_missing_scale_a_stretched_window_or_a_changed_size_fails() {
    let wanted = [100, 150, 200];
    let two = [
        at(100, [680.0, 205.0]),
        at(125, [680.0, 205.0]),
        at(150, [680.0, 205.0]),
    ];
    assert_eq!(
        verdict(&two, &wanted, 2.0).missing,
        [200],
        "125 % is only recorded"
    );
    let mut stretched = at(200, [680.0, 205.0]);
    stretched.window_dpi = USER_DEFAULT_SCREEN_DPI;
    let three = [two[0].clone(), two[2].clone(), stretched];
    assert_eq!(verdict(&three, &wanted, 2.0).blurred, [200]);
    let grown = [two[0].clone(), two[2].clone(), at(200, [700.0, 205.0])];
    assert!(!verdict(&grown, &wanted, 2.0).pass);
    let again = [
        grown[0].clone(),
        grown[1].clone(),
        grown[2].clone(),
        at(200, [681.0, 205.0]),
    ];
    assert!(
        verdict(&again, &wanted, 2.0).pass,
        "the latest run of a scale counts"
    );
}

#[test]
fn only_this_builds_runs_since_its_last_run_at_the_first_scale_count() {
    let build = |b: u64, r: Record| Record { build: b, ..r };
    let older = [
        build(1, at(100, [680.0, 205.0])),
        build(1, at(150, [680.0, 205.0])),
    ];
    let now = build(2, at(200, [680.0, 205.0]));
    let mut all = older.to_vec();
    all.push(now.clone());
    assert_eq!(
        series(&all, 100),
        std::slice::from_ref(&now),
        "a new build starts over"
    );
    let stale = build(2, at(200, [700.0, 205.0]));
    let fresh = [
        stale,
        build(2, at(100, [680.0, 205.0])),
        build(2, at(150, [680.0, 205.0])),
    ];
    let mut again = fresh.to_vec();
    again.push(now);
    assert_eq!(series(&again, 100), again[1..], "from the last 100 % run");
    assert!(verdict(&series(&again, 100), &[100, 150, 200], 2.0).pass);
    let no_first = [
        build(3, at(150, [680.0, 205.0])),
        build(3, at(200, [680.0, 205.0])),
    ];
    assert_eq!(
        series(&no_first, 100),
        no_first,
        "with no 100 % run yet, all of this build"
    );
}

#[test]
fn a_record_from_before_builds_were_kept_is_left_out() {
    let mut old = json!(at(150, [680.0, 205.0]));
    old.as_object_mut().map(|o| o.remove("build"));
    let log = format!("{}\n", json!({ "gate": "G11", "record": old }));
    assert!(records(&log).is_empty());
}

#[test]
fn records_come_from_g11_lines_of_the_results_log_only() {
    let line = json!({ "gate": "G11", "record": at(150, [680.0, 205.0]) }).to_string();
    let log =
        format!("{{\"gate\":\"G2\"}}\nnot json\n{line}\n{{\"gate\":\"G11\",\"error\":\"x\"}}\n");
    assert_eq!(records(&log), [at(150, [680.0, 205.0])]);
}

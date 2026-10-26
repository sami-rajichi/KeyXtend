//! Tests of `p7cfg`.

use super::*;
use crate::config::load;

/// The shipped file, whose every key the tests remove in turn.
const SHIPPED: &str = include_str!("../../kx-gates-p7.toml");
// Workspace paths from `tools/kx-gates/src/p7cfg`; a moved file breaks the build here, never silently.
/// The lane's names, its gate-id group among them.
const NAMES_QML: &str = include_str!("../../../../crates/kx-ui/qml/WordNames.qml");
/// The lane, which gives each chip and the lock note its id.
const LANE_QML: &str = include_str!("../../../../crates/kx-ui/qml/WordLane.qml");

/// A setting spoiled, and the key its error must name.
type Spoil = (fn(&mut GatesConfig), &'static str);

/// Runs each spoil on a fresh copy of the shipped settings and checks its error names its key.
fn refused(spoils: &[Spoil]) {
    let good = load().expect("loads");
    for (i, (spoil, key)) in spoils.iter().enumerate() {
        let mut c = good.clone();
        spoil(&mut c);
        let e = check(&c).expect_err(&format!("case {i} passed"));
        assert!(e.contains(key), "case {i}: {e}");
    }
}

/// Section `name` of the file read as its settings, or the reader's error.
fn read_section(name: &str, value: toml::Value) -> Result<(), String> {
    let e = |e: toml::de::Error| e.to_string();
    match name {
        "g27" => value.try_into::<G27>().map(drop).map_err(e),
        "g21_product" => value.try_into::<G21Product>().map(drop).map_err(e),
        "g8_predict" => value.try_into::<G8Predict>().map(drop).map_err(e),
        "owner_settings" => value.try_into::<OwnerSettings>().map(drop).map_err(e),
        other => Err(format!("no reader for [{other}]")),
    }
}

#[test]
fn the_shipped_settings_pass_with_the_owners_figure() {
    let c = load().expect("loads");
    assert_eq!(check(&c), Ok(()));
    let langs: Vec<&str> = c.g27.cases.iter().map(|k| k.lang.as_str()).collect();
    assert_eq!(langs, ["en", "fr", "ar"]);
    assert!((c.g8_predict.owner_mb - 2.0).abs() < f64::EPSILON);
}

#[test]
fn every_key_in_the_file_is_read_and_a_missing_one_is_an_error_naming_it() {
    let file: toml::Table = SHIPPED.parse().expect("parses");
    for (name, section) in &file {
        let keys = section.as_table().expect("a section");
        assert_eq!(read_section(name, section.clone()), Ok(()), "[{name}]");
        for key in keys.keys() {
            let mut less = keys.clone();
            less.remove(key);
            let e = read_section(name, toml::Value::Table(less)).expect_err(key);
            assert!(e.contains(key.as_str()), "[{name}] {key}: {e}");
        }
        let mut more = keys.clone();
        more.insert("stray".into(), toml::Value::Integer(1));
        let e = read_section(name, toml::Value::Table(more)).expect_err("unknown");
        assert!(e.contains("stray"), "[{name}]: {e}");
    }
}

#[test]
fn g27_cases_that_could_type_nothing_or_the_wrong_word_are_refused() {
    refused(&[
        (|c| c.g27.cases.clear(), "g27.cases"),
        (|c| c.g27.cases[0].prefix.clear(), "g27.cases"),
        (|c| c.g27.cases[0].prefix = "123".into(), "g27.cases"),
        (|c| c.g27.cases[0].prefix = "heا".into(), "g27.cases"),
        (|c| c.g27.cases[0].lang.clear(), "g27.cases"),
        (|c| c.g27.cases[0].lang = "EN".into(), "g27.cases"),
        (|c| c.g27.cases[0].lang = "e".into(), "g27.cases"),
        (|c| c.g27.cases[0].lang = "engl".into(), "g27.cases"),
        (|c| c.g27.cases[0].lang = "con".into(), "g27.cases"),
        (|c| c.g27.cases[1].lang = "en".into(), "g27.cases"),
        (|c| c.g27.edge = "xx".into(), "g27.edge"),
        (|c| c.g27.edge = "ar".into(), "g27.edge"),
        (|c| c.g21_product.case = "xx".into(), "g21_product.case"),
    ]);
}

#[test]
fn g27_names_and_waits_out_of_range_are_refused() {
    refused(&[
        (|c| c.g27.group.clear(), "g27.group"),
        (|c| c.g27.group = "a.b".into(), "g27.group"),
        (|c| c.g27.chip.clear(), "g27.chip"),
        (|c| c.g27.locked.clear(), "g27.locked"),
        (|c| c.g27.chip_name = "chip.nowhere".into(), "g27.chip_name"),
        (|c| c.g27.chip_word = "nothing".into(), "g27.chip_word"),
        (|c| c.g27.ready_ms = 0, "g27.ready_ms"),
        (
            |c| c.g27.ready_ms = c.mouse_guard.limit_ms + 1,
            "g27.ready_ms",
        ),
        (|c| c.g27.switch_wait_ms = 0, "g27.switch_wait_ms"),
        (|c| c.g27.key_gap_ms = 0, "g27.key_gap_ms"),
        (|c| c.g27.chip_wait_ms = 0, "g27.chip_wait_ms"),
        (|c| c.g27.settle_ms = 0, "g27.settle_ms"),
        (|c| c.g27.text_wait_ms = 0, "g27.text_wait_ms"),
        (
            |c| c.g27.text_wait_ms = c.mouse_guard.limit_ms + 1,
            "g27.text_wait_ms",
        ),
        (
            |c| c.g27.chip_wait_ms = c.mouse_guard.limit_ms + 1,
            "g27.chip_wait_ms",
        ),
        (|c| c.g27.max_chars = 0, "g27.max_chars"),
        (
            |c| c.g27.max_chars = c.g1.max_read_chars + 1,
            "g27.max_chars",
        ),
    ]);
}

#[test]
fn g8_predict_values_out_of_range_are_refused() {
    refused(&[
        (|c| c.g8_predict.owner_mb = 0.0, "g8_predict.owner_mb"),
        (|c| c.g8_predict.owner_mb = -1.0, "g8_predict.owner_mb"),
        (|c| c.g8_predict.owner_mb = f64::NAN, "g8_predict.owner_mb"),
        (
            |c| c.g8_predict.owner_mb = c.g8.memory_mb + 1.0,
            "g8_predict.owner_mb",
        ),
        (|c| c.g8_predict.warm_words = 0, "g8_predict.warm_words"),
        (|c| c.g8_predict.words = 0, "g8_predict.words"),
        (|c| c.g8_predict.words = 1_000_000, "g8_predict.words"),
        (|c| c.g8_predict.settle_ms = 0, "g8_predict.settle_ms"),
        (
            |c| c.g8_predict.settle_ms = c.mouse_guard.limit_ms + 1,
            "g8_predict.settle_ms",
        ),
    ]);
}

#[test]
fn the_lane_ids_are_the_ones_kx_ui_gives_its_chips_and_lock_note() {
    let g = load().expect("loads").g27;
    assert!(
        NAMES_QML.contains(&format!("group: \"{}\"", g.group)),
        "group"
    );
    let uid = |id: &str| format!("res.uid(names.group, \"{id}\"");
    assert!(
        LANE_QML.contains(&format!("{} + index)", uid(&g.chip))),
        "chips"
    );
    assert!(LANE_QML.contains(&format!("{})", uid(&g.locked))), "lock");
    assert_eq!(g.chip_id(0), "words.chip-0");
    assert_eq!(g.chip_start(), "words.chip-");
    assert_eq!(g.locked_id(), "words.locked");
}

#[test]
fn a_case_is_found_by_its_language() {
    let g = load().expect("loads").g27;
    assert_eq!(g.case("fr").map(|k| k.prefix.as_str()), Some("eco"));
    assert!(g.case("xx").is_none());
}

#[test]
fn a_backup_outside_the_out_folder_is_refused() {
    refused(&[
        (|c| c.owner_settings.backup.clear(), "owner_settings.backup"),
        (
            |c| c.owner_settings.backup = "a/b.json".into(),
            "owner_settings.backup",
        ),
        (
            |c| c.owner_settings.backup = "..".into(),
            "owner_settings.backup",
        ),
        (
            |c| c.owner_settings.backup = "C:b.json".into(),
            "owner_settings.backup",
        ),
    ]);
}

#[test]
fn a_backup_named_as_a_file_a_gate_deletes_or_appends_to_is_refused() {
    refused(&[
        (
            |c| c.owner_settings.backup = crate::out::RESULTS.into(),
            "owner_settings.backup",
        ),
        (
            |c| c.owner_settings.backup = "g27-keyxtend-1.json".into(),
            "owner_settings.backup",
        ),
        (
            |c| c.owner_settings.backup = "G8-predict-2.html".into(),
            "owner_settings.backup",
        ),
    ]);
    let log = "target-window.log";
    let e = backup_free("Target-Window.log", &[log]).expect_err("the log's name");
    assert!(e.contains("owner_settings.backup"), "{e}");
    assert_eq!(backup_free("owner-settings-backup.json", &[log]), Ok(()));
}

#[test]
fn the_typed_box_needs_its_focus_class() {
    let target = crate::apps::AppKind::Target.name();
    let mut c = load().expect("loads");
    c.apps.get_mut(target).expect("target").focus_class = None;
    let e = check(&c).expect_err("no focus class");
    assert!(e.contains("focus_class"), "{e}");
}

#[test]
fn each_run_length_counts_every_copy_and_every_word() {
    let c = load().expect("loads");
    let (g, t) = (&c.g27, &c.g8);
    let end = u64::from(t.quit_tries) * t.quit_try_ms + 2 * c.timing.quit_wait_ms;
    let copy = t.start_wait_ms + g.ready_ms + end;
    let keys = |lang: &str| g.case(lang).map_or(0, |k| k.prefix.chars().count() as u64);
    let word = |lang| {
        g.switch_wait_ms + keys(lang) * g.key_gap_ms + g.chip_wait_ms + g.settle_ms + g.text_wait_ms
    };
    let typed = keys(&c.g21_product.case) * g.key_gap_ms;
    let boxed = g.switch_wait_ms + typed + g.chip_wait_ms;
    assert_eq!(runs::g21_ms(&c), copy + 3 * boxed);
    let cases: u64 = g.cases.iter().map(|k| word(&k.lang)).sum();
    assert_eq!(runs::g27_ms(&c), 2 * copy + cases + word(&g.edge));
    let p = &c.g8_predict;
    let longest = g.cases.iter().map(|k| word(&k.lang)).max().expect("cases");
    let copy_warm = t.start_wait_ms + t.warm_ms + end;
    let words = (p.warm_words + p.words) as u64 * longest;
    assert_eq!(
        runs::predict_ms(&c),
        2 * copy_warm + words + 2 * p.settle_ms,
        "a settle before each of the two readings"
    );
}

#[test]
fn g21_and_g27_runs_past_the_mouse_guard_are_refused() {
    let good = load().expect("loads");
    let head = good.fit.guard_headroom_ms;
    let (g21, g27) = (runs::g21_ms(&good) + head, runs::g27_ms(&good) + head);
    assert!(g27 > g21, "G27 starts two copies and types every case");
    let mut c = good.clone();
    c.mouse_guard.limit_ms = g21 - 1;
    let e = check(&c).expect_err("G21 runs too long");
    assert!(e.contains("g21_product"), "{e}");
    c.mouse_guard.limit_ms = g21;
    let e = check(&c).expect_err("G27 runs too long");
    assert!(e.contains("g27.cases"), "{e}");
}

#[test]
fn with_the_guard_off_no_p7_run_is_refused_for_its_length() {
    let mut c = load().expect("loads");
    let longest = [runs::g21_ms(&c), runs::g27_ms(&c), runs::predict_ms(&c)];
    let waits = [c.g27.ready_ms, c.g27.text_wait_ms, c.g8_predict.settle_ms];
    c.mouse_guard.limit_ms = waits.into_iter().max().expect("waits");
    assert!(longest.iter().all(|&ms| ms > c.mouse_guard.limit_ms));
    assert!(check(&c).is_err(), "the guard on cuts them short");
    c.mouse_guard.on = false;
    assert_eq!(check(&c), Ok(()));
}

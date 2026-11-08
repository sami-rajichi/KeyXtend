//! Tests of `p10cfg`.

mod bars;
mod ids;
mod module;

use super::*;
use crate::config::load;

/// The shipped file, whose every key the tests remove in turn.
const SHIPPED: &str = include_str!("../../kx-gates-p10.toml");

/// A setting spoiled, and the key its error must name.
type Spoil = (fn(&mut G19Product), &'static str);

/// Runs each spoil on a fresh copy of the shipped settings and checks its error names its key.
fn refused(spoils: &[Spoil]) {
    let good = load().expect("loads");
    for (i, (spoil, key)) in spoils.iter().enumerate() {
        let mut c = good.clone();
        spoil(&mut c.g19_product);
        let e = check(&c).expect_err(&format!("case {i} passed"));
        assert!(e.contains(key), "case {i}: {e}");
    }
}

#[test]
fn the_shipped_settings_load_and_pass() {
    let c = load().expect("loads");
    assert_eq!(check(&c), Ok(()));
}

#[test]
fn every_key_in_the_file_is_read_and_a_missing_one_is_an_error_naming_it() {
    let file: toml::Table = SHIPPED.parse().expect("parses");
    assert_eq!(file.len(), 1, "only [g19_product]");
    let section = file["g19_product"].as_table().expect("a section").clone();
    let read = |t: toml::Table| {
        toml::Value::Table(t)
            .try_into::<G19Product>()
            .map(drop)
            .map_err(|e| e.to_string())
    };
    assert_eq!(read(section.clone()), Ok(()));
    for key in section.keys() {
        let mut less = section.clone();
        less.remove(key);
        let e = read(less).expect_err(key);
        assert!(e.contains(key.as_str()), "{key}: {e}");
    }
    let mut more = section;
    more.insert("stray".into(), toml::Value::Integer(1));
    assert!(read(more).expect_err("unknown").contains("stray"));
}

#[test]
fn the_long_arabic_line_is_long_enough_and_travels_whole() {
    let c = load().expect("loads");
    let g = &c.g19_product;
    let long = g.long_line();
    let chars = long.chars().count();
    let most = usize::from(selection_defaults().expect("defaults").translate_chars);
    assert!((g.long_min_chars..=most).contains(&chars), "{chars}");
    assert!(long.starts_with(&g.arabic) && long.ends_with(&g.arabic));
    refused(&[
        (|g| g.arabic_repeat = 1, "g19_product.arabic_repeat"),
        (|g| g.arabic_repeat = 100, "g19_product.arabic_repeat"),
        (|g| g.long_min_chars = 0, "g19_product.long_min_chars"),
        (
            |g| g.arabic = "Plain Latin text.".into(),
            "g19_product.arabic",
        ),
        (|g| g.arabic.clear(), "g19_product.arabic"),
    ]);
}

#[test]
fn the_arabic_block_is_unicodes_and_holds_the_shipped_line() {
    assert_eq!(
        (*ARABIC.start(), *ARABIC.end()),
        ('\u{0600}', '\u{06FF}'),
        "Unicode's Arabic block"
    );
    let g = load().expect("loads").g19_product;
    let letters = g.arabic.chars().filter(|c| c.is_alphabetic());
    assert!(letters.clone().count() > 0);
    for c in letters {
        assert!(ARABIC.contains(&c), "{c}");
    }
    assert!(!ARABIC.contains(&'a') && ARABIC.contains(&'ض'));
}

#[test]
fn an_empty_or_shared_test_line_is_refused() {
    refused(&[
        (|g| g.copy_line.clear(), "g19_product.copy_line"),
        (
            |g| g.cut_line.clone_from(&g.copy_line),
            "g19_product.cut_line",
        ),
        (|g| g.paste_line.clear(), "g19_product.paste_line"),
        (|g| g.term_line.clear(), "g19_product.term_line"),
        (
            |g| g.page_text.clone_from(&g.box_text),
            "g19_product.page_text",
        ),
        (|g| g.copy_line.push('\n'), "g19_product.copy_line"),
        (|g| g.box_text.push('<'), "g19_product.box_text"),
    ]);
}

#[test]
fn page_names_and_marks_the_gate_could_not_tell_apart_are_refused() {
    refused(&[
        (|g| g.box_name.clear(), "g19_product.box_name"),
        (
            |g| g.pass_name.clone_from(&g.box_name),
            "g19_product.pass_name",
        ),
        (|g| g.unknown_role.clear(), "g19_product.unknown_role"),
        (
            |g| g.pass_value = "kx\"gate".into(),
            "g19_product.pass_value",
        ),
        (|g| g.copy_mark.clear(), "g19_product.copy_mark"),
        (
            |g| g.shift_mark.clone_from(&g.copy_mark),
            "g19_product.shift_mark",
        ),
        (|g| g.shift_mark = " 1".into(), "g19_product.shift_mark"),
        (|g| g.copy_code = "Key'C".into(), "g19_product.copy_code"),
        (|g| g.ping_host = "a b".into(), "g19_product.ping_host"),
    ]);
}

#[test]
fn chips_the_strip_cannot_show_or_lists_out_of_its_order_are_refused() {
    refused(&[
        (|g| drop(g.chips.pop()), "g19_product.chips"),
        (|g| g.chips[0] = "copied".into(), "g19_product.chips"),
        (|g| g.editable_chips.clear(), "g19_product.editable_chips"),
        (|g| g.page_chips.reverse(), "g19_product.page_chips"),
        (
            |g| g.terminal_chips.push("cut".into()),
            "g19_product.terminal_chips",
        ),
        (
            |g| g.caret_chips = vec!["find".into()],
            "g19_product.caret_chips",
        ),
        (
            |g| g.strip_title = "sel.no-such".into(),
            "g19_product.strip_title",
        ),
        (|g| g.group = "sel.x".into(), "g19_product.group"),
    ]);
}

#[test]
fn keys_browsers_and_the_console_that_could_reach_another_program_are_refused() {
    refused(&[
        (|g| g.numlock = vec![0x91], "g19_product.numlock"),
        (|g| g.select_home.clear(), "g19_product.select_home"),
        (|g| g.select_end.clear(), "g19_product.select_end"),
        (|g| g.caret_left.clear(), "g19_product.caret_left"),
        (|g| g.tray.clear(), "g19_product.tray"),
        (|g| g.tray = "bar".into(), "g19_product.tray"),
        (|g| g.browsers.clear(), "g19_product.browsers"),
        (
            |g| g.browsers[0] = r"C:\x\chrome.exe".into(),
            "g19_product.browsers",
        ),
        (|g| g.console.exe.clear(), "g19_product.console"),
        (|g| g.console.class.clear(), "g19_product.console"),
        (|g| g.console.args.clear(), "g19_product.console"),
    ]);
}

#[test]
fn a_wait_or_slack_past_its_range_is_refused() {
    refused(&[
        (|g| g.ready_ms = 0, "g19_product.ready_ms"),
        (|g| g.strip_wait_ms = u64::MAX, "g19_product.strip_wait_ms"),
        (|g| g.quiet_ms = 0, "g19_product.quiet_ms"),
        (|g| g.owner_wait_ms = 0, "g19_product.owner_wait_ms"),
        (|g| g.hang_ms = u64::MAX, "g19_product.hang_ms"),
        (|g| g.hang_ms = 1, "g19_product.hang_ms"),
        (|g| g.hang_lead_ms = 0, "g19_product.hang_lead_ms"),
        (|g| g.hang_lead_ms = g.hang_ms, "g19_product.hang_ms"),
        (|g| g.place_slack_px = -1, "g19_product.place_slack_px"),
        (|g| g.caret_slack_ms = 0, "g19_product.caret_slack_ms"),
        (|g| g.hung_wait_ms = 0, "g19_product.hung_wait_ms"),
        (|g| g.hung_quiet_ms = 1, "g19_product.hung_quiet_ms"),
        (|g| g.bar_depth = 0, "g19_product.bar_depth"),
        (|g| g.box_size = 0, "g19_product.box_size"),
        (|g| g.ping_exit_ms = 0, "g19_product.ping_exit_ms"),
        (|g| g.ping_exit_ms = u64::MAX, "g19_product.ping_exit_ms"),
    ]);
}

#[test]
fn the_hung_cases_must_all_fall_within_the_hang() {
    let c = load().expect("loads");
    let g = &c.g19_product;
    let press = c.sim.step_ms + c.sim.click_ms;
    let acts =
        g.hang_lead_ms + 2 * front_ms(&c) + g.hung_quiet_ms + press + g.settle_ms + g.hung_wait_ms;
    assert_eq!(
        g.hung_acts_ms(front_ms(&c), press_ms(&c)),
        acts,
        "one click, then keys with no pause"
    );
    assert!(
        g.hang_ms > acts + g.hang_margin_ms,
        "the shipped hang holds every act and its margin"
    );
    let mut tight = c.clone();
    tight.g19_product.hang_ms = acts + g.hang_margin_ms;
    let e = check(&tight).expect_err("no margin");
    assert!(
        e.contains("g19_product.hang_ms") && e.contains("hang_margin_ms"),
        "{e}"
    );
    refused(&[
        (|g| g.hung_wait_ms += g.hang_ms, "g19_product.hang_ms"),
        (|g| g.hang_margin_ms = g.hang_ms, "g19_product.hang_ms"),
        (|g| g.hang_margin_ms = 0, "g19_product.hang_margin_ms"),
    ]);
    let cap = kx_target_window::config::load()
        .expect("target config loads")
        .hang_max_ms;
    let mut past = c.clone();
    past.g19_product.hang_ms = cap + 1;
    let e = check(&past).expect_err("past the cap");
    assert!(e.contains("hang_max_ms"), "{e}");
}

#[test]
fn a_ping_that_could_run_another_program_is_refused() {
    refused(&[
        (|g| g.ping_exe.clear(), "g19_product.ping_exe"),
        (
            |g| g.ping_exe = r"C:\x\ping.exe".into(),
            "g19_product.ping_exe",
        ),
        (|g| g.ping_args.clear(), "g19_product.ping_args"),
        (
            |g| g.ping_args = vec!["-t;calc".into()],
            "g19_product.ping_args",
        ),
        (|g| g.corner = "corner".into(), "g19_product.corner"),
    ]);
}

#[test]
fn todays_g19_fits_the_mouse_guard_and_its_waits_count() {
    let c = load().expect("loads");
    assert_eq!(held_fits(&c, "g19", g19_ms(&c)), Ok(()));
    let more = |spoil: fn(&mut GatesConfig)| {
        let mut s = c.clone();
        spoil(&mut s);
        g19_ms(&s) - g19_ms(&c)
    };
    assert_eq!(more(|s| s.g19_product.quiet_ms += 1), times(QUIET, 1));
    assert_eq!(
        more(|s| s.g19_product.hung_quiet_ms += 1),
        0,
        "within the hang"
    );
    assert_eq!(more(|s| s.g19_product.hang_ms += 1), 1);
    assert_eq!(
        more(|s| s.g19_product.ping_exit_ms += 1),
        0,
        "a longest wait"
    );
    assert_eq!(more(|s| s.g19_product.ping_ms += 1), 1);
    assert_eq!(more(|s| s.g19_product.ready_ms += 1), 1);
    assert_eq!(more(|s| s.g19_product.settle_ms += 1), times(ACTS, 1));
}

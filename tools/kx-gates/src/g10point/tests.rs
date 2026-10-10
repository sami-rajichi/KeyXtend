//! Tests of G10's check by point: where each key is aimed, the match rule, and the result block.

use windows::Win32::Foundation::RECT;

use super::*;

/// The keyboard's process in these tests.
const KB: u32 = 7;
/// Another app's process.
const OTHER: u32 = 9;

fn bx(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
    RECT {
        left,
        top,
        right,
        bottom,
    }
}

/// A button the walk read, with id `id` and box `rect`.
fn button(id: &str, rect: RECT) -> KeyRead {
    KeyRead {
        id: id.into(),
        name: id.into(),
        rect,
        invokable: true,
    }
}

/// An element UI Automation found in process `pid`, as `read_hit` keeps it: id `id` and name `name` only when the
/// keyboard shows it.
fn hit(id: &str, name: &str, pid: u32) -> Hit {
    let label = || {
        Ok(Label {
            id: id.into(),
            name: name.into(),
        })
    };
    Hit {
        label: own(pid, KB, label).expect("a label"),
        kind: "QQuickItem / type 50000".into(),
        pid,
        hwnd: 0,
    }
}

/// Key `key`, aimed at (10, 20), where UI Automation found `got`; the keyboard's window is under the point.
fn probe_of(key: &str, got: Result<Hit, String>) -> Probe {
    Probe {
        key: key.into(),
        at: Some(POINT { x: 10, y: 20 }),
        hit: got,
        under: Some(Under {
            hwnd: 42,
            pid: KB,
            window: "[Qt6110QWindowIcon] of keyxtend.exe".into(),
        }),
    }
}

/// Key `key` with no box on screen, so nothing was hit-tested.
fn no_box(key: &str) -> Probe {
    Probe {
        key: key.into(),
        at: None,
        hit: Err(NO_BOX.into()),
        under: None,
    }
}

#[test]
fn each_key_found_by_id_is_aimed_at_the_centre_of_its_box() {
    let read = [
        button("bar.grip", bx(0, 0, 400, 24)),
        button("esc", bx(100, 200, 148, 248)),
        button("lang", bx(5, 5, 5, 30)),
    ];
    let ids = ["esc".to_string(), "lang".to_string(), "tab".to_string()];
    let got = aims(&read, &ids);
    assert_eq!(
        got,
        [
            ("esc".to_string(), Some(POINT { x: 124, y: 224 })),
            ("lang".to_string(), None),
        ],
        "a box with no area has no centre; a key the walk missed is its own finding; a bar button is no board key"
    );
}

#[test]
fn only_the_keyboards_own_elements_have_their_id_and_name_read() {
    let ours = own(KB, KB, || Ok("esc"));
    assert_eq!(ours, Ok(Some("esc")));
    let theirs = own(OTHER, KB, || -> Result<&str, String> {
        panic!("another app's label is read")
    });
    assert_eq!(theirs, Ok(None), "it may hold the owner's text");
    assert_eq!(
        own(KB, KB, || Err::<(), _>("UIA id".to_string())),
        Err("UIA id".into())
    );
}

#[test]
fn a_key_matches_only_when_the_hit_is_that_key_in_the_keyboards_process() {
    let judge_of = |h: Option<&Hit>| judge("esc", KB, h);
    assert_eq!(judge_of(Some(&hit("esc", "Escape", KB))), Outcome::Matched);
    assert_eq!(
        judge_of(Some(&hit("tab", "Tab", KB))),
        Outcome::WrongElement,
        "another key"
    );
    assert_eq!(
        judge_of(Some(&hit("", "KeyXtend", KB))),
        Outcome::WrongElement,
        "the keyboard's window itself"
    );
    assert_eq!(
        judge_of(Some(&hit("esc", "Escape", OTHER))),
        Outcome::WrongWindow,
        "the same id in another app is not our key"
    );
    assert_eq!(judge_of(None), Outcome::Unread);
}

#[test]
fn the_block_counts_each_outcome_and_lists_only_the_first_failures() {
    let probes = [
        probe_of("esc", Ok(hit("esc", "Escape", KB))),
        probe_of("tab", Ok(hit("", "KeyXtend", KB))),
        probe_of("caps", Ok(hit("doc", "Report.docx", OTHER))),
        no_box("enter"),
        probe_of("lang", Err("UI Automation: access denied".into())),
    ];
    let v = tally(&probes, KB, 2);
    let counts = [
        ("keys", 5),
        ("matched", 1),
        ("wrong_element", 1),
        ("wrong_window", 1),
        ("unread", 2),
    ];
    for (field, n) in counts {
        assert_eq!(v[field], n, "{field}");
    }
    let failures = v["failures"].as_array().expect("a list");
    let keys: Vec<&str> = failures.iter().filter_map(|f| f["key"].as_str()).collect();
    assert_eq!(
        keys,
        ["tab", "caps"],
        "the first two failures, in board order"
    );
    assert_eq!(failures[0]["outcome"], "wrong_element");
    assert_eq!(v["ok"], false);
}

#[test]
fn the_block_is_ok_only_when_every_key_matched_and_some_key_was_checked() {
    let all = [
        probe_of("esc", Ok(hit("esc", "Escape", KB))),
        probe_of("tab", Ok(hit("tab", "Tab", KB))),
    ];
    let v = tally(&all, KB, 5);
    assert_eq!(
        (v["ok"].clone(), v["failures"].clone()),
        (json!(true), json!([]))
    );
    assert_eq!(
        tally(&[], KB, 5)["ok"],
        false,
        "nothing checked passes nothing"
    );
}

#[test]
fn a_failure_line_keeps_the_window_under_the_point_and_another_apps_element_keeps_no_id_or_name() {
    let other = probe_of("caps", Ok(hit("doc", "Report.docx", OTHER))).line(KB);
    assert_eq!(other["outcome"], "wrong_window");
    assert_eq!(other["at"], json!([10, 20]));
    assert_eq!(other["hit"]["pid"], OTHER);
    assert_eq!(other["hit"]["kind"], "QQuickItem / type 50000");
    assert!(
        other["hit"]["id"].is_null() && other["hit"]["name"].is_null(),
        "they may hold the owner's text"
    );
    let kept = probe_of("caps", Ok(hit("doc", "Report.docx", OTHER))).hit;
    assert!(
        !format!("{kept:?}").contains("Report"),
        "nor can a debug print show them"
    );
    assert_eq!(other["under"]["hwnd"], 42);
    assert_eq!(other["under"]["pid"], KB);
    assert_eq!(
        other["under"]["window"],
        "[Qt6110QWindowIcon] of keyxtend.exe"
    );
    let ours = probe_of("tab", Ok(hit("shift", "Shift", KB))).line(KB);
    assert_eq!(ours["hit"]["id"], "shift");
    assert_eq!(ours["hit"]["name"], "Shift", "the keyboard's own label");
}

#[test]
fn a_key_that_could_not_be_hit_tested_says_why() {
    let boxless = no_box("enter").line(KB);
    assert_eq!(boxless["outcome"], "unread");
    assert!(boxless["at"].is_null() && boxless["hit"].is_null() && boxless["under"].is_null());
    assert_eq!(boxless["error"], NO_BOX);
    let denied = probe_of("lang", Err("UI Automation: access denied".into())).line(KB);
    assert_eq!(denied["error"], "UI Automation: access denied");
    assert_eq!(
        denied["under"]["pid"], KB,
        "the window under the point still reads"
    );
}

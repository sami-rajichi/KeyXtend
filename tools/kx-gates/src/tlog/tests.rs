//! Tests of `tlog`, on fixtures of target-window's log lines.

use super::*;

#[test]
fn keeps_only_units_after_the_last_start() {
    let log = "# start\t1\n10\t0041\n# start\t20\n30\t0644\n# focus-lost\t30\n31\t0627\n";
    let units = parse(log);
    assert_eq!(
        units,
        vec![
            Unit {
                us: 30,
                unit: 0x0644
            },
            Unit {
                us: 31,
                unit: 0x0627
            }
        ]
    );
    assert_eq!(decode(&units), "لا");
}

#[test]
fn decodes_a_surrogate_pair_and_skips_bad_lines() {
    let log = "# start\t0\n5\tD83D\nnot a line\n6\tDE00\r\n7\t20AC\n8\tZZZZ\n";
    let units = parse(log);
    assert_eq!(units.len(), 3);
    assert_eq!(decode(&units), "\u{1F600}€");
}

#[test]
fn reads_mouse_presses_after_the_last_start() {
    let log = "# start\t0\n# mouse\tldown\t5\t1\t2\n# start\t10\n\
               # mouse\trdown\t20\t-4\t8\n30\t0041\n# mouse\tmenu\t21\t-4\t8\r\n\
               # mouse\tzap\t22\t0\t0\n# mouse\tlup\t23\t1\n";
    let got = mouse(log);
    let at = |press, us, x, y| Mouse { press, us, x, y };
    assert_eq!(
        got,
        vec![at(Press::RightDown, 20, -4, 8), at(Press::Menu, 21, -4, 8)]
    );
}

#[test]
fn counts_focus_losses_after_the_last_start_from_a_time() {
    let log = "# start\t0\n# focus-lost\t50\n# start\t100\n# focus-lost\t150\n\
               200\t0041\n# focus-lost\t300\r\n# focus-lost\n";
    assert_eq!(focus_lost(log, 200), 2);
    assert_eq!(focus_lost(log, 0), 3);
    assert_eq!(focus_lost("# start\t0\n10\t0041\n", 0), 0);
}

#[test]
fn reads_key_presses_after_the_last_start() {
    let log = "# start\t0\n# key\t5\t0D\t001C\n# start\t10\n\
               # key\t20\t26\tE048\r\n30\t0041\n# key\t21\tZZ\t0001\n# key\t22\t1B\n";
    let at = |us, vk, scan| KeyDown { us, vk, scan };
    assert_eq!(keys(log), vec![at(20, 0x26, 0xE048)]);
}

#[test]
fn reads_wheel_turns_after_the_last_start() {
    let log = "# start\t0\n# wheel\t5\tv\t-40\t1\t2\n# start\t10\n\
               # wheel\t20\tv\t-40\t-4\t8\n30\t0041\n# wheel\t21\th\t120\t7\t9\r\n\
               # wheel\t22\tz\t1\t0\t0\n# wheel\t23\tv\t1\n";
    let at = |us, sideways, delta, x, y| Wheel {
        us,
        sideways,
        delta,
        x,
        y,
    };
    assert_eq!(
        wheels(log),
        vec![at(20, false, -40, -4, 8), at(21, true, 120, 7, 9)]
    );
}

#[test]
fn reads_a_marks_times_after_the_last_start() {
    let log =
        "# start\t0\n# woke\t5\n# start\t10\n# hung\t20\n# woke\t30\r\n# woke\tx\n# woke\t40\n";
    assert_eq!(marks(log, "# woke"), [30, 40]);
    assert_eq!(marks(log, "# hung"), [20]);
    assert!(marks(log, "# focus-lost").is_empty());
}

#[test]
fn a_layout_kept_at_the_last_close_is_seen() {
    assert!(layout_kept("# start\t0\n10\t0041\n# layout-kept\t20\r\n"));
    let earlier = "# start\t0\n# layout-kept\t20\n# start\t30\n40\t0041\n";
    assert!(
        !layout_kept(earlier),
        "a run before the last start is not this one"
    );
    assert!(!layout_kept(""));
}

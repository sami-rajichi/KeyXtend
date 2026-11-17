//! The session table parses, each change message maps to its note, and a change told twice is told once.

use super::*;

/// `WM_WTSSESSION_CHANGE` and `WM_POWERBROADCAST`, from WinUser.h, as the table lists them.
const SESSION: u32 = 0x2B1;
const POWER: u32 = 0x218;

#[test]
fn each_change_message_maps_to_its_note_and_others_to_none() {
    let table = table().expect("the shipped table parses");
    let cases = [
        (SESSION, 7, Some(SessionNote::Locked)),
        (SESSION, 8, Some(SessionNote::Unlocked)),
        (POWER, 4, Some(SessionNote::Sleeping)),
        (POWER, 18, Some(SessionNote::Woke)),
        (POWER, 7, Some(SessionNote::Woke)),
        (SESSION, 1, None),
        (POWER, 10, None),
        (0x10, 7, None),
    ];
    for (msg, wparam, note) in cases {
        assert_eq!(table.note(msg, wparam), note, "{msg:#x} {wparam}");
    }
    assert!(table.names(POWER) && table.names(SESSION) && !table.names(0x10));
}

#[test]
fn a_change_told_twice_in_a_row_is_told_once() {
    let mut last = Last::default();
    let told: Vec<bool> = [
        SessionNote::Sleeping,
        SessionNote::Woke,
        SessionNote::Woke,
        SessionNote::Locked,
    ]
    .into_iter()
    .map(|n| last.fresh(n))
    .collect();
    assert_eq!(told, [true, true, false, true]);
}

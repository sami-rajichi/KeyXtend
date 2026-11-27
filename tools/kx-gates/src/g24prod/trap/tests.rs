//! Tests of `trap`.

use kx_platform_windows::keys::{SIM_TAG, TAG, is_replay};
use windows::Win32::UI::Input::KeyboardAndMouse::{INPUT_KEYBOARD, KEYEVENTF_KEYUP};

use super::*;

/// A key input's key, whether it lets go, and its mark.
type Key = (u16, bool, usize);

/// What key input `i` is.
#[allow(
    unsafe_code,
    reason = "Reads the keyboard member the input's type names."
)]
fn key_of(i: &INPUT) -> Key {
    assert_eq!(i.r#type, INPUT_KEYBOARD);
    // SAFETY: the type says the keyboard member is set.
    let ki = unsafe { i.Anonymous.ki };
    (
        ki.wVk.0,
        ki.dwFlags.contains(KEYEVENTF_KEYUP),
        ki.dwExtraInfo,
    )
}

/// What `press_esc` sends and says when each batch in turn gets the answer `answers` holds.
fn sent_with(answers: &[bool]) -> (Result<(), String>, Vec<Vec<Key>>) {
    let mut batches = Vec::new();
    let got = press_esc(|batch| {
        batches.push(batch.iter().map(key_of).collect::<Vec<_>>());
        let n = batches.len();
        if answers[n - 1] {
            Ok(())
        } else {
            Err(format!("refused {n}"))
        }
    });
    (got, batches)
}

#[test]
fn another_programs_esc_is_one_unmarked_press_and_release_never_ours() {
    let keys = esc_keys().map(|i| key_of(&i));
    let ours = |x: usize| x == TAG || x == SIM_TAG || is_replay(x);
    assert!(
        keys.iter()
            .all(|&(vk, _, mark)| vk == VK_ESCAPE.0 && !ours(mark))
    );
    assert!(keys.iter().all(|&(_, _, mark)| mark == UNMARKED));
    let ups = keys.map(|(_, up, _)| up);
    assert_eq!(ups, [false, true], "a press, then its release");
}

#[test]
fn a_send_windows_refuses_part_of_lets_esc_go_alone_and_unmarked() {
    let up = (VK_ESCAPE.0, true, UNMARKED);
    let (got, batches) = sent_with(&[true]);
    assert_eq!(got, Ok(()));
    assert_eq!(batches, [vec![(VK_ESCAPE.0, false, UNMARKED), up]]);
    let (got, batches) = sent_with(&[false, true]);
    assert_eq!(got, Err("refused 1".to_string()), "the first error stays");
    assert_eq!(batches[1], [up], "a lone release follows");
    let (got, batches) = sent_with(&[false, false]);
    assert_eq!(batches.len(), 2, "one release, no more");
    let why = got.expect_err("both refused");
    assert!(why.contains("refused 1") && why.contains("refused 2"));
}

#[test]
fn the_trap_taps_the_bubble_so_only_the_bubble_way_takes_it() {
    assert!(taps(SnipKeyboard::Bubble));
    assert!(!taps(SnipKeyboard::Hidden) && !taps(SnipKeyboard::Shown));
}

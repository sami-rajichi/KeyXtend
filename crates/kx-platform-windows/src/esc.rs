//! The Esc watch (ADR-0059): a low-level keyboard hook, on only while Snip's frozen screens show, hears each Esc press
//! we did not send, the hand's or an assistive program's, and passes every key on. Which hook events are a press is
//! pure and tested everywhere; the hook is in `win`.
#![cfg_attr(
    not(windows),
    allow(
        dead_code,
        reason = "Only the Windows hook runs this rule; tests cover it everywhere."
    )
)]

use crate::hook::{KEY_DOWNS, KEY_UPS, VK_ESCAPE, ours};

#[cfg(windows)]
mod win;

#[cfg(windows)]
pub use win::WinEscWatch;

/// Whether hook message `msg` for key `vk`, marked `extra`, is a new press of an Esc we did not send; `held` keeps
/// whether it is down, so a held key's repeats count once. Another program's Esc counts as the hand's, as Voice Access
/// or another on-screen keyboard sends it; our own input (`ours`) and every other key are passed over unread.
pub(crate) fn pressed(msg: u32, vk: u32, extra: usize, held: &mut bool) -> bool {
    if ours(extra) || vk != VK_ESCAPE {
        return false;
    }
    let down = KEY_DOWNS.contains(&msg);
    let fresh = down && !*held;
    if down || KEY_UPS.contains(&msg) {
        *held = down;
    }
    fresh
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::{REPLAY, SIM_TAG, TAG};

    /// `WM_KEYDOWN`, `WM_KEYUP` and `WM_SYSKEYDOWN`.
    const DOWN: u32 = 0x0100;
    const UP: u32 = 0x0101;
    const SYS_DOWN: u32 = 0x0104;
    /// The A key.
    const VK_A: u32 = 0x41;

    /// Marks of the hand's input, of another program's (Voice Access), and of ours: our keyboard's and a replay's.
    const HAND: usize = 0;
    const OTHER: usize = 0x1234;
    const REPLAYED: usize = REPLAY | 7;

    /// Which of `events` (message, key, marks) count as presses, in order.
    fn presses(events: &[(u32, u32, usize)]) -> Vec<bool> {
        let mut held = false;
        let mut hear = |&(msg, vk, extra): &(u32, u32, usize)| pressed(msg, vk, extra, &mut held);
        events.iter().map(&mut hear).collect()
    }

    #[test]
    fn a_press_counts_once_however_long_it_is_held_and_again_after_it_is_let_go() {
        let esc = |msg| (msg, VK_ESCAPE, HAND);
        let heard = presses(&[esc(DOWN), esc(DOWN), esc(DOWN), esc(UP), esc(SYS_DOWN)]);
        assert_eq!(heard, [true, false, false, false, true]);
    }

    #[test]
    fn our_own_input_and_every_other_key_never_count() {
        let heard = presses(&[
            (DOWN, VK_ESCAPE, TAG),
            (DOWN, VK_ESCAPE, REPLAYED),
            (DOWN, VK_A, HAND),
            (UP, VK_A, HAND),
            (DOWN, VK_ESCAPE, HAND),
        ]);
        assert_eq!(heard, [false, false, false, false, true]);
    }

    #[test]
    fn another_programs_esc_counts_as_the_hands() {
        let heard = presses(&[
            (DOWN, VK_ESCAPE, OTHER),
            (UP, VK_ESCAPE, OTHER),
            (DOWN, VK_ESCAPE, SIM_TAG),
        ]);
        assert_eq!(
            heard,
            [true, false, true],
            "Voice Access, then the gate tool"
        );
    }

    #[test]
    fn our_release_leaves_the_hands_held_key_held() {
        let heard = presses(&[
            (DOWN, VK_ESCAPE, HAND),
            (UP, VK_ESCAPE, TAG),
            (DOWN, VK_ESCAPE, HAND),
        ]);
        assert_eq!(heard, [true, false, false], "still the same press");
    }
}

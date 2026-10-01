//! Our windows' guard: they stay on top and never take the focus, and clicks may pass through them.
//! Which styles the guard keeps and in what order it guards are pure and tested everywhere; the OS calls are in `win`.
//! Every window it covers must start topmost (Qt's `WindowStaysOnTopHint`), as guarding a plain hidden window can leave
//! it below the topmost band; Settings is spared.
#![cfg_attr(
    not(windows),
    allow(
        dead_code,
        reason = "The style and order rules serve only the Windows guard in win."
    )
)]

#[cfg(windows)]
mod guard;
#[cfg(test)]
mod tests;
#[cfg(windows)]
mod win;

#[cfg(windows)]
pub use win::{
    WinGuard, foreground, from_raw, give_back, guard_own, is_ours, own_bounds, own_titled,
    own_windows, shape, title,
};

/// `WS_EX_TOPMOST`: above every window that is not topmost.
const WS_EX_TOPMOST: u32 = 0x0000_0008;
/// `WS_EX_TRANSPARENT`: clicks go to the window below.
const WS_EX_TRANSPARENT: u32 = 0x0000_0020;
/// `WS_EX_TOOLWINDOW`: no taskbar button and no Alt+Tab entry.
const WS_EX_TOOLWINDOW: u32 = 0x0000_0080;
/// `WS_EX_LAYERED`: needed with `WS_EX_TRANSPARENT` for clicks to pass through.
const WS_EX_LAYERED: u32 = 0x0008_0000;
/// `WS_EX_NOACTIVATE`: a click never gives the window the focus.
const WS_EX_NOACTIVATE: u32 = 0x0800_0000;

/// Extended styles every guarded window keeps: no focus, on top, no taskbar button.
pub const KEPT: u32 = WS_EX_NOACTIVATE | WS_EX_TOPMOST | WS_EX_TOOLWINDOW;
/// Extended styles that let clicks pass through a window; a guarded window keeps them once it has both.
pub const THROUGH: u32 = WS_EX_LAYERED | WS_EX_TRANSPARENT;

/// The extended style a guarded window may take when it asks for `new` over `old`.
#[must_use]
pub fn kept_ex(old: u32, new: u32) -> u32 {
    let through = if old & THROUGH == THROUGH { THROUGH } else { 0 };
    new | KEPT | through
}

/// Windows listed front first, in guard order: each guard lifts its window to the top, so the front one
/// comes last. A window just shown may sit below the others; `is_top` marks it to come last of all.
#[must_use]
pub fn guard_order<T>(mut front_first: Vec<T>, is_top: impl Fn(&T) -> bool) -> Vec<T> {
    front_first.reverse();
    let (mut rest, tops): (Vec<T>, Vec<T>) = front_first.into_iter().partition(|w| !is_top(w));
    rest.extend(tops);
    rest
}

/// A window titled `title` is left alone when it is the `spare` one.
fn spared(title: &str, spare: Option<&str>) -> bool {
    spare == Some(title)
}

/// A window of process `pid` is ours when we are process `me`, unless it is the console host's window,
/// which reports our process in a console build.
fn ours_by(pid: u32, me: u32, console: bool) -> bool {
    pid == me && !console
}

/// The focus goes back to the noted window only when it still exists, one of ours is in front, and it is
/// not ours, so we never take the focus from another app.
fn should_give_back(noted: bool, ours_in_front: bool, noted_ours: bool) -> bool {
    noted && ours_in_front && !noted_ours
}

#[cfg(test)]
mod rules {
    use super::*;

    #[test]
    fn a_guarded_window_keeps_its_styles_and_a_click_through_one_stays_so() {
        assert_eq!(kept_ex(0, 0), KEPT, "other windows keep taking clicks");
        assert_eq!(kept_ex(THROUGH, 0), KEPT | THROUGH);
        assert_eq!(kept_ex(KEPT, WS_EX_LAYERED), KEPT | WS_EX_LAYERED);
        let other = 0x0000_0100;
        assert_eq!(kept_ex(0, other), other | KEPT, "other asked styles stay");
    }

    #[test]
    fn one_click_through_bit_alone_is_not_kept() {
        assert_eq!(kept_ex(WS_EX_LAYERED, 0), KEPT, "layered alone");
        assert_eq!(kept_ex(WS_EX_TRANSPARENT, 0), KEPT, "transparent alone");
    }

    #[test]
    fn the_kept_styles_are_no_focus_top_and_tool() {
        assert_eq!(KEPT, 0x0800_0088);
        assert_eq!(THROUGH, 0x0008_0020);
        assert_eq!(KEPT & THROUGH, 0, "the two sets never overlap");
    }

    #[test]
    fn guarding_goes_back_to_front_so_the_front_window_stays_in_front() {
        assert_eq!(guard_order(vec![3, 2, 1], |_| false), vec![1, 2, 3]);
        assert!(guard_order(Vec::<u8>::new(), |_| true).is_empty());
    }

    #[test]
    fn the_window_just_shown_is_guarded_last_so_it_ends_on_top() {
        assert_eq!(guard_order(vec![3, 2, 1], |&w| w == 2), vec![1, 3, 2]);
        assert_eq!(guard_order(vec![3, 2, 1], |&w| w == 1), vec![2, 3, 1]);
    }

    #[test]
    fn only_the_spare_window_is_left_alone() {
        assert!(spared("Settings", Some("Settings")));
        assert!(!spared("Keyboard", Some("Settings")));
        assert!(!spared("Settings", None), "nothing spared");
        assert!(!spared("", None));
    }

    #[test]
    fn the_console_window_and_other_programs_are_not_ours() {
        assert!(ours_by(7, 7, false));
        assert!(!ours_by(7, 7, true), "the console host's window");
        assert!(!ours_by(8, 7, false), "another process");
    }

    #[test]
    fn the_focus_goes_back_only_from_us_to_another_app() {
        assert!(should_give_back(true, true, false));
        assert!(!should_give_back(false, true, false), "no noted window");
        assert!(
            !should_give_back(true, false, false),
            "another app is in front"
        );
        assert!(
            !should_give_back(true, true, true),
            "the noted window is ours"
        );
    }
}

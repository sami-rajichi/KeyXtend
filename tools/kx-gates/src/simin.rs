//! The simulated user's input: kx-gates' own acts, sent tagged `SIM_TAG`, which only the test build's engine hears.
#![cfg(windows)]

use kx_platform_windows::keys::SIM_TAG;
use spike_core::hold::Pt;
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, MOUSE_EVENT_FLAGS, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MOVE, MOUSEEVENTF_VIRTUALDESK,
};

use crate::mouse::{self, normalise};
use crate::{keys, win};

/// One step of the simulated user's input; the simulated user only ever presses the left button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    /// Moves the pointer there.
    Move(Pt),
    /// Presses the left button there.
    Down(Pt),
    /// Releases the left button there.
    Up(Pt),
}

/// `acts` as one `SendInput` batch marked `tag`, for the virtual desktop `(left, top, width, height)`.
pub fn inputs(acts: &[Act], desktop: (i32, i32, i32, i32), tag: usize) -> Vec<INPUT> {
    let (x0, y0, w, h) = desktop;
    let at = |p: Pt, extra: MOUSE_EVENT_FLAGS| {
        let base = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK;
        let (x, y) = (normalise(p.x, x0, w), normalise(p.y, y0, h));
        mouse::input(x, y, base | extra, tag)
    };
    acts.iter()
        .map(|&act| match act {
            Act::Move(p) => at(p, MOUSE_EVENT_FLAGS(0)),
            Act::Down(p) => at(p, MOUSEEVENTF_LEFTDOWN),
            Act::Up(p) => at(p, MOUSEEVENTF_LEFTUP),
        })
        .collect()
}

/// Plays `acts` as the simulated user, through `keys::send`, so the mouse guard's rules hold.
pub fn as_user(acts: &[Act]) -> Result<(), String> {
    keys::send(&inputs(acts, win::desktop(), SIM_TAG))
}

/// True when `p` is inside `r`: its top and left edges count, its bottom and right edges do not.
pub fn inside(r: &RECT, p: Pt) -> bool {
    (r.left..r.right).contains(&p.x) && (r.top..r.bottom).contains(&p.y)
}

#[cfg(test)]
mod tests {
    use kx_platform_windows::keys::TAG;

    use super::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::INPUT_MOUSE;

    const P: Pt = Pt { x: 5, y: 6 };

    #[test]
    fn inside_includes_the_top_left_edge_only() {
        let r = RECT {
            left: 0,
            top: 0,
            right: 10,
            bottom: 10,
        };
        assert!(inside(&r, Pt { x: 0, y: 0 }));
        assert!(inside(&r, Pt { x: 9, y: 9 }));
        assert!(!inside(&r, Pt { x: 10, y: 5 }));
    }

    #[allow(unsafe_code, reason = "Reads the union variant of the checked type.")]
    #[test]
    fn acts_become_tagged_absolute_left_button_input() {
        let acts = [Act::Down(P), Act::Up(P)];
        let got = inputs(&acts, (0, 0, 11, 11), SIM_TAG);
        assert_eq!(got.len(), 2);
        assert!(got.iter().all(|i| i.r#type == INPUT_MOUSE));
        // SAFETY: both are mouse input, as checked above.
        let (down, up) = unsafe { (got[0].Anonymous.mi, got[1].Anonymous.mi) };
        let at = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK;
        assert_eq!(down.dwFlags, at | MOUSEEVENTF_LEFTDOWN);
        assert_eq!(up.dwFlags, at | MOUSEEVENTF_LEFTUP);
        assert_eq!(
            (down.dx, down.dy, down.dwExtraInfo),
            (32768, 39321, SIM_TAG)
        );
    }

    #[allow(unsafe_code, reason = "Reads the union variant of the checked type.")]
    #[test]
    fn each_act_carries_the_tag_it_is_sent_with() {
        let acts = [Act::Move(P), Act::Down(P), Act::Up(P)];
        for tag in [TAG, SIM_TAG] {
            let got = inputs(&acts, (-10, -20, 30, 40), tag);
            // SAFETY: every act is mouse input.
            let extras: Vec<usize> = got
                .iter()
                .map(|i| unsafe { i.Anonymous.mi.dwExtraInfo })
                .collect();
            assert_eq!(extras, [tag; 3], "tag {tag:#x}");
        }
    }
}

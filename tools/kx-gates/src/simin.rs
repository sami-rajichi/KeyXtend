//! The simulated user's input: kx-gates' own acts, sent tagged `SIM_TAG`, which only the test build's engine hears.
#![cfg(windows)]

use kx_platform_windows::keys::SIM_TAG;
use spike_core::hold::Pt;
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, MOUSE_EVENT_FLAGS, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_VIRTUALDESK,
};

use crate::mouse::{self, normalise};
use crate::{keys, win};

/// A mouse button the simulated user presses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    /// The main button.
    Left,
    /// The context-menu button.
    Right,
    /// The wheel button.
    Middle,
}

/// One step of the simulated user's input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    /// Moves the pointer there.
    Move(Pt),
    /// Presses a button there.
    Down(Button, Pt),
    /// Releases a button there.
    Up(Button, Pt),
    /// Presses a virtual key.
    KeyDown(u16),
    /// Releases a virtual key.
    KeyUp(u16),
}

/// The press and release flags of `button`.
fn flags(button: Button) -> (MOUSE_EVENT_FLAGS, MOUSE_EVENT_FLAGS) {
    match button {
        Button::Left => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
        Button::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
        Button::Middle => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
    }
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
            Act::Down(b, p) => at(p, flags(b).0),
            Act::Up(b, p) => at(p, flags(b).1),
            Act::KeyDown(vk) => keys::tagged(vk, false, tag),
            Act::KeyUp(vk) => keys::tagged(vk, true, tag),
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
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        INPUT_KEYBOARD, INPUT_MOUSE, KEYEVENTF_KEYUP,
    };

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
    fn acts_become_tagged_absolute_mouse_and_key_input() {
        let acts = [Act::Down(Button::Right, P), Act::KeyUp(0x10)];
        let got = inputs(&acts, (0, 0, 11, 11), SIM_TAG);
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].r#type, INPUT_MOUSE);
        // SAFETY: the union variant matches the type checked above.
        let mi = unsafe { got[0].Anonymous.mi };
        let at = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK;
        assert_eq!(mi.dwFlags, at | MOUSEEVENTF_RIGHTDOWN);
        assert_eq!((mi.dx, mi.dy, mi.dwExtraInfo), (32768, 39321, SIM_TAG));
        assert_eq!(got[1].r#type, INPUT_KEYBOARD);
        // SAFETY: the union variant matches the type checked above.
        let ki = unsafe { got[1].Anonymous.ki };
        assert_eq!((ki.wVk.0, ki.dwExtraInfo), (0x10, SIM_TAG));
        assert!(ki.dwFlags.contains(KEYEVENTF_KEYUP));
    }

    #[allow(unsafe_code, reason = "Reads the union variant of the checked type.")]
    #[test]
    fn each_act_carries_the_tag_it_is_sent_with() {
        let acts = [Act::Move(P), Act::Up(Button::Middle, P), Act::KeyDown(0x11)];
        for tag in [TAG, SIM_TAG] {
            let got = inputs(&acts, (-10, -20, 30, 40), tag);
            // SAFETY: the first two are mouse input and the last is key input, as their acts say.
            let extras = unsafe {
                [
                    got[0].Anonymous.mi.dwExtraInfo,
                    got[1].Anonymous.mi.dwExtraInfo,
                    got[2].Anonymous.ki.dwExtraInfo,
                ]
            };
            assert_eq!(extras, [tag; 3], "tag {tag:#x}");
        }
    }
}

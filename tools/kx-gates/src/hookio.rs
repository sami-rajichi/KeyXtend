//! What the assist hooks read and write: whose input counts, hook messages as engine events, acts as input.
#![cfg(windows)]

use spike_core::hold::{Act, Button, Event, Pt};
use spike_core::inject::TAG;
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, MOUSE_EVENT_FLAGS, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_VIRTUALDESK, VK_ESCAPE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE,
    WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSKEYDOWN,
};

use crate::keys;
use crate::mouse::{self, normalise};

/// Marks the simulated user's input, which automated runs act on.
pub const SIM_TAG: usize = 0x4B58_5553;

/// Whose input the engine acts on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// Only input tagged `SIM_TAG`: a hand on the mouse cannot disturb a run.
    Simulated,
    /// Only real, non-injected input: the owner's hand try.
    Physical,
}

/// True when input with these marks belongs to `source`; our own injections never do.
pub fn accept(source: Source, injected: bool, extra: usize) -> bool {
    if extra == TAG {
        return false;
    }
    match source {
        Source::Simulated => injected && extra == SIM_TAG,
        Source::Physical => !injected,
    }
}

/// The engine event for low-level mouse message `msg` at `pt`; `ours` when it lands on our window.
pub fn event_of(msg: u32, pt: Pt, ours: bool) -> Option<Event> {
    let (down, up) = (|b| Event::Down(b, pt, ours), |b| Event::Up(b, pt));
    match msg {
        WM_LBUTTONDOWN => Some(down(Button::Left)),
        WM_LBUTTONUP => Some(up(Button::Left)),
        WM_RBUTTONDOWN => Some(down(Button::Right)),
        WM_RBUTTONUP => Some(up(Button::Right)),
        WM_MBUTTONDOWN => Some(down(Button::Middle)),
        WM_MBUTTONUP => Some(up(Button::Middle)),
        WM_MOUSEMOVE => Some(Event::Move(pt)),
        _ => None,
    }
}

/// True for an Esc key press.
pub fn is_esc(msg: u32, vk: u32) -> bool {
    matches!(msg, WM_KEYDOWN | WM_SYSKEYDOWN) && vk == u32::from(VK_ESCAPE.0)
}

/// True when `p` is inside `r`.
pub fn inside(r: &RECT, p: Pt) -> bool {
    (r.left..r.right).contains(&p.x) && (r.top..r.bottom).contains(&p.y)
}

/// The input that replays mouse `event`; Esc and ticks have none.
pub fn act_of(event: Event) -> Option<Act> {
    match event {
        Event::Down(button, at, _) => Some(Act::Down(button, at)),
        Event::Up(button, at) => Some(Act::Up(button, at)),
        Event::Move(at) => Some(Act::Move(at)),
        Event::Esc | Event::Tick => None,
    }
}

/// True for mouse input, which comes back through the mouse hook.
pub fn is_mouse(act: &Act) -> bool {
    matches!(act, Act::Move(_) | Act::Down(..) | Act::Up(..))
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

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        INPUT_KEYBOARD, INPUT_MOUSE, KEYEVENTF_KEYUP,
    };
    use windows::Win32::UI::WindowsAndMessaging::{WM_KEYUP, WM_MOUSEWHEEL};

    const P: Pt = Pt { x: 5, y: 6 };

    #[test]
    fn our_own_injections_never_count() {
        for source in [Source::Simulated, Source::Physical] {
            assert!(!accept(source, true, TAG));
            assert!(!accept(source, false, TAG));
        }
    }

    #[test]
    fn each_source_keeps_only_its_own_input() {
        assert!(accept(Source::Simulated, true, SIM_TAG));
        assert!(!accept(Source::Simulated, false, 0));
        assert!(!accept(Source::Simulated, true, 0));
        assert!(accept(Source::Physical, false, 0));
        assert!(!accept(Source::Physical, true, SIM_TAG));
    }

    #[test]
    fn mouse_messages_become_engine_events() {
        let down = event_of(WM_LBUTTONDOWN, P, true);
        assert_eq!(down, Some(Event::Down(Button::Left, P, true)));
        assert_eq!(
            event_of(WM_RBUTTONUP, P, false),
            Some(Event::Up(Button::Right, P))
        );
        assert_eq!(event_of(WM_MOUSEMOVE, P, false), Some(Event::Move(P)));
        assert_eq!(event_of(WM_MOUSEWHEEL, P, false), None);
    }

    #[test]
    fn only_an_esc_press_is_esc() {
        let esc = u32::from(VK_ESCAPE.0);
        assert!(is_esc(WM_KEYDOWN, esc));
        assert!(is_esc(WM_SYSKEYDOWN, esc));
        assert!(!is_esc(WM_KEYUP, esc));
        assert!(!is_esc(WM_KEYDOWN, 0x41));
    }

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

    #[test]
    fn a_passed_mouse_event_can_be_replayed_but_esc_and_ticks_cannot() {
        let down = Event::Down(Button::Right, P, true);
        assert_eq!(act_of(down), Some(Act::Down(Button::Right, P)));
        assert_eq!(
            act_of(Event::Up(Button::Left, P)),
            Some(Act::Up(Button::Left, P))
        );
        assert_eq!(act_of(Event::Move(P)), Some(Act::Move(P)));
        assert_eq!(act_of(Event::Esc), None);
        assert_eq!(act_of(Event::Tick), None);
        assert!(is_mouse(&Act::Move(P)) && !is_mouse(&Act::KeyUp(0x10)));
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
}

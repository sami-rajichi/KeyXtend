//! What kx-gates' own engine copy reads and writes: whose input counts, hook messages as engine events, acts as
//! input. Task 14b deletes it with that copy; the simulated user's input lives in `simin`.
#![cfg(windows)]

pub use kx_platform_windows::keys::SIM_TAG;
use kx_platform_windows::keys::TAG;
use spike_core::hold::{Act, Button, Event, Pt};
use windows::Win32::UI::Input::KeyboardAndMouse::{INPUT, VK_ESCAPE};
use windows::Win32::UI::WindowsAndMessaging::{
    WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE,
    WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSKEYDOWN,
};

use crate::simin;

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

/// The simulated user's button for the engine's `button`.
fn sim_button(button: Button) -> simin::Button {
    match button {
        Button::Left => simin::Button::Left,
        Button::Right => simin::Button::Right,
        Button::Middle => simin::Button::Middle,
    }
}

/// The simulated user's act for the engine's `act`.
fn sim_act(act: Act) -> simin::Act {
    match act {
        Act::Move(p) => simin::Act::Move(p),
        Act::Down(b, p) => simin::Act::Down(sim_button(b), p),
        Act::Up(b, p) => simin::Act::Up(sim_button(b), p),
        Act::KeyDown(vk) => simin::Act::KeyDown(vk),
        Act::KeyUp(vk) => simin::Act::KeyUp(vk),
    }
}

/// The engine's `acts` as one `SendInput` batch marked `tag`, built by `simin::inputs`.
pub fn inputs(acts: &[Act], desktop: (i32, i32, i32, i32), tag: usize) -> Vec<INPUT> {
    let acts: Vec<simin::Act> = acts.iter().map(|&a| sim_act(a)).collect();
    simin::inputs(&acts, desktop, tag)
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn each_engine_act_becomes_the_same_simulated_act() {
        let acts = [
            (Act::Move(P), simin::Act::Move(P)),
            (
                Act::Down(Button::Left, P),
                simin::Act::Down(simin::Button::Left, P),
            ),
            (
                Act::Up(Button::Middle, P),
                simin::Act::Up(simin::Button::Middle, P),
            ),
            (
                Act::Down(Button::Right, P),
                simin::Act::Down(simin::Button::Right, P),
            ),
            (Act::KeyDown(0x10), simin::Act::KeyDown(0x10)),
            (Act::KeyUp(0x10), simin::Act::KeyUp(0x10)),
        ];
        for (engine, sim) in acts {
            assert_eq!(sim_act(engine), sim, "{engine:?}");
        }
    }
}

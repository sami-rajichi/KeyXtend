//! Keys, text and raw input for the gates, sent through the product injector and tagged.
#![cfg(windows)]

use std::io;

use kx_module_api::Redacted;
use kx_module_api::ports::{InputInjector, TextRoute};
use kx_platform_windows::WinInjector;
use kx_platform_windows::inject::{self, send_keys, vk_key};
use kx_platform_windows::screen::cursor;
use kx_platform_windows::strokes::{self, KeyEvent};
use spike_core::sendkeys;
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_KEYBOARD, INPUT_MOUSE, KEYEVENTF_KEYUP, MOUSE_EVENT_FLAGS, MOUSEEVENTF_ABSOLUTE,
    MOUSEEVENTF_HWHEEL, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN,
    MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
    MOUSEEVENTF_VIRTUALDESK, MOUSEEVENTF_WHEEL, MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP, MOUSEINPUT,
};

use crate::win;
use crate::{mouse, mouseguard};

/// Mouse flags that press a button.
const DOWNS: u32 = MOUSEEVENTF_LEFTDOWN.0
    | MOUSEEVENTF_RIGHTDOWN.0
    | MOUSEEVENTF_MIDDLEDOWN.0
    | MOUSEEVENTF_XDOWN.0;
/// Mouse flags that press a button or turn a wheel.
const PRESSES: u32 = DOWNS | MOUSEEVENTF_WHEEL.0 | MOUSEEVENTF_HWHEEL.0;
/// Mouse flags that let a button go.
const RELEASES: u32 =
    MOUSEEVENTF_LEFTUP.0 | MOUSEEVENTF_RIGHTUP.0 | MOUSEEVENTF_MIDDLEUP.0 | MOUSEEVENTF_XUP.0;
/// A move this many pixels or more from the pointer surely moves it, as Windows rounds an absolute target by up to
/// a pixel; a move that ends where the pointer is reaches no low-level hook.
const SURE_MOVE_PX: i32 = 2;

/// Refuses real input in a test build: gate tests go through the `ownkeys` desk, so a call that skips it fails its
/// test instead of typing on the desktop.
fn live() {
    #[cfg(test)]
    panic!("a gate test reached a real input sender; send through the ownkeys desk");
}

/// The mouse part of `input` when it is mouse input.
#[allow(unsafe_code, reason = "Reads the union member the input's type names.")]
fn as_mouse(input: &INPUT) -> Option<MOUSEINPUT> {
    if input.r#type != INPUT_MOUSE {
        return None;
    }
    // SAFETY: the type says the mouse member is set.
    Some(unsafe { input.Anonymous.mi })
}

/// The flags of `input` when it is mouse input.
fn mouse_flags(input: &INPUT) -> Option<u32> {
    as_mouse(input).map(|mi| mi.dwFlags.0)
}

/// Whether `input` presses something and whether it lets something go; anything but a key or mouse input presses.
#[allow(unsafe_code, reason = "Reads the union member the input's type names.")]
fn press_release(input: &INPUT) -> (bool, bool) {
    if input.r#type == INPUT_KEYBOARD {
        // SAFETY: the type says the keyboard member is set.
        let up = unsafe { input.Anonymous.ki }
            .dwFlags
            .contains(KEYEVENTF_KEYUP);
        return (!up, up);
    }
    if let Some(flags) = mouse_flags(input) {
        return (flags & PRESSES != 0, flags & RELEASES != 0);
    }
    (true, false)
}

/// True when `inputs` press or release a mouse button, which every live low-level hook sees.
fn clicks(inputs: &[INPUT]) -> bool {
    let buttons = DOWNS | RELEASES;
    inputs
        .iter()
        .filter_map(mouse_flags)
        .any(|f| f & buttons != 0)
}

/// True when a virtual-desktop move in `inputs` lands `SURE_MOVE_PX` or more from where the pointer was before it,
/// starting at `at` on desktop `desk` (left, top, width, height); after a relative move the pointer is unknown.
fn moves_far(inputs: &[INPUT], at: POINT, desk: (i32, i32, i32, i32)) -> bool {
    let (left, top, width, height) = desk;
    let to_desk = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK;
    let mut was = at;
    for mi in inputs.iter().filter_map(as_mouse) {
        if !mi.dwFlags.contains(MOUSEEVENTF_MOVE) {
            continue;
        }
        if !mi.dwFlags.contains(to_desk) {
            return false;
        }
        let to = (
            mouse::pixel(mi.dx, left, width),
            mouse::pixel(mi.dy, top, height),
        );
        if (to.0 - was.x).abs().max((to.1 - was.y).abs()) >= SURE_MOVE_PX {
            return true;
        }
        was = POINT { x: to.0, y: to.1 };
    }
    false
}

/// True when `inputs` hold an absolute mouse move, the only kind `moves_far` judges.
fn moves_absolute(inputs: &[INPUT]) -> bool {
    let absolute = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE;
    inputs
        .iter()
        .filter_map(as_mouse)
        .any(|mi| mi.dwFlags.contains(absolute))
}

/// True when every live low-level hook surely sees `inputs`: a mouse button goes down or up, or an absolute move takes
/// the pointer far enough from where it is now. The test build's hook sits ahead of the guard's and swallows a held
/// press, so the guard sees that press only when the test build replays it tagged `REPLAY`.
fn hooks_see(inputs: &[INPUT]) -> bool {
    let far =
        || cursor().is_ok_and(|p| moves_far(inputs, POINT { x: p.x, y: p.y }, win::desktop()));
    clicks(inputs) || (moves_absolute(inputs) && far())
}

/// True when `inputs` let something go and press nothing, so they still go out after the mouse guard stopped a gate.
fn only_lets_go(inputs: &[INPUT]) -> bool {
    let each: Vec<_> = inputs.iter().map(press_release).collect();
    each.iter().any(|&(_, up)| up) && !each.iter().any(|&(down, _)| down)
}

/// `input` without its pointer move, so a release goes out where the pointer already is.
#[allow(unsafe_code, reason = "Reads the union member the input's type names.")]
fn without_move(mut input: INPUT) -> INPUT {
    if input.r#type == INPUT_MOUSE {
        // SAFETY: the type says the mouse member is set.
        let mut mi = unsafe { input.Anonymous.mi };
        mi.dwFlags = MOUSE_EVENT_FLAGS(mi.dwFlags.0 & RELEASES);
        (mi.dx, mi.dy) = (0, 0);
        input.Anonymous.mi = mi;
    }
    input
}

/// Only the releases of `inputs`, without their moves.
fn bare(inputs: &[INPUT]) -> Vec<INPUT> {
    let ups = inputs.iter().filter(|i| press_release(i).1);
    ups.map(|&i| without_move(i)).collect()
}

/// How a batch that only lets go goes out once the mouse guard stopped the gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum After {
    /// Without its move, so the pointer stays where it is.
    Still,
    /// With its move, so a press the gate began ends away from its key and clicks nothing.
    Away,
}

/// What a batch of `inputs` sends as the guard `allows` new input or not: all of it, else only a batch that lets go,
/// moved as `after` says, else the refusal.
fn outgoing(
    inputs: &[INPUT],
    allows: Result<(), String>,
    after: After,
) -> Result<Vec<INPUT>, String> {
    match allows {
        Ok(()) => Ok(inputs.to_vec()),
        Err(_) if only_lets_go(inputs) && after == After::Away => Ok(inputs.to_vec()),
        Err(_) if only_lets_go(inputs) => Ok(bare(inputs)),
        Err(e) => Err(e),
    }
}

/// Sends mouse or key `inputs` in one batch; the error adds Windows' reason. Once the mouse guard stopped the gate,
/// only a batch that lets go goes out, moved as `after` says, so nothing stays pressed; input every hook sees tells
/// the guard its hook must count it.
pub fn send_as(inputs: &[INPUT], after: After) -> Result<(), String> {
    live();
    let out = outgoing(inputs, mouseguard::allowed(), after)?;
    let send = || inject::send(&out).map_err(|e| format!("{e}: {}", io::Error::last_os_error()));
    if hooks_see(&out) {
        mouseguard::own_input(send)
    } else {
        send()
    }
}

/// `send_as`, keeping the pointer still after a stop.
pub fn send(inputs: &[INPUT]) -> Result<(), String> {
    send_as(inputs, After::Still)
}

/// Types `text` as Unicode characters in one batch, whatever the target's layout.
pub fn text(text: &str) -> Result<(), String> {
    live();
    mouseguard::allowed()?;
    let text = Redacted::new(text.to_owned());
    WinInjector
        .text(&text, TextRoute::Unicode)
        .map_err(|e| e.to_string())
}

/// Types `text` one character at a time, `gap_ms` apart, or in one batch when `gap_ms` is 0.
pub fn paced(text: &str, gap_ms: u64) -> Result<(), String> {
    live();
    sendkeys::text_paced(text, gap_ms)
}

/// Virtual key `vk` pressed or released (`up`), marked `tag`.
pub fn tagged(vk: u16, up: bool, tag: usize) -> INPUT {
    inject::input(vk_key(vk).event(up), tag)
}

/// The presses of `vks` in order, then the releases in reverse.
fn presses(vks: &[u16]) -> Vec<KeyEvent> {
    let keys: Vec<_> = vks.iter().map(|&vk| vk_key(vk)).collect();
    strokes::combo(&keys)
}

/// Presses `vks` in order, then releases them in reverse; after a failed send it tries to let go of what went down.
pub fn combo(vks: &[u16]) -> Result<(), String> {
    mouseguard::allowed()?;
    restore_combo(vks)
}

/// Like `combo`, but it still goes out after the mouse guard stopped the gate, for a step that puts things back.
pub fn restore_combo(vks: &[u16]) -> Result<(), String> {
    live();
    send_keys(&presses(vks)).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests;

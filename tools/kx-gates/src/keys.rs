//! Keys, text and raw input for the gates, sent through the product injector and tagged.
#![cfg(windows)]

use std::io;

use kx_module_api::Redacted;
use kx_module_api::ports::{InputInjector, TextRoute};
use kx_platform_windows::WinInjector;
use kx_platform_windows::inject::{self, send_keys, vk_key};
use kx_platform_windows::strokes::{self, KeyEvent};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_KEYBOARD, INPUT_MOUSE, KEYEVENTF_KEYUP, MOUSE_EVENT_FLAGS, MOUSEEVENTF_HWHEEL,
    MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP,
    MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, MOUSEEVENTF_XDOWN,
    MOUSEEVENTF_XUP,
};

use crate::mouseguard;
use crate::win::sleep_ms;

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

/// The flags of `input` when it is mouse input.
#[allow(unsafe_code, reason = "Reads the union member the input's type names.")]
fn mouse_flags(input: &INPUT) -> Option<u32> {
    if input.r#type != INPUT_MOUSE {
        return None;
    }
    // SAFETY: the type says the mouse member is set.
    Some(unsafe { input.Anonymous.mi }.dwFlags.0)
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

/// True when `inputs` press or release a mouse button, which every live low-level hook sees; a move may end where
/// the pointer already is, which no hook sees.
fn clicks(inputs: &[INPUT]) -> bool {
    let buttons = DOWNS | RELEASES;
    inputs
        .iter()
        .filter_map(mouse_flags)
        .any(|f| f & buttons != 0)
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

/// Sends mouse or key `inputs` in one batch; the error adds Windows' reason. Once the mouse guard stopped the gate, only
/// a batch that lets go goes out, moved as `after` says, so nothing stays pressed; clicks also tell the guard its hook
/// must count them.
pub fn send_as(inputs: &[INPUT], after: After) -> Result<(), String> {
    let out = outgoing(inputs, mouseguard::allowed(), after)?;
    let send = || inject::send(&out).map_err(|e| format!("{e}: {}", io::Error::last_os_error()));
    if clicks(&out) {
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
    mouseguard::allowed()?;
    let text = Redacted::new(text.to_owned());
    WinInjector
        .text(&text, TextRoute::Unicode)
        .map_err(|e| e.to_string())
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
    send_keys(&presses(vks)).map_err(|e| e.to_string())
}

/// Like `combo`, with a pause of `gap_ms` after each press and release, for shell shortcuts.
pub fn slow_combo(vks: &[u16], gap_ms: u64) -> Result<(), String> {
    mouseguard::allowed()?;
    let events = presses(vks);
    for (at, event) in events.iter().enumerate() {
        if let Err(e) = send_keys(std::slice::from_ref(event)) {
            let _ = send_keys(&strokes::releases(&events[..at]));
            return Err(e.to_string());
        }
        sleep_ms(gap_ms);
    }
    Ok(())
}

#[cfg(test)]
mod tests;

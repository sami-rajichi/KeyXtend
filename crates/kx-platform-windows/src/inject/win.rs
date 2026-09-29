//! The Win32 side of injection: `SendInput` with our tag, and the layout lookups of the app in front.

use kx_module_api::Redacted;
use kx_module_api::ports::{InjectError, InputInjector, Scan, TextRoute};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    HKL, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
    KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, KEYEVENTF_UNICODE, MAPVK_VK_TO_VSC_EX, MapVirtualKeyExW,
    SendInput, VIRTUAL_KEY, VkKeyScanExW,
};

use super::{Os, chord_events, deliver, type_text};
use crate::front;
use crate::keys::TAG;
use crate::strokes::{Key, KeyEvent};

/// The injector on Windows: `SendInput`, with every event tagged `TAG`.
#[derive(Clone, Copy, Debug, Default)]
pub struct WinInjector;

impl InputInjector for WinInjector {
    fn tap(&self, key: Scan) -> Result<(), InjectError> {
        deliver(&Win32, &chord_events(&[], key))
    }

    fn chord(&self, held: &[Scan], key: Scan) -> Result<(), InjectError> {
        deliver(&Win32, &chord_events(held, key))
    }

    fn text(&self, text: &Redacted<String>, route: TextRoute) -> Result<(), InjectError> {
        type_text(&Win32, text, route)
    }

    fn target_app(&self) -> Option<String> {
        front::app()
    }
}

/// The Win32 calls behind `WinInjector`.
struct Win32;

impl Os for Win32 {
    type Layout = HKL;

    fn send(&self, events: &[KeyEvent]) -> usize {
        send_count(&inputs(events))
    }

    fn layout(&self) -> HKL {
        front::layout()
    }

    #[allow(unsafe_code, reason = "A plain layout-table lookup.")]
    fn key_scan(&self, unit: u16, layout: HKL) -> i16 {
        // SAFETY: a plain table lookup; Windows checks the layout handle.
        unsafe { VkKeyScanExW(unit, layout) }
    }

    #[allow(unsafe_code, reason = "A plain layout-table lookup.")]
    fn scan_of(&self, vk: u16, layout: HKL) -> u32 {
        // SAFETY: a plain table lookup; Windows checks the layout handle.
        unsafe { MapVirtualKeyExW(u32::from(vk), MAPVK_VK_TO_VSC_EX, Some(layout)) }
    }
}

/// Sends `inputs` in one call and returns how many Windows took; a batch too long to count takes none.
#[allow(unsafe_code, reason = "SendInput with a live slice of inputs.")]
fn send_count(inputs: &[INPUT]) -> usize {
    let (Ok(size), Ok(_)) = (
        i32::try_from(size_of::<INPUT>()),
        u32::try_from(inputs.len()),
    ) else {
        return 0;
    };
    // SAFETY: `inputs` is a live slice whose length fits the call, and `size` is one `INPUT`'s size.
    let sent = unsafe { SendInput(inputs, size) };
    usize::try_from(sent).unwrap_or(0)
}

/// Sends `inputs` in one call, which other input cannot split: all, or `Refused` with the count.
///
/// # Errors
/// `Refused` when Windows takes fewer inputs than given.
pub fn send(inputs: &[INPUT]) -> Result<(), InjectError> {
    let (sent, total) = (send_count(inputs), inputs.len());
    if sent == total {
        Ok(())
    } else {
        Err(InjectError::Refused { sent, total })
    }
}

/// Sends `events` tagged as ours in one call; after a partial send it releases what is still down.
///
/// # Errors
/// `Refused` when Windows takes fewer events than given.
pub fn send_keys(events: &[KeyEvent]) -> Result<(), InjectError> {
    deliver(&Win32, events)
}

/// Virtual key `vk` with its scan code in the layout of the app in front.
#[must_use]
pub fn vk_key(vk: u16) -> Key {
    Key::vk(vk, Win32.scan_of(vk, Win32.layout()))
}

/// `event` as a Windows keyboard input marked `tag`.
#[must_use]
pub fn input(event: KeyEvent, tag: usize) -> INPUT {
    let (vk, scan, mut flags) = match event.key {
        Key::Scan(s) => (0, u16::from(s.byte()), KEYEVENTF_SCANCODE),
        Key::Vk { vk, scan, .. } => (vk, u16::from(scan), KEYBD_EVENT_FLAGS(0)),
        Key::Unit(unit) => (0, unit, KEYEVENTF_UNICODE),
    };
    if event.key.extended() {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }
    if event.up {
        flags |= KEYEVENTF_KEYUP;
    }
    let ki = KEYBDINPUT {
        wVk: VIRTUAL_KEY(vk),
        wScan: scan,
        dwFlags: flags,
        time: 0,
        dwExtraInfo: tag,
    };
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 { ki },
    }
}

/// `events` as Windows inputs tagged as ours.
fn inputs(events: &[KeyEvent]) -> Vec<INPUT> {
    events.iter().map(|&e| input(e, TAG)).collect()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::keys::{self, key_for};
    use crate::strokes::{combo, unicode};

    /// The fields Windows reads from a keyboard input: (vk, scan, flags, tag).
    #[allow(
        unsafe_code,
        reason = "Reads the keyboard member of inputs built as keyboard inputs."
    )]
    fn fields(input: &INPUT) -> (u16, u16, KEYBD_EVENT_FLAGS, usize) {
        assert_eq!(input.r#type, INPUT_KEYBOARD);
        // SAFETY: every input here was built as a keyboard input, so `ki` is the live member.
        let ki = unsafe { input.Anonymous.ki };
        (ki.wVk.0, ki.wScan, ki.dwFlags, ki.dwExtraInfo)
    }

    fn all_kinds() -> Vec<KeyEvent> {
        let keys = [
            Key::Scan(Scan::new(0x1D)),
            Key::Scan(Scan::new(0xE038)),
            Key::vk(0x23, 0x4F),
            Key::vk(0x41, 0x1E),
        ];
        let mut events = combo(&keys);
        events.extend(unicode("😀"));
        events
    }

    #[test]
    fn every_event_we_send_carries_the_tag() {
        let sent = inputs(&all_kinds());
        assert_eq!(sent.len(), all_kinds().len());
        assert!(sent.iter().all(|i| fields(i).3 == TAG));
    }

    #[test]
    fn scan_codes_go_as_scan_codes_and_extended_keys_get_their_flag() {
        let right_alt = input(Key::Scan(Scan::new(0xE038)).event(true), TAG);
        let flags = KEYEVENTF_SCANCODE | KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP;
        assert_eq!(fields(&right_alt), (0, 0x38, flags, TAG));
        let ctrl = input(Key::Scan(Scan::new(0x1D)).event(false), TAG);
        assert_eq!(fields(&ctrl), (0, 0x1D, KEYEVENTF_SCANCODE, TAG));
    }

    #[test]
    fn virtual_keys_carry_their_scan_code_and_units_go_as_unicode() {
        let end = input(Key::vk(0x23, 0x4F).event(false), TAG);
        assert_eq!(fields(&end), (0x23, 0x4F, KEYEVENTF_EXTENDEDKEY, TAG));
        let unit = input(Key::Unit(0xD83D).event(true), TAG);
        let flags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP;
        assert_eq!(fields(&unit), (0, 0xD83D, flags, TAG));
    }

    #[test]
    fn navigation_keys_in_the_real_layout_are_extended_and_letters_are_not() {
        for vk in keys::EXTENDED_VKS {
            assert!(vk_key(vk).extended(), "{vk:#04X}");
        }
        for vk in [0x41, keys::VK_SHIFT, keys::ALTGR_VKS[0], keys::ALTGR_VKS[1]] {
            assert!(!vk_key(vk).extended(), "{vk:#04X}");
        }
    }

    #[test]
    fn enter_is_enter_in_every_layout() {
        let layout = Win32.layout();
        let enter = key_for(Win32.key_scan(u16::from(b'\r'), layout)).unwrap();
        assert_eq!((enter.vk, enter.shift, enter.altgr), (0x0D, false, false));
        assert_eq!(
            key_for(Win32.key_scan(0xD83D, layout)),
            None,
            "half a surrogate pair"
        );
    }

    #[test]
    fn the_injector_names_the_app_in_front_without_a_path() {
        let port: Arc<dyn InputInjector> = Arc::new(WinInjector);
        if let Some(name) = port.target_app() {
            assert!(!name.contains(['\\', '/']), "{name}");
        }
    }
}

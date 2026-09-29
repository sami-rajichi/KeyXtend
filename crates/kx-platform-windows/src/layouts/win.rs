//! The Win32 side of the layout source: the layout list, `ToUnicodeEx` and the switch request.

use kx_module_api::ports::{KeyChars, LangFacts, LayoutId, LayoutSource, Scan, SwitchError};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, GetKeyboardLayoutList, HKL, MAPVK_VSC_TO_VK_EX, MapVirtualKeyExW, ToUnicodeEx,
};
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_INPUTLANGCHANGEREQUEST};

use super::{KeyState, Os};
use crate::keys::VK_CAPITAL;
use crate::{front, langinfo};

/// `ToUnicodeEx` flag: leave the keyboard state unchanged (Windows 10 1607 and later).
const NO_STATE_CHANGE: u32 = 0x4;
/// `GetKeyState` bit of a toggle key that is on.
const TOGGLED: i16 = 1;

/// The layout source on Windows: the user's installed layouts, read with `ToUnicodeEx`.
#[derive(Clone, Copy, Debug, Default)]
pub struct WinLayouts;

impl LayoutSource for WinLayouts {
    fn installed(&self) -> Vec<LayoutId> {
        super::installed(&Win32)
    }

    fn foreground(&self) -> Option<LayoutId> {
        super::foreground(&Win32)
    }

    fn keys(&self, layout: LayoutId, scans: &[Scan]) -> Vec<KeyChars> {
        super::keys(&Win32, layout, scans)
    }

    fn facts(&self, layout: LayoutId) -> LangFacts {
        super::facts(&Win32, layout)
    }

    #[allow(unsafe_code, reason = "A plain toggle-key query.")]
    fn caps_on(&self) -> bool {
        // SAFETY: a plain query of a toggle key's state.
        unsafe { GetKeyState(i32::from(VK_CAPITAL)) & TOGGLED != 0 }
    }

    fn types_any(&self, layout: LayoutId, key: Scan, held: &[Scan]) -> bool {
        super::types_any(&Win32, layout, key, held)
    }

    fn request(&self, layout: LayoutId) -> Result<(), SwitchError> {
        super::request(&Win32, Win32.focus(), layout)
    }
}

/// The layout id of Windows handle `hkl`, sign extension kept.
#[must_use]
pub fn id(hkl: HKL) -> LayoutId {
    LayoutId::new(u64::try_from(hkl.0.addr()).unwrap_or_default())
}

/// The Windows handle of layout `id`; `None` when it does not fit a pointer.
#[must_use]
pub fn hkl(id: LayoutId) -> Option<HKL> {
    let addr = usize::try_from(id.get()).ok()?;
    Some(HKL(std::ptr::without_provenance_mut(addr)))
}

/// Asks the window with `window`'s keyboard to switch to installed layout `layout`, as the language hotkey does.
///
/// # Errors
/// `NoTarget` when the window is gone, `NotInstalled` for an unknown layout, `Refused` when Windows says no.
pub fn ask(window: HWND, layout: LayoutId) -> Result<(), SwitchError> {
    super::request(&Win32, front::focus_of(window), layout)
}

/// `ToUnicodeEx` for `key` (virtual key `vk`) in `state`, changing no keyboard state, for the
/// injector's check of the Keys route: the count (negative for a dead key) and the text in `buf`.
pub(crate) fn to_unicode(
    vk: u16,
    key: Scan,
    state: &KeyState,
    layout: LayoutId,
    buf: &mut [u16],
) -> i32 {
    Win32.to_unicode(vk, key, state, layout, buf)
}

/// The Win32 calls behind `WinLayouts`.
struct Win32;

impl Os for Win32 {
    type Window = HWND;

    #[allow(unsafe_code, reason = "Copies the layout list into a live buffer.")]
    fn list(&self, buf: &mut [LayoutId]) -> usize {
        let mut handles = vec![HKL::default(); buf.len()];
        let into = (!handles.is_empty()).then_some(handles.as_mut_slice());
        // SAFETY: the buffer, when given, is live for its whole length, sized from Windows' own `i32` count;
        // with none Windows only counts.
        let n = unsafe { GetKeyboardLayoutList(into) };
        for (slot, &handle) in buf.iter_mut().zip(&handles) {
            *slot = id(handle);
        }
        usize::try_from(n).unwrap_or(0)
    }

    fn focus(&self) -> Option<HWND> {
        front::focus()
    }

    fn layout_of(&self, window: HWND) -> Option<LayoutId> {
        front::thread_layout(window).map(id)
    }

    #[allow(unsafe_code, reason = "A plain layout-table lookup.")]
    fn vk_of(&self, key: Scan, layout: LayoutId) -> u16 {
        let Some(handle) = hkl(layout) else {
            return 0;
        };
        // SAFETY: a plain table lookup; the prefix byte of an extended key is part of the code.
        let vk =
            unsafe { MapVirtualKeyExW(u32::from(key.code()), MAPVK_VSC_TO_VK_EX, Some(handle)) };
        u16::try_from(vk).unwrap_or(0)
    }

    #[allow(unsafe_code, reason = "ToUnicodeEx over live buffers.")]
    fn to_unicode(
        &self,
        vk: u16,
        key: Scan,
        state: &KeyState,
        layout: LayoutId,
        buf: &mut [u16],
    ) -> i32 {
        let Some(handle) = hkl(layout) else {
            return 0;
        };
        // SAFETY: both buffers are live for their lengths, and the flag keeps every keyboard state as it is.
        // The scan code goes without its prefix byte, whose top bit would mean "key up".
        unsafe {
            ToUnicodeEx(
                u32::from(vk),
                u32::from(key.byte()),
                state,
                buf,
                NO_STATE_CHANGE,
                Some(handle),
            )
        }
    }

    fn facts(&self, layout: LayoutId) -> LangFacts {
        langinfo::facts(layout)
    }

    #[allow(unsafe_code, reason = "Posts a request the window handles itself.")]
    fn post(&self, window: HWND, layout: LayoutId) -> Result<(), SwitchError> {
        let handle = hkl(layout).ok_or(SwitchError::NotInstalled)?;
        let wanted = LPARAM(handle.0.addr().cast_signed());
        // SAFETY: a plain post; the window checks and applies the request itself.
        unsafe { PostMessageW(Some(window), WM_INPUTLANGCHANGEREQUEST, WPARAM(0), wanted) }
            .map_err(|_| SwitchError::Refused)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    /// Arabic (102) AZERTY on Arabic (Tunisia): a variant, sign-extended in a 64-bit handle.
    const AR_102: LayoutId = LayoutId::new(0xFFFF_FFFF_F029_1C01);
    /// A layout id no Windows installs.
    const NONE: LayoutId = LayoutId::new(0x7FFF_0000_0000_0000);
    /// Space, Enter and Tab type something; Left and Delete (extended) type nothing.
    const TYPING: [u16; 3] = [0x39, 0x1C, 0x0F];
    const SILENT: [u16; 2] = [0xE04B, 0xE053];
    /// The top letter row: Q to P on US English, A to P on French AZERTY, ض to ح on Arabic.
    const TOP_ROW: [u16; 10] = [0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19];
    /// Languages with golden tables, whose top row is all letters; Greek and Hebrew put signs there.
    const LETTER_LANGS: [&str; 3] = ["en", "fr", "ar"];

    #[test]
    fn ids_and_handles_round_trip_with_sign_extension() {
        let handle = hkl(AR_102).unwrap();
        assert_eq!(id(handle), AR_102);
        assert_eq!(u64::try_from(handle.0.addr()).unwrap(), AR_102.get());
    }

    #[test]
    fn every_layout_names_its_language_and_the_golden_ones_type_letters() {
        let list = WinLayouts.installed();
        assert!(!list.is_empty());
        let mut checked = 0;
        for layout in list {
            let iso = WinLayouts.facts(layout).iso;
            assert!(!iso.is_empty(), "{layout}");
            if !LETTER_LANGS.contains(&iso.as_str()) {
                continue;
            }
            let keys = WinLayouts.keys(layout, &TOP_ROW.map(Scan::new));
            assert!(
                keys.iter().all(|k| is_letter(&k.base)),
                "{layout}: {keys:?}"
            );
            checked += 1;
        }
        assert!(checked > 0, "no English, French or Arabic layout installed");
    }

    fn is_letter(text: &str) -> bool {
        text.chars().any(char::is_alphabetic)
    }

    #[test]
    fn only_keys_that_type_something_end_a_dead_key_wait() {
        for layout in WinLayouts.installed() {
            for code in TYPING {
                assert!(
                    WinLayouts.types_any(layout, Scan::new(code), &[]),
                    "{code:#x}"
                );
            }
            for code in SILENT {
                assert!(
                    !WinLayouts.types_any(layout, Scan::new(code), &[]),
                    "{code:#x}"
                );
            }
        }
    }

    #[test]
    fn an_unknown_layout_types_nothing_and_no_request_is_posted() {
        assert!(!WinLayouts.installed().contains(&NONE));
        assert_eq!(
            WinLayouts.keys(NONE, &[Scan::new(TOP_ROW[0])]),
            [KeyChars::default()]
        );
        assert_eq!(WinLayouts.facts(NONE), LangFacts::default());
        let refused = WinLayouts.request(NONE);
        assert!(matches!(
            refused,
            Err(SwitchError::NotInstalled | SwitchError::NoTarget)
        ));
        assert_eq!(ask(HWND::default(), NONE), Err(SwitchError::NoTarget));
    }

    #[test]
    fn the_source_is_a_port_other_threads_can_use() {
        let port: Arc<dyn LayoutSource> = Arc::new(WinLayouts);
        let list = std::thread::spawn(move || port.installed()).join().unwrap();
        assert!(!list.is_empty());
    }
}

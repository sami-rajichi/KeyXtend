//! Layout helpers for gates G10, G22 and G25 over the product's platform: the installed layouts, the layout of a
//! window's focus, a switch request, and the step the product's language key takes.
#![cfg(windows)]

use kx_module_api::ports::{LayoutId, LayoutSource, step_to};
use kx_platform_windows::{WinLayouts, front, layouts};
use windows::Win32::Foundation::HWND;

/// Error prefix when Windows refuses the layout request.
const REFUSED: &str = "layout request";

/// Raw layout handle `h` as the product's layout id, bit for bit, so a sign-extended handle keeps its high bits.
pub fn id(h: isize) -> LayoutId {
    LayoutId::new(u64::try_from(h.cast_unsigned()).unwrap_or_default())
}

/// Layout id `id` as a raw handle, bit for bit.
fn raw(id: LayoutId) -> isize {
    isize::try_from(id.get().cast_signed()).unwrap_or_default()
}

/// The installed layouts as raw values, in the system's order.
pub fn installed() -> Vec<isize> {
    WinLayouts.installed().into_iter().map(raw).collect()
}

/// The layout of the thread that holds `hwnd`'s keyboard focus; `None` when the window is gone.
pub fn layout_of(hwnd: HWND) -> Option<isize> {
    let focus = front::focus_of(hwnd)?;
    front::thread_layout(focus).map(|h| h.0 as isize)
}

/// The layout `by` places from `cur` in `list`, as the product steps; `None` when that is `cur` itself.
pub fn step_from(list: &[isize], cur: isize, by: i32) -> Option<isize> {
    let ids: Vec<LayoutId> = list.iter().map(|&h| id(h)).collect();
    let to = step_to(&ids, Some(id(cur)), by).filter(|&to| to != id(cur))?;
    Some(raw(to))
}

/// Asks `hwnd`'s focus window to switch to `layout`, the way the system language hotkey does.
pub fn ask(hwnd: HWND, layout: isize) -> Result<(), String> {
    layouts::ask(hwnd, id(layout)).map_err(|e| format!("{REFUSED}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EN: isize = 0x0409_0409;
    const FR: isize = 0x040C_040C;
    /// Arabic (Tunisia).
    const AR: isize = 0x1C01_1C01;

    #[test]
    fn a_step_follows_the_list_both_ways_and_wraps() {
        let list = [EN, FR, AR];
        assert_eq!(step_from(&list, EN, 1), Some(FR));
        assert_eq!(step_from(&list, AR, 1), Some(EN));
        assert_eq!(step_from(&list, EN, -1), Some(AR));
    }

    #[test]
    fn an_unknown_current_layout_steps_from_the_first() {
        assert_eq!(step_from(&[EN, FR], AR, 1), Some(FR));
    }

    #[test]
    fn a_window_that_is_gone_has_no_layout() {
        assert_eq!(layout_of(HWND::default()), None);
    }

    #[test]
    fn one_layout_or_none_cannot_switch() {
        assert_eq!(step_from(&[EN], EN, 1), None);
        assert_eq!(step_from(&[], EN, 1), None);
    }

    #[test]
    fn a_sign_extended_handle_keeps_its_bits_both_ways() {
        let extended = isize::try_from(0xFFFF_FFFF_F029_1C01_u64.cast_signed()).expect("64-bit");
        assert_eq!(id(extended).get(), 0xFFFF_FFFF_F029_1C01);
        assert_eq!(raw(id(extended)), extended);
        assert_eq!(step_from(&[EN, extended], EN, 1), Some(extended));
    }

    #[test]
    fn the_installed_list_reads_without_switching_anything() {
        let list = installed();
        assert!(!list.is_empty(), "Windows has at least one layout");
        assert!(list.iter().all(|&h| h != 0));
    }
}

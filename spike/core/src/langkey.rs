//! Layout requests for gates G10, G22 and G25: asks the app in front to switch, stepping by the product's `step_to`.
//! The window and layout queries are the product's (kx-platform-windows).

use kx_module_api::ports::{LayoutId, LayoutSource, step_to};
use kx_platform_windows::{WinLayouts, front, langinfo, layouts};
use windows::Win32::Foundation::HWND;

/// Error prefix when Windows refuses the layout request.
const REFUSED: &str = "layout request";

/// The first layout in `list` for language `id`.
pub fn layout_for(list: &[isize], id: u16) -> Option<isize> {
    list.iter()
        .copied()
        .find(|&h| langinfo::lang_id(LayoutId::new(h as u64)) == id)
}

/// The installed layouts as raw values, in the system's order.
pub fn installed() -> Vec<isize> {
    WinLayouts
        .installed()
        .iter()
        .map(|id| id.get() as isize)
        .collect()
}

/// The installed layout for language `id`, such as 0x040C for French (France).
pub fn installed_for(id: u16) -> Option<isize> {
    layout_for(&installed(), id)
}

/// The layout of the thread that holds `hwnd`'s keyboard focus; `None` when the window is gone.
pub fn layout_of(hwnd: HWND) -> Option<isize> {
    let focus = front::focus_of(hwnd)?;
    front::thread_layout(focus).map(|h| h.0 as isize)
}

/// The layout `by` places from `cur` in `list`, as the product steps; `None` when that is `cur` itself.
pub fn step_from(list: &[isize], cur: isize, by: i32) -> Option<isize> {
    let id = |h: isize| LayoutId::new(h as u64);
    let ids: Vec<LayoutId> = list.iter().map(|&h| id(h)).collect();
    let to = step_to(&ids, Some(id(cur)), by).filter(|&to| to != id(cur))?;
    Some(to.get() as isize)
}

/// Asks `hwnd`'s focus window to switch to `layout`, the way the system language hotkey does.
pub fn ask(hwnd: HWND, layout: isize) -> Result<(), String> {
    layouts::ask(hwnd, LayoutId::new(layout as u64)).map_err(|e| format!("{REFUSED}: {e}"))
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
    fn the_layout_for_a_language_is_found_by_its_id() {
        assert_eq!(layout_for(&[EN, FR, AR], 0x040C), Some(FR));
        assert_eq!(layout_for(&[EN, AR], 0x040C), None);
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
}

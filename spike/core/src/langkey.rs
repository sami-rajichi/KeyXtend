//! The language key: asks the app in front to switch to the next installed layout.
//! The window and layout queries are the product's (kx-platform-windows); the stepping goes to kx-mod-keyboard later in P3.

use kx_module_api::ports::{LayoutId, LayoutSource};
use kx_platform_windows::{WinLayouts, front, langinfo, layouts};
use windows::Win32::Foundation::HWND;

/// Error when the window in front closed.
const GONE: &str = "the window is gone";
/// Error when there is no other layout to switch to.
const ONLY_ONE: &str = "only one layout is installed";
/// Error prefix when Windows refuses the layout request.
const REFUSED: &str = "layout request";

/// The layout after `cur` in `list`, wrapping and skipping copies of `cur`; `None` when there is none.
pub fn next_layout(list: &[isize], cur: isize) -> Option<isize> {
    let start = list.iter().position(|&l| l == cur).map_or(0, |i| i + 1);
    (0..list.len())
        .map(|k| list[(start + k) % list.len()])
        .find(|&l| l != cur)
}

/// The first layout in `list` for language `id`.
pub fn layout_for(list: &[isize], id: u16) -> Option<isize> {
    list.iter()
        .copied()
        .find(|&h| langinfo::lang_id(LayoutId::new(h as u64)) == id)
}

/// The layouts before and after `cur` in `list`, wrapping; `cur` itself when nothing else is installed.
pub fn neighbours(list: &[isize], cur: isize) -> (isize, isize) {
    let back: Vec<isize> = list.iter().rev().copied().collect();
    let prev = next_layout(&back, cur).unwrap_or(cur);
    (prev, next_layout(list, cur).unwrap_or(cur))
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

/// Asks `hwnd` to switch to the next installed layout; returns the layout asked for.
pub fn ask_next(hwnd: HWND) -> Result<isize, String> {
    ask_step(hwnd, false)
}

/// Asks `hwnd` to switch to the layout after its current one, or before it when `back`; returns the layout asked for.
pub fn ask_step(hwnd: HWND, back: bool) -> Result<isize, String> {
    let cur = layout_of(hwnd).ok_or(GONE)?;
    let (prev, next) = neighbours(&installed(), cur);
    let to = if back { prev } else { next };
    if to == cur {
        return Err(ONLY_ONE.into());
    }
    ask(hwnd, to)?;
    Ok(to)
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
    fn neighbours_wrap_both_ways_and_a_lone_layout_is_its_own() {
        assert_eq!(neighbours(&[EN, FR, AR], EN), (AR, FR));
        assert_eq!(neighbours(&[EN, FR, AR], AR), (FR, EN));
        assert_eq!(neighbours(&[EN], EN), (EN, EN));
    }

    #[test]
    fn next_layout_follows_the_list_and_wraps_to_the_first() {
        let list = [EN, FR, AR];
        assert_eq!(next_layout(&list, EN), Some(FR));
        assert_eq!(next_layout(&list, FR), Some(AR));
        assert_eq!(next_layout(&list, AR), Some(EN));
    }

    #[test]
    fn an_unknown_current_layout_goes_to_the_first() {
        assert_eq!(next_layout(&[EN, FR], AR), Some(EN));
    }

    #[test]
    fn a_duplicate_of_the_current_layout_is_skipped() {
        assert_eq!(next_layout(&[EN, EN, FR], EN), Some(FR));
        assert_eq!(next_layout(&[EN, EN], EN), None);
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
        assert_eq!(next_layout(&[EN], EN), None);
        assert_eq!(next_layout(&[], EN), None);
    }
}

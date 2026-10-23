//! G22's (P11) layout helpers, until it moves: the installed layout for a language; G10's and G25's are in kx-gates.

use kx_module_api::ports::{LayoutId, LayoutSource};
use kx_platform_windows::{WinLayouts, langinfo};

/// The first layout in `list` for language `id`.
pub fn layout_for(list: &[isize], id: u16) -> Option<isize> {
    list.iter()
        .copied()
        .find(|&h| langinfo::lang_id(LayoutId::new(h as u64)) == id)
}

/// The installed layouts as raw values, in the system's order.
fn installed() -> Vec<isize> {
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

#[cfg(test)]
mod tests {
    use super::*;

    const EN: isize = 0x0409_0409;
    const FR: isize = 0x040C_040C;
    /// Arabic (Tunisia).
    const AR: isize = 0x1C01_1C01;

    #[test]
    fn the_layout_for_a_language_is_found_by_its_id() {
        assert_eq!(layout_for(&[EN, FR, AR], 0x040C), Some(FR));
        assert_eq!(layout_for(&[EN, AR], 0x040C), None);
    }
}

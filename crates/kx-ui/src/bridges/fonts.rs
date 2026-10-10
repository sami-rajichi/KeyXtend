//! Qt's font list as the hub's font book (ADR-0021): it adds a bundled font file from the compiled-in resources, never
//! from disk, and drops it again.

use crate::hub::FontBook;

#[cxx_qt::bridge]
mod ffi {
    // SAFETY: `fonts.h` declares exactly these functions with these types; they run on the Qt GUI thread, as the hub's
    // font calls do.
    unsafe extern "C++" {
        include!("kx-ui/cpp/fonts.h");

        /// Adds the font file at `path`, a compiled-in resource, to Qt's font list; its id, or -1.
        #[namespace = "kxui"]
        fn font_load(path: &str) -> i32;

        /// Drops the font file added as `id`; false when Qt held none.
        #[namespace = "kxui"]
        fn font_unload(id: i32) -> bool;
    }
}

/// The resource folder of the bundled files, as a path Qt opens, from the build.
const FILES: &str = env!("KX_UI_FILES");

/// Qt's font list, fed only from the compiled-in fonts; use it on the Qt GUI thread.
#[derive(Debug, Default)]
pub(crate) struct QtFontBook;

impl FontBook for QtFontBook {
    fn load(&mut self, name: &str) -> Option<i32> {
        let id = ffi::font_load(&format!("{FILES}/{name}"));
        (id >= 0).then_some(id)
    }

    fn unload(&mut self, id: i32) {
        if !ffi::font_unload(id) {
            tracing::warn!(id, "Qt held no font to drop");
        }
    }
}

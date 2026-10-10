//! The fonts side of the hub (ADR-0021): only the bundled files of the theme shown stay loaded, and a file that does
//! not load hands its list to the next family. The app's font book runs on the Qt thread.

use super::Hub;
use kx_theme::assets::{FontFiles, SYSTEM_FONTS};
use std::collections::BTreeMap;
use std::sync::{MutexGuard, PoisonError};

/// Loads and drops bundled font files; the app's book is Qt's font list, used on the Qt thread only.
pub trait FontBook: Send {
    /// Loads the bundled file named `name`, such as `fonts/rubik/Rubik-wght-.ttf`; its id, or `None` when it did not
    /// load.
    fn load(&mut self, name: &str) -> Option<i32>;

    /// Drops the file loaded as `id`.
    fn unload(&mut self, id: i32);
}

/// The book and what it holds now, by bundle name.
#[derive(Default)]
pub(super) struct Fonts {
    book: Option<Box<dyn FontBook>>,
    loaded: BTreeMap<String, i32>,
}

impl Hub {
    fn fonts(&self) -> MutexGuard<'_, Fonts> {
        self.fonts.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Takes the font book and loads the fonts of the theme shown; call it on the Qt thread before the QML loads.
    pub fn set_fonts(&self, book: Box<dyn FontBook>) {
        self.fonts().book = Some(book);
        let _ = self.sync_fonts();
    }

    /// Loads the bundled files the theme shown needs and drops the others; true when that changed what is loaded. Call
    /// it on the Qt thread, before QML reads the families again.
    pub fn sync_fonts(&self) -> bool {
        let theme = self.lock().settings.theme.clone();
        let Some(look) = self.themes.find(&theme).map(|t| &t.look) else {
            return false;
        };
        let lists = [&look.latin_fonts, &look.arabic_fonts, &look.mark_fonts];
        let mut fonts = self.fonts();
        let Fonts { book, loaded } = &mut *fonts;
        let Some(book) = book.as_mut() else {
            return false;
        };
        let mut kept = BTreeMap::new();
        let mut added = false;
        for list in lists {
            added |= pick(list, &self.font_files, book.as_mut(), loaded, &mut kept);
        }
        let gone = std::mem::replace(loaded, kept);
        for id in gone.values() {
            book.unload(*id);
        }
        added || !gone.is_empty()
    }
}

/// Keeps the first family of `list` that Windows ships, or whose bundled files all load, in `kept`, moving its files
/// out of `loaded` or loading them; a family whose file did not load warns, and the next one is tried. True when it
/// loaded a file.
fn pick(
    list: &[String],
    table: &FontFiles,
    book: &mut dyn FontBook,
    loaded: &mut BTreeMap<String, i32>,
    kept: &mut BTreeMap<String, i32>,
) -> bool {
    for family in list {
        if SYSTEM_FONTS.contains(&family.as_str()) {
            return false;
        }
        let names = table.names(family).unwrap_or_default();
        let mut took = BTreeMap::new();
        let all = !names.is_empty()
            && names.iter().all(|name| {
                let id = kept
                    .get(name)
                    .or(loaded.get(name))
                    .copied()
                    .or_else(|| book.load(name));
                id.map(|id| took.insert(name.clone(), id)).is_some()
            });
        if all {
            let added = took
                .keys()
                .any(|n| !loaded.contains_key(n) && !kept.contains_key(n));
            loaded.retain(|n, _| !took.contains_key(n));
            kept.extend(took);
            return added;
        }
        for (name, id) in took {
            if !loaded.contains_key(&name) && !kept.contains_key(&name) {
                book.unload(id);
            }
        }
        tracing::warn!(family = %family, "a theme font did not load, so its next family is tried");
    }
    false
}

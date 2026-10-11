//! The fonts side of the hub (ADR-0021), whose book runs on the Qt thread: only the bundled files of the theme shown
//! stay loaded, and a file that does not load hands its list to the next family. A switch loads the new files before
//! QML reads the new look and drops the old ones after, so no text asks Qt for a family it lacks.

use super::Hub;
use kx_theme::assets::{FontFiles, SYSTEM_FONTS};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{MutexGuard, PoisonError};

/// Loads and drops bundled font files; the app's book is Qt's font list, used on the Qt thread only.
pub trait FontBook: Send {
    /// Loads the bundled file named `name`, such as `fonts/rubik/Rubik-wght-.ttf`; its id, or `None` when it did not
    /// load.
    fn load(&mut self, name: &str) -> Option<i32>;

    /// Drops the file loaded as `id`.
    fn unload(&mut self, id: i32);
}

/// The book, what it holds now by bundle name, the files the last load picked for the theme shown, and whether a
/// load added files since the last drop.
#[derive(Default)]
pub(super) struct Fonts {
    book: Option<Box<dyn FontBook>>,
    loaded: BTreeMap<String, i32>,
    picked: BTreeSet<String>,
    added: bool,
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
        let added = self.load_fonts();
        self.drop_fonts() || added
    }

    /// Loads the bundled files the theme shown needs and keeps the others until `drop_fonts`; true when it loaded one.
    /// Call it on the Qt thread, before QML reads the new look.
    pub fn load_fonts(&self) -> bool {
        let theme = self.lock().settings.theme.clone();
        let Some(look) = self.themes.find(&theme).map(|t| &t.look) else {
            return false;
        };
        let lists = [&look.latin_fonts, &look.arabic_fonts, &look.mark_fonts];
        let mut fonts = self.fonts();
        let Fonts {
            book,
            loaded,
            picked,
            added,
        } = &mut *fonts;
        let Some(book) = book.as_mut() else {
            return false;
        };
        let mut kept = BTreeMap::new();
        let mut now = false;
        for list in lists {
            now |= pick(list, &self.font_files, book.as_mut(), loaded, &mut kept);
        }
        *picked = kept.keys().cloned().collect();
        loaded.extend(kept);
        *added |= now;
        now
    }

    /// Drops the bundled files the last `load_fonts` did not pick; true when the files changed since the last drop, so
    /// QML reads the families again. Call it on the Qt thread, once QML reads the new look.
    pub fn drop_fonts(&self) -> bool {
        let mut fonts = self.fonts();
        let Fonts {
            book,
            loaded,
            picked,
            added,
        } = &mut *fonts;
        let Some(book) = book.as_mut() else {
            return false;
        };
        let gone: Vec<i32> = loaded
            .iter()
            .filter(|(name, _)| !picked.contains(*name))
            .map(|(_, id)| *id)
            .collect();
        loaded.retain(|name, _| picked.contains(name));
        for id in &gone {
            book.unload(*id);
        }
        std::mem::take(added) || !gone.is_empty()
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

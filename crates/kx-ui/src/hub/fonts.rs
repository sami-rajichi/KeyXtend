//! The fonts side of the hub (ADR-0021), whose book runs on the Qt thread: only the bundled files of the theme shown
//! stay loaded, and a file that does not load hands its list to the next family. Each read of the look loads its
//! theme's files first and a wake drops the old ones after, so no text asks Qt for a family it lacks.

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

/// The book, what it holds now by bundle name, the files the last load picked for the theme shown, whether a load
/// added files since the last drop, and the families whose files did not load, which are not asked for again.
#[derive(Default)]
pub(super) struct Fonts {
    book: Option<Box<dyn FontBook>>,
    loaded: BTreeMap<String, i32>,
    picked: BTreeSet<String>,
    added: bool,
    failed: BTreeSet<String>,
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
        self.load_theme(&theme)
    }

    /// Loads the bundled files theme `id` needs, as `load_fonts` does for the theme shown.
    pub(super) fn load_theme(&self, id: &str) -> bool {
        let Some(look) = self.themes.find(id).map(|t| &t.look) else {
            return false;
        };
        let lists = [&look.latin_fonts, &look.arabic_fonts, &look.mark_fonts];
        let mut fonts = self.fonts();
        let Fonts {
            book,
            loaded,
            picked,
            added,
            failed,
        } = &mut *fonts;
        let Some(book) = book.as_mut() else {
            return false;
        };
        let mut picker = Picker {
            book: book.as_mut(),
            loaded: &mut *loaded,
            kept: BTreeMap::new(),
            failed,
        };
        let mut now = false;
        for list in lists {
            now |= picker.pick(list, &self.font_files);
        }
        let kept = picker.kept;
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
            ..
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

/// One load's pick: the book, the files loaded before it, the files it keeps for the theme, and the families whose
/// files did not load.
struct Picker<'a> {
    book: &'a mut dyn FontBook,
    loaded: &'a mut BTreeMap<String, i32>,
    kept: BTreeMap<String, i32>,
    failed: &'a mut BTreeSet<String>,
}

impl Picker<'_> {
    /// Keeps the first family of `list` that Windows ships, or whose bundled files all load, moving its files out of
    /// `loaded` or loading them; a family whose file did not load warns once, joins `failed`, and the next one is
    /// tried. True when it loaded a file.
    fn pick(&mut self, list: &[String], table: &FontFiles) -> bool {
        for family in list {
            if SYSTEM_FONTS.contains(&family.as_str()) {
                return false;
            }
            if self.failed.contains(family) {
                continue;
            }
            let names = table.names(family).unwrap_or_default();
            if let Some(took) = self.take(&names) {
                let added = took.keys().any(|n| self.new_file(n));
                self.loaded.retain(|n, _| !took.contains_key(n));
                self.kept.extend(took);
                return added;
            }
            self.failed.insert(family.clone());
            tracing::warn!(family = %family, "a theme font did not load, so its next family is tried");
        }
        false
    }

    /// The id of each file in `names`, kept, loaded before or loaded now; `None` when there are none or one did not
    /// load, after dropping the ones it loaded.
    fn take(&mut self, names: &[String]) -> Option<BTreeMap<String, i32>> {
        let mut took = BTreeMap::new();
        let all = !names.is_empty()
            && names.iter().all(|name| {
                let had = self.kept.get(name).or(self.loaded.get(name)).copied();
                let id = had.or_else(|| self.book.load(name));
                id.map(|id| took.insert(name.clone(), id)).is_some()
            });
        if all {
            return Some(took);
        }
        for (name, id) in took {
            if self.new_file(&name) {
                self.book.unload(id);
            }
        }
        None
    }

    /// True when file `name` was neither loaded before this load nor kept by it yet.
    fn new_file(&self, name: &str) -> bool {
        !self.loaded.contains_key(name) && !self.kept.contains_key(name)
    }
}

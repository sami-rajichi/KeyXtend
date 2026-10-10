//! The theme's fonts (ADR-0021): only the bundled files of the theme shown stay loaded, a theme switch drops the last
//! theme's, and a file that does not load hands its list to the next family, with a warning.

use crate::logs::warnings_of;
use crate::{FakeFace, Rig};
use kx_test_support::host::FakeHost;
use kx_theme::assets::FontFiles;
use kx_ui::{FontBook, MANIFEST, UiSettings};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, PoisonError};

/// What the fake book holds: each loaded file by its id, and the next id.
#[derive(Default)]
struct Shelf {
    loaded: BTreeMap<i32, String>,
    next: i32,
}

/// A font book that loads every file but those it refuses, and keeps what it holds where the test reads it.
#[derive(Clone, Default)]
struct Book {
    shelf: Arc<Mutex<Shelf>>,
    refuse: Vec<String>,
}

impl Book {
    /// The files loaded now, by bundle name.
    fn loaded(&self) -> BTreeSet<String> {
        let held = self.shelf.lock().unwrap_or_else(PoisonError::into_inner);
        held.loaded.values().cloned().collect()
    }
}

impl FontBook for Book {
    fn load(&mut self, name: &str) -> Option<i32> {
        if self.refuse.iter().any(|r| r == name) {
            return None;
        }
        let mut held = self.shelf.lock().unwrap_or_else(PoisonError::into_inner);
        held.next += 1;
        let id = held.next;
        held.loaded.insert(id, name.to_owned());
        Some(id)
    }

    fn unload(&mut self, id: i32) {
        let mut held = self.shelf.lock().unwrap_or_else(PoisonError::into_inner);
        assert!(
            held.loaded.remove(&id).is_some(),
            "only a loaded file is dropped"
        );
    }
}

/// The bundle names of every file of `families`.
fn files(families: &[&str]) -> BTreeSet<String> {
    let table = FontFiles::shipped().unwrap();
    let names = families.iter().map(|f| table.names(f).unwrap());
    names.flatten().collect()
}

/// Shows theme `id`, as a Settings-window change does, then syncs the fonts as the bridge does on its wake.
fn show(rig: &Rig, id: &str) -> bool {
    let settings = UiSettings {
        theme: id.to_owned(),
        ..UiSettings::from_defaults().unwrap()
    };
    let mut host = FakeHost::new(&MANIFEST);
    rig.hub
        .attach(settings, Arc::new(FakeFace::default()), host.handle());
    rig.hub.sync_fonts()
}

#[test]
fn native_needs_no_bundled_font_as_windows_ships_segoe_ui() {
    let rig = Rig::new();
    let book = Book::default();
    rig.hub.set_fonts(Box::new(book.clone()));
    assert!(book.loaded().is_empty());
    assert!(!rig.hub.sync_fonts(), "nothing changed");
}

#[test]
fn each_theme_switch_keeps_only_that_themes_files() {
    let rig = Rig::new();
    let book = Book::default();
    rig.hub.set_fonts(Box::new(book.clone()));
    let steps: [(&str, &[&str]); 4] = [
        ("dolch", &["Rubik", "Noto Sans Arabic"]),
        ("et66", &["IBM Plex Sans", "IBM Plex Sans Arabic"]),
        ("native", &[]),
        ("dolch", &["Rubik", "Noto Sans Arabic"]),
    ];
    for (id, families) in steps {
        assert!(show(&rig, id), "{id}: the fonts changed");
        assert_eq!(book.loaded(), files(families), "{id}");
    }
    assert!(!show(&rig, "dolch"), "the same theme loads nothing again");
}

#[test]
fn a_file_that_does_not_load_hands_its_list_to_the_next_family_with_a_warning() {
    let rig = Rig::new();
    let book = Book {
        refuse: files(&["Rubik"]).into_iter().collect(),
        ..Book::default()
    };
    rig.hub.set_fonts(Box::new(book.clone()));
    let warned = warnings_of(|| {
        show(&rig, "dolch");
    });
    assert_eq!(
        book.loaded(),
        files(&["Noto Sans Arabic"]),
        "Arabic falls back"
    );
    assert!(
        !warned.is_empty() && warned.iter().all(|w| w.contains("next family")),
        "{warned:?}"
    );
}

#[test]
fn without_a_book_nothing_loads() {
    let rig = Rig::new();
    assert!(!show(&rig, "dolch"));
}

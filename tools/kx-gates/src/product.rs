//! The product app the gates G2-G4, G8-G11, G13, G25, G26 and `close` aim at: its windows, found by program, its
//! keyboard, and the running copy's data files and keyboard settings, read without changing them.
#![cfg(windows)]

use std::collections::HashSet;
use std::env::consts::EXE_SUFFIX;
use std::path::Path;

use kx_mod_keyboard::{ID, KeyboardSettings, Role, SPEC};
use kx_platform_windows::dirs;
use kx_settings::{Files, peek, peek_section};
use windows::Win32::Foundation::{HWND, POINT, RECT};

use crate::keymap::{KeyMap, Reader};
use crate::{apps, board, win};

/// The product's name on the command line; its program is this name and the system's program suffix.
pub const NAME: &str = "keyxtend";
/// The app's package manifest, from this crate's folder; its package name is `NAME`.
#[cfg(test)]
const APP_MANIFEST: &str = "../../apps/keyxtend/Cargo.toml";

/// True when `path` is the product's program, whatever its folder and letter case.
pub fn is_ours(path: &str) -> bool {
    apps::same_program(path, &format!("{NAME}{EXE_SUFFIX}"))
}

/// The longest a keyboard with `settings` takes to show the layout of the app in front: one relabel period and one
/// `poll_ms` more. A key clicked before then may step from, or wait for, the old layout.
pub fn follow_after(settings: &KeyboardSettings, poll_ms: u64) -> u64 {
    settings.relabel_ms + poll_ms
}

/// `follow_after` with the running copy's settings.
pub fn follow_ms(poll_ms: u64) -> Result<u64, String> {
    keyboard().map(|s| follow_after(&s, poll_ms))
}

/// The data files of the product at `program`: beside it in a portable copy, else in the user's folder.
pub fn files(program: &Path) -> Result<Files, String> {
    let local = dirs::local_data().map_err(|e| e.to_string())?;
    let app = dirs::from_paths(program, &local).map_err(|e| e.to_string())?;
    Ok(Files::new(app.data_dir().path()))
}

/// The `[keyboard]` settings in settings file `file`, as a later copy of the app reads a section.
fn keyboard_in(file: Option<&toml::Table>) -> Result<KeyboardSettings, String> {
    peek_section(file, ID, &SPEC).map_err(|e| format!("the keyboard's defaults: {e}"))
}

/// The keyboard settings of the product at `program`, read without changing its files; an error when its data
/// folder cannot be named, so a gate never acts on the defaults by mistake.
fn keyboard_of(program: &Path) -> Result<KeyboardSettings, String> {
    let file = peek(&files(program)?);
    keyboard_in(file.as_ref())
}

/// The running copy's keyboard settings; the defaults when no copy runs.
pub fn keyboard() -> Result<KeyboardSettings, String> {
    match running() {
        Some((_, exe)) => keyboard_of(Path::new(&exe)),
        None => keyboard_in(None),
    }
}

/// True when window `w` belongs to the product.
fn owns(w: HWND) -> bool {
    win::program(w).is_ok_and(|p| is_ours(&p))
}

/// The product's windows a person can see now, topmost first.
pub fn shown() -> Vec<HWND> {
    win::seen_windows()
        .into_iter()
        .filter(|&w| owns(w))
        .collect()
}

/// Presses the button with UI Automation id `id` in whichever shown product window has it.
pub fn invoke(reader: &Reader, id: &str) -> Result<(), String> {
    let mut why = format!("no shown {NAME} window has {id}");
    for hwnd in shown() {
        match reader.invoke(hwnd, id) {
            Ok(()) => return Ok(()),
            Err(e) => why = e,
        }
    }
    Err(why)
}

/// The running product's process id and program path, found by any of its windows, shown or hidden.
pub fn running() -> Option<(u32, String)> {
    let mut asked = HashSet::new();
    let mut fresh = win::all_windows()
        .into_iter()
        .filter(|&w| asked.insert(win::owner(w).0));
    fresh.find_map(|w| {
        let path = win::program(w).ok().filter(|p| is_ours(p))?;
        Some((win::owner(w).0, path))
    })
}

/// The product's keyboard on screen: its window and every key's box.
pub struct Keyboard {
    /// The keyboard's window.
    pub hwnd: HWND,
    keys: KeyMap,
}

impl Keyboard {
    /// The topmost shown product window whose buttons hold the language key. An owned window sits above its owner,
    /// so the keyboard comes before its shadow, whose tree may list the keyboard's buttons too.
    pub fn find() -> Result<Self, String> {
        let anchor = board::id_of(Role::Lang)?;
        let reader = Reader::new()?;
        for hwnd in shown() {
            let Ok(keys) = reader.read(hwnd) else {
                continue;
            };
            if keys.at(&anchor).is_some() {
                return Ok(Self { hwnd, keys });
            }
        }
        Err(format!("no {NAME} keyboard is showing; start {NAME} first"))
    }

    /// The screen centre of key `id`, in physical pixels.
    pub fn at(&self, id: &str) -> Result<POINT, String> {
        self.keys.at(id).ok_or_else(|| missing(id))
    }

    /// The screen box of key `id`, in physical pixels.
    pub fn rect(&self, id: &str) -> Result<RECT, String> {
        self.keys.rect(id).ok_or_else(|| missing(id))
    }
}

/// Why key `id` has no place.
fn missing(id: &str) -> String {
    format!("key {id} not found by its UI Automation id")
}

#[cfg(test)]
mod tests {
    use kx_mod_keyboard::LangKey;
    use kx_platform::PORTABLE_DIR;
    use kx_test_support::tempdir::TempDir;

    use super::*;

    #[test]
    fn the_product_is_its_program_in_any_folder_and_case() {
        assert!(is_ours(
            r"C:\Program Files\KeyXtend-dev\keyxtend\keyxtend.exe"
        ));
        assert!(is_ours(r"D:\Projects\KeyXtend\target\debug\KeyXtend.EXE"));
        assert!(!is_ours(r"C:\Program Files\KeyXtend-dev\qt-kb\qt-kb.exe"));
        assert!(!is_ours(r"C:\x\keyxtend.exe.bak"));
        assert!(!is_ours(""));
    }

    fn table(text: &str) -> toml::Table {
        text.parse().expect("TOML")
    }

    fn defaults() -> KeyboardSettings {
        KeyboardSettings::from_defaults().expect("defaults")
    }

    #[test]
    fn the_keyboard_follows_within_one_relabel_period_and_one_poll() {
        let mut s = defaults();
        s.relabel_ms = 900;
        assert_eq!(follow_after(&s, 50), 950);
    }

    #[test]
    fn the_keyboard_settings_are_the_files_section_over_the_defaults() {
        assert_eq!(keyboard_in(None), Ok(defaults()));
        assert_eq!(keyboard_in(Some(&table("[ui]\nx = 1"))), Ok(defaults()));
        let file = table("[keyboard]\nversion = 1\nrelabel_ms = 900\nlang_key = \"list\"");
        let s = keyboard_in(Some(&file)).expect("reads");
        assert_eq!((s.relabel_ms, s.lang_key), (900, LangKey::List));
        assert_eq!(
            s.modifiers,
            defaults().modifiers,
            "the rest keeps its default"
        );
    }

    #[test]
    fn a_portable_copy_keeps_its_settings_beside_its_program() {
        let dir = TempDir::new("gates-portable").expect("a temp folder");
        let data = dir.path().join(PORTABLE_DIR);
        std::fs::create_dir(&data).expect("the data folder");
        let want = Files::new(&data);
        std::fs::write(&want.settings, "[keyboard]\nrelabel_ms = 700\n").expect("written");
        let program = dir.path().join(format!("{NAME}{EXE_SUFFIX}"));
        assert_eq!(files(&program), Ok(want));
        assert_eq!(keyboard_of(&program).map(|s| s.relabel_ms), Ok(700));
    }

    #[test]
    fn a_program_whose_data_folder_cannot_be_named_is_an_error_not_the_defaults() {
        let bare = Path::new("keyxtend.exe");
        assert!(
            files(bare).is_err(),
            "no folder to read the settings beside"
        );
        assert!(
            keyboard_of(bare).is_err(),
            "G25 must not click with defaults"
        );
    }

    #[test]
    fn the_name_is_the_app_package_name() {
        let file = Path::new(env!("CARGO_MANIFEST_DIR")).join(APP_MANIFEST);
        let text = std::fs::read_to_string(&file).expect("the app's Cargo.toml reads");
        let manifest: toml::Table = text.parse().expect("is TOML");
        assert_eq!(manifest["package"]["name"].as_str(), Some(NAME));
    }
}

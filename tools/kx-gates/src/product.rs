//! The product app the gates G2-G4, G8-G13, G25, G26 and `close` aim at: its windows, found by program, and its
//! keyboard.
#![cfg(windows)]

use std::collections::HashSet;
use std::env::consts::EXE_SUFFIX;
use std::path::Path;

use kx_mod_keyboard::{KeyboardSettings, Role};
use windows::Win32::Foundation::{HWND, POINT, RECT};

use crate::keymap::{KeyMap, Reader};
use crate::{board, win};

/// The product's name on the command line; its program is this name and the system's program suffix.
pub const NAME: &str = "keyxtend";
/// The app's package manifest, from this crate's folder; its package name is `NAME`.
#[cfg(test)]
const APP_MANIFEST: &str = "../../apps/keyxtend/Cargo.toml";

/// True when `path` is the product's program, whatever its folder and letter case.
pub fn is_ours(path: &str) -> bool {
    let program = format!("{NAME}{EXE_SUFFIX}");
    Path::new(path)
        .file_name()
        .is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case(&program))
}

/// The longest the keyboard takes to show the layout of the app in front: one relabel period of its default
/// settings, and one `poll_ms` more. A key clicked before then may step from, or wait for, the old layout.
pub fn follow_ms(poll_ms: u64) -> Result<u64, String> {
    let settings =
        KeyboardSettings::from_defaults().map_err(|e| format!("the keyboard's defaults: {e}"))?;
    Ok(settings.relabel_ms + poll_ms)
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

    #[test]
    fn the_keyboard_follows_within_one_relabel_period_and_one_poll() {
        let relabel = KeyboardSettings::from_defaults()
            .expect("defaults")
            .relabel_ms;
        assert_eq!(follow_ms(50), Ok(relabel + 50));
    }

    #[test]
    fn the_name_is_the_app_package_name() {
        let file = Path::new(env!("CARGO_MANIFEST_DIR")).join(APP_MANIFEST);
        let text = std::fs::read_to_string(&file).expect("the app's Cargo.toml reads");
        let manifest: toml::Table = text.parse().expect("is TOML");
        assert_eq!(manifest["package"]["name"].as_str(), Some(NAME));
    }
}

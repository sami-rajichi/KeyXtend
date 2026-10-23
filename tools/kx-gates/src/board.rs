//! The product's board as kx-mod-keyboard ships it: its keys' ids, its character keys, its language key and the
//! shortcuts layer's pages.
#![cfg(windows)]

use std::collections::HashSet;

use kx_mod_keyboard::spec::{RowKey, char_id};
use kx_mod_keyboard::{BoardSpec, Latch, Layer, Nav, Role};
use kx_module_api::ports::{Scan, ShortcutId, Shortcuts};
use kx_platform_windows::WinShortcuts;

/// The board compiled into the product.
fn shipped() -> Result<BoardSpec, String> {
    BoardSpec::shipped().map_err(|e| format!("the product's board: {e}"))
}

/// Every character key's scan code and id, row by row.
pub fn char_keys() -> Result<Vec<(Scan, String)>, String> {
    let scans = shipped()?.char_scans();
    Ok(scans.into_iter().map(|sc| (sc, char_id(sc))).collect())
}

/// Every key's id: the main rows left to right, then the side keys.
pub fn ids() -> Result<Vec<String>, String> {
    Ok(shipped()?.roles().into_iter().map(|(id, ..)| id).collect())
}

/// Every key's id, role and scan code: the main rows left to right, then the side keys.
pub fn roles() -> Result<Vec<(String, Role, Option<Scan>)>, String> {
    Ok(shipped()?.roles())
}

/// The id of the key that does `nav` while one of the product's panels is open.
pub fn nav_key(nav: Nav) -> Result<String, String> {
    let board = shipped()?;
    let mut keys = (0..board.rows.len()).flat_map(|r| board.row_keys(r));
    let found = keys.find_map(|k| match k {
        RowKey::Named(n) if n.nav == Some(nav) => Some(n.id.clone()),
        _ => None,
    });
    found.ok_or_else(|| format!("the product's board has no {nav:?} key"))
}

/// The id of the first key whose role is `role`.
pub fn id_of(role: Role) -> Result<String, String> {
    let roles = shipped()?.roles();
    let found = roles.into_iter().find(|(_, r, _)| *r == role);
    found
        .map(|(id, ..)| id)
        .ok_or_else(|| format!("the product's board has no {role:?} key"))
}

/// The id of the first key that latches `latch`, whether or not a second click sends it alone.
pub fn latch_id(latch: Latch) -> Result<String, String> {
    let roles = shipped()?.roles();
    let found = roles
        .into_iter()
        .find(|(_, r, _)| matches!(r, Role::Latch(l) if l.latch == latch));
    found
        .map(|(id, ..)| id)
        .ok_or_else(|| format!("the product's board has no {latch:?} key"))
}

/// The id of the character key with scan code `code`.
pub fn char_key(code: u16) -> Result<String, String> {
    let found = char_keys()?.into_iter().find(|(sc, _)| sc.code() == code);
    found
        .map(|(_, id)| id)
        .ok_or_else(|| format!("the product's board has no character key {code:#06X}"))
}

/// Each page's key ids in reading order, Next last, as the product draws its `shortcut_pages` setting `pages` on
/// Windows.
pub fn layer_pages(pages: &[Vec<ShortcutId>]) -> Result<Vec<Vec<String>>, String> {
    let known: HashSet<ShortcutId> = WinShortcuts.known().into_iter().map(|s| s.id).collect();
    let mut layer = Layer::new(shipped()?.layer_grid(), pages, &known);
    layer.toggle();
    let mut all = Vec::new();
    for _ in 0..layer.page_count() {
        all.push(layer.cells().into_iter().map(|c| c.key.id()).collect());
        layer.next_page();
    }
    Ok(all)
}

#[cfg(test)]
mod tests {
    use kx_mod_keyboard::KeyboardSettings;
    use kx_mod_keyboard::spec::NEXT_ID;

    use super::*;

    #[test]
    fn character_keys_are_named_by_their_scan_code() {
        let keys = char_keys().expect("the board reads");
        assert_eq!(
            keys.len(),
            BoardSpec::shipped().expect("reads").char_scans().len()
        );
        assert!(keys.iter().all(|(sc, id)| *id == char_id(*sc)));
        assert!(
            keys.iter()
                .any(|(sc, id)| sc.code() == 0x1E && id == "c-0x001E")
        );
    }

    #[test]
    fn the_key_that_closes_a_panel_and_every_role_come_from_the_board() {
        assert_eq!(nav_key(Nav::Esc).expect("one Esc key"), "esc");
        let roles = roles().expect("the board reads");
        assert_eq!(roles.len(), ids().expect("the board reads").len());
        assert!(
            roles
                .iter()
                .any(|(id, r, _)| id == "caps" && *r == Role::Caps)
        );
    }

    #[test]
    fn the_win_latch_and_a_character_key_come_from_the_board() {
        assert_eq!(latch_id(Latch::Win).expect("one Win key"), "win");
        assert_eq!(latch_id(Latch::Shift).expect("a Shift key"), "shift");
        assert_eq!(char_key(0x2F).expect("V's place"), "c-0x002F");
        assert!(char_key(0x01).is_err(), "Esc is a named key");
        assert!(char_key(0xFFFF).is_err());
    }

    #[test]
    fn the_layer_pages_are_the_products_with_next_last() {
        let defaults = KeyboardSettings::from_defaults().expect("defaults");
        let pages = layer_pages(&defaults.shortcut_pages).expect("the board reads");
        assert_eq!(pages.len(), defaults.shortcut_pages.len());
        assert!(
            pages
                .iter()
                .all(|p| p.last().map(String::as_str) == Some(NEXT_ID))
        );
        assert!(pages[0].contains(&"sc-select-all".to_string()));
        assert!(pages[1].contains(&"sc-switch-app".to_string()));
        assert!(
            layer_pages(&[]).expect("reads").is_empty(),
            "nothing to show"
        );
    }

    #[test]
    fn the_language_key_and_every_id_come_from_the_board() {
        assert_eq!(id_of(Role::Lang).expect("one language key"), "lang");
        let all = ids().expect("the board reads");
        assert!(all.contains(&"esc".to_string()) && all.contains(&"c-0x0022".to_string()));
        assert!(id_of(Role::Later).is_ok(), "the first of several");
    }
}

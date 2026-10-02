//! The product's board as kx-mod-keyboard ships it: its keys' ids, its character keys and its language key.
#![cfg(windows)]

use kx_mod_keyboard::spec::{RowKey, char_id};
use kx_mod_keyboard::{BoardSpec, Nav, Role};
use kx_module_api::ports::Scan;

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

#[cfg(test)]
mod tests {
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
    fn the_language_key_and_every_id_come_from_the_board() {
        assert_eq!(id_of(Role::Lang).expect("one language key"), "lang");
        let all = ids().expect("the board reads");
        assert!(all.contains(&"esc".to_string()) && all.contains(&"c-0x0022".to_string()));
        assert!(id_of(Role::Later).is_ok(), "the first of several");
    }
}

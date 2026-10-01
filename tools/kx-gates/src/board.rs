//! The product's board as kx-mod-keyboard ships it: its keys' ids, its character keys and its language key.
#![cfg(windows)]

use kx_mod_keyboard::spec::char_id;
use kx_mod_keyboard::{BoardSpec, Role};
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
    fn the_language_key_and_every_id_come_from_the_board() {
        assert_eq!(id_of(Role::Lang).expect("one language key"), "lang");
        let all = ids().expect("the board reads");
        assert!(all.contains(&"esc".to_string()) && all.contains(&"c-0x0022".to_string()));
        assert!(id_of(Role::Later).is_ok(), "the first of several");
    }
}

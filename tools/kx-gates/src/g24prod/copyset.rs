//! The settings file of G24's copy: the snip module's defaults with the keyboard way under test, word prediction and
//! the clipboard history off, so the copy's own history keeps none of the gate's pictures.

use kx_mod_snip::SnipSettings;
use kx_module_api::ports::SnipKeyboard;
use toml::{Table, Value};

use crate::ownset;

/// The snip module's defaults with the keyboard `way` during a capture, as the copy runs them.
pub fn snip(way: SnipKeyboard) -> Result<SnipSettings, String> {
    let mut s = SnipSettings::from_defaults().map_err(|e| format!("the snip defaults: {e}"))?;
    s.keyboard = way;
    Ok(s)
}

/// The copy's settings with the keyboard `way` during a capture.
pub fn table(way: SnipKeyboard) -> Result<Table, String> {
    let mut t = ownset::predict(&Table::new(), false, &[kx_mod_clipboard::ID])?;
    let section = Table::try_from(snip(way)?).map_err(|e| format!("the snip settings: {e}"))?;
    t.insert(kx_mod_snip::ID.as_str().to_owned(), Value::Table(section));
    Ok(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_copy_takes_the_way_asked_with_the_clipboard_history_off() {
        for way in [SnipKeyboard::Bubble, SnipKeyboard::Hidden] {
            let t = table(way).expect("a table");
            let section = t[kx_mod_snip::ID.as_str()].clone();
            let read: SnipSettings = section.try_into().expect("the module reads it");
            assert_eq!(read, snip(way).expect("the defaults"), "{way:?}");
            let off = t[kx_kernel::KERNEL.as_str()]["disabled"]
                .as_array()
                .expect("a list");
            assert!(
                off.iter()
                    .any(|m| m.as_str() == Some(kx_mod_clipboard::ID.as_str())),
                "{off:?}"
            );
        }
    }
}

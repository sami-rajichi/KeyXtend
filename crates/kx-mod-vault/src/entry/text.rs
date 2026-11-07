//! The text rules: a shown text loses the bidi marks that copied Arabic text often holds; then a text holding
//! any other hidden mark, a line break among them, or too many characters is refused. ZWNJ and ZWJ stay, as Arabic and
//! Persian words need them.

use kx_module_api::is_hidden;

use super::EntryError;

/// The marks a shown text loses: LRM, RLM and ALM. A hidden text holding one is refused, as nobody sees it to check.
pub const STRIPPED: [char; 3] = ['\u{200e}', '\u{200f}', '\u{061c}'];

/// Fits `text` to a field in place, so a wiped buffer stays the only copy: a shown text loses `STRIPPED`, then any
/// hidden mark or more than `max` characters refuses it.
///
/// # Errors
/// `Refused` when it holds a hidden mark or more than `max` characters.
pub fn fit(text: &mut String, hidden: bool, max: usize) -> Result<(), EntryError> {
    if !hidden {
        strip(text);
    }
    held(text, max)
}

/// Takes `STRIPPED` out of `text` in place, so a wiped buffer stays the only copy.
pub fn strip(text: &mut String) {
    text.retain(|c| !STRIPPED.contains(&c));
}

/// Checks a saved `text` without changing it: no hidden mark and at most `max` characters.
///
/// # Errors
/// `Refused` when it holds a hidden mark or more than `max` characters.
pub fn held(text: &str, max: usize) -> Result<(), EntryError> {
    let marked = text.chars().any(is_hidden);
    if marked || text.chars().count() > max {
        Err(EntryError::Refused)
    } else {
        Ok(())
    }
}

/// True when the user may type `c` into a line: anything but a hidden mark other than `STRIPPED`, which a shown text
/// loses and a hidden one refuses on saving.
#[must_use]
pub fn typable(c: char) -> bool {
    !is_hidden(c) || STRIPPED.contains(&c)
}

/// True when `text` holds nothing but blanks, which an own label may not.
#[must_use]
pub fn blank(text: &str) -> bool {
    text.trim().is_empty()
}

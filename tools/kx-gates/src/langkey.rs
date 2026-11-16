//! G22's layout helpers, its own copies of the spike's: a language's installed layout by its LANGID, which G22 puts in
//! front of target-window before each clip.
#![cfg(windows)]

use kx_platform_windows::langinfo;

use crate::langs;

/// Digits in a LANGID written in hex.
const DIGITS: usize = 4;
/// Base of the LANGID digits.
const HEX: u32 = 16;

/// The LANGID `text` names; `None` unless it is exactly 4 hex digits.
pub fn lang_id(text: &str) -> Option<u16> {
    let digits = text.len() == DIGITS && text.bytes().all(|b| b.is_ascii_hexdigit());
    digits
        .then(|| u16::from_str_radix(text, HEX).ok())
        .flatten()
}

/// The first layout in `list` for language `id`.
#[cfg_attr(
    not(feature = "voice"),
    allow(dead_code, reason = "Only the voice gates call it.")
)]
pub fn layout_for(list: &[isize], id: u16) -> Option<isize> {
    list.iter()
        .copied()
        .find(|&h| langinfo::lang_id(langs::id(h)) == id)
}

/// The installed layout for language `id`, such as 0x040C for French (France).
#[cfg_attr(
    not(feature = "voice"),
    allow(dead_code, reason = "Only the voice gates call it.")
)]
pub fn installed_for(id: u16) -> Option<isize> {
    layout_for(&langs::installed(), id)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EN: isize = 0x0409_0409;
    const FR: isize = 0x040C_040C;
    /// Arabic (Tunisia).
    const AR: isize = 0x1C01_1C01;

    #[test]
    fn the_layout_for_a_language_is_found_by_its_id() {
        assert_eq!(layout_for(&[EN, FR, AR], 0x040C), Some(FR));
        assert_eq!(layout_for(&[EN, FR, AR], 0x1C01), Some(AR));
        assert_eq!(layout_for(&[EN, AR], 0x040C), None);
        assert_eq!(layout_for(&[], 0x0409), None);
    }

    #[test]
    fn the_first_of_two_layouts_in_one_language_wins() {
        let us_intl = isize::try_from(0xFFFF_FFFF_F001_0409_u64.cast_signed()).expect("64-bit");
        assert_eq!(layout_for(&[FR, us_intl, EN], 0x0409), Some(us_intl));
    }

    #[test]
    fn a_lang_id_is_exactly_four_hex_digits() {
        assert_eq!(lang_id("0409"), Some(0x0409));
        assert_eq!(lang_id("040c"), Some(0x040c));
        assert_eq!(lang_id("1C01"), Some(0x1c01));
        for bad in ["+409", "-409", "409", "04090", "04 9", "040g", ""] {
            assert_eq!(lang_id(bad), None, "{bad:?}");
        }
    }
}

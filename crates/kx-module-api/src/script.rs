//! Which text holds Arabic letters, from the Unicode blocks that hold them; the layouts module names languages by
//! it and the keyboard picks each legend's font by it.

use std::ops::RangeInclusive;

/// The Unicode blocks that hold Arabic letters; Presentation Forms-B stops before the byte-order mark.
pub const ARABIC_BLOCKS: [RangeInclusive<char>; 7] = [
    // Arabic.
    '\u{0600}'..='\u{06FF}',
    // Arabic Supplement.
    '\u{0750}'..='\u{077F}',
    // Arabic Extended-B.
    '\u{0870}'..='\u{089F}',
    // Arabic Extended-A.
    '\u{08A0}'..='\u{08FF}',
    // Arabic Presentation Forms-A.
    '\u{FB50}'..='\u{FDFF}',
    // Arabic Presentation Forms-B.
    '\u{FE70}'..='\u{FEFC}',
    // Arabic Extended-C.
    '\u{10EC0}'..='\u{10EFF}',
];

/// True when `text` holds any character from `ARABIC_BLOCKS`.
#[must_use]
pub fn is_arabic(text: &str) -> bool {
    text.chars()
        .any(|c| ARABIC_BLOCKS.iter().any(|block| block.contains(&c)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arabic_script_languages_and_signs_are_found_by_their_letters() {
        for text in [
            "العربية",
            "فارسی",
            "اردو",
            "کوردی",
            "ئۇيغۇرچە",
            "\u{FEFB}",
            "\u{FDFC}",
        ] {
            assert!(is_arabic(text), "{text}");
        }
    }

    #[test]
    fn other_scripts_even_right_to_left_ones_are_not_arabic() {
        for text in [
            "English",
            "français",
            "עברית",
            "ދިވެހި",
            "",
            "\u{FEFF}",
            "\u{25CC}",
        ] {
            assert!(!is_arabic(text), "{text}");
        }
    }

    #[test]
    fn the_blocks_are_ordered_and_apart() {
        let pairs = ARABIC_BLOCKS.windows(2);
        assert!(pairs.into_iter().all(|p| p[0].end() < p[1].start()));
    }
}

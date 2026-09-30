//! Which text is written in the Arabic script, from the Unicode blocks that hold it.

use std::ops::RangeInclusive;

/// The Unicode blocks of the Arabic script; Presentation Forms-B stops before the byte-order mark.
const ARABIC_BLOCKS: [RangeInclusive<char>; 7] = [
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

/// True when `text` holds any character of the Arabic script.
pub fn is_arabic(text: &str) -> bool {
    text.chars()
        .any(|c| ARABIC_BLOCKS.iter().any(|block| block.contains(&c)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arabic_script_languages_are_found_by_their_letters() {
        for name in ["العربية", "فارسی", "اردو", "کوردی", "ئۇيغۇرچە", "\u{FEFB}"]
        {
            assert!(is_arabic(name), "{name}");
        }
    }

    #[test]
    fn other_scripts_even_right_to_left_ones_are_not_arabic() {
        for name in ["English", "français", "עברית", "ދިވެހި", "", "\u{FEFF}"] {
            assert!(!is_arabic(name), "{name}");
        }
    }
}

//! Which text holds Arabic letters and which reads right to left, from Unicode's blocks: the layouts module names
//! languages and the keyboard picks each legend's font by the first, and the clipboard panel sets each row's way.

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

/// The ranges whose letters read right to left: the default R and AL ranges of Unicode's `DerivedBidiClass.txt`.
pub const RTL_BLOCKS: [RangeInclusive<char>; 5] = [
    // Hebrew, Arabic, Syriac, Thaana, N'Ko, Samaritan, Mandaic and the Arabic extensions.
    '\u{0590}'..='\u{08FF}',
    // Hebrew and Arabic presentation forms.
    '\u{FB1D}'..='\u{FDFF}',
    // Arabic Presentation Forms-B, before the byte-order mark.
    '\u{FE70}'..='\u{FEFC}',
    // The right-to-left scripts of the first supplementary plane, Arabic Extended-C among them.
    '\u{10800}'..='\u{10FFF}',
    // Mende Kikakui, Adlam, the Siyaq numbers and the Arabic mathematical letters.
    '\u{1E800}'..='\u{1EFFF}',
];

/// True when `text` holds any character from `ARABIC_BLOCKS`.
#[must_use]
pub fn is_arabic(text: &str) -> bool {
    text.chars()
        .any(|c| ARABIC_BLOCKS.iter().any(|block| block.contains(&c)))
}

/// True when the first letter of `text` reads right to left, as a paragraph takes the direction of its first strong
/// character (UAX #9); digits, signs and spaces before it do not count.
#[must_use]
pub fn is_rtl(text: &str) -> bool {
    let first = text.chars().find(|c| c.is_alphabetic());
    first.is_some_and(|c| RTL_BLOCKS.iter().any(|block| block.contains(&c)))
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
        let pairs = RTL_BLOCKS.windows(2);
        assert!(pairs.into_iter().all(|p| p[0].end() < p[1].start()));
    }

    #[test]
    fn the_first_letter_sets_the_direction() {
        let rtl = [
            "مرحبا",
            "شكراً hello",
            "שלום",
            "١٢٣ مرحبا",
            "(«مرحبا»)",
            "ދިވެހި",
            "ܫܠܡܐ",
            "\u{FEFB}",
        ];
        for text in rtl {
            assert!(is_rtl(text), "{text}");
        }
        let ltr = [
            "hello مرحبا",
            "école",
            "Ελληνικά",
            "",
            " ",
            "123",
            "١٢٣",
            "«»",
        ];
        for text in ltr {
            assert!(!is_rtl(text), "{text}");
        }
    }

    #[test]
    fn every_arabic_block_lies_in_a_right_to_left_range() {
        for block in &ARABIC_BLOCKS {
            let inside =
                |r: &RangeInclusive<char>| r.start() <= block.start() && block.end() <= r.end();
            assert!(RTL_BLOCKS.iter().any(inside), "{block:?}");
        }
    }
}

//! Fill text: one line with no control character or other hidden mark, so a fill never sends Enter or a Tab.

use super::*;
use proptest::prelude::*;

/// Texts a fill types: Latin, Arabic ("password"), a no-break space, Persian with its ZWNJ ("I want"), and its solar
/// year's mark with its ZWJ.
const KEPT: [&str; 5] = [
    "zebra quartz",
    "كلمة السر",
    "a\u{a0}b",
    "می\u{200c}خواهم",
    "ه\u{200d}.ش",
];
/// Texts it refuses: a line feed, a carriage return, a Tab, a bell, a line separator, a bidi override, and the LRM, RLM
/// and ALM marks that Save and Paste strip first.
const REFUSED: [&str; 9] = [
    "a\nb",
    "a\r",
    "\tb",
    "\u{7}",
    "a\u{2028}b",
    "\u{202e}abc",
    "a\u{200e}b",
    "a\u{200f}b",
    "a\u{61c}b",
];

/// `text` as fill text, if it may be.
fn typed(text: &str) -> Option<FillText> {
    FillText::new(Secret::new(Zeroizing::new(text.to_owned())))
}

/// Every control character Unicode names (`Cc`).
fn controls() -> Vec<char> {
    ('\0'..='\u{1f}').chain('\u{7f}'..='\u{9f}').collect()
}

#[test]
fn fill_text_keeps_one_line_and_refuses_controls_and_hidden_marks() {
    for text in KEPT {
        let kept = typed(text).map(|t| t.text().expose().to_string());
        assert_eq!(kept.as_deref(), Some(text), "{text:?}");
    }
    for text in REFUSED {
        assert_eq!(typed(text), None, "{text:?}");
    }
}

proptest! {
    #[test]
    fn a_text_holding_any_control_character_is_refused(
        text in "\\PC*",
        at in any::<prop::sample::Index>(),
        control in prop::sample::select(controls()),
    ) {
        let mut chars: Vec<char> = text.chars().collect();
        chars.insert(at.index(chars.len() + 1), control);
        prop_assert!(typed(&String::from_iter(chars)).is_none());
    }
}

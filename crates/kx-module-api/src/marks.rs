//! Characters that must never reach a log line or a notice as they are.

/// The line and paragraph separators and the bidi controls: not control characters, yet each can
/// split a line or turn it around.
pub const HIDDEN_MARKS: [char; 14] = [
    '\u{2028}', '\u{2029}', '\u{061c}', '\u{200e}', '\u{200f}', '\u{202a}', '\u{202b}', '\u{202c}',
    '\u{202d}', '\u{202e}', '\u{2066}', '\u{2067}', '\u{2068}', '\u{2069}',
];

/// True for a control character, line breaks included, or one of `HIDDEN_MARKS`.
/// Logs escape these characters and notices drop them.
#[must_use]
pub fn is_hidden(c: char) -> bool {
    c.is_control() || HIDDEN_MARKS.contains(&c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// Line breaks: LF, CR, NEL, and the line and paragraph separators.
    const BREAKS: [char; 5] = ['\n', '\r', '\u{85}', '\u{2028}', '\u{2029}'];
    /// Controls a value may hide: BEL, ESC, DEL, RLO, the first-strong isolate and ALM.
    const CONTROLS: [char; 6] = [
        '\u{7}', '\u{1b}', '\u{7f}', '\u{202e}', '\u{2068}', '\u{61c}',
    ];
    /// Characters texts need: letters, a space, the no-break spaces, a fatha, ZWNJ and ZWJ.
    const KEPT: [char; 8] = [
        'a', 'ع', ' ', '\u{a0}', '\u{202f}', '\u{64e}', '\u{200c}', '\u{200d}',
    ];

    #[test]
    fn line_breaks_and_controls_are_hidden_marks() {
        for c in BREAKS.iter().chain(&CONTROLS) {
            assert!(is_hidden(*c), "{c:?}");
        }
    }

    #[test]
    fn letters_spaces_and_joiners_are_kept() {
        for c in KEPT {
            assert!(!is_hidden(c), "{c:?}");
        }
    }

    #[test]
    fn the_table_holds_only_what_is_control_misses_once_each() {
        let unique: HashSet<_> = HIDDEN_MARKS.iter().collect();
        assert_eq!(unique.len(), HIDDEN_MARKS.len());
        assert!(HIDDEN_MARKS.iter().all(|c| !c.is_control()));
    }
}

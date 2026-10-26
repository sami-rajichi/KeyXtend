//! Characters that must never reach a log line, a notice or a typed word as they are: they draw nothing, yet change
//! what a text says or how it reads.

/// The line and paragraph separators and the bidi controls: not control characters, yet each can
/// split a line or turn it around.
pub const HIDDEN_MARKS: [char; 14] = [
    '\u{2028}', '\u{2029}', '\u{061c}', '\u{200e}', '\u{200f}', '\u{202a}', '\u{202b}', '\u{202c}',
    '\u{202d}', '\u{202e}', '\u{2066}', '\u{2067}', '\u{2068}', '\u{2069}',
];

/// Characters that draw nothing, as first and last of each run: `Default_Ignorable` ones (the soft hyphen, the Hangul
/// filler letters, the Mongolian vowel separator, the zero-width space, the word joiner and invisible operators, the
/// deprecated inhibitors, the BOM, the shorthand and musical format controls, the tags) and the interlinear annotation
/// formats. ZWNJ and ZWJ stay out, as Persian words and emoji need them.
pub const INVISIBLE: [(char, char); 14] = [
    ('\u{ad}', '\u{ad}'),
    ('\u{115f}', '\u{1160}'),
    ('\u{180e}', '\u{180e}'),
    ('\u{200b}', '\u{200b}'),
    ('\u{2060}', '\u{2064}'),
    ('\u{206a}', '\u{206f}'),
    ('\u{3164}', '\u{3164}'),
    ('\u{feff}', '\u{feff}'),
    ('\u{ffa0}', '\u{ffa0}'),
    ('\u{fff9}', '\u{fffb}'),
    ('\u{1bca0}', '\u{1bca3}'),
    ('\u{1d173}', '\u{1d17a}'),
    ('\u{e0001}', '\u{e0001}'),
    ('\u{e0020}', '\u{e007f}'),
];

/// True for a control character, line breaks included, one of `HIDDEN_MARKS`, or an `INVISIBLE` character, which may
/// be a format or a letter (the Hangul fillers). Logs escape them, notices and chips drop them, and the keyboard types
/// no word holding one.
#[must_use]
pub fn is_hidden(c: char) -> bool {
    let invisible = INVISIBLE
        .iter()
        .any(|(first, last)| (*first..=*last).contains(&c));
    c.is_control() || HIDDEN_MARKS.contains(&c) || invisible
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
    fn invisible_formats_are_hidden_marks() {
        // Soft hyphen, Mongolian vowel separator, zero-width space, word joiner, invisible plus, an inhibitor, BOM and
        // an interlinear annotation mark.
        let formats = [
            '\u{ad}', '\u{180e}', '\u{200b}', '\u{2060}', '\u{2064}', '\u{206a}', '\u{feff}',
            '\u{fff9}',
        ];
        for c in formats {
            assert!(is_hidden(c), "{c:?}");
        }
    }

    #[test]
    fn hangul_fillers_shorthand_and_musical_formats_and_tags_are_hidden_marks() {
        // The four Hangul fillers, a shorthand format control, a musical format, the language tag and a tag letter.
        let blanks = [
            '\u{115f}',
            '\u{1160}',
            '\u{3164}',
            '\u{ffa0}',
            '\u{1bca0}',
            '\u{1d173}',
            '\u{e0001}',
            '\u{e0061}',
        ];
        for c in blanks {
            assert!(is_hidden(c), "{c:?}");
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

    #[test]
    fn the_invisible_runs_rise_and_hold_no_control_mark_or_joiner() {
        let marks_or_kept =
            |c: char| c.is_control() || HIDDEN_MARKS.contains(&c) || KEPT.contains(&c);
        for (at, (first, last)) in INVISIBLE.iter().enumerate() {
            assert!(first <= last, "{first:?}");
            assert!(!(*first..=*last).any(marks_or_kept), "{first:?}");
            let next = INVISIBLE.get(at + 1);
            assert!(next.is_none_or(|n| n.0 > *last), "{first:?}");
        }
    }
}

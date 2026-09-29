//! Text of the probe documents: RTF for Word, and numbered lines for the scroll gates.
#![cfg(windows)]

use std::fmt::Write;

/// Ends an RTF paragraph.
const RTF_PAR: &str = r"\par ";
/// End of the RTF file.
const RTF_TAIL: &str = "}";
/// Letters that fill the scroll lines.
const FILLER: &str = "abcdefghijklmnopqrstuvwxyz";

/// `text` as RTF in `font` at `half_points`, one paragraph per line; specials escaped, other than ASCII as `\uN?`.
pub fn rtf(text: &str, font: &str, half_points: u32) -> String {
    let mut out = format!(r"{{\rtf1\ansi\deff0{{\fonttbl{{\f0 {font};}}}}\f0\fs{half_points} ");
    for line in text.lines() {
        for c in line.chars() {
            match c {
                '\\' | '{' | '}' => {
                    out.push('\\');
                    out.push(c);
                }
                c if c.is_ascii() => out.push(c),
                c => {
                    for unit in c.encode_utf16(&mut [0; 2]) {
                        // Writing to a `String` cannot fail.
                        let _ = write!(out, "\\u{}?", unit.cast_signed());
                    }
                }
            }
        }
        out.push_str(RTF_PAR);
    }
    out.push_str(RTF_TAIL);
    out
}

/// `rows` numbered lines of `cols` characters each, for the scroll documents.
pub fn scroll_text(rows: usize, cols: usize) -> String {
    let line = |i: usize| {
        let head = format!("{i:04} ");
        let fill = FILLER.chars().cycle().take(cols.saturating_sub(head.len()));
        head.chars().chain(fill).take(cols).collect::<String>()
    };
    (1..=rows).map(line).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rtf_escapes_specials_and_non_ascii_and_ends_each_line() {
        let got = rtf("a{b}\\c\nلا", "Calibri", 28);
        assert!(got.starts_with(r"{\rtf1\ansi\deff0{\fonttbl{\f0 Calibri;}}\f0\fs28 "));
        assert!(got.ends_with(RTF_TAIL));
        assert!(got.contains(r"a\{b\}\\c\par "));
        let lam_alef = ["\\", "u1604?", "\\", "u1575?", "\\par "].concat();
        assert!(got.contains(&lam_alef));
    }

    #[test]
    fn scroll_text_has_numbered_lines_of_the_asked_width() {
        let text = scroll_text(3, 12);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines, ["0001 abcdefg", "0002 abcdefg", "0003 abcdefg"]);
        assert_eq!(scroll_text(1, 2), "00");
    }
}

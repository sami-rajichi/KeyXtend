//! The text of the fill switch and of kx-gates' scroll documents: numbered lines, each as wide as asked, so a box or a
//! page scrolls both ways. The fill is never logged.

/// Where an edit box breaks a line: it breaks only at CR LF.
pub const LINE_END: &str = "\r\n";
/// Digits in each line's number, and in each of G6's Explorer file names.
pub const NUMBER: usize = 4;
/// Letters that fill each line after its number.
const FILLER: &str = "abcdefghijklmnopqrstuvwxyz";

/// `rows` numbered lines of `cols` characters each, first to last.
#[must_use]
pub fn lines(rows: usize, cols: usize) -> Vec<String> {
    (1..=rows)
        .map(|n| {
            let line = format!("{n:0NUMBER$} ");
            line.chars()
                .chain(FILLER.chars().cycle())
                .take(cols)
                .collect()
        })
        .collect()
}

/// The fill switch's text for an edit box: `lines` joined by `LINE_END`.
#[must_use]
pub fn text(rows: usize, cols: usize) -> String {
    lines(rows, cols).join(LINE_END)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_line_is_numbered_and_as_wide_as_asked() {
        assert_eq!(
            lines(3, 12),
            ["0001 abcdefg", "0002 abcdefg", "0003 abcdefg"]
        );
        assert_eq!(
            lines(1, 2),
            ["00"],
            "a narrow line keeps the number's start"
        );
        assert!(lines(0, 12).is_empty());
    }

    #[test]
    fn the_box_text_breaks_each_line_with_cr_lf() {
        assert_eq!(text(2, 6), "0001 a\r\n0002 a");
    }
}

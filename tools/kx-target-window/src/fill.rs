//! The text of the fill switch and of kx-gates' scroll documents: numbered lines, each as wide as asked, so a box or a
//! page scrolls both ways. Each line's letters start one on from the line above, so a scroll of one line changes the
//! picture. The fill is never logged.

/// Where an edit box breaks a line: it breaks only at CR LF.
pub const LINE_END: &str = "\r\n";
/// Digits in each line's number, and in each of G6's Explorer file names.
pub const NUMBER: usize = 4;
/// Letters that fill each line after its number.
const FILLER: &str = "abcdefghijklmnopqrstuvwxyz";

/// `rows` numbered lines of `cols` characters each, first to last; line `n`'s letters start at `FILLER`'s `n`th.
#[must_use]
pub fn lines(rows: usize, cols: usize) -> Vec<String> {
    (1..=rows)
        .map(|n| {
            let line = format!("{n:0NUMBER$} ");
            let letters = FILLER.chars().cycle().skip((n - 1) % FILLER.len());
            line.chars().chain(letters).take(cols).collect()
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
            ["0001 abcdefg", "0002 bcdefgh", "0003 cdefghi"]
        );
        assert_eq!(
            lines(1, 2),
            ["00"],
            "a narrow line keeps the number's start"
        );
        assert!(lines(0, 12).is_empty());
    }

    #[test]
    fn a_whole_line_shift_changes_every_filler_letter() {
        let rows = lines(100, 40);
        let filler = |l: &String| l.chars().skip(NUMBER + 1).collect::<Vec<_>>();
        for (a, b) in rows.iter().zip(&rows[1..]) {
            let same = filler(a).iter().zip(&filler(b)).any(|(x, y)| x == y);
            assert!(!same, "{a} / {b}");
        }
        assert!(rows[26].starts_with("0027 abc"), "the letters come round");
    }

    #[test]
    fn the_box_text_breaks_each_line_with_cr_lf() {
        assert_eq!(text(2, 6), "0001 a\r\n0002 b");
    }
}

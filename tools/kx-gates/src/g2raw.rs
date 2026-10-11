//! G2's raw click file: one line per click with its scan code, its press and release times and its latency, in µs.
#![cfg(windows)]

use std::fmt::Write;
use std::path::Path;

use crate::stats::Click;

/// Suffix of the raw click file.
pub const CLICKS_FILE: &str = "-clicks.tsv";
/// The raw click file's columns, in order.
const COLUMNS: [&str; 5] = ["index", "code", "click_us", "release_us", "latency_us"];
/// Where the click time sits among them.
const CLICK_AT: usize = 2;

/// The raw click file's first line.
fn header() -> String {
    format!("{}\n", COLUMNS.join("\t"))
}

/// One line per click: index, scan code, press and release times and latency in µs.
pub fn tsv(clicks: &[Click], released: &[i64], lat: &[Option<i64>]) -> String {
    let mut s = header();
    for (i, ((c, r), l)) in clicks.iter().zip(released).zip(lat).enumerate() {
        let l = l.map(|v| v.to_string()).unwrap_or_default();
        // Writing to a `String` cannot fail.
        let _ = writeln!(s, "{i}\t{:#X}\t{}\t{r}\t{l}", c.code, c.us);
    }
    s
}

/// The click times, in QPC µs, in raw click file `text`; an error when G2 did not write it.
pub fn times(text: &str) -> Result<Vec<i64>, String> {
    let mut lines = text.lines();
    if lines.next() != Some(header().trim_end()) {
        return Err("not a G2 click file".to_string());
    }
    let time = |l: &str| l.split('\t').nth(CLICK_AT)?.parse().ok();
    lines
        .map(|l| time(l).ok_or_else(|| "a G2 click line has no time".to_string()))
        .collect()
}

/// The click times, in QPC µs, in the raw click file at `path`.
pub fn read(path: &Path) -> Result<Vec<i64>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    times(&text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tsv_has_a_header_and_one_line_per_click_with_its_press_and_release() {
        let click = |code, us| Click {
            code,
            us,
            expect: vec![],
        };
        let clicks = [click(0x1E, 100), click(0xE01D, 250)];
        let got = tsv(&clicks, &[500, 650], &[Some(30), None]);
        let head = "index\tcode\tclick_us\trelease_us\tlatency_us\n";
        let want = format!("{head}0\t0x1E\t100\t500\t30\n1\t0xE01D\t250\t650\t\n");
        assert_eq!(got, want);
    }

    #[test]
    fn the_click_times_read_back_from_the_file_g2_wrote() {
        let click = |us| Click {
            code: 0x1E,
            us,
            expect: vec![],
        };
        let file = tsv(&[click(100), click(250)], &[500, 650], &[Some(30), None]);
        assert_eq!(times(&file), Ok(vec![100, 250]));
        assert_eq!(times(&header()), Ok(vec![]), "no clicks");
        assert!(times("").is_err(), "no header");
        assert!(
            times("index\tcode\n0\t0x1E\n").is_err(),
            "no click time column"
        );
        assert!(
            times(&file.replacen("100", "x", 1)).is_err(),
            "a time that is no number"
        );
    }
}

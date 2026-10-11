//! G2's raw click file: one line per click with its scan code, its time and its latency, in µs.
#![cfg(windows)]

use std::fmt::Write;

use crate::stats::Click;

/// Suffix of the raw click file.
pub const CLICKS_FILE: &str = "-clicks.tsv";
/// First line of the raw click file.
const TSV_HEADER: &str = "index\tcode\tclick_us\tlatency_us\n";

/// One line per click: index, scan code, click time and latency in µs.
pub fn tsv(clicks: &[Click], lat: &[Option<i64>]) -> String {
    let mut s = String::from(TSV_HEADER);
    for (i, (c, l)) in clicks.iter().zip(lat).enumerate() {
        let l = l.map(|v| v.to_string()).unwrap_or_default();
        // Writing to a `String` cannot fail.
        let _ = writeln!(s, "{i}\t{:#X}\t{}\t{l}", c.code, c.us);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tsv_has_a_header_and_one_line_per_click() {
        let click = |code, us| Click {
            code,
            us,
            expect: vec![],
        };
        let got = tsv(&[click(0x1E, 100), click(0xE01D, 250)], &[Some(30), None]);
        let want = "index\tcode\tclick_us\tlatency_us\n0\t0x1E\t100\t30\n1\t0xE01D\t250\t\n";
        assert_eq!(got, want);
    }
}

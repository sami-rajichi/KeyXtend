//! Palette tokens set after loading from one shared source: a palette that writes one is refused.

use super::*;

/// The shipped file with `line` written into Native light, after its hover.
fn with(line: &str) -> String {
    let at = "hover = \"#0000000F\"";
    crate::THEMES.replacen(at, &format!("{at}\n{line}"), 1)
}

#[test]
fn dim_colours_come_from_the_share_and_cannot_be_written() {
    let e = parse(&with("dim = \"#FFFFFF\"")).expect_err("dim is derived");
    assert!(e.contains("native.light") && e.contains("dim"), "{e}");
}

#[test]
fn the_badge_shadow_comes_from_common_and_cannot_be_written() {
    let e = parse(&with("badge_shadow = []")).expect_err("one shadow for every theme");
    assert!(
        e.contains("native.light") && e.contains("badge_shadow"),
        "{e}"
    );
}

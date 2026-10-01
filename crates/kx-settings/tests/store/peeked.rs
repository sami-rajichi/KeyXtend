//! `peek_section`: a later copy reads one module's section over its defaults, without the store or any file.

#![allow(clippy::unwrap_used, reason = "a failed setup should fail the test")]

use kx_settings::peek_section;
use toml::Table;

use crate::spec::{HOLD, Hold, SPEC};

/// `text` as a settings table.
fn table(text: &str) -> Table {
    text.parse().unwrap()
}

/// The hold section `file` gives, as its minimum and maximum.
fn read(file: Option<&Table>) -> (u32, u32) {
    let h: Hold = peek_section(file, HOLD, &SPEC).unwrap();
    (h.min_ms, h.max_ms)
}

#[test]
fn the_section_is_read_over_the_defaults_and_its_version_is_ignored() {
    assert_eq!(
        read(Some(&table("[hold]\nversion = 3\nmin_ms = 200"))),
        (200, 500)
    );
}

#[test]
fn no_section_or_one_that_does_not_read_gives_the_defaults() {
    let defaults = (100, 500);
    assert_eq!(read(None), defaults);
    for text in [
        "[other]\nmin_ms = 200",
        "[hold]\nmin_ms = 900",
        "[hold]\nmin_ms = \"x\"",
        "[hold]\nno_such = 1",
        "hold = 3",
    ] {
        assert_eq!(read(Some(&table(text))), defaults, "{text}");
    }
}

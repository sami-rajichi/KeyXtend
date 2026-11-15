//! The tested languages, which the gate's bench clips follow; the check clips' languages come from the speech port, so
//! the table names none.

use super::*;

/// The compiled-in text.
const TEXT: &str = include_str!("../../../models.toml");

#[test]
fn the_tested_languages_are_the_ones_with_their_own_order() {
    let tested: Vec<&str> = shipped().tested().collect();
    assert_eq!(tested, ["ar", "en", "fr"]);
}

#[test]
fn a_table_that_names_check_clips_is_refused() {
    let text = format!("clips = [\"en\"]\n{TEXT}");
    assert!(matches!(Table::parse(&text), Err(TableError::Parse(_))));
}

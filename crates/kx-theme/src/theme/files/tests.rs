//! Each theme part's errors name its own file, never the joined text.

use super::*;
use crate::THEME_PARTS;

/// The shipped parts with part `at` written as `text`, and their joined text.
fn with(at: usize, text: &'static str) -> (Vec<Part>, String) {
    let pick = |(i, p): (usize, &Part)| Part {
        text: if i == at { text } else { p.text },
        ..*p
    };
    let parts: Vec<Part> = THEME_PARTS.iter().enumerate().map(pick).collect();
    let joined = parts.iter().map(|p| p.text).collect();
    (parts, joined)
}

#[test]
fn a_part_that_does_not_parse_alone_names_its_own_file() {
    for (at, part) in THEME_PARTS.iter().enumerate() {
        let (parts, joined) = with(at, "[[theme]\n");
        let e = load(&parts, &joined).expect_err("broken");
        assert_eq!(e.file, part.file);
    }
}

#[test]
fn a_reason_names_the_theme_it_starts_with_else_the_shared_colours() {
    let (common, themes) = THEME_PARTS.split_first().expect("the parts");
    let file = |why: String| file_of(THEME_PARTS, &why);
    for p in themes {
        assert_eq!(file(format!("{}.light: missing", p.name)), p.file);
        assert_eq!(file(format!("{}: odd", p.name)), p.file);
        assert_eq!(
            file(format!("{}x: odd", p.name)),
            common.file,
            "a longer id"
        );
    }
    assert_eq!(file("unknown field `x`".into()), common.file);
}

#[test]
fn a_theme_written_twice_names_its_file() {
    let (parts, mut joined) = with(THEME_PARTS.len(), "");
    let twice = &THEME_PARTS[1];
    joined.push_str(twice.text);
    let e = load(&parts, &joined).expect_err("twice");
    assert_eq!(e.file, twice.file, "{}", e.why);
}

#[test]
fn every_part_is_a_file_in_the_themes_folder() {
    for p in THEME_PARTS {
        assert_eq!(p.file, format!("themes/{}.toml", p.name));
    }
}

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

/// `text` kept for the life of the tests, as a part's text is compiled in.
fn kept(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

/// Each theme part with its text changed by `change`, and that part.
fn each_theme(change: impl Fn(&Part) -> String) -> impl Iterator<Item = (Part, Vec<Part>, String)> {
    let themes = THEME_PARTS.iter().enumerate().skip(1);
    themes.map(move |(at, p)| {
        let (parts, joined) = with(at, kept(change(p)));
        (*p, parts, joined)
    })
}

#[test]
fn a_stray_key_or_table_in_a_theme_part_names_that_part_at_its_own_line() {
    for (part, parts, joined) in each_theme(|p| format!("extra = 1\n{}", p.text)) {
        let e = load(&parts, &joined).expect_err("a stray key");
        assert_eq!(e.file, part.file, "{}", e.why);
        assert!(e.why.contains("line 1,"), "{}", e.why);
    }
    for (part, parts, joined) in each_theme(|p| format!("{}\n[common]\n", p.text)) {
        let e = load(&parts, &joined).expect_err("another part's table");
        assert_eq!(e.file, part.file, "{}", e.why);
    }
}

#[test]
fn a_misspelt_or_missing_id_names_its_part() {
    let id = |p: &Part| format!("id = \"{}\"\n", p.name);
    let misspelt = each_theme(|p| p.text.replace(&id(p), "id = \"x\"\n"));
    for (part, parts, joined) in misspelt.chain(each_theme(|p| p.text.replace(&id(p), ""))) {
        let e = load(&parts, &joined).expect_err("not its id");
        assert_eq!(e.file, part.file, "{}", e.why);
    }
}

#[test]
fn a_theme_in_the_shared_colours_names_their_part() {
    let (parts, joined) = with(
        0,
        kept(format!("{}{}", THEME_PARTS[0].text, THEME_PARTS[1].text)),
    );
    let e = load(&parts, &joined).expect_err("a theme in common");
    assert_eq!(e.file, THEME_PARTS[0].file, "{}", e.why);
}

#[test]
fn a_theme_written_twice_names_the_later_part() {
    let (mut parts, _) = with(THEME_PARTS.len(), "");
    let copy = Part {
        file: "themes/copy.toml",
        ..THEME_PARTS[1]
    };
    parts.push(copy);
    let joined: String = parts.iter().map(|p| p.text).collect();
    let e = load(&parts, &joined).expect_err("twice");
    assert_eq!(e.file, copy.file, "{}", e.why);
}

#[test]
fn every_part_is_a_file_in_the_themes_folder() {
    for p in THEME_PARTS {
        assert_eq!(p.file, format!("themes/{}.toml", p.name));
    }
}

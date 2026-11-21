//! The themes' token parts (`tokens/themes/`): joined in a fixed order, they give every theme, mode, token and value the
//! golden file holds, and each part holds only its own table.

use super::parse;
use kx_test_support::golden::assert_golden;

/// The joined themes as the loader reads them; a token change rewrites it with `KX_BLESS=1`.
const GOLDEN: &str = "tests/golden/themes.txt";

#[test]
fn the_joined_parts_give_every_theme_token_and_value_as_before() {
    let (themes, common) = parse(crate::THEMES).expect("the themes parse");
    assert_golden(GOLDEN, &format!("{themes:#?}\n{common:#?}\n"));
}

/// The top-level keys of `part`, which must parse alone.
fn keys(part: &str) -> Vec<String> {
    let table: toml::Table = toml::from_str(part).expect("a part parses alone");
    table.keys().cloned().collect()
}

#[test]
fn each_part_holds_only_its_own_table_in_the_theme_buttons_order() {
    let (common, themes) = crate::THEME_PARTS.split_first().expect("the parts");
    assert_eq!(
        (common.name, keys(common.text)),
        ("common", vec!["common".to_owned()])
    );
    let (list, _) = parse(crate::THEMES).expect("the themes parse");
    let ids: Vec<&str> = list.iter().map(|t| t.look.id.as_str()).collect();
    let names: Vec<&str> = themes.iter().map(|p| p.name).collect();
    assert_eq!(ids, names, "one part per theme, in the button's order");
    for &super::Part { name, text, .. } in themes {
        assert_eq!(keys(text), ["theme"], "{name}");
        let table: toml::Table = toml::from_str(text).expect("a part parses alone");
        let theme = table["theme"].as_array().expect("one [[theme]]");
        assert_eq!(theme.len(), 1, "{name}");
        assert_eq!(theme[0]["id"].as_str(), Some(name), "{name}");
    }
}

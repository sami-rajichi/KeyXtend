//! The tokens a mode table may leave out, against the shipped `themes/*.toml`: each then takes its fallback's value,
//! and a palette writes its own only where the two differ. A token may also name another of its table, whose colour
//! it then takes, so a colour two tokens share is written once.

use super::*;
use crate::rgba::Rgba;

fn text() -> String {
    crate::THEMES.to_string()
}

/// The tokens a light table may leave out, what they then take, and a theme whose light table writes its own.
const LEFT_OUT: [(&str, &str, usize); 3] = [
    ("legend_2_plate", "legend_2", 1),
    ("ring_act", "ring", 2),
    ("rec_light", "rec", 1),
];

/// Dolch, whose light side ring is its lit LED, written by that token's name.
const DOLCH: usize = 2;

#[test]
fn a_palette_without_plate_text_side_ring_or_red_light_takes_its_fallback() {
    for (name, from, at) in LEFT_OUT {
        let (shipped, _) = parse(&text()).expect("themes/*.toml parses");
        let own = &shipped[at].light;
        assert_ne!(
            own.get(name),
            own.get(from),
            "{name}: written where it differs"
        );
        let mut file: toml::Table = toml::from_str(&text()).expect("themes/*.toml parses");
        let themes = file.get_mut("theme").and_then(toml::Value::as_array_mut);
        let light = themes.and_then(|t| t[at].get_mut("light")?.as_table_mut());
        assert!(
            light.and_then(|l| l.remove(name)).is_some(),
            "{name} was there"
        );
        let gone = toml::to_string(&file).expect("the file writes back");
        let (themes, _) = parse(&gone).expect("the token is optional");
        assert_eq!(
            themes[at].light.get(name),
            themes[at].light.get(from),
            "{name}"
        );
    }
}

#[test]
fn plate_text_side_ring_and_red_light_are_written_only_where_they_differ() {
    let file: toml::Table = toml::from_str(&text()).expect("themes/*.toml parses");
    let themes = file["theme"].as_array().expect("a theme list");
    for (t, mode) in themes.iter().flat_map(|t| Mode::ALL.map(|m| (t, m))) {
        let p = &t[mode.key()];
        // A token written as another's name stands for that token's colour.
        let colour = |v: Option<&toml::Value>| {
            let s = v.and_then(toml::Value::as_str)?;
            Some(Rgba::parse(
                p.get(s).and_then(toml::Value::as_str).unwrap_or(s),
            ))
        };
        for (name, from, _) in LEFT_OUT {
            if let Some(own) = p.get(name) {
                let at = format!("{}.{}: {name}", t["id"], mode.key());
                assert_ne!(colour(Some(own)), colour(p.get(from)), "{at}");
            }
        }
    }
}

#[test]
fn a_token_that_names_another_takes_its_colour_and_a_name_of_none_is_refused() {
    let (themes, _) = parse(&text()).expect("themes/*.toml parses");
    let dolch = &themes[DOLCH].light;
    assert_eq!(
        dolch.ring_act, dolch.led_on,
        "Dolch's side ring is its lit LED"
    );
    let file: toml::Table = toml::from_str(&text()).expect("themes/*.toml parses");
    let written = file["theme"][DOLCH]["light"]["ring_act"].as_str();
    assert_eq!(written, Some("led_on"), "written once, by the token's name");
    let named = "ring_act = \"led_on\"";
    let none = text().replacen(named, "ring_act = \"nope\"", 1);
    let e = parse(&none).expect_err("a name of no token");
    assert!(e.contains("dolch.light") && e.contains("nope"), "{e}");
}

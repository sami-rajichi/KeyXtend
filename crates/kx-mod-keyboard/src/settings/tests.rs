use super::*;

/// The defaults as a table.
fn defaults() -> toml::Table {
    SPEC.defaults.parse().unwrap()
}

/// The defaults with `key` set to `value`.
fn with(key: &str, value: impl Into<toml::Value>) -> toml::Table {
    let mut t = defaults();
    t.insert(key.to_owned(), value.into());
    t
}

fn invalid(t: &toml::Table) -> bool {
    matches!(
        validate_as::<KeyboardSettings>(t),
        Err(SettingsError::Invalid(_))
    )
}

fn unparsed(t: &toml::Table) -> bool {
    matches!(
        validate_as::<KeyboardSettings>(t),
        Err(SettingsError::Parse(_))
    )
}

#[test]
fn the_defaults_validate_and_hold_the_agreed_values() {
    assert!(SPEC.is_consistent());
    assert_eq!((SPEC.validate)(&defaults()), Ok(()));
    let s = KeyboardSettings::from_defaults().unwrap();
    assert_eq!(
        (s.modifiers, s.lang_key),
        (Modifiers::Release, LangKey::Carousel)
    );
    assert_eq!(
        (s.size, s.presets.as_slice()),
        (1.0, [1.0, 1.25, 1.5].as_slice())
    );
    assert_eq!((s.size_step, s.size_min, s.size_max), (0.05, 0.8, 1.8));
    assert_eq!((s.relabel_ms, s.key_apps.len()), (500, 0));
    assert_eq!((s.x, s.y, s.placed), (0, 0, false));
    assert_eq!(s.bubble_corner, Corner::Right);
}

#[test]
fn each_choice_takes_only_its_names() {
    assert!(!invalid(&with("modifiers", "stay")) && !unparsed(&with("modifiers", "stay")));
    for (key, good) in [
        ("lang_key", "cycle"),
        ("lang_key", "list"),
        ("bubble_corner", "left"),
    ] {
        assert_eq!(
            validate_as::<KeyboardSettings>(&with(key, good)),
            Ok(()),
            "{key}"
        );
    }
    let bad = [
        ("modifiers", "hold"),
        ("lang_key", "wheel"),
        ("bubble_corner", "top"),
    ];
    for (key, value) in bad {
        assert!(unparsed(&with(key, value)), "{key} = {value}");
    }
}

#[test]
fn the_size_limits_must_make_a_range_above_zero() {
    assert!(invalid(&with("size_min", 1.8)), "min equals max");
    assert!(invalid(&with("size_min", 2.0)), "min above max");
    let mut none = with("size_min", 0.0);
    none.insert("presets".into(), toml::Value::Array(vec![1.0.into()]));
    none.insert("size_max".into(), 1.0.into());
    assert!(invalid(&none), "a size of zero");
}

#[test]
fn the_size_and_every_preset_sit_within_the_limits() {
    assert!(invalid(&with("size", 0.5)));
    assert!(invalid(&with("size", 1.9)));
    assert_eq!(validate_as::<KeyboardSettings>(&with("size", 1.8)), Ok(()));
    let presets = |p: Vec<f64>| with("presets", p);
    assert!(invalid(&presets(vec![1.0, 2.0])));
    assert!(invalid(&presets(vec![])), "no preset");
    assert_eq!(
        validate_as::<KeyboardSettings>(&presets(vec![0.8, 1.8])),
        Ok(())
    );
}

#[test]
fn no_more_presets_than_the_top_bar_names() {
    let presets = |n: usize| with("presets", vec![1.0; n]);
    assert_eq!(
        validate_as::<KeyboardSettings>(&presets(MAX_PRESETS)),
        Ok(())
    );
    assert!(invalid(&presets(MAX_PRESETS + 1)));
}

#[test]
fn the_step_and_the_relabel_time_are_above_zero() {
    assert!(invalid(&with("size_step", 0.0)));
    assert!(invalid(&with("size_step", -0.05)));
    assert!(invalid(&with("relabel_ms", 0)));
    assert!(unparsed(&with("relabel_ms", -1)));
}

#[test]
fn the_relabel_time_has_a_ceiling() {
    let top = i64::try_from(*RELABEL_MS.end()).unwrap();
    let relabel = |ms: i64| validate_as::<KeyboardSettings>(&with("relabel_ms", ms));
    assert_eq!(relabel(top), Ok(()));
    assert!(invalid(&with("relabel_ms", top + 1)));
    assert!(
        invalid(&with("relabel_ms", i64::MAX)),
        "a hand-edited huge value"
    );
}

#[test]
fn a_size_that_is_not_a_number_is_refused() {
    for key in ["size", "size_step", "size_min", "size_max"] {
        for odd in [f64::NAN, f64::INFINITY] {
            assert!(invalid(&with(key, odd)), "{key} = {odd}");
        }
    }
    assert!(invalid(&with("presets", vec![1.0, f64::NAN])));
}

#[test]
fn an_app_name_may_not_be_blank() {
    assert_eq!(
        validate_as::<KeyboardSettings>(&with("key_apps", vec!["app.exe"])),
        Ok(())
    );
    assert!(invalid(&with("key_apps", vec!["app.exe", " "])));
}

#[test]
fn a_missing_or_unknown_key_does_not_parse() {
    let mut missing = defaults();
    missing.remove("size");
    assert!(unparsed(&missing), "size has no default in code");
    assert!(unparsed(&with("poll_ms", 16)), "poll_ms is a UI setting");
    assert!(unparsed(&with("x", 1_i64 << 40)), "x is a pixel");
}

#[test]
fn the_saved_place_holds_only_once_placed() {
    let mut s = KeyboardSettings::from_defaults().unwrap();
    (s.x, s.y) = (-1200, 40);
    assert_eq!(s.place(), None);
    s.placed = true;
    assert_eq!(s.place(), Some(Point { x: -1200, y: 40 }));
}

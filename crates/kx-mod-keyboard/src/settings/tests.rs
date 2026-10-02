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

/// `n` as a float, for small counts.
fn float(n: usize) -> f64 {
    f64::from(u32::try_from(n).unwrap())
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
    let rising = |n: usize| (0..n).map(|i| 1.0 + 0.1 * float(i)).collect::<Vec<f64>>();
    let presets = |n: usize| with("presets", rising(n));
    assert_eq!(
        validate_as::<KeyboardSettings>(&presets(MAX_PRESETS)),
        Ok(())
    );
    assert!(invalid(&presets(MAX_PRESETS + 1)));
}

#[test]
fn size_max_has_a_ceiling_so_a_hand_edited_size_cannot_open_off_every_screen() {
    let top = |max: f64| {
        let mut t = with("size_max", max);
        t.insert("size".into(), max.into());
        t
    };
    assert_eq!(validate_as::<KeyboardSettings>(&top(SIZE_CEILING)), Ok(()));
    assert!(invalid(&top(SIZE_CEILING + 0.05)), "just past the ceiling");
    assert!(invalid(&top(1000.0)), "the reviewer's case");
}

/// The smallest target WCAG 2.2 asks for (SC 2.5.8, Target Size Minimum), in CSS px, which are logical pixels.
const TARGET_PX: f64 = 24.0;

/// The shortest side of any key cell on the shipped board at `size`, in logical pixels.
fn smallest_key(size: f64) -> f64 {
    let b = crate::geom::board(&crate::spec::BoardSpec::shipped().unwrap(), 0.0, size);
    b.keys
        .iter()
        .map(|k| k.cell.w.min(k.cell.h))
        .fold(f64::INFINITY, f64::min)
}

#[test]
fn size_min_has_a_floor_so_a_hand_edited_size_cannot_shrink_the_keys_past_clicking() {
    let low = |min: f64| with("size_min", min);
    assert_eq!(validate_as::<KeyboardSettings>(&low(SIZE_FLOOR)), Ok(()));
    assert!(invalid(&low(SIZE_FLOOR - 0.05)), "just under the floor");
    assert!(invalid(&low(0.01)), "a damaged file's case");
}

#[test]
fn the_floor_is_the_lowest_default_step_where_every_key_is_a_full_target() {
    let step = KeyboardSettings::from_defaults().unwrap().size_step;
    assert!(smallest_key(SIZE_FLOOR) >= TARGET_PX);
    assert!(
        smallest_key(SIZE_FLOOR - step) < TARGET_PX,
        "a step lower, a key is below the target"
    );
}

#[test]
fn the_defaults_name_the_hard_bounds_with_an_example() {
    let bounds = format!("down to size_min = {SIZE_FLOOR} and up to size_max = {SIZE_CEILING}");
    assert!(SPEC.defaults.contains(&bounds), "{bounds}");
}

#[test]
fn the_step_fits_inside_the_limits_and_divides_into_a_finite_grid() {
    let (min, max) = (0.8, 1.8);
    let step = |s: f64| validate_as::<KeyboardSettings>(&with("size_step", s));
    assert_eq!(step(max - min), Ok(()), "one step spans the range");
    assert!(
        invalid(&with("size_step", max - min + 0.05)),
        "wider than the range"
    );
    assert!(invalid(&with("size_step", 5e-324)), "no finite grid");
}

#[test]
fn the_presets_rise_as_the_buttons_name_them() {
    let presets = |p: Vec<f64>| with("presets", p);
    assert!(invalid(&presets(vec![1.25, 1.0])), "falling");
    assert!(invalid(&presets(vec![1.0, 1.0])), "repeated");
    assert_eq!(
        validate_as::<KeyboardSettings>(&presets(vec![0.8, 1.0, 1.8])),
        Ok(())
    );
}

#[test]
fn the_step_and_the_relabel_time_are_above_zero() {
    assert!(invalid(&with("size_step", 0.0)));
    assert!(invalid(&with("size_step", -0.05)));
    assert!(invalid(&with("relabel_ms", 0)));
    assert!(unparsed(&with("relabel_ms", -1)));
}

/// The fastest relabel the settings take, in ms: an engineering bound, as a faster poll only costs CPU time.
const FLOOR_MS: i64 = 50;

#[test]
fn the_relabel_time_has_a_floor_and_a_ceiling() {
    let low = i64::try_from(*RELABEL_MS.start()).unwrap();
    let top = i64::try_from(*RELABEL_MS.end()).unwrap();
    assert_eq!(low, FLOOR_MS);
    let relabel = |ms: i64| validate_as::<KeyboardSettings>(&with("relabel_ms", ms));
    assert_eq!(relabel(low), Ok(()));
    assert!(invalid(&with("relabel_ms", low - 1)));
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

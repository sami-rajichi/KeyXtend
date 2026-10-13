use super::*;
use proptest::prelude::*;

/// Allowed rounding error on a size.
const EPS: f64 = 1e-9;
/// The widest pointer move the tests try, in pixels.
const FAR: i32 = 10_000;

fn settings() -> KeyboardSettings {
    KeyboardSettings::from_defaults().unwrap()
}

fn limits() -> Limits {
    Limits::of(&settings())
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < EPS
}

/// True when `v` is a whole number of steps.
fn on_grid(v: f64) -> bool {
    let n = v / settings().size_step;
    (n - n.round()).abs() < EPS
}

fn at(x: i32, y: i32) -> Point {
    Point { x, y }
}

#[test]
fn steps_move_by_one_step_and_stop_at_the_limits() {
    let (l, s) = (limits(), settings());
    assert!(close(l.up(1.0), 1.05) && close(l.down(1.0), 0.95));
    assert!(
        close(l.up(s.size_max), s.size_max),
        "bigger at the top does nothing"
    );
    assert!(
        close(l.down(s.size_min), s.size_min),
        "smaller at the bottom does nothing"
    );
    let mut size = s.size_min;
    for _ in 0..100 {
        size = l.up(size);
    }
    assert!(close(size, s.size_max), "no drift after many steps: {size}");
}

#[test]
fn a_stepped_size_is_written_as_a_short_decimal() {
    let (l, s) = (limits(), settings());
    let mut size = s.size_min;
    for _ in 0..100 {
        size = l.up(size);
        let text = size.to_string();
        assert!(text.len() <= "0.85".len(), "{text}");
    }
}

#[test]
fn presets_are_s_m_and_l_and_a_size_knows_its_preset() {
    let l = limits();
    assert_eq!(l.presets(), [1.0, 1.25, 1.5]);
    assert_eq!((l.preset(1), l.preset(3)), (Some(1.25), None));
    assert_eq!((l.preset_of(1.25), l.preset_of(1.05)), (Some(1), None));
    assert_eq!(l.preset_of(l.up(l.up(l.up(l.up(l.up(1.0)))))), Some(1));
}

#[test]
fn clamping_keeps_any_value_inside_the_limits() {
    let (l, s) = (limits(), settings());
    assert!(close(l.clamp(5.0), s.size_max) && close(l.clamp(0.0), s.size_min));
    assert!(close(l.clamp(1.3), 1.3));
    let odd = l.clamp(f64::NAN);
    assert!((s.size_min..=s.size_max).contains(&odd), "{odd}");
}

#[test]
fn the_settings_take_only_sizes_inside_their_limits_whatever_the_room() {
    let (mut l, s) = (limits(), settings());
    assert!(l.allows(s.size_min) && l.allows(s.size_max) && l.allows(1.3));
    assert!(!l.allows(s.size_min - s.size_step) && !l.allows(s.size_max + s.size_step));
    assert!(!l.allows(f64::NAN));
    l.room = s.size_min / 2.0;
    assert!(
        l.allows(1.0) && !l.allows(l.clamp(1.0)),
        "a fitted size below the floor"
    );
}

#[test]
fn a_resize_follows_the_pointer_in_steps_and_stops_at_the_limits() {
    let (l, s) = (limits(), settings());
    let r = Resize::begin(1.0, at(100, 100), (600.0, 200.0));
    assert!(close(r.at(at(400, 100), &l), 1.5), "half as wide again");
    assert!(close(r.at(at(100, 150), &l), 1.25), "down grows too");
    assert!(close(r.at(at(FAR, 100), &l), s.size_max));
    assert!(close(r.at(at(-FAR, 100), &l), s.size_min));
    assert!(close(r.at(at(103, 100), &l), 1.0), "snapped to a step");
    assert!(close(r.start(), 1.0));
    let flat = Resize::begin(1.0, at(0, 0), (0.0, 0.0));
    assert!(
        close(flat.at(at(0, 0), &l), 1.0),
        "an empty plate never divides by zero"
    );
}

proptest! {
    #[test]
    fn any_steps_and_presets_stay_on_the_grid_within_the_limits(ops in prop::collection::vec(0u8..4, 0..64)) {
        let (l, s) = (limits(), settings());
        let mut size = s.size;
        for op in ops {
            let before = size;
            size = match op {
                0 => l.up(size),
                1 => l.down(size),
                n => l.preset(usize::from(n)).unwrap_or(size),
            };
            prop_assert!((s.size_min..=s.size_max).contains(&size), "{size}");
            prop_assert!(on_grid(size), "{size}");
            if op == 0 { prop_assert!(size >= before); }
            if op == 1 { prop_assert!(size <= before); }
        }
    }

    #[test]
    fn a_step_up_then_down_comes_back_inside_the_limits(n in 17u32..36) {
        let (l, s) = (limits(), settings());
        let size = f64::from(n) * s.size_step;
        prop_assert!(close(l.down(l.up(size)), size));
    }

    #[test]
    fn a_resize_stays_on_the_grid_for_any_pointer(x in -FAR..FAR, y in -FAR..FAR) {
        let (l, s) = (limits(), settings());
        let size = Resize::begin(1.25, at(0, 0), (840.0, 245.0)).at(at(x, y), &l);
        prop_assert!((s.size_min..=s.size_max).contains(&size), "{size}");
        prop_assert!(on_grid(size), "{size}");
    }
}

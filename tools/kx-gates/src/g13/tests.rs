//! Tests of G13's change names, its pass rule and its backdrop names.

use kx_platform::ui::Accent;

use super::*;

fn change(followed_ms: Option<f64>) -> Change {
    of(&[DARK], followed_ms)
}

fn of(what: &[&'static str], followed_ms: Option<f64>) -> Change {
    Change {
        what: what.to_vec(),
        dark: true,
        contrast: false,
        followed_ms,
        backdrop: "none",
        picture: PathBuf::new(),
    }
}

fn needs(kinds: &[&str]) -> Vec<String> {
    kinds.iter().map(ToString::to_string).collect()
}

#[test]
fn each_part_of_windows_look_is_named_when_it_changes() {
    let light = SysLook::default();
    let dark = SysLook {
        dark: true,
        ..light
    };
    assert_eq!(what_changed(&light, &dark), ["dark"]);
    let accent = SysLook {
        accent: Some(Accent {
            on_light: [0, 95, 184, 255],
            on_dark: [96, 205, 255, 255],
        }),
        ..dark
    };
    assert_eq!(what_changed(&light, &accent), ["dark", "accent"]);
    assert!(what_changed(&accent, &accent).is_empty());
    let moved = SysLook {
        anim_off: true,
        ..light
    };
    assert!(
        what_changed(&light, &moved).is_empty(),
        "animations are not G13's"
    );
}

#[test]
fn g13_passes_when_every_change_was_followed_in_time() {
    let dark = needs(&[DARK]);
    assert!(passed(
        &[change(Some(120.0)), change(Some(0.0))],
        &dark,
        1000.0
    ));
    assert!(
        !passed(&[change(Some(120.0)), change(Some(1500.0))], &dark, 1000.0),
        "too slow"
    );
    assert!(!passed(&[change(None)], &dark, 1000.0), "never redrawn");
    assert!(!passed(&[], &dark, 1000.0), "the owner changed nothing");
}

#[test]
fn a_part_that_changed_while_the_keyboard_followed_joins_that_change() {
    assert_eq!(
        joined(vec![DARK], &[DARK, ACCENT]),
        [DARK, ACCENT],
        "a theme switch flips dark, then the accent"
    );
    assert_eq!(joined(vec![DARK], &[]), [DARK]);
}

#[test]
fn g13_fails_until_each_needed_part_of_the_look_changed() {
    let all = needs(&KINDS);
    let two = [of(&[DARK], Some(100.0)), of(&[DARK, ACCENT], Some(200.0))];
    assert!(!passed(&two, &all, 1000.0), "contrast never changed");
    let three = [two[0].clone(), two[1].clone(), of(&[CONTRAST], Some(300.0))];
    assert!(passed(&three, &all, 1000.0));
}

#[test]
fn every_backdrop_the_product_sets_has_a_name() {
    let names: Vec<&str> = BACKDROPS.iter().map(|(_, n)| *n).collect();
    for kind in ["none", "mica", "acrylic"] {
        assert!(names.contains(&kind), "{kind}");
    }
}

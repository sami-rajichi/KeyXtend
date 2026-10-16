//! Tests of `keymap`.

use super::*;

#[test]
fn a_description_is_the_full_one_else_the_help_text() {
    assert_eq!(description(Some("Fast".into()), Some("x".into())), "Fast");
    assert_eq!(
        description(Some(String::new()), Some("Slow".into())),
        "Slow"
    );
    assert_eq!(description(None, None), "", "a part that says nothing");
}

fn bx(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
    RECT {
        left,
        top,
        right,
        bottom,
    }
}

#[test]
fn a_key_is_aimed_at_the_centre_of_the_box_ui_automation_gives_for_its_id() {
    let keys = KeyMap::new(vec![
        ("bar.grip".into(), bx(0, 0, 400, 24)),
        ("c-0x001E".into(), bx(100, 200, 148, 248)),
    ]);
    let p = keys.at("c-0x001E").expect("found");
    assert_eq!((p.x, p.y), (124, 224));
    assert!(
        keys.at("c-0x001F").is_none(),
        "a key UI Automation did not list"
    );
}

#[test]
fn the_aim_follows_the_box_ui_automation_reports() {
    let at = |down: i32| {
        let keys = KeyMap::new(vec![("esc".into(), bx(8, 30 + down, 40, 62 + down))]);
        keys.at("esc").map(|p| p.y)
    };
    assert_eq!(at(0), Some(46));
    assert_eq!(
        at(24),
        Some(70),
        "the box comes from the keyboard, not from a key map"
    );
}

#[test]
fn a_toggle_state_reads_as_on_or_off_and_a_button_without_one_as_none() {
    use windows::Win32::UI::Accessibility::{ToggleState_Off, ToggleState_On};
    assert_eq!(toggle_state(&VARIANT::from(ToggleState_On.0)), Some(true));
    assert_eq!(toggle_state(&VARIANT::from(ToggleState_Off.0)), Some(false));
    assert_eq!(toggle_state(&VARIANT::default()), None, "no Toggle pattern");
    assert_eq!(
        toggle_state(&VARIANT::from(true)),
        None,
        "not a toggle state"
    );
}

#[test]
fn an_empty_id_names_no_button_so_it_is_refused_before_ui_automation() {
    assert_eq!(wanted("esc"), Ok("esc"));
    let e = wanted("").expect_err("the S, M and L buttons have no id");
    assert!(e.contains("empty"), "{e}");
}

#[test]
fn a_box_with_no_area_is_not_on_screen_and_the_first_of_two_ids_wins() {
    let keys = KeyMap::new(vec![
        ("lang".into(), bx(5, 5, 5, 30)),
        ("esc".into(), bx(0, 0, 10, 10)),
        ("esc".into(), bx(50, 50, 60, 60)),
    ]);
    assert!(keys.at("lang").is_none());
    let p = keys.at("esc").expect("found");
    assert_eq!((p.x, p.y), (5, 5));
    assert!(keys.at("").is_none());
}

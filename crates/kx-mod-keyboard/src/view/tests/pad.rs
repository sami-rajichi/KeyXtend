//! The scroll pad: the board gives its box and name, and the state dims it with a tip until the scroll module runs,
//! then shows the way and the speed it scrolls.

use super::*;
use kx_module_api::ports::Dir;

/// The scroll module running, scrolling towards `dir` (none while still), fast or slow.
fn live(dir: Option<Dir>, fast: bool) -> ScrollChanged {
    ScrollChanged {
        live: true,
        dir,
        fast,
    }
}

fn with(scroll: ScrollChanged) -> StateView {
    state_of(&none(), false, None, AssistChanged::OFF, scroll)
}

#[test]
fn the_board_gives_the_pad_its_box_and_name_only() {
    let v = board_view(&spec(), &settings(), 1.0);
    assert_eq!(v.dpad.name, DPAD_NAME);
    let json = serde_json::to_value(&v.dpad).unwrap();
    let mut fields: Vec<&String> = json.as_object().unwrap().keys().collect();
    fields.sort_unstable();
    assert_eq!(fields, ["h", "name", "w", "x", "y"]);
}

#[test]
fn until_the_scroll_module_runs_the_pad_is_dim_and_says_it_is_off() {
    let stopped = ScrollChanged {
        live: false,
        ..live(Some(Dir::Up), true)
    };
    for s in [with(ScrollChanged::OFF), with(stopped)] {
        let want = PadState {
            dim: true,
            tip: Some(SCROLL_OFF_TIP),
            dir: None,
            fast: false,
        };
        assert_eq!(s.dpad, want);
    }
}

#[test]
fn a_running_scroll_module_shows_the_way_and_the_speed() {
    for (dir, fast) in [
        (None, false),
        (Some(Dir::Down), false),
        (Some(Dir::Left), true),
    ] {
        let s = with(live(dir, fast));
        let want = PadState {
            dim: false,
            tip: None,
            dir,
            fast,
        };
        assert_eq!(s.dpad, want, "{dir:?} {fast}");
    }
}

#[test]
fn the_state_json_names_the_way_in_lowercase() {
    let s = with(live(Some(Dir::Right), true));
    let json = serde_json::to_value(&s).unwrap();
    let want = serde_json::json!({ "dim": false, "tip": null, "dir": "right", "fast": true });
    assert_eq!(json["dpad"], want);
}

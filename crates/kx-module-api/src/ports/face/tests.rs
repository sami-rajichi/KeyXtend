//! The face port: the exact JSON of every action, redacted key ids and the reply defaults.

use super::*;

const KEY: &str = "key-q";
/// The number of `FaceAction` variants, the same in every build; `row` fails to build when one is added.
const VARIANTS: usize = 17;

/// Each variant's place in `wire`, by an exhaustive match.
fn row(action: &FaceAction) -> usize {
    match action {
        FaceAction::Tap { .. } => 0,
        FaceAction::Smaller => 1,
        FaceAction::Bigger => 2,
        FaceAction::Preset { .. } => 3,
        FaceAction::ResizeBegin { .. } => 4,
        FaceAction::ResizeAt { .. } => 5,
        FaceAction::ResizeEnd => 6,
        FaceAction::Placed { .. } => 7,
        FaceAction::Relabel => 8,
        FaceAction::LangPick { .. } => 9,
        FaceAction::PanelKey { .. } => 10,
        FaceAction::Press { .. } => 11,
        FaceAction::Tick => 12,
        FaceAction::LetGo { .. } => 13,
        FaceAction::Leave => 14,
        FaceAction::Pad { .. } => 15,
        FaceAction::PadHover { .. } => 16,
    }
}

/// The tap, placement and panel-key samples, which need built values.
fn built() -> [FaceAction; 3] {
    let rect = Rect {
        x: -8,
        y: 40,
        w: 900,
        h: 300,
    };
    let tap = FaceAction::Tap {
        id: KeyId::new(KEY),
        back: true,
    };
    let screen = Screen {
        rect,
        dpi: 144,
        primary: true,
    };
    let placed = FaceAction::Placed {
        rect,
        screens: vec![screen],
        overhang: 10.5,
    };
    let panel = FaceAction::PanelKey {
        key: KeyId::new(KEY),
        rows: 5,
    };
    [tap, placed, panel]
}

/// The held-key and pad actions, which come last, and their JSON.
fn tail() -> Vec<(FaceAction, &'static str)> {
    let press = FaceAction::Press {
        id: KeyId::new(KEY),
    };
    let let_go = FaceAction::LetGo {
        id: KeyId::new(KEY),
    };
    let up = FaceAction::Pad { dir: Some(Dir::Up) };
    vec![
        (press, r#"{"act":"press","id":"key-q"}"#),
        (FaceAction::Tick, r#"{"act":"tick"}"#),
        (let_go, r#"{"act":"let_go","id":"key-q"}"#),
        (FaceAction::Leave, r#"{"act":"leave"}"#),
        (up, r#"{"act":"pad","dir":"up"}"#),
        (
            FaceAction::PadHover { on: true },
            r#"{"act":"pad_hover","on":true}"#,
        ),
    ]
}

/// Every action and the exact JSON the UI sends for it.
fn wire() -> Vec<(FaceAction, &'static str)> {
    let [tap, placed, panel] = built();
    let mut table = vec![
        (tap, r#"{"act":"tap","id":"key-q","back":true}"#),
        (FaceAction::Smaller, r#"{"act":"smaller"}"#),
        (FaceAction::Bigger, r#"{"act":"bigger"}"#),
        (
            FaceAction::Preset { index: 2 },
            r#"{"act":"preset","index":2}"#,
        ),
        (
            FaceAction::ResizeBegin { x: 10, y: -20 },
            r#"{"act":"resize_begin","x":10,"y":-20}"#,
        ),
        (
            FaceAction::ResizeAt { x: 30, y: 40 },
            r#"{"act":"resize_at","x":30,"y":40}"#,
        ),
        (FaceAction::ResizeEnd, r#"{"act":"resize_end"}"#),
        (
            placed,
            r#"{"act":"placed","rect":{"x":-8,"y":40,"w":900,"h":300},"screens":[{"rect":{"x":-8,"y":40,"w":900,"h":300},"dpi":144,"primary":true}],"overhang":10.5}"#,
        ),
        (FaceAction::Relabel, r#"{"act":"relabel"}"#),
        (
            FaceAction::LangPick { index: 1 },
            r#"{"act":"lang_pick","index":1}"#,
        ),
        (panel, r#"{"act":"panel_key","key":"key-q","rows":5}"#),
    ];
    table.extend(tail());
    table
}

#[test]
fn every_action_has_one_exact_json_form() {
    let table = wire();
    let rows: Vec<usize> = table.iter().map(|(a, _)| row(a)).collect();
    assert_eq!(
        rows,
        (0..VARIANTS).collect::<Vec<_>>(),
        "one row per variant, in order"
    );
    for (action, json) in table {
        assert_eq!(serde_json::to_string(&action).unwrap(), json);
        assert_eq!(serde_json::from_str::<FaceAction>(json).unwrap(), action);
    }
}

#[test]
fn pointer_and_window_places_may_have_fractions() {
    let at = r#"{"act":"resize_at","x":10.5,"y":-0.4}"#;
    let want = FaceAction::ResizeAt { x: 11, y: 0 };
    assert_eq!(serde_json::from_str::<FaceAction>(at).unwrap(), want);
    let screen = r#"{"rect":{"x":0,"y":0,"w":1919.6,"h":1040},"dpi":96,"primary":false}"#;
    let placed = format!(
        r#"{{"act":"placed","rect":{{"x":0.5,"y":1,"w":2,"h":3}},"screens":[{screen}],"overhang":10}}"#
    );
    let FaceAction::Placed { rect, screens, .. } = serde_json::from_str(&placed).unwrap() else {
        panic!("not a placement");
    };
    assert_eq!((rect.x, screens[0].rect.w), (1, 1920));
}

#[test]
fn a_placement_carries_how_far_the_resize_corner_hangs_past_the_plate() {
    let rect = r#"{"x":0,"y":0,"w":2,"h":3}"#;
    let bare = format!(r#"{{"act":"placed","rect":{rect},"screens":[]}}"#);
    assert!(
        serde_json::from_str::<FaceAction>(&bare).is_err(),
        "the overhang is needed"
    );
    let full = format!(r#"{{"act":"placed","rect":{rect},"screens":[],"overhang":10}}"#);
    let FaceAction::Placed { overhang, .. } = serde_json::from_str(&full).unwrap() else {
        panic!("not a placement");
    };
    assert!((overhang - 10.0).abs() < f64::EPSILON);
}

#[test]
fn the_ui_names_the_action_in_act() {
    let tap: FaceAction = serde_json::from_str(r#"{"act":"tap","id":"key-q"}"#).unwrap();
    let want = FaceAction::Tap {
        id: KeyId::new(KEY),
        back: false,
    };
    assert_eq!(tap, want);
    let pick: FaceAction = serde_json::from_str(r#"{"act":"lang_pick","index":3}"#).unwrap();
    assert_eq!(pick, FaceAction::LangPick { index: 3 });
    assert!(serde_json::from_str::<FaceAction>(r#"{"act":"jump"}"#).is_err());
    let gone = r#"{"act":"flip_caps"}"#;
    assert!(
        serde_json::from_str::<FaceAction>(gone).is_err(),
        "ADR-0033: the churn turns Caps Lock over through the layout source"
    );
    let extra = r#"{"act":"preset","index":1,"size":2}"#;
    assert!(serde_json::from_str::<FaceAction>(extra).is_err());
}

#[test]
fn debug_never_shows_which_key_was_pressed() {
    let actions: Vec<FaceAction> = wire().into_iter().map(|(a, _)| a).collect();
    let shown = format!("{actions:?}");
    assert!(!shown.contains(KEY), "{shown}");
    assert!(shown.contains(REDACTED_MARKER));
    assert_eq!(KeyId::new(KEY).as_str(), KEY);
}

#[test]
fn a_reply_redraws_nothing_by_default() {
    let reply = FaceReply::default();
    assert!(!reply.board && !reply.state && reply.notice.is_none() && !reply.holding);
}

#[test]
fn the_face_needs_keyboard_ui() {
    assert_eq!(KeyboardFaceKey::ID.as_str(), "keyboard-face");
    assert_eq!(KeyboardFaceKey::CAPABILITY, Some(Capability::KeyboardUi));
}

#[test]
fn a_panel_key_carries_the_uis_page_size() {
    let bare = r#"{"act":"panel_key","key":"down"}"#;
    assert!(
        serde_json::from_str::<FaceAction>(bare).is_err(),
        "rows is needed"
    );
    let key = r#"{"act":"panel_key","key":"down","rows":3}"#;
    let FaceAction::PanelKey { rows, .. } = serde_json::from_str(key).unwrap() else {
        panic!("not a panel key");
    };
    assert_eq!(rows, 3);
}

#[test]
fn the_pads_stop_has_no_direction() {
    let stop = FaceAction::Pad { dir: None };
    let json = r#"{"act":"pad","dir":null}"#;
    assert_eq!(serde_json::to_string(&stop).unwrap(), json);
    assert_eq!(serde_json::from_str::<FaceAction>(json).unwrap(), stop);
    let bare = serde_json::from_str::<FaceAction>(r#"{"act":"pad"}"#).unwrap();
    assert_eq!(bare, stop, "no direction is Stop");
    let left = serde_json::from_str::<FaceAction>(r#"{"act":"pad","dir":"left"}"#).unwrap();
    assert_eq!(
        left,
        FaceAction::Pad {
            dir: Some(Dir::Left)
        }
    );
    assert!(serde_json::from_str::<FaceAction>(r#"{"act":"pad","dir":"top"}"#).is_err());
}

#[test]
fn a_pad_hover_needs_on() {
    let bare = r#"{"act":"pad_hover"}"#;
    assert!(
        serde_json::from_str::<FaceAction>(bare).is_err(),
        "on is needed"
    );
    let off = serde_json::from_str::<FaceAction>(r#"{"act":"pad_hover","on":false}"#).unwrap();
    assert_eq!(off, FaceAction::PadHover { on: false });
}

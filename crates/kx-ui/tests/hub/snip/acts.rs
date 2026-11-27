//! Clicks on Snip's windows reach the module as `SnipAct`s: keys by id, a press on a frozen screen in physical desktop
//! pixels by that screen's own place and scale, the editor's presses and strokes in the picture's pixels, the theme's
//! fit, the flattened picture (cut in `flat.rs`), the tray's Open, and each screen's work area at a snip's start with
//! `Bare` after it while a capture window's shield was refused.

use super::{editing, heard, idle, picking, rect, screens, shot, strip};
use crate::Rig;
use kx_module_api::Redacted;
use kx_module_api::ports::{
    AskKey, BarKey, CropKey, EditKey, FaceAction, FolderKey, Handle, Ink, Keep, Point, RecKey,
    Scale, Screen, SnipClick, SnipFit, SnipKeyboard, SnipTab, SnipView, StripKey, TextSize,
    Thickness, Tool, VideoRow,
};
use kx_platform::ui::{Screens, UiError};
use kx_ui::snip_acts::{
    ASK, BAR, CARET, CLOSE, CROP, EDIT, FOLDER, HANDLE, INK, KEYBOARD, REC, SIZE, STEP, STRIP, TAB,
    TOOL,
};
use kx_ui::{Mode, Wake};

#[test]
fn each_key_reaches_the_module_by_id_with_the_round_qml_saw() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, strip(9));
    let shown = SnipClick::Keyboard(SnipKeyboard::Shown);
    let row = VideoRow::Longest;
    let cases = [
        (BAR, "video-region", 0, SnipClick::Bar(BarKey::VideoRegion)),
        (STRIP, "copy", 0, SnipClick::Strip(StripKey::Copy)),
        (REC, "cancel", 0, SnipClick::Record(RecKey::Cancel)),
        (TOOL, "highlighter", 0, SnipClick::Tool(Tool::Highlighter)),
        (INK, "white", 0, SnipClick::Ink(Ink::White)),
        (SIZE, "", 1, SnipClick::Size { up: true }),
        (SIZE, "", -1, SnipClick::Size { up: false }),
        (EDIT, "redo", 0, SnipClick::Edit(EditKey::Redo)),
        (STRIP, "play", 0, SnipClick::Strip(StripKey::Play)),
        (STRIP, "speed", 0, SnipClick::Strip(StripKey::Speed)),
        (ASK, "drop", 0, SnipClick::Ask(AskKey::Drop)),
        (TAB, "video", 0, SnipClick::Tab(SnipTab::Video)),
        (KEYBOARD, "shown", 0, shown),
        (STEP, "longest", -1, SnipClick::Step { row, up: false }),
        (FOLDER, "change", 0, SnipClick::Folder(FolderKey::Change)),
        (CLOSE, "", 0, SnipClick::ClosePanel),
        (CARET, "", 3, SnipClick::Caret(Redacted::new(3))),
        (HANDLE, "top-left", 0, SnipClick::Handle(Handle::TopLeft)),
        (CROP, "ok", 0, SnipClick::Crop(CropKey::Ok)),
        (CROP, "cancel", 0, SnipClick::Crop(CropKey::Cancel)),
    ];
    for (kind, id, arg, click) in cases {
        rig.hub.snip_act(kind, id, arg, 8);
        let sent = heard.sent.acts();
        assert_eq!(sent.len(), 1, "{kind} {id}");
        assert_eq!((sent[0].round, &sent[0].click), (8, &click), "{kind} {id}");
    }
}

#[test]
fn a_bad_click_sends_nothing() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, strip(2));
    let bad = [
        ("shout", "copy", 0, 2),
        (STRIP, "paste", 0, 2),
        (SIZE, "", 0, 2),
        ("speed", "", 2, 2),
        ("keep", "save", 0, 2),
        (EDIT, "play", 0, 2),
        (CARET, "", -1, 2),
        (STEP, "fps", 0, 2),
        (HANDLE, "middle", 0, 2),
        (CROP, "crop", 0, 2),
        (STRIP, "copy", 0, -1),
    ];
    for (kind, id, arg, round) in bad {
        rig.hub.snip_act(kind, id, arg, round);
    }
    assert!(heard.sent.acts().is_empty());
}

#[test]
fn a_press_on_a_frozen_screen_goes_in_physical_desktop_pixels_by_that_screens_place_and_scale() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, picking(3, screens()));
    rig.hub.snip_spot(0, 2.4, 3.6, 3);
    rig.hub.snip_spot(1, 2.0, 3.0, 3);
    let want = [
        SnipClick::Spot(Point { x: 2, y: 4 }),
        SnipClick::Spot(Point { x: 8 + 3, y: 5 }),
    ];
    assert_eq!(
        heard.sent.clicks(),
        want,
        "150 % scales the press, not the screen's corner"
    );
    rig.hub.snip_spot(2, 1.0, 1.0, 3);
    rig.hub.snip_spot(-1, 1.0, 1.0, 3);
    assert!(heard.sent.acts().is_empty(), "no such screen");
}

#[test]
fn the_editors_presses_and_strokes_go_in_the_pictures_pixels_with_the_scale_shown() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, editing(5, None));
    rig.hub.snip_press(10.0, 20.0, 0.5, 5);
    rig.hub.snip_press(1.0, 1.0, 0.0004, 5);
    rig.hub.snip_stroke(&[[1.0, 2.0], [3.0, 4.0]], 2.0, 5);
    let want = [
        SnipClick::Press {
            at: Point { x: 20, y: 40 },
            scale: Scale::new(500).unwrap(),
        },
        SnipClick::Press {
            at: Point { x: 2500, y: 2500 },
            scale: Scale::new(1).unwrap(),
        },
        SnipClick::Stroke(vec![Point { x: 1, y: 1 }, Point { x: 2, y: 2 }]),
    ];
    assert_eq!(heard.sent.clicks(), want);
    for scale in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        rig.hub.snip_press(1.0, 1.0, scale, 5);
        rig.hub.snip_stroke(&[[1.0, 1.0]], scale, 5);
    }
    rig.hub.snip_stroke(&[], 1.0, 5);
    assert!(heard.sent.acts().is_empty(), "no scale, or no point");
}

#[test]
fn the_themes_fit_and_the_flattened_picture_reach_the_module() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, editing(6, None));
    let fit = r#"{"strip":44,"bar":40,"gap":8,"thick":[{"pen":2,"marker":12,"disc":20,"head":10}],
        "text":[{"height":18,"letter":8}],"slop":6,"handle":32}"#;
    assert!(rig.hub.snip_fit(fit));
    assert!(
        !rig.hub.snip_fit(r#"{"strip":-1}"#),
        "a fit that does not read"
    );
    rig.hub.snip_flat("copy", shot(2, 2), 6);
    rig.hub.snip_flat("print", shot(2, 2), 6);
    let want = SnipFit {
        strip: 44,
        bar: 40,
        gap: 8,
        thick: vec![Thickness {
            pen: 2,
            marker: 12,
            disc: 20,
            head: 10,
        }],
        text: vec![TextSize {
            height: 18,
            letter: 8,
        }],
        slop: 6,
        handle: 32,
    };
    let sent = heard.sent.acts();
    assert_eq!((sent[0].round, &sent[0].click), (6, &SnipClick::Fit(want)));
    let flat = SnipClick::Flat {
        keep: Keep::Copy,
        shot: shot(2, 2),
    };
    assert_eq!((sent[1].round, &sent[1].click), (6, &flat));
    assert_eq!(sent.len(), 2, "an unknown keep sends nothing");
}

#[test]
fn the_tray_opens_the_panel_and_shows_a_closed_keyboard_first() {
    let rig = Rig::new();
    let heard = heard(&rig);
    rig.hub.snip_open();
    assert_eq!(heard.sent.clicks(), [SnipClick::Open], "before any view");
    rig.hub.keep_mode(Mode::Away);
    rig.wakes();
    rig.hub.snip_open();
    assert_eq!(rig.wakes(), [Wake::Show]);
    assert_eq!(heard.sent.clicks(), [SnipClick::Open]);
}

#[test]
fn each_screens_work_area_goes_at_a_snips_start_and_when_a_screen_changes() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard
        .bus
        .publish(&SnipView::new(picking(1, screens())))
        .unwrap();
    rig.hub.snip_woken();
    let areas = rig.screens.work_areas();
    assert_eq!(heard.sent.clicks(), [SnipClick::Screens(areas.clone())]);
    heard.shown(&rig, picking(2, screens()));
    rig.hub.snip_woken();
    assert!(heard.sent.acts().is_empty(), "once a snip");
    let second = Screen {
        rect: rect(1920, 0, 1280, 1000),
        dpi: 144,
        primary: false,
    };
    rig.screens.set_areas(vec![areas[0], second]);
    rig.hub.snip_screens();
    assert_eq!(
        heard.sent.clicks(),
        [SnipClick::Screens(vec![areas[0], second])]
    );
    heard.shown(&rig, idle(3));
    heard
        .bus
        .publish(&SnipView::new(picking(4, screens())))
        .unwrap();
    rig.hub.snip_woken();
    assert_eq!(heard.sent.acts().len(), 1, "the next snip's start");
}

#[test]
fn bare_follows_the_screens_at_a_snips_start_while_a_capture_window_was_refused() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let areas = rig.screens.work_areas();
    let start = |round| {
        let page = picking(round, screens());
        heard.bus.publish(&SnipView::new(page)).unwrap();
        rig.hub.snip_woken();
        heard.sent.clicks()
    };
    assert_eq!(
        start(1),
        [SnipClick::Screens(areas.clone())],
        "none refused"
    );
    heard.shown(&rig, idle(2));
    rig.guard.set_shield_error(Some(UiError::Refused(5)));
    assert!(!rig.hub.snip_shield("record", "Snip record"));
    assert_eq!(heard.sent.clicks(), [], "no snip runs");
    let bare = [SnipClick::Screens(areas.clone()), SnipClick::Bare];
    assert_eq!(start(3), bare);
    heard.shown(&rig, idle(4));
    rig.guard.set_shield_error(None);
    assert!(rig.hub.snip_shield("record", "Snip record"));
    assert_eq!(start(5), [SnipClick::Screens(areas)], "held again");
}

#[test]
fn a_press_on_a_snip_window_aims_the_keyboards_keys_at_it() {
    let (rig, face, _host) = Rig::with_face();
    let _heard = heard(&rig);
    let _ = rig.hub.aim_snip();
    assert_eq!(face.acts(), [FaceAction::AimSnip]);
}

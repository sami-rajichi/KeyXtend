//! The editor's window (edges 9 and 19): it opens at the theme's default size on the pointer's screen, or where it was
//! last placed, kept on a screen; smaller than its least it stops there; maximised it takes its screen's work area and
//! opens so next time, over its box, or at its default on the main screen once that screen is gone; each place it ends
//! at goes to the module, which keeps it for next time; QML reads where it opens as `{place, back}`; and Snip's windows
//! sit under a system dialog.

use super::{editing, heard, rect, strip};
use crate::Rig;
use kx_module_api::ports::{
    BASE_DPI, Point, Rect, Screen, SnipClick, SnipPage, SnipStage, physical,
};
use kx_platform::ui::Screens;
use kx_ui::EditorAt;
use kx_ui::snip_acts::STRIP;
use serde_json::json;

/// A 150 % screen to the right of the main one.
fn second() -> Screen {
    Screen {
        rect: rect(1920, 0, 2560, 1400),
        dpi: 144,
        primary: false,
    }
}

/// The theme's default and least editor sizes, at 100 %.
fn sizes() -> ([i32; 2], [i32; 2]) {
    let shape = kx_theme::Themes::shipped().unwrap().shape;
    let whole = |v: [f32; 2]| v.map(|n| physical(f64::from(n), BASE_DPI));
    (whole(shape.snip.editor_px), whole(shape.snip.editor_min_px))
}

/// Where the editor opens now, and the box it comes back to if it opens maximised.
fn opens(rig: &Rig) -> Option<(Rect, Option<Rect>)> {
    rig.hub.editor_at().map(|at| (at.place, at.back))
}

/// Round `round` with the editor's box at `place` if kept, maximised over work area `full`.
fn maximised(round: u32, place: Option<Rect>, full: Rect) -> SnipPage {
    let mut page = editing(round, place);
    if let SnipStage::Edit(editor) = &mut page.stage {
        editor.full = Some(full);
    }
    page
}

#[test]
fn a_first_editor_opens_at_its_default_size_centred_on_the_pointers_screen() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, editing(1, None));
    let ([w, h], _) = sizes();
    let first = rect((1920 - w) / 2, (1040 - h) / 2, w, h);
    assert_eq!(opens(&rig), Some((first, None)));
    let main = rig.screens.work_areas()[0];
    rig.screens.set_areas(vec![main, second()]);
    rig.screens.set_cursor(Point { x: 2000, y: 10 });
    let (w, h) = (w * 3 / 2, h * 3 / 2);
    let want = rect(1920 + (2560 - w) / 2, (1400 - h) / 2, w, h);
    assert_eq!(opens(&rig), Some((want, None)), "at that screen's scale");
}

#[test]
fn its_size_and_place_come_back_next_time_kept_on_a_screen() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let kept = rect(100, 80, 1200, 700);
    heard.shown(&rig, editing(1, Some(kept)));
    assert_eq!(opens(&rig), Some((kept, None)));
    heard.shown(&rig, editing(2, Some(rect(1500, 900, 800, 500))));
    let pulled = rect(1120, 540, 800, 500);
    assert_eq!(opens(&rig), Some((pulled, None)), "pulled onto its screen");
    heard.shown(&rig, editing(3, Some(rect(-50, 0, 4000, 3000))));
    let whole = rect(0, 0, 1920, 1040);
    assert_eq!(
        opens(&rig),
        Some((whole, None)),
        "no larger than its screen"
    );
}

#[test]
fn smaller_than_its_least_it_stops_there_and_the_place_goes_to_the_module() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, editing(4, None));
    let (_, [lw, lh]) = sizes();
    assert_eq!(
        rig.hub.editor_fit(rect(100, 100, 300, 200)),
        Some(rect(100, 100, lw, lh))
    );
    assert!(
        heard.sent.acts().is_empty(),
        "a step of the resize tells nothing"
    );
    let placed = rig.hub.editor_placed(rect(100, 100, 300, 200));
    assert_eq!(placed, Some(rect(100, 100, lw, lh)));
    let screens = rig.screens.work_areas();
    let want = SnipClick::Placed {
        rect: rect(100, 100, lw, lh),
        full: None,
        screens,
    };
    assert_eq!(heard.sent.clicks(), [want]);
    let main = rig.screens.work_areas()[0];
    rig.screens.set_areas(vec![main, second()]);
    let at = rig
        .hub
        .editor_fit(rect(2000, 100, 10, 10))
        .map(|r: Rect| (r.w, r.h));
    assert_eq!(
        at,
        Some((lw * 3 / 2, lh * 3 / 2)),
        "its least at that screen's scale"
    );
}

#[test]
fn maximised_it_takes_its_screens_work_area_and_the_module_keeps_its_box_under_it() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let kept = rect(100, 80, 1200, 700);
    heard.shown(&rig, editing(1, Some(kept)));
    let work = rect(0, 0, 1920, 1040);
    assert_eq!(rig.hub.editor_full(kept), Some(work));
    let screens = rig.screens.work_areas();
    let full = SnipClick::Placed {
        rect: kept,
        full: Some(work),
        screens: screens.clone(),
    };
    assert_eq!(heard.sent.clicks(), [full], "the maximise is kept");
    assert_eq!(rig.hub.editor_placed(kept), Some(kept), "the restore");
    let back = SnipClick::Placed {
        rect: kept,
        full: None,
        screens,
    };
    assert_eq!(heard.sent.clicks(), [back], "kept unmaximised");
    rig.screens.set_areas(Vec::new());
    assert_eq!(rig.hub.editor_full(kept), None, "no screen");
    assert_eq!(rig.hub.editor_placed(kept), None);
    assert!(heard.sent.acts().is_empty());
}

#[test]
fn a_maximised_editor_opens_over_its_screens_work_area_as_it_is_now() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let main = rig.screens.work_areas()[0];
    let now = Screen {
        rect: rect(1920, 40, 2560, 1360),
        ..second()
    };
    rig.screens.set_areas(vec![main, now]);
    let kept = rect(2000, 100, 1200, 700);
    heard.shown(&rig, maximised(1, Some(kept), second().rect));
    assert_eq!(opens(&rig), Some((now.rect, Some(kept))), "over its box");
    heard.shown(&rig, maximised(2, None, second().rect));
    let ([w, h], _) = sizes();
    let (w, h) = (w * 3 / 2, h * 3 / 2);
    let back = rect(1920 + (2560 - w) / 2, 40 + (1360 - h) / 2, w, h);
    assert_eq!(
        opens(&rig),
        Some((now.rect, Some(back))),
        "with no box, over its default size centred on that screen"
    );
}

#[test]
fn where_the_editor_opens_reaches_qml_as_its_place_and_the_box_it_comes_back_to() {
    let (place, back) = (rect(0, 0, 1920, 1040), rect(100, 80, 1200, 700));
    let box_of = |r: Rect| json!({"x": r.x, "y": r.y, "w": r.w, "h": r.h});
    let full = serde_json::to_value(EditorAt {
        place,
        back: Some(back),
    })
    .unwrap();
    assert_eq!(full, json!({"place": box_of(place), "back": box_of(back)}));
    let plain = serde_json::to_value(EditorAt { place, back: None }).unwrap();
    assert_eq!(plain, json!({"place": box_of(place), "back": null}));
}

#[test]
fn a_maximised_editor_whose_screen_went_opens_at_its_default_centred_on_the_main_screen() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let main = rig.screens.work_areas()[0];
    let left = Screen {
        rect: rect(-1280, 0, 1280, 984),
        dpi: 120,
        primary: false,
    };
    rig.screens.set_areas(vec![left, main]);
    rig.screens.set_cursor(Point { x: -100, y: 10 });
    let ([w, h], _) = sizes();
    let centred = rect((1920 - w) / 2, (1040 - h) / 2, w, h);
    // The box, from the 150 % screen that went, never sets the size, nor the pointer's screen the place.
    for (round, kept) in [(1, None), (2, Some(rect(2000, 100, 1200, 700)))] {
        heard.shown(&rig, maximised(round, kept, second().rect));
        let at = opens(&rig);
        assert_eq!(
            at,
            Some((centred, None)),
            "at the main screen's scale: {kept:?}"
        );
    }
}

#[test]
fn snips_windows_drop_below_a_system_dialog_while_the_module_says_one_shows() {
    let (rig, _face, _host) = Rig::with_face();
    let heard = heard(&rig);
    let low = |rig: &Rig| super::json(rig)["low"].clone();
    heard.shown(&rig, strip(1));
    rig.hub.snip_act(STRIP, "save", 0, 1);
    assert_eq!(
        low(&rig),
        json!(false),
        "only the module says a dialog shows"
    );
    let mut asking = strip(2);
    asking.dialog = true;
    heard.shown(&rig, asking);
    assert_eq!(low(&rig), json!(true), "the Save dialog sits in front");
    let _ = rig.hub.aim_snip();
    assert_eq!(low(&rig), json!(true), "a press changes nothing");
    heard.shown(&rig, strip(3));
    assert_eq!(low(&rig), json!(false), "it answered: on top again");
}

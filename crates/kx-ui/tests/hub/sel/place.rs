//! Where the strip goes: the theme's gap under the last line and centred on it, above when there is no room or it would
//! cover the keyboard or a panel, at the selection screen's scale, from the strip's measured size; a stale round or an
//! absurd size places nothing, and any numbers land on the work area or nowhere.

use super::{AREA, heard, line, spot, strip, ticked, usual, view};
use crate::Rig;
use kx_module_api::ports::{BASE_DPI, Legend, Point, Rect, Screen, SelChip, StripSpot, physical};
use kx_theme::Themes;
use proptest::collection::vec;
use proptest::prelude::*;

/// The strip's measured size in these tests, in physical pixels.
const W: i32 = 120;
const H: i32 = 40;
/// Random runs per property.
const CASES: u32 = 256;
/// Most boxes QML asks the strip to keep clear of.
const MAX_CLEAR: usize = 3;

/// The theme's gap between the text and the strip at `dpi`, in physical pixels.
fn gap(dpi: u32) -> i32 {
    let px = Themes::shipped().unwrap().shape.pill.gap_px;
    physical(f64::from(px), dpi)
}

/// The keyboard's box, under the usual line.
fn keyboard() -> Rect {
    Rect {
        x: 0,
        y: 330,
        w: AREA.w,
        h: 400,
    }
}

#[test]
fn the_strip_sits_the_gap_under_the_last_line_and_centred_on_it() {
    let rig = Rig::new();
    let (bus, _sent, _subs) = heard(&rig);
    bus.publish(&view(1)).unwrap();
    let want = Point {
        x: 100 + 200 / 2 - W / 2,
        y: 300 + 20 + gap(BASE_DPI),
    };
    assert_eq!(rig.hub.sel_at(1, W, H, &[]), Some(want));
    let far = Rect {
        x: 1500,
        y: 900,
        w: 50,
        h: 50,
    };
    assert_eq!(rig.hub.sel_at(1, W, H, &[far]), Some(want), "clear already");
}

#[test]
fn the_gap_follows_the_selection_screens_dpi() {
    let rig = Rig::new();
    let (bus, _sent, _subs) = heard(&rig);
    let dpi = BASE_DPI * 2;
    bus.publish(&strip(1, spot(100, 300, dpi), None, Legend::Both))
        .unwrap();
    let at = rig.hub.sel_at(1, W, H, &[]).unwrap();
    assert_eq!(at.y, 320 + gap(dpi));
    assert!(gap(dpi) > gap(BASE_DPI), "the theme's gap is scaled");
}

#[test]
fn the_strip_goes_above_the_first_line_over_the_keyboard_or_a_panel() {
    let rig = Rig::new();
    let (bus, _sent, _subs) = heard(&rig);
    let two = StripSpot {
        first: line(100, 260),
        ..usual()
    };
    bus.publish(&strip(1, two, None, Legend::Both)).unwrap();
    let above = Point {
        x: 200 - W / 2,
        y: 260 - gap(BASE_DPI) - H,
    };
    assert_eq!(rig.hub.sel_at(1, W, H, &[keyboard()]), Some(above));
    let panel = Rect {
        x: 150,
        y: 325,
        w: 300,
        h: 200,
    };
    let none = Rect { w: 0, ..keyboard() };
    assert_eq!(
        rig.hub.sel_at(1, W, H, &[none, panel]),
        Some(above),
        "a panel"
    );
}

#[test]
fn with_no_room_below_the_strip_goes_above() {
    let rig = Rig::new();
    let (bus, _sent, _subs) = heard(&rig);
    let low = spot(100, AREA.h - 20, BASE_DPI);
    bus.publish(&strip(1, low, None, Legend::Both)).unwrap();
    let at = rig.hub.sel_at(1, W, H, &[]).unwrap();
    assert_eq!(at.y, AREA.h - 20 - gap(BASE_DPI) - H);
}

#[test]
fn a_ticked_strip_still_places_as_its_done_word_may_widen_it() {
    let rig = Rig::new();
    let (bus, _sent, _subs) = heard(&rig);
    bus.publish(&ticked(3, SelChip::Copy)).unwrap();
    assert!(rig.hub.sel_at(3, W + 20, H, &[]).is_some());
}

#[test]
fn a_stale_round_no_strip_or_an_absurd_size_places_nothing() {
    let rig = Rig::new();
    let (bus, _sent, _subs) = heard(&rig);
    assert_eq!(rig.hub.sel_at(1, W, H, &[]), None, "no strip yet");
    bus.publish(&view(2)).unwrap();
    for round in [1, 3, -2, i64::from(u32::MAX) + 3] {
        assert_eq!(rig.hub.sel_at(round, W, H, &[]), None, "round {round}");
    }
    let sizes = [
        (0, H),
        (W, 0),
        (-W, H),
        (W, -1),
        (AREA.w + 1, H),
        (W, AREA.h + 1),
    ];
    for (w, h) in sizes {
        assert_eq!(rig.hub.sel_at(2, w, h, &[]), None, "{w} by {h}");
    }
    assert!(
        rig.hub.sel_at(2, AREA.w, AREA.h, &[]).is_some(),
        "a whole screen"
    );
}

fn rect() -> impl Strategy<Value = Rect> {
    (any::<i32>(), any::<i32>(), any::<i32>(), any::<i32>()).prop_map(|(x, y, w, h)| Rect {
        x,
        y,
        w,
        h,
    })
}

fn near() -> impl Strategy<Value = Rect> {
    (-50..AREA.w + 50, -50..AREA.h + 50, -2..400, -2..60).prop_map(|(x, y, w, h)| Rect {
        x,
        y,
        w,
        h,
    })
}

fn boxes() -> impl Strategy<Value = Rect> {
    prop_oneof![3 => near(), 1 => rect()]
}

fn screen() -> impl Strategy<Value = Screen> {
    let area = prop_oneof![3 => Just(AREA), 1 => rect()];
    (area, prop_oneof![3 => Just(BASE_DPI), 1 => any::<u32>()]).prop_map(|(rect, dpi)| Screen {
        rect,
        dpi,
        primary: true,
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    /// Whatever boxes, screen, size and boxes to clear come, the strip lands wholly on the work area, or nowhere.
    #[test]
    fn any_numbers_land_on_the_work_area_or_nowhere(
        first in boxes(),
        last in boxes(),
        screen in screen(),
        w in prop_oneof![3 => 1..400_i32, 1 => any::<i32>()],
        h in prop_oneof![3 => 1..60_i32, 1 => any::<i32>()],
        clear in vec(boxes(), 0..=MAX_CLEAR),
    ) {
        let rig = Rig::new();
        let (bus, _sent, _subs) = heard(&rig);
        bus.publish(&strip(1, StripSpot { first, last, screen }, None, Legend::Both)).unwrap();
        if let Some(at) = rig.hub.sel_at(1, w, h, &clear) {
            let a = screen.rect;
            let (right, bottom) = (i64::from(a.x) + i64::from(a.w), i64::from(a.y) + i64::from(a.h));
            prop_assert!(at.x >= a.x && at.y >= a.y, "{:?} {:?}", at, a);
            prop_assert!(i64::from(at.x) + i64::from(w) <= right, "{:?} {:?}", at, a);
            prop_assert!(i64::from(at.y) + i64::from(h) <= bottom, "{:?} {:?}", at, a);
        }
    }
}

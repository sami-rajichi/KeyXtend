//! A region's frame on the frozen screens (plan 2026-11-26 task 4, edges 1-4): each handle picked up and set, the
//! frame kept on the screens and at least `frame_min_px`, a new region from a click away, the adjusted part handed to
//! Copy, Edit, Save and Record, and Esc dropping a held handle before it closes.

use super::*;
use crate::{frame, part};
use kx_module_api::ports::{Grip, Handle};

mod grip;
mod reach;

/// The handle's click area in the tests, in logical pixels: two pixels each way on the left screen.
const HIT: u16 = 4;
/// The strip showing.
fn strip(rig: &Rig) -> &Strip {
    match rig.stage() {
        Stage::Strip(strip) => strip,
        other => panic!("no strip: {other:?}"),
    }
}

/// Where `handle` sits on the part.
fn on(handle: Handle) -> Point {
    frame::spot(PART, handle).unwrap()
}

fn pt(x: i32, y: i32) -> Point {
    Point { x, y }
}

/// A picture's strip whose handles take clicks.
fn framed() -> Rig {
    let mut rig = Rig::new();
    rig.picture();
    rig.on(In::HitPx(HIT));
    rig
}

/// The strip with `rect` taken as the part, a picture's pixels (none for a video) and its screen.
fn holds(rig: &Rig, rect: Rect) {
    let part = Part::new(rect).unwrap();
    let screens = &rig.flow.frozen().unwrap().screens;
    let s = strip(rig);
    assert_eq!(s.part, part);
    let pixels = matches!(s.held, Held::Picture(_)).then(|| part.cut(screens));
    assert_eq!(s.picture(), pixels.flatten().as_ref());
    assert_eq!(Some(s.home), part::home(part, screens));
}

#[test]
fn a_region_shows_its_frame_with_no_handle_held() {
    let rig = framed();
    assert_eq!(strip(&rig).frame, Some(region_frame()));
}

#[test]
fn each_handle_is_picked_up_then_set_by_the_pointers_travel() {
    let mid = pt(PART.x + PART.w / 2, PART.y + PART.h / 2);
    for handle in Handle::ALL {
        let mut rig = framed();
        let at = on(handle);
        assert_eq!(rig.on(In::Spot(at)), [], "{handle:?}");
        let grip = Grip { handle, at };
        assert_eq!(strip(&rig).frame.unwrap().grip, Some(grip));
        holds(&rig, PART);
        // Outwards, three across and two down.
        let to = pt(
            at.x + (at.x - mid.x).signum() * 3,
            at.y + (at.y - mid.y).signum() * 2,
        );
        assert_eq!(rig.on(In::Spot(to)), []);
        let want = frame::moved(PART, grip, to, DESKTOP, rig.settings.frame_min_px).unwrap();
        assert_ne!(want, PART, "{handle:?} moved");
        holds(&rig, want);
        assert_eq!(strip(&rig).frame.unwrap().grip, None, "{handle:?} set");
    }
}

#[test]
fn a_set_handle_moves_its_side_and_the_strip_takes_the_new_part() {
    let mut rig = framed();
    rig.on(In::Spot(on(Handle::Right)));
    rig.on(In::Spot(pt(19, 10)));
    holds(&rig, Rect { w: 15, ..PART });
}

#[test]
fn edge_3_a_set_on_the_pick_up_spot_leaves_the_part_unchanged() {
    let mut rig = framed();
    let near = pt(15, 11);
    rig.on(In::Spot(near));
    assert!(
        strip(&rig).frame.unwrap().grip.is_some(),
        "picked up off its spot"
    );
    rig.on(In::Spot(near));
    holds(&rig, PART);
}

#[test]
fn edge_2_a_frame_stops_at_the_screens_and_keeps_frame_min_px() {
    let mut rig = framed();
    rig.on(In::Spot(pt(6, 8)));
    rig.on(In::Spot(pt(0, 0)));
    holds(
        &rig,
        Rect {
            x: 0,
            y: 0,
            w: 14,
            h: 14,
        },
    );
    let mut rig = framed();
    rig.on(In::Spot(on(Handle::Left)));
    rig.on(In::Spot(pt(13, 10)));
    let min = i32::from(rig.settings.frame_min_px);
    holds(
        &rig,
        Rect {
            x: 14 - min,
            w: min,
            ..PART
        },
    );
}

#[test]
fn edge_2_a_video_regions_frame_keeps_to_its_screen_with_even_sides() {
    let mut rig = Rig::new();
    rig.ready();
    rig.on(In::HitPx(HIT));
    assert_eq!(strip(&rig).frame.unwrap().within, LEFT);
    rig.on(In::Spot(on(Handle::Right)));
    rig.on(In::Spot(pt(RIGHT.x + 20, 10)));
    holds(
        &rig,
        Rect {
            w: LEFT.w - PART.x,
            ..PART
        },
    );
    let bottom = frame::spot(strip(&rig).part.rect(), Handle::Bottom).unwrap();
    rig.on(In::Spot(bottom));
    rig.on(In::Spot(pt(bottom.x, bottom.y + 3)));
    let s = strip(&rig).part.rect();
    assert_eq!((s.h, s.w % 2), (10, 0), "an odd side loses its far pixel");
}

#[test]
fn a_click_away_from_the_frame_starts_a_new_region_from_it() {
    for (video, way) in [(false, BarKey::Region), (true, BarKey::VideoRegion)] {
        let mut rig = Rig::new();
        if video {
            rig.ready();
        } else {
            rig.picture();
        }
        rig.on(In::HitPx(HIT));
        let one = pt(30, 25);
        assert_eq!(rig.on(In::Spot(one)), []);
        let picking = Picking {
            way,
            corner: Some(one),
        };
        assert_eq!(rig.stage(), &Stage::Picking(picking), "{way:?}");
        rig.on(In::Spot(pt(35, 28)));
        assert_eq!(
            strip(&rig).part.rect(),
            Rect {
                x: 30,
                y: 25,
                w: 6,
                h: 4
            }
        );
    }
}

#[test]
fn a_click_inside_the_frame_off_its_handles_does_nothing() {
    let mut rig = framed();
    let before = rig.stage().clone();
    assert_eq!(rig.on(In::Spot(pt(9, 10))), []);
    assert_eq!(rig.stage(), &before);
}

#[test]
fn copy_edit_and_save_take_the_adjusted_picture() {
    let keys = [StripKey::Copy, StripKey::Save, StripKey::Edit];
    for key in keys {
        let mut rig = framed();
        rig.on(In::Spot(on(Handle::Right)));
        rig.on(In::Spot(pt(19, 10)));
        holds(&rig, Rect { w: 15, ..PART });
        let shot = strip(&rig).picture().unwrap().clone();
        let want = match key {
            StripKey::Edit => Out::Edit(shot),
            StripKey::Copy => Out::Keep {
                keep: Keep::Copy,
                what: What::Picture(shot),
            },
            _ => Out::Keep {
                keep: Keep::Save,
                what: What::Picture(shot),
            },
        };
        assert!(rig.on(In::Strip(key)).contains(&want), "{key:?}");
    }
}

#[test]
fn record_takes_the_adjusted_part() {
    let mut rig = Rig::new();
    rig.ready();
    rig.on(In::HitPx(HIT));
    rig.on(In::Spot(on(Handle::Right)));
    rig.on(In::Spot(pt(20, 10)));
    let part = strip(&rig).part;
    rig.on(In::Strip(StripKey::Record));
    let Stage::Count(count) = rig.stage() else {
        panic!("no countdown");
    };
    assert_eq!((count.film.part, part.rect().w), (part, 16));
}

#[test]
fn edge_4_esc_drops_a_held_handle_first_then_closes_as_before() {
    let mut rig = framed();
    rig.on(In::Spot(on(Handle::Top)));
    assert_eq!(rig.on(In::Esc), []);
    assert!(!rig.flow.asking(), "the hold went, no question");
    assert_eq!(strip(&rig).frame.unwrap().grip, None);
    holds(&rig, PART);
    assert_eq!(rig.on(In::Esc), []);
    assert!(rig.flow.asking(), "the unkept picture asks, as before");
    assert!(
        ended(&rig.on(In::Esc)),
        "over frozen screens a second Esc ends it"
    );
}

#[test]
fn a_window_or_a_screen_has_no_frame_and_its_spots_do_nothing() {
    for (way, at) in [(BarKey::Window, INSIDE), (BarKey::Full, INSIDE)] {
        let mut rig = Rig::new();
        rig.picking();
        rig.on(In::Bar(way));
        rig.on(In::Spot(at));
        rig.on(In::HitPx(HIT));
        assert_eq!(strip(&rig).frame, None, "{way:?}");
        let before = rig.stage().clone();
        rig.on(In::Spot(pt(30, 25)));
        assert_eq!(rig.stage(), &before, "{way:?}");
    }
}

#[test]
fn a_frame_takes_no_click_while_a_save_dialog_or_the_question_shows() {
    let mut rig = framed();
    rig.on(In::Strip(StripKey::Save));
    rig.on(In::Spot(on(Handle::Right)));
    assert_eq!(strip(&rig).frame.unwrap().grip, None, "the dialog");
    rig.on(In::Kept(Outcome::Cancelled));
    rig.on(In::Strip(StripKey::Close));
    assert!(rig.flow.asking());
    rig.on(In::Spot(pt(30, 25)));
    assert!(matches!(rig.stage(), Stage::Strip(_)), "the question");
}

//! A region's frame through the actor (plan 2026-11-26 task 4): the view shows the frame and the handle held, hides
//! the strip while one is held, a reader's press picks one up, and the theme's handle size from `SnipClick::Fit` is
//! the click area the frame rules hit within.

use kx_module_api::ports::{Grip, Handle, PanelKey, Point, SnipClick, SnipFit, SnipStage, Taken};

use super::rig::Rig;

/// The theme's handle click area in the tests, in logical pixels: four pixels each way on the 100 % screen.
const HIT: u16 = 8;

/// The strip told last.
fn taken(rig: &Rig) -> Taken {
    let SnipStage::Strip(taken) = rig.stage() else {
        panic!("a strip shows");
    };
    taken
}

/// The theme's sizes with handles `hit` across, and nothing else.
fn fit(hit: u16) -> SnipFit {
    SnipFit {
        strip: 0,
        bar: 0,
        gap: 0,
        thick: Vec::new(),
        text: Vec::new(),
        slop: 0,
        handle: hit,
    }
}

/// A click three pixels right of the right handle of `rig`'s part and one below it, nearer it than any other.
fn by_right(rig: &Rig) -> Point {
    let r = taken(rig).part.rect();
    Point {
        x: r.x + r.w + 3,
        y: r.y + r.h / 2 + 1,
    }
}

#[test]
fn the_view_shows_a_regions_frame_and_the_handle_picked_up_within_the_themes_size() {
    let mut rig = Rig::new();
    rig.act(SnipClick::Fit(fit(HIT)));
    rig.snap();
    let part = taken(&rig).part.rect();
    assert!(taken(&rig).frame.is_some_and(|f| f.grip.is_none()));
    let at = by_right(&rig);
    rig.act(SnipClick::Spot(at));
    let grip = Grip {
        handle: Handle::Right,
        at,
    };
    assert_eq!(taken(&rig).frame.and_then(|f| f.grip), Some(grip));
    rig.act(SnipClick::Spot(Point { x: at.x + 10, ..at }));
    assert_eq!(
        taken(&rig).part.rect().w,
        part.w + 10,
        "the strip's new part"
    );
}

#[test]
fn the_strip_hides_while_a_handle_is_held_and_shows_again_at_the_set_or_the_old_part() {
    let mut rig = Rig::new();
    rig.act(SnipClick::Fit(fit(HIT)));
    rig.snap();
    let (part, keys) = (taken(&rig).part, taken(&rig).keys);
    assert!(!keys.is_empty());
    let at = by_right(&rig);
    rig.act(SnipClick::Spot(at));
    assert!(taken(&rig).keys.is_empty(), "no strip while held");
    rig.line(PanelKey::Esc);
    assert_eq!(
        (taken(&rig).part, taken(&rig).keys),
        (part, keys.clone()),
        "dropped"
    );
    rig.act(SnipClick::Spot(at));
    rig.act(SnipClick::Spot(Point { x: at.x + 10, ..at }));
    let set = taken(&rig);
    assert_eq!(
        (set.part.rect().w, set.keys),
        (part.rect().w + 10, keys),
        "set"
    );
}

#[test]
fn the_hands_esc_drops_a_held_handle_first_then_asks_then_ends_the_snip() {
    let mut rig = Rig::new();
    rig.act(SnipClick::Fit(fit(HIT)));
    rig.snap();
    let part = taken(&rig).part;
    rig.act(SnipClick::Spot(by_right(&rig)));
    rig.act(SnipClick::Esc);
    assert!(rig.page().ask.is_none(), "the hold went, no question");
    let frame = taken(&rig).frame;
    assert_eq!((frame.and_then(|f| f.grip), taken(&rig).part), (None, part));
    rig.act(SnipClick::Esc);
    assert!(rig.page().ask.is_some(), "a picture not kept asks");
    rig.act(SnipClick::Esc);
    assert_eq!(rig.stage(), SnipStage::Idle, "a third Esc ends it");
}

#[test]
fn a_readers_press_on_a_handle_picks_it_up_at_its_spot() {
    let mut rig = Rig::new();
    rig.snap();
    rig.act(SnipClick::Handle(Handle::Left));
    let r = taken(&rig).part.rect();
    let at = Point {
        x: r.x,
        y: r.y + r.h / 2,
    };
    let grip = Grip {
        handle: Handle::Left,
        at,
    };
    assert_eq!(taken(&rig).frame.and_then(|f| f.grip), Some(grip));
}

#[test]
fn before_the_theme_tells_its_size_a_click_off_the_handles_pixel_starts_a_new_region() {
    let mut rig = Rig::new();
    rig.snap();
    let at = by_right(&rig);
    rig.act(SnipClick::Spot(at));
    let SnipStage::Pick(picking) = rig.stage() else {
        panic!("a new region is picked");
    };
    assert_eq!(picking.corner, Some(at));
}

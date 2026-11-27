//! A handle's reach at each screen's scale, and a frame that takes no click while a kept picture's tick ends the snip
//! (task 4 review Minors 4 and 5).

use super::*;

/// A picture region taken on the 150 % right screen, its handles taking clicks.
fn on_right() -> Rig {
    let mut rig = Rig::new();
    rig.picking();
    rig.spot(RIGHT.x + 6, 8);
    rig.spot(RIGHT.x + 19, 17);
    rig.on(In::HitPx(HIT));
    rig
}

#[test]
fn a_handles_reach_follows_the_scale_of_the_screen_holding_the_click() {
    let right = frame::spot(strip(&on_right()).part.rect(), Handle::Right).unwrap();
    // Half of `HIT` is 2 logical pixels: 3 physical at 150 %, 2 at 100 %.
    let mut rig = on_right();
    rig.on(In::Spot(pt(right.x + 3, right.y)));
    let held = strip(&rig).frame.unwrap().grip.map(|g| g.handle);
    assert_eq!(held, Some(Handle::Right), "3 px off at 150 %");
    let mut rig = on_right();
    rig.on(In::Spot(pt(right.x + 4, right.y)));
    assert!(
        matches!(rig.stage(), Stage::Picking(_)),
        "4 px off at 150 %"
    );
    let mut rig = framed();
    let right = on(Handle::Right);
    rig.on(In::Spot(pt(right.x + 3, right.y)));
    assert!(
        matches!(rig.stage(), Stage::Picking(_)),
        "3 px off at 100 %"
    );
}

#[test]
fn while_a_kept_pictures_tick_ends_the_snip_its_frame_takes_no_click() {
    let mut rig = framed();
    rig.on(In::Strip(StripKey::Copy));
    rig.on(In::Kept(Outcome::Done));
    let before = rig.stage().clone();
    rig.on(In::Spot(on(Handle::Right)));
    rig.on(In::Handle(Handle::Top));
    rig.on(In::Spot(pt(30, 25)));
    assert_eq!(
        rig.stage(),
        &before,
        "no handle picked up and no new region"
    );
    assert!(ended(&rig.wait(rig.shown())), "the tick ends the snip");
}

//! A held handle (task 4 fix round 1): the strip's keys cannot act while one is held, as the strip hides; a reader's
//! press picks a handle up by name; the Snip key lets go of it; and a video region keeps an odd `frame_min_px` as the
//! pair above it.

use super::*;

/// The keys of a picture's strip and a video's before it records.
const KEYS: [StripKey; 5] = [
    StripKey::Copy,
    StripKey::Edit,
    StripKey::Save,
    StripKey::Close,
    StripKey::Record,
];

#[test]
fn a_strip_key_cannot_act_while_a_handle_is_held_and_acts_on_the_set_part_after() {
    for key in KEYS {
        let mut rig = framed();
        if key == StripKey::Record {
            rig = Rig::new();
            rig.ready();
            rig.on(In::HitPx(HIT));
        }
        rig.on(In::Spot(on(Handle::Bottom)));
        let before = rig.stage().clone();
        assert_eq!(rig.on(In::Strip(key)), [], "{key:?}");
        assert_eq!(rig.stage(), &before, "{key:?}: still held, part as it was");
    }
    let mut rig = framed();
    rig.on(In::Spot(on(Handle::Bottom)));
    rig.on(In::Strip(StripKey::Copy));
    rig.on(In::Spot(pt(9, 18)));
    holds(&rig, Rect { h: 12, ..PART });
    let what = What::Picture(strip(&rig).picture().unwrap().clone());
    let kept = Out::Keep {
        keep: Keep::Copy,
        what,
    };
    assert_eq!(rig.on(In::Strip(StripKey::Copy)), [kept]);
}

#[test]
fn a_readers_press_picks_a_handle_up_at_its_spot_and_another_takes_its_place() {
    let mut rig = framed();
    assert_eq!(rig.on(In::Handle(Handle::Right)), []);
    let right = Grip {
        handle: Handle::Right,
        at: on(Handle::Right),
    };
    assert_eq!(strip(&rig).frame.unwrap().grip, Some(right));
    rig.on(In::Handle(Handle::Left));
    let left = Grip {
        handle: Handle::Left,
        at: on(Handle::Left),
    };
    assert_eq!(strip(&rig).frame.unwrap().grip, Some(left), "Right dropped");
    holds(&rig, PART);
    rig.on(In::Handle(Handle::Left));
    assert_eq!(strip(&rig).frame.unwrap().grip, Some(left), "pressed again");
    rig.on(In::Spot(pt(1, 10)));
    holds(
        &rig,
        Rect {
            x: 1,
            w: 13,
            ..PART
        },
    );
}

#[test]
fn a_readers_press_does_nothing_without_a_frame_or_while_the_question_shows() {
    let mut rig = Rig::new();
    rig.picking();
    rig.on(In::Bar(BarKey::Window));
    rig.on(In::Spot(INSIDE));
    let before = rig.stage().clone();
    rig.on(In::Handle(Handle::Top));
    assert_eq!(rig.stage(), &before, "a window has no frame");
    let mut rig = framed();
    rig.on(In::Strip(StripKey::Close));
    assert!(rig.flow.asking());
    rig.on(In::Handle(Handle::Top));
    assert_eq!(strip(&rig).frame.unwrap().grip, None, "the question");
}

#[test]
fn the_snip_key_lets_go_of_a_held_handle_so_none_is_held_under_the_question_or_after_back() {
    let mut rig = framed();
    rig.on(In::Spot(on(Handle::Bottom)));
    assert!(strip(&rig).gripped());
    rig.on(In::Key);
    assert!(rig.flow.asking(), "a picture not kept asks first");
    assert_eq!(strip(&rig).frame.unwrap().grip, None, "none held under it");
    rig.on(In::Ask(AskKey::Back));
    assert!(!rig.flow.asking());
    assert!(
        !strip(&rig).gripped(),
        "so the strip shows again after Back"
    );
    holds(&rig, PART);
}

#[test]
fn edge_2_a_video_region_keeps_an_odd_frame_min_px_as_the_pair_above_it() {
    let mut rig = Rig::with(|s| s.frame_min_px = 7);
    rig.ready();
    rig.on(In::HitPx(HIT));
    assert_eq!(strip(&rig).frame.unwrap().min, 8);
    rig.on(In::Spot(on(Handle::Left)));
    rig.on(In::Spot(pt(13, 10)));
    holds(&rig, Rect { x: 6, w: 8, ..PART });
    let mut rig = Rig::with(|s| s.frame_min_px = 7);
    rig.picture();
    assert_eq!(strip(&rig).frame.unwrap().min, 7, "a picture keeps it");
}

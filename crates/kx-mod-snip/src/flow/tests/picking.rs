//! Start, the freeze and picking: the keyboard steps aside before the freeze, Region is ready at once, a way drops the
//! corner, and the Snip key, Esc or Close while picking end with nothing kept (edge 1).

use super::*;

#[test]
fn the_key_lights_then_steps_aside_then_freezes() {
    let mut rig = Rig::new();
    let outs = rig.on(In::Key);
    assert_eq!(outs, [Out::Lit(SnipLit::On), Out::Aside(SnipAside::Bubble)]);
    assert_eq!(rig.on(In::Aside), [Out::Freeze]);
    assert_eq!(rig.stage(), &Stage::Freezing);
}

#[test]
fn hidden_hides_the_keyboard_and_shown_leaves_it() {
    let mut rig = Rig::with(|s| s.keyboard = SnipKeyboard::Hidden);
    let hidden = [Out::Lit(SnipLit::On), Out::Aside(SnipAside::Hidden)];
    assert_eq!(rig.on(In::Key), hidden);
    let mut rig = Rig::with(|s| s.keyboard = SnipKeyboard::Shown);
    assert_eq!(rig.on(In::Key), [Out::Lit(SnipLit::On), Out::Freeze]);
}

#[test]
fn a_freeze_makes_region_ready() {
    let mut rig = Rig::new();
    rig.picking();
    let region = Picking {
        way: BarKey::Region,
        corner: None,
    };
    assert_eq!(rig.stage(), &Stage::Picking(region));
    assert_eq!(rig.flow.frozen(), Some(&frozen()));
}

#[test]
fn a_freeze_past_the_cap_notes_and_ends() {
    let mut rig = Rig::new();
    rig.on(In::Key);
    rig.on(In::Aside);
    let outs = rig.on(In::Frozen(Err(FreezeError::TooBig)));
    assert!(outs.contains(&Out::Note(SnipNote::TooBig)));
    assert!(ended(&outs));
    assert_eq!(rig.stage(), &Stage::Idle);
}

#[test]
fn a_refused_freeze_notes_and_ends() {
    let mut rig = Rig::new();
    rig.on(In::Key);
    rig.on(In::Aside);
    let outs = rig.on(In::Frozen(Err(FreezeError::Refused(5))));
    assert!(
        outs.contains(&Out::Note(SnipNote::FreezeFailed)),
        "{outs:?}"
    );
    assert!(ended(&outs));
    assert_eq!(rig.stage(), &Stage::Idle);
}

#[test]
fn the_key_while_freezing_ends_and_the_late_freeze_is_dropped() {
    let mut rig = Rig::new();
    rig.on(In::Key);
    rig.on(In::Aside);
    assert!(ended(&rig.on(In::Key)));
    assert_eq!(rig.on(In::Frozen(Ok(frozen()))), []);
    assert_eq!(rig.stage(), &Stage::Idle);
    assert_eq!(rig.flow.frozen(), None);
}

#[test]
fn edge_2_a_corner_waits_and_the_same_spot_twice_drops_it() {
    let mut rig = Rig::new();
    rig.picking();
    rig.on(In::Spot(NEAR));
    let waiting = Picking {
        way: BarKey::Region,
        corner: Some(NEAR),
    };
    assert_eq!(rig.stage(), &Stage::Picking(waiting));
    rig.on(In::Spot(NEAR));
    let dropped = Picking {
        corner: None,
        ..waiting
    };
    assert_eq!(rig.stage(), &Stage::Picking(dropped));
    rig.take();
    assert!(matches!(rig.stage(), Stage::Strip(_)), "Region waits again");
}

#[test]
fn a_way_drops_the_corner() {
    let mut rig = Rig::new();
    rig.picking();
    rig.on(In::Spot(NEAR));
    rig.on(In::Bar(BarKey::Window));
    let window = Picking {
        way: BarKey::Window,
        corner: None,
    };
    assert_eq!(rig.stage(), &Stage::Picking(window));
}

#[test]
fn a_one_click_way_takes_no_corner_and_a_miss_takes_nothing() {
    let off_window = Point { x: 30, y: 25 };
    for (way, miss) in [(BarKey::Window, off_window), (BarKey::Full, AWAY)] {
        let mut rig = Rig::new();
        rig.picking();
        rig.on(In::Bar(way));
        assert_eq!(rig.on(In::Spot(miss)), [], "{way:?}");
        let waiting = Picking { way, corner: None };
        assert_eq!(rig.stage(), &Stage::Picking(waiting), "{way:?}");
    }
}

#[test]
fn a_picture_way_takes_the_part_cut_from_the_freeze() {
    let ways = [
        (BarKey::Region, vec![NEAR, FAR], PART),
        (BarKey::Window, vec![INSIDE], PART),
        (BarKey::Full, vec![INSIDE], LEFT),
    ];
    for (way, spots, rect) in ways {
        let mut rig = Rig::new();
        rig.picking();
        rig.on(In::Bar(way));
        let outs: Vec<Out> = spots
            .into_iter()
            .flat_map(|at| rig.on(In::Spot(at)))
            .collect();
        assert_eq!(outs, [], "{way:?}");
        let part = Part::new(rect).unwrap();
        let strip = Strip {
            part,
            home: home(),
            held: Held::Picture(part.cut(&frozen().screens).unwrap()),
            done: None,
            frame: (way == BarKey::Region).then(region_frame),
        };
        assert_eq!(rig.stage(), &Stage::Strip(strip), "{way:?}");
        assert!(rig.flow.frozen().is_some());
    }
}

#[test]
fn a_video_way_gets_ready_with_the_start_switches() {
    for (way, spots) in [
        (BarKey::VideoRegion, vec![NEAR, FAR]),
        (BarKey::VideoFull, vec![INSIDE]),
    ] {
        let mut rig = Rig::with(|s| s.mic_on = true);
        rig.picking();
        rig.on(In::Bar(way));
        for at in spots {
            rig.on(In::Spot(at));
        }
        let sounds = Sounds {
            mic: true,
            sound: false,
        };
        let Stage::Strip(strip) = rig.stage() else {
            panic!("no strip for {way:?}");
        };
        assert_eq!(strip.held, Held::Ready(sounds));
    }
}

#[test]
fn a_video_holds_no_frozen_pixels_on_its_strip_or_its_countdown() {
    let mut rig = Rig::new();
    rig.ready();
    let Stage::Strip(strip) = rig.stage() else {
        panic!("no strip");
    };
    assert_eq!(strip.picture(), None);
    let Held::Ready(sounds) = strip.held else {
        panic!("not ready");
    };
    rig.on(In::Strip(StripKey::Record));
    let film = Film {
        part: part(),
        home: home(),
        sounds,
    };
    assert!(matches!(rig.stage(), Stage::Count(count) if count.film == film));
}

#[test]
fn a_part_on_no_screen_is_not_taken() {
    let mut rig = Rig::new();
    rig.picking();
    rig.on(In::Spot(AWAY));
    rig.spot(AWAY.x + 3, AWAY.y + 3);
    assert!(matches!(rig.stage(), Stage::Picking(_)));
}

#[test]
fn edge_1_the_key_esc_or_close_while_picking_keeps_nothing() {
    for input in [In::Key, In::Esc, In::Bar(BarKey::Close)] {
        let mut rig = Rig::new();
        rig.picking();
        rig.on(In::Spot(NEAR));
        let outs = rig.on(input.clone());
        assert!(ended(&outs), "{input:?}");
        assert!(!kept_any(&outs), "{input:?}");
        assert_eq!(rig.stage(), &Stage::Idle);
        assert_eq!(rig.flow.frozen(), None);
    }
}

#[test]
fn edge_1_shown_has_no_keyboard_to_bring_back() {
    let mut rig = Rig::with(|s| s.keyboard = SnipKeyboard::Shown);
    rig.picking();
    assert_eq!(rig.on(In::Esc), [Out::Lit(SnipLit::Off), Out::End]);
}

#[test]
fn settings_ends_the_snip_then_opens_the_panel() {
    let mut rig = Rig::new();
    rig.picking();
    let outs = rig.on(In::Bar(BarKey::Settings));
    assert!(ended(&outs));
    let (end, panel) = (at(&outs, &Out::End), at(&outs, &Out::Panel));
    assert!(end.zip(panel).is_some_and(|(e, p)| e < p), "{outs:?}");
}

#[test]
fn bar_keys_and_spots_wait_for_picking() {
    let mut rig = Rig::new();
    assert_eq!(rig.on(In::Bar(BarKey::Close)), []);
    assert_eq!(rig.on(In::Spot(INSIDE)), []);
    assert_eq!(rig.on(In::Esc), []);
    assert_eq!(rig.stage(), &Stage::Idle);
}

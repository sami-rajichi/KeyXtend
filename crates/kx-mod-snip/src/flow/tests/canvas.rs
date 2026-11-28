//! The editor's model and the flow together, as the actor joins them: a change the model tells makes the snip unkept,
//! and picks, or an Undo with nothing to take back, leave a kept snip kept.

use super::*;
use crate::edit::{Canvas, Did};
use kx_module_api::ports::{Ink, Scale, SnipFit, TextSize, Thickness, Tool};

/// The picture shown at its own size.
const SAME: Scale = match Scale::new(Scale::PER) {
    Some(scale) => scale,
    None => panic!("a scale of PER is not 0"),
};

/// The theme's sizes: one of each, with no slop, as no click here aims at a drawing.
fn fit() -> SnipFit {
    SnipFit {
        strip: 0,
        bar: 0,
        gap: 0,
        thick: vec![Thickness::default()],
        text: vec![TextSize::default()],
        slop: 0,
        handle: 0,
    }
}

/// The editor on a picture, with its model as the actor makes it when the editor opens, and a Copy kept.
fn kept_editor() -> (Rig, Canvas) {
    let mut rig = Rig::new();
    rig.editor();
    let picture = shot();
    let (undo_max, min) = (rig.settings.undo_max, rig.settings.frame_min_px);
    let canvas = Canvas::new(picture.width(), picture.height(), undo_max, min, fit()).unwrap();
    rig.flat_kept(Keep::Copy);
    (rig, canvas)
}

/// Tells the flow what the model did, as the actor does.
fn tell(rig: &mut Rig, did: Did) {
    if did == Did::Change {
        rig.on(In::Changed);
    }
}

#[test]
fn a_change_the_model_tells_makes_close_ask() {
    let (mut rig, mut canvas) = kept_editor();
    canvas.tool(Tool::Steps);
    let did = canvas.press(Point { x: 1, y: 1 }, SAME);
    tell(&mut rig, did);
    assert_eq!(rig.on(In::Edit(EditKey::Close)), []);
    assert!(rig.flow.asking());
}

#[test]
fn picks_and_an_undo_with_nothing_leave_a_kept_snip_kept() {
    let (mut rig, mut canvas) = kept_editor();
    let dids = [
        canvas.tool(Tool::Box),
        canvas.ink(Ink::Blue),
        canvas.size(true),
        canvas.press(Point { x: 1, y: 1 }, SAME),
        canvas.undo(),
        canvas.undo(),
        canvas.redo(),
    ];
    for did in dids {
        tell(&mut rig, did);
    }
    let outs = rig.on(In::Edit(EditKey::Close));
    assert!(outs.contains(&Out::End), "{outs:?}");
}

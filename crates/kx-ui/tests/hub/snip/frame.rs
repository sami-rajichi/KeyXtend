//! A region's frame and Crop's reach QML (plan 2026-11-26 tasks 4 and 5): its box, its smallest side and the handle
//! held by id with where it was picked up, Crop's with its part, and none without a frame.

use super::{editing, heard, json, rect, strip};
use crate::Rig;
use kx_module_api::ports::{CropFrame, Frame, Grip, Handle, Point, SnipStage};
use serde_json::json as j;

#[test]
fn a_regions_frame_reaches_qml_with_its_box_its_least_side_and_the_handle_held_by_id() {
    let rig = Rig::new();
    let heard = heard(&rig);
    for (handle, id) in [(Handle::TopLeft, "top-left"), (Handle::Right, "right")] {
        let mut page = strip(7);
        if let SnipStage::Strip(taken) = &mut page.stage {
            let at = Point { x: 5, y: 2 };
            let grip = Some(Grip { handle, at });
            let within = rect(0, 0, 8, 6);
            taken.frame = Some(Frame {
                within,
                min: 8,
                grip,
            });
        }
        heard.shown(&rig, page);
        let f = &json(&rig)["strip"]["frame"];
        assert_eq!(f["within"], j!({"x": 0, "y": 0, "w": 8, "h": 6}));
        assert_eq!(f["min"], j!(8));
        assert_eq!(f["grip"], j!({"handle": id, "at": {"x": 5, "y": 2}}));
    }
}

#[test]
fn a_strip_without_a_frame_or_a_handle_held_says_null() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, strip(8));
    assert_eq!(json(&rig)["strip"]["frame"], j!(null));
    let mut page = strip(9);
    if let SnipStage::Strip(taken) = &mut page.stage {
        let within = rect(0, 0, 8, 6);
        taken.frame = Some(Frame {
            within,
            min: 8,
            grip: None,
        });
    }
    heard.shown(&rig, page);
    assert_eq!(json(&rig)["strip"]["frame"]["grip"], j!(null));
}

#[test]
fn crops_frame_reaches_qml_with_its_part_its_box_its_least_side_and_the_handle_held_by_id() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, editing(4, None));
    let none = &json(&rig)["edit"]["picture"]["frame"];
    assert_eq!(none, &j!(null), "none waits");
    let mut page = editing(5, None);
    if let SnipStage::Edit(editor) = &mut page.stage {
        let at = Point { x: 2, y: 3 };
        let grip = Some(Grip {
            handle: Handle::Bottom,
            at,
        });
        let frame = Frame {
            within: rect(0, 0, 4, 3),
            min: 8,
            grip,
        };
        let part = rect(1, 0, 2, 3);
        editor.body.frame = Some(CropFrame { part, frame });
    }
    heard.shown(&rig, page);
    let want = j!({"part": {"x": 1, "y": 0, "w": 2, "h": 3}, "within": {"x": 0, "y": 0, "w": 4, "h": 3}, "min": 8,
        "grip": {"handle": "bottom", "at": {"x": 2, "y": 3}}});
    assert_eq!(json(&rig)["edit"]["picture"]["frame"], want);
}

//! A picture editor with drawings reaches QML as JSON: its tool, ink and steps, each drawing, and the Arabic text being
//! typed, right to left.

use super::{heard, json, page, rect, shot};
use crate::Rig;
use kx_module_api::Redacted;
use kx_module_api::ports::{
    Crop, Dim, Drawing, Editor, Ink, Keep, Mark, PictureEdit, Point, Shape, SnipStage, TextEdit,
    Tool,
};
use serde_json::{Value, json as j};

/// A box, an Arabic text and a step, in blue at sizes 0, 1 and 2.
fn drawings() -> Vec<Drawing> {
    let at = |x, y| Point { x, y };
    let marks = [
        Mark::Shape {
            shape: Shape::Box,
            from: at(1, 2),
            to: at(9, 8),
        },
        Mark::Text {
            at: at(30, 4),
            text: Redacted::new("مرحبا".to_owned()),
            rtl: true,
        },
        Mark::Step {
            at: at(5, 20),
            n: 1,
        },
    ];
    let drawing = |(mark, size)| Drawing {
        mark,
        ink: Ink::Blue,
        size,
    };
    marks.into_iter().zip(0..).map(drawing).collect()
}

/// A picture editor with `drawings`, typing more Arabic with its caret at 1.
pub(super) fn drawn() -> Editor {
    let body = PictureEdit {
        crop: Crop::new(rect(0, 0, 40, 30), 40, 30).unwrap(),
        drawings: drawings(),
        tool: Tool::Text,
        ink: Ink::Red,
        size: 1,
        pending: None,
        frame: None,
        typing: Some(TextEdit {
            at: Point { x: 30, y: 12 },
            text: Redacted::new("نص".to_owned()),
            caret: Redacted::new(1),
            rtl: true,
        }),
        undo: true,
        redo: false,
    };
    Editor {
        picture: shot(40, 30),
        body,
        save_dim: Some(Dim::Hidden),
        done: Some(Keep::Save),
        place: None,
        full: None,
    }
}

/// The view's `edit` once `drawn` shows.
fn edited() -> Value {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(
        &rig,
        page(7, Vec::new(), SnipStage::Edit(Box::new(drawn()))),
    );
    json(&rig)["edit"].clone()
}

#[test]
fn a_picture_editor_carries_its_tool_ink_steps_and_part_kept() {
    let e = edited();
    assert_eq!(
        (e["save_dim"].clone(), e["done"].clone()),
        (j!("hidden"), j!("save"))
    );
    assert_eq!(e.get("video"), None, "the editor has no video page");
    let p = &e["picture"];
    assert_eq!(
        (p["tool"].clone(), p["ink"].clone(), p["size"].clone()),
        (j!("text"), j!("red"), j!(1))
    );
    assert_eq!(
        (p["undo"].clone(), p["redo"].clone()),
        (j!(true), j!(false))
    );
    assert_eq!(p["crop"], j!({"x": 0, "y": 0, "w": 40, "h": 30}));
}

#[test]
fn a_picture_editor_carries_every_drawing_and_the_arabic_text_right_to_left() {
    let p = &edited()["picture"];
    let marks = &p["drawings"];
    assert_eq!(
        marks[0],
        j!({"mark": {"kind": "shape", "shape": "box", "from": {"x": 1, "y": 2}, "to": {"x": 9, "y": 8}}, "ink": "blue", "size": 0})
    );
    assert_eq!(
        marks[1]["mark"],
        j!({"kind": "text", "at": {"x": 30, "y": 4}, "text": "مرحبا", "rtl": true})
    );
    assert_eq!(
        marks[2]["mark"],
        j!({"kind": "step", "at": {"x": 5, "y": 20}, "n": 1})
    );
    assert_eq!(
        p["typing"],
        j!({"at": {"x": 30, "y": 12}, "text": "نص", "caret": 1, "rtl": true})
    );
}

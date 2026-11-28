//! The picture editor: it opens in one view, its clicks reach its model and its changes the flow, none while the close
//! question shows, and the text being typed ends before Copy, Save, Close and the Snip key.

use kx_module_api::Redacted;
use kx_module_api::ports::{
    AskKey, EditKey, Editor, Keep, Mark, PanelKey, PanelLine, Point, Scale, Shot, SnipClick,
    SnipFit, SnipStage, StripKey, TextSize, Thickness, Tool,
};

use super::rig::Rig;

/// The scale a click on the editor's picture was shown at: one window pixel per picture pixel.
fn same() -> Scale {
    Scale::new(Scale::PER).unwrap()
}

/// The editor told last.
fn editor(rig: &Rig) -> Editor {
    let SnipStage::Edit(editor) = rig.stage() else {
        panic!("the editor shows");
    };
    *editor
}

/// The drawings of the editor told last.
fn marks(rig: &Rig) -> Vec<Mark> {
    let drawings = editor(rig).body.drawings;
    drawings.into_iter().map(|d| d.mark).collect()
}

/// The picture of the editor's size, flattened.
fn flat(rig: &Rig) -> Shot {
    let picture = editor(rig).picture;
    let bytes = picture.bgrx().expose().to_vec();
    Shot::new(picture.width(), picture.height(), bytes).unwrap()
}

/// The theme's sizes as the ui sends them at display scale `k`: the editor's steps scale, the rest do not.
fn fit_at(k: u16) -> SnipFit {
    SnipFit {
        strip: 0,
        bar: 0,
        gap: 0,
        thick: vec![Thickness {
            pen: k,
            marker: 2 * k,
            disc: 3 * k,
            head: 4 * k,
        }],
        text: vec![TextSize {
            height: 8 * k,
            letter: 4 * k,
        }],
        slop: 1,
        handle: 0,
    }
}

/// A picture in the editor with a line drawn.
fn drawn() -> Rig {
    let mut rig = Rig::new();
    rig.snap();
    rig.act(SnipClick::Strip(StripKey::Edit));
    rig.act(SnipClick::Tool(Tool::Line));
    rig.act(SnipClick::Press {
        at: Point { x: 1, y: 1 },
        scale: same(),
    });
    rig.act(SnipClick::Press {
        at: Point { x: 9, y: 5 },
        scale: same(),
    });
    rig
}

#[test]
fn edit_opens_the_editor_in_one_view_and_its_fit_at_the_parts_scale_adds_none() {
    let mut rig = Rig::new();
    rig.act(SnipClick::Fit(fit_at(1)));
    rig.snap();
    let before = rig.views().len();
    rig.act(SnipClick::Strip(StripKey::Edit));
    assert_eq!(rig.views().len(), before + 1, "one view opens the editor");
    let picture = editor(&rig).picture;
    assert!(
        picture.width() > 0 && picture.height() > 0,
        "with its picture"
    );
    rig.act(SnipClick::Fit(fit_at(2)));
    assert_eq!(
        rig.views().len(),
        before + 1,
        "the fit sent on open tells no view"
    );
}

#[test]
fn a_pictures_clicks_draw_and_a_change_makes_close_ask() {
    let mut rig = drawn();
    assert_eq!(marks(&rig).len(), 1);
    rig.act(SnipClick::Edit(EditKey::Close));
    assert!(rig.page().ask.is_some(), "unkept since the line");
}

#[test]
fn a_stroke_keeps_its_first_stroke_max_points() {
    let least = i64::from(*crate::STROKE_MAX.start());
    let mut rig = Rig::build(|_, host| host.put("stroke_max", toml::Value::Integer(least)));
    rig.snap();
    rig.act(SnipClick::Strip(StripKey::Edit));
    rig.act(SnipClick::Tool(Tool::Pen));
    let from = Point { x: 0, y: 0 };
    rig.act(SnipClick::Press {
        at: from,
        scale: same(),
    });
    let most = usize::try_from(rig.actor.held.stroke_max).unwrap();
    let two = [Point { x: 1, y: 1 }, Point { x: 2, y: 1 }];
    let path = two.into_iter().cycle().take(most + 1).collect();
    rig.act(SnipClick::Stroke(path));
    let [Mark::Pen(points)] = &marks(&rig)[..] else {
        panic!("one pen stroke: {:?}", marks(&rig));
    };
    assert_eq!(
        points.len(),
        most + 1,
        "its first click and the points kept"
    );
}

#[test]
fn no_drawing_while_the_close_question_shows() {
    let mut rig = drawn();
    rig.act(SnipClick::Edit(EditKey::Close));
    rig.act(SnipClick::Edit(EditKey::Undo));
    rig.act(SnipClick::Press {
        at: Point { x: 2, y: 2 },
        scale: same(),
    });
    rig.act(SnipClick::Ask(AskKey::Back));
    assert_eq!(marks(&rig).len(), 1, "the question is modal");
}

#[test]
fn copy_ends_the_text_being_typed_first_so_it_is_kept() {
    let mut rig = drawn();
    rig.act(SnipClick::Tool(Tool::Text));
    rig.act(SnipClick::Press {
        at: Point { x: 3, y: 3 },
        scale: same(),
    });
    assert_eq!(
        rig.told_line(),
        Some(PanelLine::Private),
        "the text line takes the letters"
    );
    rig.line(PanelKey::Text(Redacted::new("Hi".into())));
    let shot = flat(&rig);
    rig.act(SnipClick::Flat {
        keep: Keep::Copy,
        shot,
    });
    assert_eq!(marks(&rig).len(), 2, "the text joined the drawings");
    assert_eq!(rig.fake.clip.writes().len(), 1);
    rig.act(SnipClick::Edit(EditKey::Close));
    assert_eq!(
        rig.stage(),
        SnipStage::Idle,
        "kept with its text, so Close asks nothing"
    );
}

#[test]
fn the_snip_key_ends_the_text_first_then_asks() {
    let mut rig = Rig::new();
    rig.snap();
    rig.act(SnipClick::Strip(StripKey::Edit));
    rig.act(SnipClick::Tool(Tool::Text));
    rig.act(SnipClick::Press {
        at: Point { x: 3, y: 3 },
        scale: same(),
    });
    rig.line(PanelKey::Text(Redacted::new("ok".into())));
    rig.key();
    assert_eq!(marks(&rig).len(), 1);
    assert!(rig.page().ask.is_some());
}

#[test]
fn esc_ends_the_text_then_closes_the_editor() {
    let mut rig = drawn();
    rig.act(SnipClick::Tool(Tool::Text));
    rig.act(SnipClick::Press {
        at: Point { x: 3, y: 3 },
        scale: same(),
    });
    rig.line(PanelKey::Esc);
    assert_eq!(rig.told_line(), Some(PanelLine::None), "the text ended");
    rig.line(PanelKey::Esc);
    assert!(rig.page().ask.is_some(), "Esc is Close");
    rig.line(PanelKey::Esc);
    assert!(rig.page().ask.is_none(), "and Back at the question");
}

//! Crop's frame through the actor (plan 2026-11-26 task 5): Crop's keys reach the editor's model and its ✓ is a change
//! the flow hears, a waiting frame alone changes nothing, our Esc lets go of a held handle then drops it before it
//! closes, a reader's press on a handle reaches the editor, and nothing acts under the close question.

use kx_module_api::ports::{
    AskKey, CropKey, EditKey, Editor, Grip, Handle, Keep, PanelKey, Point, Rect, Scale, Shot,
    SnipClick, SnipStage, StripKey, Tool,
};

use super::rig::Rig;

/// The part Crop's two clicks frame, both pixels in.
const PART: Rect = Rect {
    x: 1,
    y: 1,
    w: 9,
    h: 5,
};

/// The editor told last.
fn editor(rig: &Rig) -> Editor {
    let SnipStage::Edit(editor) = rig.stage() else {
        panic!("the editor shows");
    };
    *editor
}

/// The whole picture the editor shows.
fn whole(rig: &Rig) -> Rect {
    let picture = editor(rig).picture;
    let side = |len: u32| i32::try_from(len).unwrap();
    Rect {
        x: 0,
        y: 0,
        w: side(picture.width()),
        h: side(picture.height()),
    }
}

/// The editor's picture copied, so the snip is kept.
fn copy(rig: &mut Rig) {
    let picture = editor(rig).picture;
    let bytes = picture.bgrx().expose().to_vec();
    let shot = Shot::new(picture.width(), picture.height(), bytes).unwrap();
    rig.act(SnipClick::Flat {
        keep: Keep::Copy,
        shot,
    });
}

/// A picture in the editor, copied first when `kept`, with Crop's frame round `PART` waiting.
fn framed(kept: bool) -> Rig {
    let mut rig = Rig::new();
    rig.snap();
    rig.act(SnipClick::Strip(StripKey::Edit));
    if kept {
        copy(&mut rig);
    }
    rig.act(SnipClick::Tool(Tool::Crop));
    let scale = Scale::new(Scale::PER).unwrap();
    for at in [Point { x: 1, y: 1 }, Point { x: 9, y: 5 }] {
        rig.act(SnipClick::Press { at, scale });
    }
    assert_eq!(editor(&rig).body.frame.map(|f| f.part), Some(PART));
    rig
}

#[test]
fn a_waiting_frame_changes_nothing_so_close_asks_nothing() {
    let mut rig = framed(true);
    assert_eq!(
        editor(&rig).body.crop.rect(),
        whole(&rig),
        "nothing cut yet"
    );
    rig.act(SnipClick::Edit(EditKey::Close));
    assert_eq!(rig.stage(), SnipStage::Idle, "still kept");
}

#[test]
fn crops_ok_cuts_as_a_change_so_close_asks() {
    let mut rig = framed(true);
    rig.act(SnipClick::Crop(CropKey::Ok));
    let body = editor(&rig).body;
    assert_eq!(
        (body.crop.rect(), body.frame, body.tool),
        (PART, None, Tool::Crop)
    );
    rig.act(SnipClick::Edit(EditKey::Close));
    assert!(rig.page().ask.is_some(), "unkept since the cut");
}

#[test]
fn a_readers_press_picks_a_handle_up_and_cancel_drops_the_frame_once_it_is_set() {
    let mut rig = framed(false);
    rig.act(SnipClick::Handle(Handle::Right));
    let right = Grip {
        handle: Handle::Right,
        at: Point { x: 10, y: 3 },
    };
    let frame = editor(&rig).body.frame.unwrap().frame;
    assert_eq!(frame.grip, Some(right), "the editor's frame takes it");
    rig.act(SnipClick::Crop(CropKey::Cancel));
    assert!(editor(&rig).body.frame.is_some(), "it waits for the set");
    let scale = Scale::new(Scale::PER).unwrap();
    rig.act(SnipClick::Press {
        at: right.at,
        scale,
    });
    rig.act(SnipClick::Crop(CropKey::Cancel));
    let body = editor(&rig).body;
    assert_eq!((body.frame, body.crop.rect()), (None, whole(&rig)));
}

#[test]
fn our_esc_lets_go_of_a_held_handle_then_drops_the_frame_then_closes_the_editor() {
    let mut rig = framed(false);
    rig.act(SnipClick::Handle(Handle::Right));
    rig.line(PanelKey::Esc);
    let frame = editor(&rig).body.frame.map(|f| (f.part, f.frame.grip));
    assert_eq!(frame, Some((PART, None)), "only the hold went");
    rig.line(PanelKey::Esc);
    assert_eq!(editor(&rig).body.frame, None, "the frame went");
    assert!(rig.page().ask.is_none(), "and nothing else");
    rig.line(PanelKey::Esc);
    assert!(rig.page().ask.is_some(), "then Esc is Close");
}

#[test]
fn crops_keys_do_nothing_while_the_close_question_shows() {
    let mut rig = framed(false);
    rig.act(SnipClick::Edit(EditKey::Close));
    rig.act(SnipClick::Crop(CropKey::Ok));
    rig.act(SnipClick::Handle(Handle::Top));
    rig.act(SnipClick::Ask(AskKey::Back));
    let body = editor(&rig).body;
    assert_eq!(body.crop.rect(), whole(&rig), "the question is modal");
    assert_eq!(
        body.frame.map(|f| (f.part, f.frame.grip)),
        Some((PART, None))
    );
}

#[test]
fn our_esc_under_the_close_question_answers_back_and_the_frame_still_waits() {
    let mut rig = framed(false);
    rig.act(SnipClick::Edit(EditKey::Close));
    assert!(rig.page().ask.is_some(), "the question shows");
    rig.line(PanelKey::Esc);
    assert!(rig.page().ask.is_none(), "Esc answered Back");
    let frame = editor(&rig).body.frame.map(|f| f.part);
    assert_eq!(frame, Some(PART), "the frame still waits");
}

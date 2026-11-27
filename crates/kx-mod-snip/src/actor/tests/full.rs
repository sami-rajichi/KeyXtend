//! A maximised editor: kept over its box in one write, it opens maximised on its screen next time; a restore opens it
//! unmaximised again; and with its screen gone the ui is still told its work area, to open it at its default (edge 9).

use kx_module_api::WriteError;
use kx_module_api::ports::{Editor, Rect, Screen, SnipClick, SnipStage, StripKey};

use super::rig::Rig;
use crate::settings::keys;

/// The editor's box on the one screen.
const BOX: Rect = Rect {
    x: 8,
    y: 4,
    w: 40,
    h: 30,
};
/// That screen's work area, which a maximised editor fills.
const WORK: Rect = Rect {
    x: 0,
    y: 0,
    w: 64,
    h: 40,
};
/// A box on a screen to the right, gone by the next snip.
const GONE_BOX: Rect = Rect { x: 72, ..BOX };
/// That screen's work area.
const GONE_WORK: Rect = Rect { x: 64, ..WORK };

/// The editor ended at `rect`, maximised over `full` or not, as the ui tells it.
fn placed(rig: &mut Rig, rect: Rect, full: Option<Rect>) {
    let screens = vec![Screen {
        rect: WORK,
        dpi: 96,
        primary: true,
    }];
    rig.act(SnipClick::Placed {
        rect,
        full,
        screens,
    });
}

/// The editor of a new snip.
fn reopened(rig: &mut Rig) -> Editor {
    rig.snap();
    rig.act(SnipClick::Strip(StripKey::Edit));
    let SnipStage::Edit(editor) = rig.stage() else {
        panic!("the editor shows");
    };
    *editor
}

#[test]
fn a_maximised_editor_opens_maximised_on_its_screen_over_its_box() {
    let mut rig = Rig::new();
    placed(&mut rig, BOX, Some(WORK));
    let saved = rig.saved();
    assert_eq!((saved.editor, saved.editor_full), (Some(BOX), Some(WORK)));
    let editor = reopened(&mut rig);
    assert_eq!((editor.place, editor.full), (Some(BOX), Some(WORK)));
}

#[test]
fn a_restored_editor_opens_unmaximised_at_its_box() {
    let mut rig = Rig::new();
    placed(&mut rig, BOX, Some(WORK));
    placed(&mut rig, BOX, None);
    assert_eq!(rig.saved().editor_full, None);
    let editor = reopened(&mut rig);
    assert_eq!((editor.place, editor.full), (Some(BOX), None));
}

#[test]
fn a_maximised_editor_whose_screen_went_still_tells_its_work_area_without_its_box() {
    let mut rig = Rig::new();
    placed(&mut rig, GONE_BOX, Some(GONE_WORK));
    let editor = reopened(&mut rig);
    let want = (None, Some(GONE_WORK));
    assert_eq!(
        (editor.place, editor.full),
        want,
        "the ui finds no screen for it"
    );
}

#[test]
fn a_refused_write_keeps_the_box_and_the_work_area_as_they_were() {
    let mut rig = Rig::new();
    placed(&mut rig, BOX, None);
    let refused = WriteError::Refused(crate::ID);
    rig.host.sink.refuse_key(keys::EDITOR_FULL, refused);
    placed(&mut rig, Rect { x: 12, ..BOX }, Some(WORK));
    let saved = rig.saved();
    assert_eq!(
        (saved.editor, saved.editor_full),
        (Some(BOX), None),
        "all or none"
    );
    let editor = reopened(&mut rig);
    assert_eq!(
        (editor.place, editor.full),
        (Some(BOX), None),
        "nor its own copy"
    );
}

#[test]
fn an_unmaximised_editor_whose_screen_went_opens_at_its_default() {
    let mut rig = Rig::new();
    placed(&mut rig, GONE_BOX, None);
    assert_eq!(reopened(&mut rig).place, None);
}

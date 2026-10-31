//! Joins only before an entry in hand: an entry found gone gets none, none trails the last, and an entry read
//! before it is deleted still follows its join.

use kx_module_api::ports::{ClipAct, ClipNote, ClipPart};

use super::super::Msg;
use super::rig::Rig;
use crate::fixture::{app, image, text};

/// The plan's join: two line breaks.
const JOIN: &str = "\n\n";

/// Ticks `rows` in order, then clicks Insert.
fn insert(rig: &mut Rig, rows: &[u8]) {
    for &row in rows {
        rig.tick_row(row);
    }
    rig.send(Msg::Act(ClipAct::Insert));
}

/// Answers each paste and waits its gap, until no new one is asked.
fn paste_all(rig: &mut Rig) {
    loop {
        let asked = rig.pastes().len();
        rig.answer(true);
        rig.advance(rig.settings.paste_gap());
        if rig.pastes().len() == asked {
            return;
        }
    }
}

/// A rig holding text copies of `texts`, the last at the top, with the panel open.
fn opened(texts: &[&str]) -> Rig {
    let mut rig = Rig::new();
    for t in texts {
        rig.text(t);
    }
    rig.open();
    rig
}

#[test]
fn a_text_found_gone_between_two_texts_leaves_one_join() {
    let mut rig = opened(&["a", "b", "c"]);
    rig.lose("b");
    insert(&mut rig, &[2, 1, 0]);
    paste_all(&mut rig);
    let want = [vec![text("a")], vec![text(JOIN)], vec![text("c")]];
    assert_eq!(rig.fakes.writer.writes(), want);
}

#[test]
fn a_last_entry_found_gone_leaves_no_join_and_the_last_paste_stays() {
    let mut rig = opened(&["a", "b"]);
    rig.lose("b");
    insert(&mut rig, &[1, 0]);
    paste_all(&mut rig);
    assert_eq!(rig.fakes.writer.writes(), [vec![text("a")]]);
    assert_eq!(rig.panels().last(), Some(&false), "the panel closes");
}

#[test]
fn an_image_found_gone_between_two_texts_leaves_their_join() {
    let mut rig = Rig::new();
    rig.text("a");
    rig.copy(vec![image()], None);
    rig.text("c");
    rig.open();
    rig.lose("");
    insert(&mut rig, &[2, 1, 0]);
    paste_all(&mut rig);
    let want = [vec![text("a")], vec![text(JOIN)], vec![text("c")]];
    assert_eq!(rig.fakes.writer.writes(), want);
}

#[test]
fn a_text_deleted_after_it_was_read_still_follows_its_join_and_no_join_precedes_the_image() {
    let mut rig = Rig::new();
    let notes = app("notes.exe");
    rig.copy(vec![image()], None);
    rig.text("a");
    rig.copy(vec![text("b")], Some(notes.clone()));
    rig.open();
    insert(&mut rig, &[1, 0, 2]);
    rig.answer(true);
    rig.advance(rig.settings.paste_gap());
    assert_eq!(rig.fakes.writer.writes().last(), Some(&vec![text(JOIN)]));
    rig.push(ClipNote::Cleared { app: Some(notes) });
    paste_all(&mut rig);
    let want: [Vec<ClipPart>; 4] = [
        vec![text("a")],
        vec![text(JOIN)],
        vec![text("b")],
        vec![image()],
    ];
    assert_eq!(rig.fakes.writer.writes(), want);
}

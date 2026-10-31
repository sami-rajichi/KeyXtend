//! Inserting: tick order with one paste request per step, the join pasted as text, the gap, the last entry staying
//! and moving up, the notes that stop it, file entries gone or out of reach, and a close that stops it.

use std::path::PathBuf;
use std::time::Duration;

use kx_module_api::Redacted;
use kx_module_api::ports::{ClipAct, ClipError, ClipPart, FileFact, PanelNote};
use toml::Value;

use super::super::Msg;
use super::rig::Rig;
use crate::fixture::{files, text};

/// A rig holding text copies of `texts`, the last at the top, with the panel open.
fn opened(texts: &[&str]) -> Rig {
    let mut rig = Rig::new();
    for t in texts {
        rig.text(t);
    }
    rig.open();
    rig
}

/// Ticks `rows` in order, then clicks Insert.
fn insert(rig: &mut Rig, rows: &[u8]) {
    for &row in rows {
        rig.tick_row(row);
    }
    rig.send(Msg::Act(ClipAct::Insert));
}

/// Answers the last paste and waits the gap.
fn pasted(rig: &mut Rig) {
    rig.answer(true);
    rig.advance(rig.settings.paste_gap());
}

#[test]
fn ticked_entries_go_in_tick_order_with_one_paste_request_each() {
    let mut rig = opened(&["a", "b", "c"]);
    insert(&mut rig, &[2, 0]);
    assert_eq!(rig.fakes.writer.writes(), [vec![text("a")]]);
    assert_eq!(rig.pastes().len(), 1);
    pasted(&mut rig);
    assert_eq!(rig.pastes().len(), 2);
    pasted(&mut rig);
    let writes = rig.fakes.writer.writes();
    assert_eq!(
        writes,
        [vec![text("a")], vec![text("\n\n")], vec![text("c")]]
    );
    assert_eq!(rig.pastes().len(), 3, "one request per step");
    pasted(&mut rig);
    assert_eq!(rig.panels().last(), Some(&false), "the panel then closes");
    assert_eq!(
        rig.fakes.writer.writes().last(),
        Some(&vec![text("c")]),
        "the last entry stays"
    );
}

#[test]
fn the_join_is_pasted_text_with_its_rule_never_a_key() {
    let mut rig = Rig::build(|_, host| {
        host.put("join_breaks", Value::Integer(1));
        host.put("join_rule", Value::String("— ✦ —".to_owned()));
    });
    rig.text("a");
    rig.text("b");
    rig.open();
    insert(&mut rig, &[0, 1]);
    pasted(&mut rig);
    assert_eq!(rig.fakes.writer.writes()[1], [text("\n— ✦ —\n")]);
    assert_eq!(rig.pastes().len(), rig.fakes.writer.writes().len());
}

#[test]
fn each_step_waits_for_its_paste_then_paste_gap_ms() {
    let mut rig = opened(&["a", "b"]);
    insert(&mut rig, &[0, 1]);
    rig.advance(rig.settings.paste_gap());
    assert_eq!(rig.fakes.writer.tries(), 1, "no step before the paste went");
    rig.answer(true);
    rig.advance(
        rig.settings
            .paste_gap()
            .checked_sub(Duration::from_millis(1))
            .unwrap(),
    );
    assert_eq!(rig.fakes.writer.tries(), 1, "the gap");
    rig.advance(Duration::from_millis(1));
    assert_eq!(rig.fakes.writer.tries(), 2);
}

#[test]
fn an_inserted_entry_moves_to_the_top() {
    let mut rig = opened(&["a", "b"]);
    insert(&mut rig, &[1]);
    pasted(&mut rig);
    rig.open();
    assert_eq!(rig.titles(), ["a", "b"]);
}

#[test]
fn a_busy_clipboard_stops_the_insert_with_a_note() {
    let mut rig = opened(&["a", "b"]);
    rig.fakes.writer.queue(Err(ClipError::Busy));
    insert(&mut rig, &[0, 1]);
    assert!(rig.pastes().is_empty());
    assert_eq!(rig.page().note, Some(PanelNote::Busy));
    assert_eq!(rig.panels().last(), Some(&true), "open to show the note");
}

#[test]
fn a_paste_that_failed_or_never_came_stops_with_a_note() {
    let mut rig = opened(&["a", "b"]);
    insert(&mut rig, &[0, 1]);
    rig.answer(false);
    assert_eq!(rig.page().note, Some(PanelNote::PasteFailed));
    rig.send(Msg::Act(ClipAct::Insert));
    assert_eq!(rig.page().note, None, "a new insert clears it");
    rig.advance(rig.settings.paste_wait());
    assert_eq!(rig.page().note, Some(PanelNote::PasteFailed));
    assert_eq!(rig.fakes.writer.tries(), 2, "no step after the wait");
}

#[test]
fn a_file_entry_pastes_its_files_still_there_and_leaves_when_all_are_gone() {
    let mut rig = Rig::new();
    for path in ["C:\\a", "C:\\b", "C:\\c"] {
        rig.fakes.clip.set_file(path, FileFact::Size(1));
    }
    rig.copy(vec![files(&["C:\\c"])], None);
    rig.copy(vec![files(&["C:\\a", "C:\\b"])], None);
    rig.open();
    rig.fakes.clip.set_file("C:\\b", FileFact::Gone);
    rig.fakes.clip.set_file("C:\\c", FileFact::Gone);
    insert(&mut rig, &[1, 0]);
    let kept = ClipPart::Files(Redacted::new(vec![PathBuf::from("C:\\a")]));
    assert_eq!(rig.fakes.writer.writes(), [vec![kept]]);
    assert_eq!(
        rig.titles(),
        ["a b"],
        "the entry whose files are all gone left"
    );
}

#[test]
fn a_file_out_of_reach_is_skipped_and_kept() {
    let mut rig = Rig::new();
    rig.fakes.clip.set_file("E:\\song.mp3", FileFact::Size(9));
    rig.copy(vec![files(&["E:\\song.mp3"])], None);
    rig.text("words");
    rig.open();
    rig.fakes.clip.set_file("E:\\song.mp3", FileFact::Away);
    insert(&mut rig, &[1, 0]);
    assert_eq!(rig.fakes.writer.writes(), [vec![text("words")]]);
    pasted(&mut rig);
    rig.open();
    assert_eq!(rig.titles(), ["words", "song.mp3"]);
}

#[test]
fn closing_the_panel_stops_the_insert() {
    let mut rig = opened(&["a", "b"]);
    insert(&mut rig, &[0, 1]);
    rig.send(Msg::Open);
    pasted(&mut rig);
    assert_eq!(rig.fakes.writer.tries(), 1);
}

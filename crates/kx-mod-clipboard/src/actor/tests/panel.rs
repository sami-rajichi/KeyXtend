//! The panel: held until the ui's rows, the toggle, the chord flag and its ask, the stop, redraws, the footer's
//! moves, the preview, and entries whose files or folder went.

use kx_module_api::Redacted;
use kx_module_api::ports::{ClipAct, ClipNote, FileFact, PanelKey, Preview};
use toml::Value;

use super::super::Msg;
use super::rig::{LIST, PINNED, Rig, TODAY};
use crate::fixture::{files, text};

#[test]
fn an_open_panel_waits_for_the_uis_rows_then_shows_one_view_per_message() {
    let mut rig = Rig::new();
    for t in ["1", "2", "3", "4", "5", "6", "7"] {
        rig.text(t);
    }
    rig.send(Msg::Open);
    assert_eq!(rig.panels(), [true]);
    assert!(rig.views().is_empty(), "no page size of its own");
    rig.send(Msg::Act(ClipAct::Rows(3)));
    assert_eq!(rig.views().len(), 1);
    assert_eq!(rig.page().rows.len(), 3);
    assert_eq!(rig.page().pages.count, 3);
}

#[test]
fn clip_open_toggles_the_panel() {
    let mut rig = Rig::new();
    rig.open();
    rig.send(Msg::Open);
    assert_eq!(rig.panels(), [true, false]);
    rig.send(Msg::Open);
    assert_eq!(rig.panels(), [true, false, true]);
}

#[test]
fn a_history_ask_gets_the_chord_flag_and_the_panels_state() {
    let mut rig = Rig::new();
    assert_eq!(rig.wanted(), [true], "told at the start");
    rig.send(Msg::HistoryAsk);
    assert_eq!(
        (rig.wanted(), rig.panels()),
        (vec![true, true], vec![false])
    );
    rig.open();
    rig.send(Msg::HistoryAsk);
    assert_eq!(rig.panels(), [false, true, true]);
}

#[test]
fn history_wanted_follows_history_key() {
    let mut rig = Rig::build(|_, host| host.put("history_key", Value::Boolean(false)));
    rig.send(Msg::HistoryAsk);
    assert_eq!(rig.wanted(), [false, false]);
}

#[test]
fn the_stop_saves_closes_the_panel_frees_the_chord_and_hears_no_more() {
    let mut rig = Rig::new();
    rig.text("a");
    rig.open();
    rig.stop();
    assert!(rig.names(TODAY).contains(&LIST.to_owned()), "saved at once");
    assert_eq!(
        (rig.panels().last(), rig.wanted().last()),
        (Some(&false), Some(&false))
    );
    let (views, saves) = (rig.views().len(), rig.names(TODAY).len());
    assert!(!rig.fakes.clip.watching(), "the watch went first");
    rig.send(Msg::Note(ClipNote::Copied {
        parts: vec![text("late")],
        app: None,
    }));
    assert_eq!((rig.views().len(), rig.names(TODAY).len()), (views, saves));
}

#[test]
fn the_open_panel_draws_again_on_each_history_change() {
    let mut rig = Rig::new();
    rig.open();
    assert!(rig.page().empty);
    rig.text("new");
    assert_eq!(rig.titles(), ["new"]);
}

#[test]
fn pin_unpin_and_delete_act_on_the_ticked_rows_and_move_their_blobs() {
    let mut rig = Rig::new();
    rig.text("a");
    rig.text("b");
    rig.open();
    rig.tick_row(0);
    rig.send(Msg::Act(ClipAct::Pin));
    assert!(rig.page().rows[0].pinned && rig.page().unpin);
    assert!(!rig.names(PINNED).is_empty(), "b's blobs moved");
    rig.send(Msg::Act(ClipAct::Pin));
    assert!(
        !rig.page().rows[0].pinned,
        "all ticked were pinned, so it unpinned"
    );
    assert!(rig.names(PINNED).is_empty());
    rig.send(Msg::Act(ClipAct::Delete));
    assert_eq!(rig.titles(), ["a"]);
}

#[test]
fn delete_all_deletes_the_unpinned_on_yes_only() {
    let mut rig = Rig::new();
    for t in ["a", "b", "c"] {
        rig.text(t);
    }
    rig.open();
    rig.tick_row(2);
    rig.send(Msg::Act(ClipAct::Pin));
    rig.send(Msg::Act(ClipAct::DeleteAll));
    assert_eq!(rig.page().confirm, Some(2));
    rig.send(Msg::Act(ClipAct::Confirm(true)));
    assert_eq!(rig.titles(), ["a"]);
    rig.advance(rig.settings.save());
    assert!(rig.names(TODAY).is_empty(), "the emptied day left the disk");
}

#[test]
fn a_texts_preview_shows_it_with_its_count() {
    let mut rig = Rig::new();
    rig.text("hello");
    rig.open();
    rig.send(Msg::Act(ClipAct::Preview));
    let want = Preview::Text {
        text: Redacted::new("hello".to_owned()),
        chars: 5,
    };
    assert_eq!(rig.page().preview, Some(want));
    rig.send(Msg::Key(PanelKey::Esc));
    assert_eq!(rig.page().preview, None);
}

#[test]
fn a_file_row_shows_its_size_and_leaves_at_the_next_look_once_gone() {
    let mut rig = Rig::new();
    rig.fakes
        .clip
        .set_file("C:\\report.pdf", FileFact::Size(2_527_068));
    rig.copy(vec![files(&["C:\\report.pdf"])], None);
    rig.open();
    assert_eq!(rig.page().rows[0].bytes, Some(2_527_068));
    rig.fakes.clip.set_file("C:\\report.pdf", FileFact::Gone);
    rig.send(Msg::Open);
    rig.send(Msg::Open);
    assert!(rig.page().empty);
}

#[test]
fn a_day_folder_deleted_by_hand_leaves_at_the_next_look() {
    let mut rig = Rig::new();
    rig.text("a");
    rig.advance(rig.settings.save());
    let day = kx_module_api::ports::BlobName::new(TODAY).unwrap();
    rig.fakes.shelf.delete_by_hand(&day);
    rig.open();
    assert!(rig.page().empty);
}

#[test]
fn delete_all_answered_no_deletes_nothing() {
    let mut rig = Rig::new();
    rig.text("a");
    rig.text("b");
    rig.open();
    rig.send(Msg::Act(ClipAct::DeleteAll));
    rig.send(Msg::Act(ClipAct::Confirm(false)));
    assert_eq!(
        (rig.titles(), rig.page().confirm),
        (vec!["b".to_owned(), "a".to_owned()], None)
    );
}

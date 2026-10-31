//! Copies: the top row and the debounced save, skipped, marked and missed copies, follow-the-clear, duplicates, the
//! notes for a refused copy, the caps, and a locked shelf.

use std::time::Duration;

use kx_module_api::keys;
use kx_module_api::ports::{ClipAct, ClipNote, PanelNote, SealedError};
use kx_platform_fake::ShelfCall;
use toml::Value;

use super::super::Msg;
use super::rig::{LIST, Rig, TODAY};
use crate::fixture::{FOLLOW, LIMITS, MB, ROOMY, app, text};

/// The content blobs in `folder`, without its list.
fn content(rig: &Rig, folder: &str) -> Vec<String> {
    let names = rig.names(folder).into_iter();
    names.filter(|n| !n.starts_with(LIST)).collect()
}

#[test]
fn a_copy_shows_at_the_top_and_its_list_saves_after_save_ms() {
    let mut rig = Rig::new();
    rig.text("first");
    rig.text("second");
    rig.open();
    assert_eq!(rig.titles(), ["second", "first"]);
    assert_eq!(
        content(&rig, TODAY).len(),
        2,
        "each copy's content saves at once"
    );
    assert!(
        !rig.names(TODAY).contains(&LIST.to_owned()),
        "the list waits"
    );
    rig.advance(rig.settings.save());
    assert!(rig.names(TODAY).contains(&LIST.to_owned()));
}

#[test]
fn a_marked_copy_and_a_skipped_apps_copy_keep_nothing() {
    let skip = Value::Array(vec!["KeePass.exe".into()]);
    let mut rig = Rig::build(|_, host| host.put("skip_apps", skip));
    rig.push(ClipNote::Marked {
        app: Some(app("other.exe")),
    });
    rig.copy(vec![text("secret")], Some(app("keepass.exe")));
    rig.copy(vec![], None);
    rig.open();
    assert!(
        rig.page().empty,
        "a copy with no part kept makes no row either"
    );
    assert_eq!(rig.fakes.shelf.calls(ShelfCall::Save), 0, "nothing written");
}

#[test]
fn a_clear_by_the_copying_app_within_the_window_deletes_the_copy_for_good() {
    let mut rig = Rig::new();
    rig.text("kept");
    rig.copy(vec![text("password")], Some(app("vault.exe")));
    rig.advance(FOLLOW.checked_sub(Duration::from_secs(1)).unwrap());
    rig.push(ClipNote::Cleared {
        app: Some(app("other.exe")),
    });
    rig.open();
    assert_eq!(rig.titles(), ["password", "kept"], "another app's clear");
    rig.push(ClipNote::Cleared {
        app: Some(app("VAULT.exe")),
    });
    assert_eq!(rig.titles(), ["kept"]);
    assert_eq!(
        content(&rig, TODAY).len(),
        1,
        "its blobs left the disk at once"
    );
}

#[test]
fn a_clear_after_the_window_keeps_the_copy() {
    let mut rig = Rig::new();
    rig.copy(vec![text("note")], Some(app("editor.exe")));
    rig.advance(*FOLLOW + Duration::from_secs(1));
    rig.push(ClipNote::Cleared {
        app: Some(app("editor.exe")),
    });
    rig.open();
    assert_eq!(rig.titles(), ["note"]);
}

#[test]
fn a_copy_of_kept_content_moves_it_up_and_a_pinned_one_stays_pinned() {
    let mut rig = Rig::new();
    rig.text("a");
    rig.text("b");
    rig.text("a");
    rig.open();
    assert_eq!(rig.titles(), ["a", "b"], "one entry for a, at the top");
    rig.tick_row(1);
    rig.send(Msg::Act(ClipAct::Pin));
    rig.text("c");
    rig.text("b");
    assert_eq!(rig.titles(), ["b", "c", "a"]);
    assert!(rig.page().rows[0].pinned, "still pinned");
}

#[test]
fn a_copy_over_the_entry_cap_writes_nothing_and_shows_a_note() {
    let mut rig = Rig::new();
    rig.push(ClipNote::TooBig {
        app: None,
        bytes: 200 * MB,
    });
    rig.open();
    assert_eq!(rig.page().note, Some(PanelNote::TooBig));
    assert!(rig.page().empty);
    assert_eq!(rig.fakes.shelf.calls(ShelfCall::Save), 0);
}

#[test]
fn a_skipped_apps_copy_over_the_cap_shows_no_note() {
    let skip = Value::Array(vec!["KeePass.exe".into()]);
    let mut rig = Rig::build(|_, host| host.put("skip_apps", skip));
    rig.push(ClipNote::TooBig {
        app: Some(app("keepass.exe")),
        bytes: 200 * MB,
    });
    rig.open();
    assert_eq!(rig.page().note, None, "a skipped app's copy is never told");
    assert!(rig.page().empty);
}

#[test]
fn a_copy_missed_while_the_clipboard_was_busy_keeps_nothing_and_shows_no_note() {
    let mut rig = Rig::new();
    rig.push(ClipNote::Busy);
    rig.open();
    assert_eq!(
        rig.page().note,
        None,
        "edge 17: the missed copy is not told"
    );
    assert!(rig.page().empty);
    assert_eq!(rig.fakes.shelf.calls(ShelfCall::Save), 0);
}

#[test]
fn under_the_free_floor_nothing_new_is_kept_until_space_comes_back() {
    let mut rig = Rig::new();
    rig.fakes.shelf.set_free(LIMITS.min_free_bytes);
    rig.text("lost");
    rig.open();
    assert_eq!(rig.page().note, Some(PanelNote::Disk));
    assert!(rig.page().empty);
    rig.fakes.shelf.set_free(ROOMY);
    rig.text("kept");
    assert_eq!(rig.titles(), ["kept"]);
    assert_eq!(rig.page().note, None, "a kept copy ends the note");
}

#[test]
fn pinned_entries_filling_a_cap_refuse_a_new_copy_with_a_note() {
    let mut rig = Rig::build(|_, host| host.put("max_entries", Value::Integer(1)));
    rig.text("a");
    rig.open();
    rig.tick_row(0);
    rig.send(Msg::Act(ClipAct::Pin));
    rig.text("b");
    assert_eq!(rig.titles(), ["a"]);
    assert_eq!(rig.page().note, Some(PanelNote::PinnedFull));
}

#[test]
fn past_max_entries_the_oldest_unpinned_leaves_with_its_blobs() {
    let mut rig = Rig::build(|_, host| host.put("max_entries", Value::Integer(2)));
    for t in ["a", "b", "c"] {
        rig.text(t);
    }
    rig.open();
    assert_eq!(rig.titles(), ["c", "b"]);
    assert_eq!(content(&rig, TODAY).len(), 2);
}

#[test]
fn a_locked_shelf_keeps_nothing_and_says_so_once() {
    let mut rig = Rig::new();
    for t in ["a", "b"] {
        rig.fakes.shelf.fail(ShelfCall::Save, SealedError::Locked);
        rig.text(t);
    }
    rig.open();
    assert!(rig.page().empty);
    assert_eq!(rig.notices(), [keys::CLIP_LOCKED]);
}

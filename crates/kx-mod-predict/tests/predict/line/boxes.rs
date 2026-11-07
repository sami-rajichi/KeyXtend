//! The app's box behind a line: it is never read or asked, its kind counts for nothing, focus changes are only noted, and
//! when the aim leaves the lane follows it again as a new box.

use super::{aimed, in_box, search};
use kx_mod_predict::Msg;
use kx_module_api::ports::{FieldChange, FieldKind, Lane, PanelLine, Piece, TypingMode};

#[test]
fn the_box_is_never_read_or_asked_on_a_search() {
    let mut rig = in_box("en", &["hello", "help"], "we said ");
    let (reads, ends) = (rig.fakes.field.before_calls(), rig.fakes.field.ends_calls());
    rig.line(search("hel"));
    rig.send(Msg::Pressed(false));
    rig.lang("en");
    rig.pick(0);
    assert_eq!(rig.typed_words(), [(0, "lo ".to_owned())]);
    assert_eq!(rig.fakes.field.before_calls(), reads);
    assert_eq!(rig.fakes.field.ends_calls(), ends);
}

#[test]
fn a_password_box_behind_a_search_never_locks_its_lane() {
    let mut rig = in_box("en", &["hello", "help"], "");
    rig.focus(FieldKind::Password);
    assert_eq!(rig.lane().lane, Lane::Locked);
    rig.line(search("hel"));
    assert_eq!(rig.words(), ["hello", "help"]);
    rig.pick(0);
    assert_eq!(rig.typed_words(), [(0, "lo ".to_owned())]);
    assert_eq!(rig.word_targets(), [true]);
}

#[test]
fn a_password_box_the_watch_sees_changes_nothing_on_a_search_until_the_aim_leaves() {
    let mut rig = aimed("hell");
    rig.fakes.field.focus(FieldKind::Password);
    rig.late(Msg::Mode(TypingMode::default()));
    assert_eq!(rig.words(), ["hello"], "the watch's kind does not matter");
    rig.pick(0);
    assert_eq!(rig.typed_words(), [(0, "o ".to_owned())]);
    rig.line(PanelLine::None);
    assert_eq!(rig.lane().lane, Lane::Locked, "the app's box again");
}

#[test]
fn a_private_line_shows_an_empty_lane_whatever_the_apps_box() {
    for kind in [FieldKind::Text, FieldKind::Password, FieldKind::Unknown] {
        let mut rig = in_box("en", &["hello", "help"], "hel");
        rig.focus(kind);
        rig.line(PanelLine::Private);
        assert_eq!(rig.lane().lane, Lane::Empty, "{kind:?}");
        assert_eq!(rig.presses().last().copied(), Some(false), "{kind:?}");
        rig.typed("hel");
        rig.pick(0);
        assert_eq!(rig.lane().lane, Lane::Empty, "{kind:?}");
        assert!(rig.typed_words().is_empty(), "{kind:?}");
    }
}

#[test]
fn a_private_line_reads_nothing_from_the_box() {
    let mut rig = in_box("en", &["hello"], "hel");
    let reads = rig.fakes.field.before_calls();
    rig.line(PanelLine::Private);
    rig.send(Msg::Pressed(false));
    assert_eq!(rig.fakes.field.before_calls(), reads);
}

#[test]
fn the_aim_leaving_reads_the_apps_box_again_as_a_new_box() {
    let mut rig = in_box("en", &["hello", "help"], "");
    rig.line(search("xyz"));
    assert!(rig.words().is_empty());
    rig.fakes.field.set_before(Some("we said hel"));
    let reads = rig.fakes.field.before_calls();
    rig.line(PanelLine::None);
    assert_eq!(rig.fakes.field.before_calls(), reads + 1, "read once");
    assert_eq!(rig.words(), ["hello", "help"]);
    rig.pick(0);
    assert_eq!(rig.word_targets(), [false], "offered for the app");
}

#[test]
fn a_blind_box_is_read_afresh_when_the_aim_leaves() {
    let mut rig = in_box("en", &["hello"], "");
    rig.fakes.field.set_before(None);
    rig.send(Msg::Pressed(false));
    rig.line(search(""));
    rig.fakes.field.set_before(Some("we said hel"));
    rig.line(PanelLine::None);
    rig.pick(0);
    assert_eq!(rig.typed_words(), [(0, "lo ".to_owned())], "it confirms");
}

#[test]
fn a_focus_event_on_a_line_only_notes_the_box() {
    let mut rig = aimed("hel");
    let round = rig.lane().round;
    let reads = rig.fakes.field.before_calls();
    rig.fakes.field.set_before(Some("other "));
    rig.focus(FieldKind::Text);
    assert_eq!(rig.lane().round, round, "the lane stays");
    assert_eq!(rig.fakes.field.before_calls(), reads);
    rig.pick(0);
    assert_eq!(
        rig.typed_words(),
        [(0, "lo ".to_owned())],
        "the tracker too"
    );
}

#[test]
fn a_box_focused_during_a_line_kind_and_all_counts_when_the_aim_leaves() {
    let mut rig = aimed("hel");
    rig.focus(FieldKind::Password);
    assert_eq!(rig.words(), ["hello", "help"]);
    rig.line(PanelLine::None);
    assert_eq!(rig.lane().lane, Lane::Locked);
}

#[test]
fn a_box_noted_during_a_line_is_the_same_box_once_the_aim_left() {
    let mut rig = aimed("");
    let change = FieldChange {
        kind: FieldKind::Text,
        box_id: 99,
    };
    rig.send(Msg::Focus(change));
    rig.line(PanelLine::None);
    rig.typed("hell");
    assert_eq!(rig.words(), ["hello"]);
    rig.send(Msg::Focus(change));
    assert_eq!(rig.words(), ["hello"], "the word it holds stays");
}

#[test]
fn a_click_outside_changes_nothing_about_the_line() {
    let mut rig = aimed("hel");
    rig.send(Msg::Pressed(false));
    assert_eq!(
        rig.words(),
        ["hello", "help"],
        "the keyboard tells the change"
    );
    rig.pick(0);
    assert_eq!(rig.word_targets(), [true]);
}

#[test]
fn a_piece_for_the_app_during_a_line_changes_nothing_on_it() {
    let mut rig = aimed("hel");
    let round = rig.lane().round;
    rig.typed("zzz");
    rig.send(Msg::Typed(Piece::Break));
    rig.send(Msg::Typed(Piece::Erase));
    assert_eq!(rig.lane().round, round);
    assert_eq!(rig.words(), ["hello", "help"]);
}

#[test]
fn presses_are_not_wanted_on_a_line_and_are_again_once_the_box_holds_a_word() {
    let mut rig = in_box("en", &["hello"], "we said ");
    assert_eq!(rig.presses().last(), Some(&true));
    rig.line(search("hel"));
    assert_eq!(rig.presses().last(), Some(&false));
    rig.line(PanelLine::None);
    assert_eq!(rig.presses().last(), Some(&true));
}

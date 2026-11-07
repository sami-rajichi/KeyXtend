//! The line the keyboard hears (ADR-0040): the search on the list, no line on a preview or under a question, each change
//! once, and again on an ask.

use kx_module_api::Redacted;
use kx_module_api::ports::{ClipAct, PanelKey, PanelLine};

use super::super::Msg;
use super::rig::Rig;

/// The search line, holding `text`.
fn search(text: &str) -> PanelLine {
    PanelLine::Search(Redacted::new(text.to_owned()))
}

/// The key that types `text` into the search.
fn typed(text: &str) -> Msg {
    Msg::Key(PanelKey::Text(Redacted::new(text.to_owned())))
}

#[test]
fn a_begun_actor_has_told_no_line() {
    assert!(
        Rig::new().lines().is_empty(),
        "shut, as the keyboard assumes"
    );
}

#[test]
fn the_panel_opening_tells_the_search_and_closing_tells_no_line() {
    let mut rig = Rig::new();
    rig.send(Msg::Open);
    assert_eq!(rig.lines(), [search("")], "before the ui tells its rows");
    rig.send(Msg::Open);
    assert_eq!(rig.lines(), [search(""), PanelLine::None]);
}

#[test]
fn a_search_is_told_as_it_changes_and_only_then() {
    let mut rig = Rig::new();
    rig.text("alpha");
    rig.open();
    rig.send(typed("a"));
    rig.send(Msg::Key(PanelKey::Down));
    rig.send(typed("l"));
    assert_eq!(rig.lines(), [search(""), search("a"), search("al")]);
}

#[test]
fn a_caret_move_tells_the_search_before_it_again_and_gives_one_view_each() {
    let mut rig = Rig::new();
    rig.open();
    rig.send(typed("bank"));
    let views = rig.views().len();
    rig.send(Msg::Key(PanelKey::Left));
    assert_eq!(rig.lines().last(), Some(&search("ban")));
    assert_eq!(*rig.page().caret.expose(), 3);
    rig.send(Msg::Act(ClipAct::Caret { at: 1.into() }));
    assert_eq!(rig.lines().last(), Some(&search("b")));
    assert_eq!(*rig.page().caret.expose(), 1);
    assert_eq!(
        rig.views().len(),
        views + 2,
        "the chips and the line follow"
    );
    let told = rig.lines().len();
    rig.send(Msg::Key(PanelKey::Delete));
    assert_eq!(rig.page().search.expose(), "bnk");
    assert_eq!(rig.lines().len(), told, "nothing before the caret changed");
}

#[test]
fn a_chip_acts_at_the_caret_and_prediction_hears_what_it_left_before_it() {
    let mut rig = Rig::new();
    rig.open();
    rig.send(typed("ba x"));
    rig.send(Msg::Act(ClipAct::Caret { at: 2.into() }));
    rig.send(Msg::Key(PanelKey::Word {
        erase: 2,
        text: Redacted::new("bank".to_owned()),
    }));
    assert_eq!(rig.page().search.expose(), "bank x");
    assert_eq!(*rig.page().caret.expose(), 4);
    assert_eq!(rig.lines().last(), Some(&search("bank")));
}

#[test]
fn a_chip_gives_one_view_and_one_line_in_english_and_arabic() {
    for (typed_first, erase, chip, want) in [
        ("hel", 3, "Hello ", "Hello "),
        ("مرحب", 2, "حبا ", "مرحبا "),
        ("he", 9, "x", "x"),
    ] {
        let mut rig = Rig::new();
        rig.open();
        rig.send(typed(typed_first));
        let (views, lines) = (rig.views().len(), rig.lines().len());
        rig.send(Msg::Key(PanelKey::Word {
            erase,
            text: Redacted::new(chip.to_owned()),
        }));
        assert_eq!(rig.views().len(), views + 1, "{want}");
        assert_eq!(rig.lines().len(), lines + 1, "{want}");
        assert_eq!(rig.lines().last(), Some(&search(want)));
    }
}

#[test]
fn a_preview_tells_no_line_and_coming_back_tells_the_search() {
    let mut rig = Rig::new();
    rig.text("alpha");
    rig.open();
    rig.send(Msg::Act(ClipAct::Preview));
    assert_eq!(rig.lines(), [search(""), PanelLine::None]);
    rig.send(Msg::Key(PanelKey::Esc));
    assert_eq!(rig.lines(), [search(""), PanelLine::None, search("")]);
}

#[test]
fn a_question_tells_no_line_and_its_answer_tells_the_search() {
    let mut rig = Rig::new();
    rig.text("alpha");
    rig.open();
    rig.send(Msg::Act(ClipAct::DeleteAll));
    assert_eq!(rig.lines(), [search(""), PanelLine::None]);
    rig.send(Msg::Act(ClipAct::Confirm(false)));
    assert_eq!(rig.lines(), [search(""), PanelLine::None, search("")]);
}

#[test]
fn an_ask_tells_the_line_again_even_unchanged() {
    let mut rig = Rig::new();
    rig.send(Msg::HistoryAsk);
    assert_eq!(rig.lines(), [PanelLine::None], "shut");
    rig.open();
    rig.send(Msg::HistoryAsk);
    assert_eq!(rig.lines(), [PanelLine::None, search(""), search("")]);
}

#[test]
fn the_stop_tells_no_line_and_nothing_after_it() {
    let mut rig = Rig::new();
    rig.open();
    rig.stop();
    rig.send(Msg::HistoryAsk);
    assert_eq!(rig.lines(), [search(""), PanelLine::None]);
}

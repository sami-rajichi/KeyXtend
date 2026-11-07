//! The search line's caret: keys and chips act at it, Left and Right never switch filters while the line holds text, a
//! click places it, and only a change of text moves the list.

use kx_module_api::Redacted;
use kx_module_api::ports::Filter;

use super::*;

/// Beh with a fatha, then teh, which reads right to left.
const BEH_FATHA_TEH: &str = "\u{628}\u{64e}\u{62a}";

/// The key of a chip that erases `erase` characters, then types `text`.
fn word(erase: u16, text: &str) -> PanelKey {
    PanelKey::Word {
        erase,
        text: Redacted::new(text.to_owned()),
    }
}

/// The search line holding `text`.
fn line_of(text: &str) -> PanelLine {
    PanelLine::Search(Redacted::new(text.to_owned()))
}

/// The search's text and its caret.
fn held(p: &Panel) -> (&str, u32) {
    (p.query().expose(), p.caret())
}

/// A panel over `h` whose search holds `text`, its caret at `at`.
fn searching(h: &History, text: &str, at: u32) -> Panel {
    let mut p = opened();
    p.key(&chars(text), h);
    p.act(ClipAct::Caret { at: at.into() }, h);
    p
}

#[test]
fn the_search_starts_empty_and_follows_what_is_typed() {
    let h = history(&["a"]);
    let mut p = opened();
    assert_eq!(p.caret(), 0);
    p.key(&chars("abc"), &h);
    assert_eq!(p.caret(), 3);
}

#[test]
fn space_types_at_the_caret_while_the_line_holds_text_and_ticks_nothing() {
    let h = history(&["a"]);
    let mut p = searching(&h, "te", 2);
    assert_eq!(p.key(&PanelKey::Tick, &h), Do::Show);
    assert_eq!(held(&p), ("te ", 3));
    assert_eq!(p.line(), line_of("te "));
    assert!(p.ticks().is_empty());
    p.key(&chars("st"), &h);
    assert_eq!(held(&p), ("te st", 5));
}

#[test]
fn space_goes_in_the_middle_of_arabic_too_and_two_stay_one() {
    let h = history(&["a"]);
    let mut p = searching(&h, "\u{643}\u{62a}\u{627}\u{628}", 2);
    assert_eq!(p.key(&PanelKey::Tick, &h), Do::Show);
    assert_eq!(held(&p), ("\u{643}\u{62a} \u{627}\u{628}", 3));
    assert_eq!(
        p.key(&PanelKey::Tick, &h),
        Do::Stay,
        "blanks that meet are one"
    );
    assert_eq!(held(&p), ("\u{643}\u{62a} \u{627}\u{628}", 3));
}

#[test]
fn space_on_an_empty_line_ticks_the_highlighted_row_and_types_nothing() {
    let h = history(&["a"]);
    let mut p = opened();
    assert_eq!(p.key(&PanelKey::Tick, &h), Do::Show);
    assert_eq!(held(&p), ("", 0));
    assert_eq!(p.ticks(), ids(&h, &["a"]));
}

#[test]
fn left_and_right_move_the_caret_and_never_the_filter_while_the_line_holds_text() {
    let h = history(&["a"]);
    let mut p = opened();
    p.act(ClipAct::Filter(Filter::Text), &h);
    p.key(&chars("ab"), &h);
    assert_eq!(p.key(&PanelKey::Right, &h), Do::Stay, "at the end");
    assert_eq!(p.filter(), Filter::Text);
    assert_eq!(p.key(&PanelKey::Left, &h), Do::Show);
    assert_eq!(p.caret(), 1);
    p.key(&PanelKey::Left, &h);
    assert_eq!(p.key(&PanelKey::Left, &h), Do::Stay, "at the start");
    assert_eq!((p.caret(), p.filter()), (0, Filter::Text));
}

#[test]
fn an_empty_search_still_switches_filters() {
    let h = history(&["a"]);
    let mut p = opened();
    assert_eq!(p.key(&PanelKey::Right, &h), Do::Show);
    assert_eq!(p.filter(), Filter::Text);
    assert_eq!(p.key(&PanelKey::Left, &h), Do::Show);
    assert_eq!(p.filter(), Filter::All);
    p.key(&chars("a"), &h);
    p.key(&PanelKey::Erase, &h);
    p.key(&PanelKey::Right, &h);
    assert_eq!(p.filter(), Filter::Text, "erased empty again");
    let mut p = opened();
    p.key(&chars("\u{200f}"), &h);
    p.key(&PanelKey::Right, &h);
    assert_eq!(p.filter(), Filter::Text, "a hidden mark draws nothing");
}

#[test]
fn a_right_to_left_line_takes_left_as_on_and_steps_over_its_harakat() {
    let h = history(&["a"]);
    let mut p = opened();
    p.key(&chars(BEH_FATHA_TEH), &h);
    assert_eq!(p.caret(), 3);
    assert_eq!(
        p.key(&PanelKey::Left, &h),
        Do::Stay,
        "the end is on the left"
    );
    assert_eq!(p.key(&PanelKey::Right, &h), Do::Show);
    assert_eq!(p.caret(), 2, "before the teh");
    p.key(&PanelKey::Right, &h);
    assert_eq!(p.caret(), 0, "over the beh and its fatha as one");
    p.key(&PanelKey::Left, &h);
    assert_eq!(p.caret(), 2);
}

#[test]
fn moving_the_caret_keeps_the_results_and_the_highlight() {
    let h = history(&["ab one", "ab two", "ab three"]);
    let mut p = opened();
    p.key(&chars("ab"), &h);
    p.key(&PanelKey::Down, &h);
    let hi = p.highlight(&p.list(&h));
    assert_eq!(hi, Some(1));
    assert_eq!(p.key(&PanelKey::Left, &h), Do::Show);
    assert_eq!(p.act(ClipAct::Caret { at: 0.into() }, &h), Do::Show);
    assert_eq!(p.highlight(&p.list(&h)), hi);
    assert_eq!(p.list(&h).len(), 3);
    assert_eq!(p.query().expose(), "ab");
}

#[test]
fn a_change_in_the_middle_brings_the_highlight_to_the_top() {
    let h = history(&["ab 1", "ab 2", "a 3"]);
    let mut p = searching(&h, "ab", 1);
    p.key(&PanelKey::Down, &h);
    assert_eq!(p.highlight(&p.list(&h)), Some(1));
    assert_eq!(p.key(&PanelKey::Delete, &h), Do::Show);
    assert_eq!(p.highlight(&p.list(&h)), Some(0));
    assert_eq!(p.list(&h).len(), 3, "the list follows the whole line");
}

#[test]
fn letters_erase_delete_and_a_chip_act_in_the_middle() {
    let h = history(&["a"]);
    let mut p = searching(&h, "ak", 1);
    p.key(&chars("n"), &h);
    assert_eq!(held(&p), ("ank", 2));
    assert_eq!(p.key(&PanelKey::Erase, &h), Do::Show);
    assert_eq!(held(&p), ("ak", 1));
    assert_eq!(p.key(&PanelKey::Delete, &h), Do::Show);
    assert_eq!(held(&p), ("a", 1));
    assert_eq!(
        p.key(&PanelKey::Delete, &h),
        Do::Stay,
        "nothing after the caret"
    );
    p.key(&PanelKey::Left, &h);
    p.key(&word(0, "b"), &h);
    assert_eq!(held(&p), ("ba", 1));
    p.key(&word(1, "ban"), &h);
    assert_eq!(held(&p), ("bana", 3));
}

#[test]
fn delete_takes_a_whole_grapheme_and_erase_one_character() {
    let h = history(&["a"]);
    let mut p = searching(&h, BEH_FATHA_TEH, 0);
    assert_eq!(p.key(&PanelKey::Delete, &h), Do::Show);
    assert_eq!(held(&p), ("\u{62a}", 0));
    let mut p = searching(&h, BEH_FATHA_TEH, 2);
    assert_eq!(p.key(&PanelKey::Erase, &h), Do::Show);
    assert_eq!(held(&p), ("\u{628}\u{62a}", 1), "the haraka goes alone");
}

#[test]
fn the_cap_in_the_middle_cuts_the_end_and_keeps_the_caret_after_what_went_in() {
    let h = history(&["a"]);
    let mut p = opened_with(PanelConfig {
        query_chars: 3,
        ..*PANEL
    });
    p.key(&chars("abc"), &h);
    p.act(ClipAct::Caret { at: 1.into() }, &h);
    assert_eq!(p.key(&chars("z"), &h), Do::Show);
    assert_eq!(held(&p), ("azb", 2));
    assert_eq!(p.key(&chars("xyz"), &h), Do::Show);
    assert_eq!(held(&p), ("azx", 3), "clamped at the cap");
    assert_eq!(p.key(&chars("q"), &h), Do::Stay, "full, at the end");
}

#[test]
fn what_is_typed_joins_the_line_flat_around_the_caret() {
    let h = history(&["a"]);
    let mut p = searching(&h, "a b", 1);
    assert_eq!(
        p.key(&chars(" "), &h),
        Do::Show,
        "the same text, a new caret"
    );
    assert_eq!(held(&p), ("a b", 2));
    assert_eq!(p.key(&PanelKey::Left, &h), Do::Show);
    assert_eq!(p.key(&chars("\r\n"), &h), Do::Show);
    assert_eq!(held(&p), ("a b", 2), "one space, and past it");
    let mut p = searching(&h, "a b c", 3);
    p.key(&PanelKey::Erase, &h);
    assert_eq!(held(&p), ("a c", 2), "the blanks it left join");
    let mut p = searching(&h, "a b c", 2);
    p.key(&PanelKey::Delete, &h);
    assert_eq!(held(&p), ("a c", 2));
}

#[test]
fn a_click_places_the_caret_clamped_and_inside_no_grapheme() {
    let h = history(&["a"]);
    let mut p = opened();
    p.key(&chars(BEH_FATHA_TEH), &h);
    assert_eq!(p.act(ClipAct::Caret { at: 0.into() }, &h), Do::Show);
    assert_eq!(p.caret(), 0);
    assert_eq!(
        p.act(ClipAct::Caret { at: 1.into() }, &h),
        Do::Stay,
        "inside it"
    );
    assert_eq!(p.caret(), 0);
    assert_eq!(p.act(ClipAct::Caret { at: 2.into() }, &h), Do::Show);
    assert_eq!(p.caret(), 2);
    assert_eq!(
        p.act(
            ClipAct::Caret {
                at: u32::MAX.into()
            },
            &h
        ),
        Do::Show
    );
    assert_eq!(p.caret(), 3);
}

#[test]
fn the_caret_counts_what_the_line_draws_never_a_hidden_mark() {
    let h = history(&["a"]);
    let mut p = opened();
    p.key(&chars("a\u{200f}b"), &h);
    assert_eq!(p.caret(), 2);
    p.act(ClipAct::Caret { at: 0.into() }, &h);
    p.key(&PanelKey::Right, &h);
    assert_eq!(p.caret(), 1);
    assert_eq!(p.key(&PanelKey::Right, &h), Do::Show);
    assert_eq!(p.caret(), 2, "one press, over the mark too");
    assert_eq!(p.key(&PanelKey::Left, &h), Do::Show);
    assert_eq!(p.caret(), 1);
    p.key(&PanelKey::Delete, &h);
    assert_eq!(
        p.query().expose(),
        "a\u{200f}",
        "it took the letter, not the mark"
    );
}

#[test]
fn prediction_hears_only_the_search_before_the_caret() {
    let h = history(&["a"]);
    let mut p = searching(&h, "bank", 2);
    assert_eq!(p.line(), line_of("ba"));
    p.key(&PanelKey::Right, &h);
    assert_eq!(p.line(), line_of("ban"));
    p.key(&PanelKey::Delete, &h);
    assert_eq!(p.query().expose(), "ban");
    assert_eq!(p.line(), line_of("ban"), "nothing before it changed");
}

#[test]
fn debug_prints_no_caret() {
    let h = history(&["a"]);
    let p = searching(&h, "abc", 2);
    assert_eq!(p.caret(), 2);
    let shown = format!("{p:?}");
    assert!(!shown.contains("caret: 2"), "{shown}");
}

#[test]
fn esc_clears_the_line_and_its_caret_and_a_new_panel_starts_fresh() {
    let h = history(&["a"]);
    let mut p = searching(&h, "ab", 1);
    assert_eq!(p.key(&PanelKey::Esc, &h), Do::Show);
    assert_eq!(held(&p), ("", 0));
    let mut p = searching(&h, "ab", 1);
    p.close();
    p.open();
    assert_eq!(held(&p), ("", 0));
}

#[test]
fn a_question_or_a_preview_takes_neither_the_keys_nor_the_click() {
    let h = history(&["a"]);
    let mut p = searching(&h, "ab", 1);
    p.act(ClipAct::DeleteAll, &h);
    for key in [PanelKey::Left, PanelKey::Right, PanelKey::Delete] {
        assert_eq!(p.key(&key, &h), Do::Stay, "a question: {key:?}");
    }
    assert_eq!(p.act(ClipAct::Caret { at: 0.into() }, &h), Do::Stay);
    assert_eq!(p.line(), PanelLine::None);
    p.act(ClipAct::Confirm(false), &h);
    p.show(id(&h, "a"), content());
    for key in [PanelKey::Left, PanelKey::Right, PanelKey::Delete] {
        assert_eq!(p.key(&key, &h), Do::Stay, "a preview: {key:?}");
    }
    assert_eq!(p.act(ClipAct::Caret { at: 0.into() }, &h), Do::Stay);
    assert_eq!(held(&p), ("ab", 1));
}

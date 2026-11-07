//! The caret's steps, ends and conversions, in Arabic, emoji and hidden marks.

use super::*;
use proptest::prelude::*;

/// Beh with a fatha, then teh: two letters and three characters.
const BEH_FATHA_TEH: &str = "\u{628}\u{64e}\u{62a}";
/// A family of three joined by ZWJ: one grapheme of five characters.
const FAMILY: &str = "\u{1f468}\u{200d}\u{1f469}\u{200d}\u{1f467}";
/// An emoji outside the first plane: one character of two UTF-16 units.
const EMOJI: &str = "\u{1f600}";
/// A right-to-left mark, which the drawn text loses.
const RLM: &str = "\u{200f}";

/// The bytes of beh and its fatha.
const BEH_FATHA: usize = 4;

#[test]
fn a_step_takes_a_base_letter_and_its_harakat_as_one() {
    assert_eq!(on(BEH_FATHA_TEH, 0), BEH_FATHA);
    assert_eq!(on(BEH_FATHA_TEH, BEH_FATHA), BEH_FATHA_TEH.len());
    assert_eq!(back(BEH_FATHA_TEH, BEH_FATHA_TEH.len()), BEH_FATHA);
    assert_eq!(back(BEH_FATHA_TEH, BEH_FATHA), 0);
}

#[test]
fn a_step_takes_an_emoji_and_a_joined_family_as_one() {
    for one in [EMOJI, FAMILY] {
        let text = format!("a{one}b");
        assert_eq!(on(&text, 1), 1 + one.len(), "{one}");
        assert_eq!(back(&text, 1 + one.len()), 1, "{one}");
    }
}

#[test]
fn a_step_stops_at_the_ends_where_it_stands() {
    assert_eq!(back("ab", 0), 0);
    assert_eq!(on("ab", 2), 2);
    assert_eq!((back("", 0), on("", 0)), (0, 0));
}

#[test]
fn a_step_inside_a_grapheme_reaches_its_edges() {
    assert_eq!(
        on(BEH_FATHA_TEH, 2),
        BEH_FATHA,
        "after the beh, before its fatha"
    );
    assert_eq!(back(BEH_FATHA_TEH, 2), 0);
}

#[test]
fn a_step_never_rests_on_a_hidden_mark_alone_as_nothing_would_move() {
    let text = format!("a{RLM}b");
    assert_eq!(on(&text, 1), text.len(), "over the mark and the b");
    assert_eq!(back(&text, text.len() - 1), 0, "over the mark and the a");
    assert_eq!(on(&format!("a{RLM}"), 1), 1, "only a mark is left");
    assert_eq!(
        back(&format!("{RLM}a"), RLM.len()),
        RLM.len(),
        "only a mark is left"
    );
}

#[test]
fn an_offset_past_the_text_or_inside_a_character_is_clamped() {
    assert_eq!(clamped("ab", 9), 2);
    assert_eq!(clamped("\u{628}a", 1), 0, "inside the beh");
    assert_eq!(clamped("", 3), 0);
    assert_eq!(before("\u{628}a", 1), "");
    assert_eq!(before("\u{628}a", 2), "\u{628}");
    assert_eq!(before("ab", 9), "ab");
}

#[test]
fn the_arrows_follow_the_lines_direction() {
    let latin = "ab";
    assert_eq!(arrow(latin, 1, Side::Left), 0);
    assert_eq!(arrow(latin, 1, Side::Right), 2);
    let arabic = "\u{628}\u{62a}";
    assert_eq!(
        arrow(arabic, 2, Side::Left),
        4,
        "left goes on in a right-to-left line"
    );
    assert_eq!(arrow(arabic, 2, Side::Right), 0);
    assert_eq!(
        arrow(BEH_FATHA_TEH, 0, Side::Left),
        BEH_FATHA,
        "and takes the harakat along"
    );
}

#[test]
fn a_line_with_only_hidden_marks_draws_nothing() {
    assert!(empty(""));
    assert!(empty(RLM));
    assert!(!empty("a"));
    assert!(!empty(" "));
}

#[test]
fn the_units_before_the_caret_count_what_the_line_draws() {
    assert_eq!(units(BEH_FATHA_TEH, BEH_FATHA), 2);
    assert_eq!(units(EMOJI, EMOJI.len()), 2, "a surrogate pair");
    assert_eq!(units(FAMILY, FAMILY.len()), 8);
    assert_eq!(
        units(&format!("a{RLM}b"), 1 + RLM.len()),
        1,
        "the drawn text loses the mark"
    );
    assert_eq!(units("ab", 9), 2, "clamped");
}

#[test]
fn a_units_count_comes_back_to_its_grapheme_clamped() {
    assert_eq!(of_units(BEH_FATHA_TEH, 0), 0);
    assert_eq!(of_units(BEH_FATHA_TEH, 1), 0, "inside the beh's grapheme");
    assert_eq!(of_units(BEH_FATHA_TEH, 2), BEH_FATHA);
    assert_eq!(of_units(BEH_FATHA_TEH, 3), BEH_FATHA_TEH.len());
    assert_eq!(of_units(BEH_FATHA_TEH, u32::MAX), BEH_FATHA_TEH.len());
    assert_eq!(of_units(EMOJI, 1), 0, "never inside a surrogate pair");
    assert_eq!(of_units(FAMILY, 4), 0, "never inside a joined family");
    assert_eq!(of_units(&format!("a{EMOJI}b"), 3), 1 + EMOJI.len());
    assert_eq!(of_units("", 5), 0);
}

#[test]
fn dots_count_every_character_and_come_back_to_their_own_spot() {
    assert_eq!(dots(BEH_FATHA_TEH, BEH_FATHA), 2);
    assert_eq!(dots(EMOJI, EMOJI.len()), 1, "one dot, two units");
    assert_eq!(
        dots(&format!("a{RLM}"), 1 + RLM.len()),
        2,
        "a hidden mark has its dot"
    );
    assert_eq!(
        of_dots(BEH_FATHA_TEH, 1),
        2,
        "between the beh and its fatha"
    );
    assert_eq!(of_dots(BEH_FATHA_TEH, 2), BEH_FATHA);
    assert_eq!(of_dots(EMOJI, 1), EMOJI.len());
    assert_eq!(of_dots(FAMILY, 1), EMOJI.len(), "inside the joined family");
    assert_eq!(of_dots("ab", u32::MAX), 2);
}

#[test]
fn a_dots_step_takes_one_character_whatever_it_joins() {
    assert_eq!(dot_on(BEH_FATHA_TEH, 0), 2, "the beh, not its fatha");
    assert_eq!(dot_back(BEH_FATHA_TEH, BEH_FATHA), 2);
    assert_eq!(dot_on(FAMILY, 0), EMOJI.len());
    let marked = format!("a{RLM}");
    assert_eq!(dot_on(&marked, 1), marked.len(), "a last hidden mark too");
    assert_eq!(dot_back(&marked, marked.len()), 1);
    assert_eq!((dot_back("ab", 0), dot_on("ab", 2)), (0, 2), "the ends");
    assert_eq!((dot_back("", 3), dot_on("", 3)), (0, 0));
    assert_eq!(dot_on("\u{628}a", 1), 2, "from inside a character");
}

/// Text mixing what a line may hold: Latin, Arabic with harakat, emoji, joiners and hidden marks.
fn line() -> impl Strategy<Value = String> {
    "[a\u{628}\u{64e}\u{651}\u{1f600}\u{200d}\u{200c}\u{200f}\u{61c} ]{0,12}"
}

proptest! {
    #[test]
    fn every_result_is_a_char_boundary_within_the_text(
        text in line(),
        at in 0usize..64,
        count in 0u32..64,
    ) {
        let results = [
            clamped(&text, at),
            back(&text, at),
            on(&text, at),
            arrow(&text, at, Side::Left),
            arrow(&text, at, Side::Right),
            of_units(&text, count),
            of_dots(&text, count),
            dot_back(&text, at),
            dot_on(&text, at),
        ];
        for result in results {
            prop_assert!(text.is_char_boundary(result), "{result}");
            prop_assert!(result <= text.len());
        }
    }

    #[test]
    fn a_step_moves_what_the_line_draws_or_stays(text in line(), at in 0usize..64) {
        let at = clamped(&text, at);
        let (was, ahead, behind) = (units(&text, at), on(&text, at), back(&text, at));
        prop_assert!(ahead == at || units(&text, ahead) > was);
        prop_assert!(behind == at || units(&text, behind) < was);
    }

    #[test]
    fn steps_walk_the_whole_drawn_line_both_ways(text in line()) {
        let mut at = 0;
        while on(&text, at) != at {
            at = on(&text, at);
        }
        prop_assert_eq!(units(&text, at), units(&text, text.len()));
        while back(&text, at) != at {
            at = back(&text, at);
        }
        prop_assert_eq!(units(&text, at), 0);
    }

    #[test]
    fn a_count_never_comes_back_past_itself(text in line(), count in 0u32..64) {
        prop_assert!(units(&text, of_units(&text, count)) <= count);
        prop_assert!(dots(&text, of_dots(&text, count)) <= count);
    }

    #[test]
    fn every_dot_is_reached_by_one_step_and_by_its_own_count(text in line()) {
        let (mut at, total) = (0, count(text.chars().count()));
        for dot in 1..=total {
            at = dot_on(&text, at);
            prop_assert_eq!(dots(&text, at), dot);
            prop_assert_eq!(of_dots(&text, dot), at);
        }
        prop_assert_eq!(dot_on(&text, at), at);
        for dot in (0..total).rev() {
            at = dot_back(&text, at);
            prop_assert_eq!(dots(&text, at), dot);
        }
        prop_assert_eq!(dot_back(&text, at), at);
    }

    #[test]
    fn a_step_lands_where_its_count_comes_back(text in line(), at in 0usize..64) {
        let there = on(&text, at);
        let count = units(&text, there);
        prop_assert_eq!(units(&text, of_units(&text, count)), count);
    }
}

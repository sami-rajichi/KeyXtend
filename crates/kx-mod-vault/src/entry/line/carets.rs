//! A line's caret: typing, erasing and deleting at it, its moves and clicks in Arabic, the cap in the middle of a line,
//! and a middle insert that never leaves a copy of a wiped text behind.

use super::*;

/// Beh with a fatha, then teh: two letters and three characters.
const BEH_FATHA_TEH: &str = "\u{628}\u{64e}\u{62a}";
/// The bytes of beh and its fatha.
const BEH_FATHA: usize = 4;
/// An emoji of two UTF-16 units.
const EMOJI: &str = "\u{1f600}";
/// A right-to-left mark, one a shown text loses.
const RLM: char = '\u{200f}';
/// A family of three joined by ZWJ: one grapheme of five characters.
const FAMILY: &str = "\u{1f468}\u{200d}\u{1f469}\u{200d}\u{1f467}";
/// The dots of `joined`.
const DOTS: u32 = 8;

/// Beh with a fatha, a joined family and a last right-to-left mark: what a grapheme's step would jump.
fn joined() -> String {
    format!("\u{628}\u{64e}{FAMILY}{RLM}")
}

#[test]
fn a_new_line_starts_at_its_start_and_a_saved_one_at_its_end() {
    assert_eq!(Line::new(5).caret(), 0);
    let line = Line::with("abc", 5).unwrap();
    assert_eq!(line.caret(), 3);
    assert_eq!(line.before(), "abc");
}

#[test]
fn letters_go_in_at_the_caret_and_it_stays_after_them() {
    let mut line = Line::with("ac", 5).unwrap();
    assert!(line.arrow(Side::Left));
    line.push('b').unwrap();
    assert_eq!(line.text(), "abc");
    assert_eq!((line.caret(), line.before()), (2, "ab"));
    line.push('\u{628}').unwrap();
    assert_eq!(line.text(), "ab\u{628}c");
    assert_eq!(line.before(), "ab\u{628}");
}

#[test]
fn a_refused_letter_leaves_the_caret_and_the_text() {
    let mut line = Line::with("ab", 2).unwrap();
    assert!(line.arrow(Side::Left));
    assert_eq!(
        line.push('z'),
        Err(EntryError::Refused),
        "the cap, mid-line"
    );
    assert_eq!(line.push('\n'), Err(EntryError::Refused));
    assert_eq!((line.text(), line.caret(), line.chars()), ("ab", 1, 2));
}

#[test]
fn pop_takes_one_character_before_the_caret_so_a_haraka_goes_alone() {
    let mut line = Line::with(BEH_FATHA_TEH, 5).unwrap();
    assert!(line.arrow(Side::Right), "back in a right-to-left line");
    assert_eq!(line.caret(), BEH_FATHA);
    assert_eq!(line.pop(), Some('\u{64e}'));
    assert_eq!(line.text(), "\u{628}\u{62a}");
    assert_eq!((line.caret(), line.chars()), (2, 2));
    assert_eq!(line.pop(), Some('\u{628}'));
    assert_eq!(line.pop(), None, "nothing before the caret");
    assert_eq!((line.text(), line.chars()), ("\u{62a}", 1));
}

#[test]
fn delete_takes_one_grapheme_after_the_caret() {
    let mut line = Line::with(BEH_FATHA_TEH, 5).unwrap();
    line.place(0);
    assert!(line.delete());
    assert_eq!(line.text(), "\u{62a}", "the beh and its fatha as one");
    assert_eq!((line.caret(), line.chars()), (0, 1));
    assert!(line.delete());
    assert!(!line.delete(), "nothing after the caret");
    assert_eq!((line.text(), line.chars()), ("", 0));
}

#[test]
fn delete_keeps_the_counts_of_a_stripped_mark() {
    let mut line = Line::new(2);
    for c in ['a', RLM, 'b'] {
        line.push(c).unwrap();
    }
    line.place(1);
    assert!(line.delete(), "the mark and the b");
    assert_eq!((line.text(), line.chars()), ("a", 1));
    for c in ['c', RLM, RLM] {
        line.push(c).unwrap();
    }
    assert_eq!(line.push(RLM), Err(EntryError::Refused), "two marks fit");
}

#[test]
fn delete_with_only_a_mark_after_the_caret_does_nothing_as_nothing_shows() {
    let mut line = Line::new(3);
    for c in ['a', RLM] {
        line.push(c).unwrap();
    }
    line.place(1);
    assert!(!line.delete());
    assert_eq!(line.text().chars().count(), 2);
}

#[test]
fn the_arrows_stop_at_the_ends_and_follow_the_lines_direction() {
    let mut latin = Line::with("ab", 5).unwrap();
    assert!(!latin.arrow(Side::Right), "already at the end");
    assert!(latin.arrow(Side::Left));
    assert_eq!(latin.caret(), 1);
    assert!(latin.arrow(Side::Left));
    assert!(!latin.arrow(Side::Left), "already at the start");
    let mut arabic = Line::with(BEH_FATHA_TEH, 5).unwrap();
    assert!(!arabic.arrow(Side::Left), "the end is on the left");
    assert!(arabic.arrow(Side::Right));
    assert_eq!(arabic.caret(), BEH_FATHA);
    assert!(arabic.arrow(Side::Left));
    assert_eq!(arabic.caret(), BEH_FATHA_TEH.len());
}

#[test]
fn a_masked_line_steps_one_dot_whatever_its_text_reads() {
    let mut arabic = Line::with(BEH_FATHA_TEH, 5).unwrap();
    assert!(arabic.step_dot(false), "left goes back along the dots");
    assert_eq!(arabic.caret(), BEH_FATHA);
    assert_eq!(arabic.dots(), 2);
    assert!(arabic.step_dot(false), "onto the fatha's own dot");
    assert_eq!(arabic.dots(), 1);
    assert!(arabic.step_dot(true));
    assert!(arabic.step_dot(true));
    assert!(!arabic.step_dot(true), "already at the end");
}

#[test]
fn a_masked_line_reaches_and_deletes_every_dot_whatever_joins_them() {
    let (text, dots) = (joined(), DOTS);
    let mut line = Line::with(&text, 9).unwrap();
    for dot in (0..dots).rev() {
        assert!(line.step_dot(false));
        assert_eq!(line.dots(), dot);
    }
    assert!(!line.step_dot(false));
    for dot in 1..=dots {
        assert!(line.step_dot(true));
        assert_eq!(line.dots(), dot);
    }
    assert!(!line.step_dot(true));
    for dot in 0..=dots {
        line.place_dots(dot);
        assert_eq!(line.dots(), dot, "a click lands on its own dot");
    }
    line.place_dots(0);
    for left in (0..dots).rev() {
        assert!(line.delete_dot());
        assert_eq!(line.text().chars().count(), usize::try_from(left).unwrap());
        assert_eq!(line.dots(), 0);
    }
    assert!(!line.delete_dot(), "nothing is left");
    assert_eq!(line.chars(), 0);
}

#[test]
fn delete_dot_keeps_the_counts_of_a_stripped_mark() {
    let mut line = Line::new(2);
    for c in ['a', RLM, 'b'] {
        line.push(c).unwrap();
    }
    line.place_dots(1);
    assert!(line.delete_dot(), "the mark alone");
    assert_eq!((line.text(), line.chars()), ("ab", 2));
    for c in [RLM, RLM] {
        line.push(c).unwrap();
    }
    assert_eq!(line.push(RLM), Err(EntryError::Refused), "two marks fit");
}

#[test]
fn a_delete_or_a_step_from_inside_a_grapheme_stays_on_a_boundary() {
    let mut line = Line::with(BEH_FATHA_TEH, 5).unwrap();
    line.place_dots(1);
    assert_eq!(line.drawn(), 1, "after the beh, before its fatha");
    assert!(line.arrow(Side::Right), "back in a right-to-left line");
    assert_eq!(line.caret(), 0);
    line.place_dots(1);
    assert!(line.delete(), "the rest of the grapheme");
    assert_eq!(line.text(), "\u{628}\u{62a}");
    assert_eq!((line.caret(), line.chars()), (2, 2));
}

#[test]
fn a_click_places_the_caret_in_what_the_line_draws_clamped() {
    let text = format!("a{EMOJI}b");
    let mut line = Line::with(&text, 5).unwrap();
    assert!(line.place_drawn(1));
    assert_eq!((line.caret(), line.drawn()), (1, 1));
    assert!(!line.place_drawn(2), "inside the pair stays before it");
    assert!(line.place_drawn(3));
    assert_eq!((line.caret(), line.drawn()), (1 + EMOJI.len(), 3));
    assert!(line.place_drawn(u32::MAX));
    assert_eq!((line.caret(), line.drawn()), (text.len(), 4));
}

#[test]
fn a_click_on_a_mask_counts_dots() {
    let mut line = Line::with(BEH_FATHA_TEH, 5).unwrap();
    assert!(line.place_dots(1), "its own dot, inside the grapheme");
    assert_eq!((line.caret(), line.dots()), (2, 1));
    assert!(line.place_dots(2));
    assert_eq!((line.caret(), line.dots()), (BEH_FATHA, 2));
    assert!(line.place_dots(99));
    assert_eq!(line.caret(), BEH_FATHA_TEH.len());
}

#[test]
fn a_place_between_bytes_keeps_a_char_boundary() {
    let mut line = Line::with("\u{628}a", 5).unwrap();
    assert!(line.place(1));
    assert_eq!(line.caret(), 0, "inside the beh");
    assert!(line.place(99));
    assert_eq!(line.caret(), 3);
    assert!(!line.place(99), "it did not move");
}

#[test]
fn inserts_in_the_middle_never_grow_the_line_so_no_copy_stays_behind() {
    let (mut line, wide) = (Line::new(4), '\u{1e900}');
    let room = line.text.capacity();
    for at in [0, 0, 4, 8] {
        line.place(at);
        line.push(wide).unwrap();
    }
    for at in [4, 12, 0, 8] {
        line.place(at);
        line.push(RLM).unwrap();
    }
    assert_eq!(line.push(RLM), Err(EntryError::Refused));
    assert_eq!(line.text.capacity(), room, "never reallocated");
    assert!(line.delete());
    line.pop();
    assert_eq!(line.text.capacity(), room, "nor by taking back");
}

#[test]
fn debug_never_shows_the_caret() {
    let line = Line::with("abc", 73_519).unwrap();
    let shown = format!("{line:?}");
    let hidden = format!("at: {}", kx_module_api::REDACTED_MARKER);
    assert!(shown.contains(&hidden), "{shown}");
    assert!(
        !shown.chars().any(|c| c.is_ascii_digit()),
        "no count tells its length: {shown}"
    );
}

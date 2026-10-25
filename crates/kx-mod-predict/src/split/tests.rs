//! The shared rules: where a word stands among its neighbours, and where each apostrophe piece stands.

use super::{Edges, Place, edges, places, spans};

#[cfg(feature = "build")]
mod text;

/// The edges of each word of `text`, with whether only spaces part it from the word before.
fn all_edges(text: &str) -> Vec<(Edges, bool)> {
    let found = spans(text, '\'');
    (0..found.len()).map(|k| edges(text, &found, k)).collect()
}

/// Edges free before and after as given.
fn free(before: bool, after: bool) -> Edges {
    Edges { before, after }
}

/// A place.
fn place(alone: bool, joined: bool) -> Place {
    Place { alone, joined }
}

/// The words `spans` finds in `text`.
fn words(text: &str) -> Vec<&str> {
    let found = spans(text, '\'');
    found.iter().filter_map(|&(s, e)| text.get(s..e)).collect()
}

#[test]
fn a_joiner_inside_a_word_stays_and_one_at_its_edge_is_trimmed() {
    assert_eq!(words("کتاب\u{200C}ها خوب"), ["کتاب\u{200C}ها", "خوب"]);
    assert_eq!(
        words("می\u{200C}\u{200D}خواهم"),
        ["می\u{200C}\u{200D}خواهم"]
    );
    assert_eq!(words("\u{200C}کتاب\u{200D} ها"), ["کتاب", "ها"]);
    assert_eq!(words("l'\u{200C}x"), ["l'", "x"], "after an apostrophe");
    assert_eq!(all_edges("کتاب\u{200C} ها").len(), 2);
}

#[test]
fn a_mark_then_a_space_leaves_a_side_free_but_breaks_the_chain() {
    let expected = [
        (free(true, true), true),
        (free(true, true), true),
        (free(true, true), false),
    ];
    assert_eq!(all_edges("say hello, world"), expected);
}

#[test]
fn the_text_edges_are_free_and_leading_marks_part_the_first_word() {
    assert_eq!(all_edges("(hello)"), [(free(true, true), false)]);
    assert_eq!(all_edges("hello"), [(free(true, true), true)]);
}

#[test]
fn a_mark_with_no_space_glues_its_neighbours() {
    let expected = [(free(true, false), true), (free(false, true), false)];
    assert_eq!(all_edges("a.b"), expected);
}

#[test]
fn an_elision_frees_the_side_it_parts_and_lets_the_chain_through() {
    let pieces = [("l'", true), ("homme", false)];
    let found: Vec<Place> = places(&pieces, free(true, true), false).collect();
    assert_eq!(found, [place(true, false), place(true, true)]);
    let glued: Vec<Place> = places(&pieces, free(true, false), true).collect();
    assert_eq!(glued, [place(true, true), place(false, true)]);
}

#[test]
fn a_split_apostrophe_glues_both_pieces_and_breaks_the_chain() {
    let pieces = [("Muiriel", false), ("s", false)];
    let found: Vec<Place> = places(&pieces, free(true, true), true).collect();
    assert_eq!(found, [place(false, true), place(false, false)]);
}

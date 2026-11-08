//! A read that hit its cap (plan edge case 9): the selection goes on past the text read, so Translate's text ends at a
//! whole word even when cleaning a leading space or a hidden mark made it shorter than `translate_chars`.

use super::*;

/// The `translate_chars` these reads were capped for, one past it.
const CHARS: u16 = 50;

/// The text Google's link carries for a read of `read`, which hit its cap.
fn capped(read: &str) -> String {
    assert_eq!(read.chars().count(), usize::from(CHARS) + 1, "{read:?}");
    sent(google().cut_link(read, "fr", CHARS).unwrap().expose())
}

#[test]
fn a_leading_space_never_leaves_the_half_word_the_read_cut() {
    let read = format!(" {}hotel", "word ".repeat(9));
    assert_eq!(
        capped(&read),
        "word ".repeat(9).trim_end(),
        "never hotel of hotelier"
    );
    let arabic = format!(" {}با", "مرحبا ".repeat(8));
    assert_eq!(
        capped(&arabic),
        "مرحبا ".repeat(8).trim_end(),
        "never با of بالعالم"
    );
}

#[test]
fn a_right_to_left_mark_inside_never_leaves_the_half_word_the_read_cut() {
    let read = format!("wo\u{200f}rd {}hotel", "word ".repeat(8));
    assert_eq!(capped(&read), "word ".repeat(9).trim_end());
    let arabic = format!("مرحبا\u{200f} {}با", "مرحبا ".repeat(7));
    assert_eq!(capped(&arabic), "مرحبا ".repeat(8).trim_end());
}

#[test]
fn a_read_that_ended_on_a_space_keeps_its_last_word() {
    let read = format!(" {}", "word ".repeat(10));
    assert_eq!(capped(&read), "word ".repeat(10).trim_end());
    let marked = format!("{}word \u{200b}", "word ".repeat(9));
    assert_eq!(
        capped(&marked),
        "word ".repeat(10).trim_end(),
        "a hidden mark after the space"
    );
    let arabic = format!("  {}مرحبا\u{2028}\u{200f}", "مرحبا ".repeat(7));
    assert_eq!(
        capped(&arabic),
        "مرحبا ".repeat(8).trim_end(),
        "a hidden break, then a mark"
    );
}

#[test]
fn inside_one_long_word_a_letter_never_loses_the_mark_the_read_cut() {
    // The read ended on a ba whose kasra it did not reach.
    let read = format!(" \u{200f}{}ب", "بِ".repeat(24));
    assert_eq!(capped(&read), "بِ".repeat(24));
}

#[test]
fn a_whole_text_keeps_its_last_word() {
    let text = " word word hotelier";
    assert_eq!(sent(&link(text, CHARS).unwrap()), "word word hotelier");
}

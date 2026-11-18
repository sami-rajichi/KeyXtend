use kx_module_api::REDACTED_MARKER;

use super::Line;

/// The most a test line holds.
const MOST: usize = 8;
/// A letter of two UTF-16 units and four bytes.
const WIDE: char = '\u{1f600}';
/// "é" as e and a combining acute: two characters, one grapheme.
const E_ACUTE: &str = "e\u{301}";

/// A shown line holding `text`, the caret at its end.
fn shown(text: &str) -> Line {
    Line::with(text, MOST)
}

/// A hidden line holding `text`, the caret at its end.
fn hidden(text: &str) -> Line {
    let mut line = Line::new(MOST, true);
    line.write(text);
    line
}

#[test]
fn letters_go_in_at_the_caret_which_stays_after_them() {
    let mut line = shown("ab");
    assert!(line.step(false));
    assert!(line.write("xy"));
    assert_eq!((line.text(), line.caret()), ("axyb", 3));
}

#[test]
fn backspace_takes_the_character_before_the_caret_and_delete_the_one_after() {
    let mut line = shown("abc");
    line.step(false);
    assert!(line.erase());
    assert_eq!((line.text(), line.caret()), ("ac", 1));
    assert!(line.delete());
    assert_eq!((line.text(), line.caret()), ("a", 1));
    assert!(!line.delete(), "nothing after the caret");
    line.step(false);
    assert!(!line.erase(), "nothing before it");
}

#[test]
fn a_shown_line_moves_and_deletes_whole_letters_and_a_hidden_one_a_dot_at_a_time() {
    let mut line = shown(E_ACUTE);
    assert!(line.step(false));
    assert_eq!(line.caret(), 0, "the accent moves with its letter");
    assert!(line.delete());
    assert!(line.is_empty(), "and goes with it");
    let mut key = hidden(E_ACUTE);
    assert_eq!((key.count(), key.caret()), (2, 2), "one dot per character");
    assert!(key.step(false));
    assert_eq!(key.caret(), 1, "one dot back");
    assert!(key.delete());
    assert_eq!(key.text(), "e");
}

#[test]
fn the_caret_counts_utf16_units_on_a_shown_line_and_dots_on_a_hidden_one() {
    let text = format!("a{WIDE}b");
    let mut line = shown(&text);
    assert_eq!(line.caret(), 4);
    assert!(line.place(3));
    assert_eq!(line.caret(), 3, "after the wide letter");
    assert!(line.place(2), "never inside it");
    assert_eq!(line.caret(), 1);
    let mut key = hidden(&text);
    assert_eq!(key.caret(), 3);
    assert!(key.place(1));
    assert_eq!(key.caret(), 1);
    assert!(!key.place(1), "it stays");
    assert!(key.place(99));
    assert_eq!(key.caret(), 3, "clamped to the end");
}

#[test]
fn the_caret_stops_at_both_ends() {
    let mut line = shown("a");
    assert!(!line.step(true));
    assert!(line.step(false));
    assert!(!line.step(false));
}

#[test]
fn a_line_takes_letters_until_past_its_most_and_never_grows() {
    let mut line = Line::new(MOST, true);
    let room = line.room();
    assert!(room >= MOST + char::MAX_LEN_UTF8);
    assert!(line.write(&"a".repeat(MOST + 3)));
    assert_eq!(line.count(), 9, "one past the most, so Save can say why");
    assert!(!line.write("b"));
    assert_eq!(line.room(), room, "never reallocated");
    let mut wide = Line::new(MOST, false);
    wide.write(&WIDE.to_string().repeat(4));
    assert_eq!(wide.room(), Line::new(MOST, false).room());
}

#[test]
fn hidden_marks_and_line_breaks_never_go_in() {
    let mut line = Line::new(MOST, false);
    assert!(!line.write("\n\r\t\u{200e}\u{202e}\u{200b}"));
    assert!(line.is_empty());
    assert!(line.write("a\nb"), "the rest still goes in");
    assert_eq!(line.text(), "ab");
    assert_eq!(
        shown("a\u{200f}b").text(),
        "ab",
        "a kept text loses them too"
    );
}

#[test]
fn a_copy_keeps_its_text_caret_and_room() {
    let mut line = shown("abc");
    line.step(false);
    let copy = line.clone();
    assert_eq!((copy.text(), copy.caret()), ("abc", 2));
    assert_eq!(copy.room(), line.room());
}

#[test]
fn debug_never_shows_the_text_or_the_caret() {
    let shown = format!("{:?}", hidden("hunter2"));
    assert!(
        !shown.contains("hunter2") && !shown.contains('7') && shown.contains(REDACTED_MARKER),
        "{shown}"
    );
}

#[test]
fn its_text_leaves_as_a_secret() {
    assert_eq!(hidden("abc").into_secret().expose().as_str(), "abc");
}

mod props;

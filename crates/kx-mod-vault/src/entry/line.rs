//! A line the user types letter by letter at its caret, such as a value on the New page. It reserves room for its most
//! characters and as many stripped marks at once and is wiped when dropped, so a growing value, or a letter put in its
//! middle, never leaves an unwiped copy.

use std::fmt;

use kx_module_api::caret::{self, Side};
use kx_module_api::{Redacted, Secret, Zeroizing};

use super::EntryError;
use super::text::{STRIPPED, typable};

/// The bytes one character and one stripped mark beside it may take at most.
const ROOM_PER_CHAR: usize = 2 * char::MAX_LEN_UTF8;

/// A typed line of at most `max` characters, holding no line break or other hidden mark but those a shown text loses.
/// Those marks do not count toward `max`, as a saved shown text loses them, but at most `max` of them fit.
pub struct Line {
    text: Zeroizing<String>,
    /// The caret, in bytes, on a char boundary; it tells a secret's length, so `Debug` redacts it.
    at: usize,
    chars: usize,
    marks: usize,
    max: usize,
}

impl Line {
    /// An empty line of at most `max` characters and `max` stripped marks, with room for all of them.
    #[must_use]
    pub fn new(max: usize) -> Self {
        let room = max.saturating_mul(ROOM_PER_CHAR);
        Self {
            text: Zeroizing::new(String::with_capacity(room)),
            at: 0,
            chars: 0,
            marks: 0,
            max,
        }
    }

    /// A line holding `text`, such as a saved value to edit, with the caret at its end.
    ///
    /// # Errors
    /// `Refused` when it holds more than `max` characters or a character no line takes.
    pub fn with(text: &str, max: usize) -> Result<Self, EntryError> {
        let mut line = Self::new(max);
        text.chars().try_for_each(|c| line.push(c))?;
        Ok(line)
    }

    /// Types `c` at the caret, which stays after it; the room `new` reserved means it never reallocates.
    ///
    /// # Errors
    /// `Refused` for a line break or another hidden mark, or once the line holds `max` characters or `max` marks.
    pub fn push(&mut self, c: char) -> Result<(), EntryError> {
        let max = self.max;
        let count = self.count(c);
        if !typable(c) || *count >= max {
            return Err(EntryError::Refused);
        }
        *count += 1;
        self.text.insert(self.at, c);
        self.at += c.len_utf8();
        Ok(())
    }

    /// Removes the character before the caret, so a haraka goes alone.
    pub fn pop(&mut self) -> Option<char> {
        let last = self.before().chars().next_back()?;
        self.at -= last.len_utf8();
        self.take();
        Some(last)
    }

    /// Removes the one character after the caret, a hidden mark too, as a mask's dot; false when none is left.
    pub fn delete_dot(&mut self) -> bool {
        let there = self.at < self.text.len();
        if there {
            self.take();
        }
        there
    }

    /// Removes the character at the caret and its count; one must be there.
    fn take(&mut self) {
        let gone = self.text.remove(self.at);
        let count = self.count(gone);
        *count = count.saturating_sub(1);
    }

    /// Removes the grapheme after the caret; false when nothing shows there.
    pub fn delete(&mut self) -> bool {
        let end = caret::on(&self.text, self.at);
        let gone: (usize, usize) = self.text.drain(self.at..end).fold((0, 0), |(n, marks), c| {
            (n + 1, marks + usize::from(STRIPPED.contains(&c)))
        });
        self.marks = self.marks.saturating_sub(gone.1);
        self.chars = self.chars.saturating_sub(gone.0 - gone.1);
        gone.0 > 0
    }

    /// Moves the caret as the arrow on `side` does; false when it stays.
    pub fn arrow(&mut self, side: Side) -> bool {
        self.place(caret::arrow(&self.text, self.at, side))
    }

    /// Moves the caret one of a mask's dots along the text, on when `on`, whatever way it reads; false when it stays.
    pub fn step_dot(&mut self, on: bool) -> bool {
        let to = if on {
            caret::dot_on(&self.text, self.at)
        } else {
            caret::dot_back(&self.text, self.at)
        };
        self.place(to)
    }

    /// Puts the caret at byte `at`, clamped to a char boundary; false when it stays.
    pub fn place(&mut self, at: usize) -> bool {
        let at = caret::clamped(&self.text, at);
        let moved = at != self.at;
        self.at = at;
        moved
    }

    /// Puts the caret `at` UTF-16 units into the text the line draws; false when it stays.
    pub fn place_drawn(&mut self, at: u32) -> bool {
        self.place(caret::of_units(&self.text, at))
    }

    /// Puts the caret `at` dots into the line's mask; false when it stays.
    pub fn place_dots(&mut self, at: u32) -> bool {
        self.place(caret::of_dots(&self.text, at))
    }

    /// Moves a caret a mask's dot left inside a grapheme to its start, as a shown line's letter must not split it.
    pub fn to_edge(&mut self) {
        self.place_drawn(self.drawn());
    }

    /// The caret, in bytes.
    #[must_use]
    pub const fn caret(&self) -> usize {
        self.at
    }

    /// The UTF-16 units the line draws before the caret.
    #[must_use]
    pub fn drawn(&self) -> u32 {
        caret::units(&self.text, self.at)
    }

    /// The dots the line's mask draws before the caret.
    #[must_use]
    pub fn dots(&self) -> u32 {
        caret::dots(&self.text, self.at)
    }

    /// The text before the caret.
    #[must_use]
    pub fn before(&self) -> &str {
        caret::before(&self.text, self.at)
    }

    /// The count `c` adds to: the marks for a stripped mark, else the characters.
    fn count(&mut self, c: char) -> &mut usize {
        if STRIPPED.contains(&c) {
            &mut self.marks
        } else {
            &mut self.chars
        }
    }

    /// How many characters it holds, the stripped marks left out.
    #[must_use]
    pub const fn chars(&self) -> usize {
        self.chars
    }

    /// The text typed so far.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The text typed, for a field's value.
    #[must_use]
    pub fn into_secret(self) -> Secret {
        Redacted::new(self.text)
    }
}

/// Never shows the text, the caret or a count: each tells its length, and so does `max`, as `Editor::of` fits it to a
/// long saved text.
impl fmt::Debug for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Line")
            .field("text", &Redacted::new(()))
            .field("at", &Redacted::new(self.at))
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kx_module_api::REDACTED_MARKER;

    /// A character of four bytes in UTF-8, the Adlam letter alif.
    const WIDE: char = '\u{1e900}';

    #[test]
    fn a_line_holds_its_room_from_the_start_and_never_grows() {
        let mut line = Line::new(5);
        let room = line.text.capacity();
        assert!(room >= 5 * char::MAX_LEN_UTF8);
        for _ in 0..5 {
            line.push(WIDE).unwrap();
        }
        assert_eq!(line.text.capacity(), room, "never reallocated");
        assert_eq!(line.push('a'), Err(EntryError::Refused));
        assert_eq!(line.chars(), 5);
    }

    #[test]
    fn a_line_takes_no_line_break_or_hidden_mark_but_the_stripped_ones() {
        let mut line = Line::new(10);
        for c in ['\n', '\r', '\u{2028}', '\t', '\u{202e}', '\u{200b}'] {
            assert_eq!(line.push(c), Err(EntryError::Refused), "{c:?}");
        }
        for c in ['\u{200e}', '\u{200f}', '\u{61c}', '\u{200c}', 'ع'] {
            assert_eq!(line.push(c), Ok(()), "{c:?}");
        }
        assert_eq!(line.chars(), 2, "the stripped marks do not count");
    }

    #[test]
    fn the_stripped_marks_do_not_count_toward_the_most() {
        let mut line = Line::new(3);
        for c in ['a', '\u{200e}', 'b', '\u{61c}', 'c', '\u{200f}'] {
            assert_eq!(line.push(c), Ok(()), "{c:?}");
        }
        assert_eq!(line.chars(), 3);
        assert_eq!(line.push('d'), Err(EntryError::Refused));
        assert_eq!(line.pop(), Some('\u{200f}'));
        assert_eq!(line.chars(), 3, "a mark taken back leaves the count");
        assert!(
            Line::with("\u{200f}+33 6\u{200e}", 5).is_ok(),
            "as a saved shown text counts"
        );
    }

    #[test]
    fn marks_past_the_most_are_refused_so_the_line_never_grows() {
        let mut line = Line::new(2);
        let room = line.text.capacity();
        for c in [WIDE, WIDE, '\u{200e}', '\u{200f}'] {
            line.push(c).unwrap();
        }
        assert_eq!(line.push('\u{61c}'), Err(EntryError::Refused));
        assert_eq!(line.text.capacity(), room, "never reallocated");
    }

    #[test]
    fn pop_takes_the_last_character_back() {
        let mut line = Line::with("سلام", 10).unwrap();
        assert_eq!(line.pop(), Some('م'));
        assert_eq!((line.text(), line.chars()), ("سلا", 3));
        assert_eq!(Line::new(3).pop(), None);
        assert_eq!(line.into_secret().expose().as_str(), "سلا");
    }

    #[test]
    fn a_saved_text_too_long_or_broken_is_refused() {
        assert!(Line::with("abc", 3).is_ok());
        assert_eq!(Line::with("abcd", 3).err(), Some(EntryError::Refused));
        assert_eq!(Line::with("a\nb", 9).err(), Some(EntryError::Refused));
    }

    #[test]
    fn debug_never_shows_the_text() {
        let shown = format!("{:?}", Line::with("hunter2", 9).unwrap());
        assert!(
            !shown.contains("hunter2") && shown.contains(REDACTED_MARKER),
            "{shown}"
        );
    }
}

#[cfg(test)]
mod carets;

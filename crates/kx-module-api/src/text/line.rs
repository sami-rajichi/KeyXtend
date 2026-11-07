//! One line for search: a run of blanks, line breaks and control characters reads as one space, so a pasted query and
//! the text it looks in line up in every module.

/// What a run of blanks reads as, in a search line and the text it looks in.
pub const SPACE: char = ' ';

/// `chars` on one line: each run of blanks, line breaks and control characters as one space.
pub fn one_line(chars: impl IntoIterator<Item = char>) -> impl Iterator<Item = char> {
    let mut gap = false;
    chars.into_iter().filter_map(move |c| {
        let blank = c.is_whitespace() || c.is_control();
        let was = std::mem::replace(&mut gap, blank);
        match (blank, was) {
            (false, _) => Some(c),
            (true, false) => Some(SPACE),
            (true, true) => None,
        }
    })
}

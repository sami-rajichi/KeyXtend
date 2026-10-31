//! The worker's command line as the C runtime reads it back: the program in quotes, then each argument, quoted only
//! when it must be, with backslashes doubled before a quote.

use std::ffi::OsStr;
use std::iter;
use std::os::windows::ffi::OsStrExt;

/// A quote, in UTF-16.
const QUOTE: u16 = b'"' as u16;
/// A backslash, in UTF-16.
const SLASH: u16 = b'\\' as u16;
/// The space between arguments, in UTF-16.
const SPACE: u16 = b' ' as u16;
/// What makes an argument need quotes.
const SPECIAL: [char; 5] = [' ', '\t', '\n', '\x0b', '"'];

/// `exe`, then `args`, as one null-ended command line.
pub(super) fn command_line(exe: &OsStr, args: &[String]) -> Vec<u16> {
    let mut line = vec![QUOTE];
    line.extend(exe.encode_wide());
    line.push(QUOTE);
    for arg in args {
        line.push(SPACE);
        quote(arg, &mut line);
    }
    line.push(0);
    line
}

/// Writes `arg` to `line`, in quotes when it is empty or holds a space, a tab, a line break or a quote.
fn quote(arg: &str, line: &mut Vec<u16>) {
    if !arg.is_empty() && !arg.contains(SPECIAL) {
        line.extend(arg.encode_utf16());
        return;
    }
    line.push(QUOTE);
    let mut slashes = 0;
    for unit in arg.encode_utf16() {
        if unit == SLASH {
            slashes += 1;
            continue;
        }
        // Backslashes before a quote are doubled, and the quote escaped; elsewhere they stay as they are.
        let doubled = if unit == QUOTE {
            2 * slashes + 1
        } else {
            slashes
        };
        line.extend(iter::repeat_n(SLASH, doubled));
        line.push(unit);
        slashes = 0;
    }
    // Backslashes before the closing quote are doubled, so it still closes.
    line.extend(iter::repeat_n(SLASH, 2 * slashes));
    line.push(QUOTE);
}

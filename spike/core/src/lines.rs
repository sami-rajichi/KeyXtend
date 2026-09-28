//! JSON lines between a face and its worker, capped in length so a runaway writer cannot fill memory.

use std::io::{BufRead, Read};

/// Bytes a line end may take: a carriage return and a line feed.
const LINE_END: u64 = 2;

/// The next line without its line end, invalid UTF-8 replaced; a line over `max` bytes is skipped and reported.
pub fn next_line(r: &mut impl BufRead, max: usize) -> Option<Result<String, String>> {
    let mut line = Vec::new();
    let limit = u64::try_from(max)
        .unwrap_or(u64::MAX)
        .saturating_add(LINE_END);
    let n = r.by_ref().take(limit).read_until(b'\n', &mut line).ok()?;
    if n == 0 {
        return None;
    }
    let ended = line.last() == Some(&b'\n');
    let body = line.strip_suffix(b"\n").unwrap_or(&line);
    let body = body.strip_suffix(b"\r").unwrap_or(body);
    if body.len() > max {
        if !ended {
            skip_line(r);
        }
        return Some(Err(format!("a line over {max} bytes was skipped")));
    }
    Some(Ok(String::from_utf8_lossy(body).trim_end().to_string()))
}

/// Drops the rest of the current line.
fn skip_line(r: &mut impl BufRead) {
    while let Ok(buf) = r.fill_buf() {
        if buf.is_empty() {
            return;
        }
        let (n, end) = match buf.iter().position(|&b| b == b'\n') {
            Some(i) => (i + 1, true),
            None => (buf.len(), false),
        };
        r.consume(n);
        if end {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn lines_come_one_by_one_without_their_ends() {
        let mut r = Cursor::new(b"one\r\ntwo\nlast".to_vec());
        assert_eq!(next_line(&mut r, 8), Some(Ok("one".into())));
        assert_eq!(next_line(&mut r, 8), Some(Ok("two".into())));
        assert_eq!(next_line(&mut r, 8), Some(Ok("last".into())));
        assert_eq!(next_line(&mut r, 8), None);
    }

    #[test]
    fn a_long_line_is_skipped_and_the_next_one_is_read() {
        let mut r = Cursor::new(b"0123456789abcdef\nok\n".to_vec());
        assert!(next_line(&mut r, 4).is_some_and(|l| l.is_err()));
        assert_eq!(next_line(&mut r, 4), Some(Ok("ok".into())));
        let mut exact = Cursor::new(b"abcd\n".to_vec());
        assert_eq!(
            next_line(&mut exact, 4),
            Some(Ok("abcd".into())),
            "a line of exactly max bytes is kept"
        );
    }

    #[test]
    fn the_cap_counts_the_words_not_the_line_end() {
        let mut r = Cursor::new(b"abcd\r\nabcde\nok\n".to_vec());
        assert_eq!(next_line(&mut r, 4), Some(Ok("abcd".into())), "CRLF");
        assert!(next_line(&mut r, 4).is_some_and(|l| l.is_err()));
        assert_eq!(next_line(&mut r, 4), Some(Ok("ok".into())));
    }

    #[test]
    fn invalid_utf8_is_replaced_not_fatal() {
        let mut r = Cursor::new(b"a\xFFb\n".to_vec());
        assert_eq!(next_line(&mut r, 8), Some(Ok("a\u{FFFD}b".into())));
    }
}

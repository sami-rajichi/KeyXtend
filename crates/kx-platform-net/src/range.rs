//! Byte ranges: the `Range` header a resume sends, the `Content-Range` and `Content-Length` a server answers, and
//! where an answer's body starts in the file.

use crate::{Head, NetError};

/// HTTP's whole-file answer.
pub(crate) const OK: u16 = 200;
/// HTTP's partial answer, to a `Range` request.
pub(crate) const PARTIAL: u16 = 206;
/// The start of the header line a resume sends, before its unit.
const RANGE: &str = "Range: ";
/// Between the unit and the first byte asked.
const EQUALS: char = '=';
/// Ends a header line.
const CRLF: &str = "\r\n";
/// The only range unit, matched in any case.
pub(crate) const UNIT: &str = "bytes";
/// Between a span's first and last bytes; it also ends an open range, which runs to the file's end.
pub(crate) const DASH: char = '-';
/// Between a span and the total.
pub(crate) const OVER: char = '/';
/// A total the server does not know.
const UNKNOWN: &str = "*";

/// The bytes a partial answer holds, from `Content-Range`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    /// The first byte, counted from 0.
    pub first: u64,
    /// The last byte, at least `first`.
    pub last: u64,
    /// The whole file's size, above `last`, when the server knows it.
    pub total: Option<u64>,
}

/// The header line that asks for the file from byte `from` on; none for 0, which asks for the whole file.
#[cfg_attr(
    not(windows),
    allow(
        dead_code,
        reason = "Only WinHTTP sends it; the tests check it everywhere."
    )
)]
pub(crate) fn range_header(from: u64) -> Option<String> {
    (from > 0).then(|| format!("{RANGE}{UNIT}{EQUALS}{from}{DASH}{CRLF}"))
}

/// The span `text` names, as in `bytes 4-9/10` or `bytes 4-9/*`; none for anything else.
pub(crate) fn span(text: &str) -> Option<Span> {
    let (unit, rest) = text.trim().split_once(' ')?;
    if !unit.eq_ignore_ascii_case(UNIT) {
        return None;
    }
    let (bytes, total) = rest.split_once(OVER)?;
    let (first, last) = bytes.split_once(DASH)?;
    let (first, last) = (number(first)?, number(last)?);
    let total = if total == UNKNOWN {
        None
    } else {
        Some(number(total)?)
    };
    let fits = first <= last && total.is_none_or(|t| last < t);
    fits.then_some(Span { first, last, total })
}

/// The `Content-Length` header's number; none when there is none.
///
/// # Errors
/// `Server` when it is not digits only, or does not fit 64 bits.
pub(crate) fn length(text: Option<&str>) -> Result<Option<u64>, NetError> {
    text.map(|t| number(t.trim()).ok_or(NetError::Server))
        .transpose()
}

/// Where `head`'s body starts in the file for a request from byte `from`, and the file's size when known.
///
/// # Errors
/// `Range` for a partial answer that starts elsewhere, `Server` for a bad length, `Status` for any other status.
pub(crate) fn start(head: &Head, from: u64) -> Result<(u64, Option<u64>), NetError> {
    match head.status {
        OK => Ok((0, length(head.length.as_deref())?)),
        PARTIAL => {
            let span = head
                .range
                .as_deref()
                .and_then(span)
                .ok_or(NetError::Range)?;
            if span.first == from {
                Ok((from, span.total))
            } else {
                Err(NetError::Range)
            }
        }
        code => Err(NetError::Status(code)),
    }
}

/// `text` as a number, when it is ASCII digits only and fits 64 bits.
fn number(text: &str) -> Option<u64> {
    let digits = !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit());
    digits.then(|| text.parse().ok()).flatten()
}

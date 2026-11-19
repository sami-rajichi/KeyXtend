//! What a reply's status means, as each shape's table says: a done reply holds a piece's text where its shape reads it,
//! or a key test's JSON object; otherwise the first row whose statuses and error words take the answer wins, and a
//! status no row takes is strange. A 429's wait is read from its `Retry-After`.

use super::reply;
use super::shape::Verdict;
use kx_ipc::NetFault;
use kx_platform_net::Head;
use std::ops::RangeInclusive;
use zeroize::Zeroizing;

/// Statuses whose reply holds what was asked.
pub const DONE: RangeInclusive<u16> = 200..=299;
/// The answer header a 429's wait is read from.
pub const RETRY: &str = "retry-after";

/// What a status means.
#[derive(Copy, Clone)]
pub enum Then {
    /// This fault.
    Fault(NetFault),
    /// Too many requests: a used allowance, or a wait.
    Busy,
}

/// One row of a shape's table: the statuses it covers, what the error must say, and what it means.
pub struct Row<S> {
    codes: RangeInclusive<u16>,
    says: S,
    then: Then,
}

/// A table row.
pub const fn row<S>(codes: RangeInclusive<u16>, says: S, then: Then) -> Row<S> {
    Row { codes, says, then }
}

/// What a piece's reply with `head` and `body` says: once done, the text `read` finds, or strange without one; else
/// `judge`'s reading.
pub fn text(
    head: &Head,
    body: &[u8],
    read: impl FnOnce(&[u8]) -> Option<Zeroizing<String>>,
    judge: impl FnOnce() -> Verdict<Zeroizing<String>>,
) -> Verdict<Zeroizing<String>> {
    if DONE.contains(&head.status) {
        return read(body).map_or(Verdict::Fault(NetFault::Strange), Verdict::Said);
    }
    judge()
}

/// What a key test's reply with `head` and `body` says: the key works once done with a JSON object, else `judge`'s
/// reading.
pub fn tested(head: &Head, body: &[u8], judge: impl FnOnce() -> Verdict<()>) -> Verdict<()> {
    match (DONE.contains(&head.status), reply::object(body)) {
        (true, true) => Verdict::Said(()),
        (true, false) => Verdict::Fault(NetFault::Strange),
        (false, _) => judge(),
    }
}

/// What `head`'s answer means by the first row of `table` whose error `said` takes; a 429's row is `busy`'s.
pub fn judge<S: Copy, T>(
    table: &[Row<S>],
    head: &Head,
    said: impl Fn(S) -> bool,
    busy: impl FnOnce(&Head) -> Verdict<T>,
) -> Verdict<T> {
    let takes = |r: &&Row<S>| r.codes.contains(&head.status) && said(r.says);
    match table.iter().find(takes).map(|r| r.then) {
        Some(Then::Fault(fault)) => Verdict::Fault(fault),
        Some(Then::Busy) => busy(head),
        None => Verdict::Fault(NetFault::Strange),
    }
}

/// A 429's wait: the whole seconds its `Retry-After` asks for, when it names them.
pub fn wait<T>(head: &Head) -> Verdict<T> {
    Verdict::Busy(head.header(RETRY).and_then(reply::seconds))
}

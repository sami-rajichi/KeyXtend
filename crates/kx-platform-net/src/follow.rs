//! One download's request: each address is checked before it is asked, and each redirect is followed by hand.

use crate::range::start;
use crate::{Answer, Get, Hop, Http, MAX_HOPS, NetError, Target};

/// HTTP's "found" redirect, the one the fake sends.
pub(crate) const FOUND: u16 = 302;
/// The statuses that name another address in `Location`.
const MOVED: [u16; 5] = [301, FOUND, 303, 307, 308];

/// Gets `ask.url` over `http`, following at most `MAX_HOPS` redirects, each to a listed HTTPS host.
///
/// # Errors
/// `NotHttps` or `Unlisted` before an address is asked, `Hops` past the limit, `Server` for a redirect without its
/// address, and any error from `send` or `start`.
pub(crate) fn get<H: Http + ?Sized>(http: &H, ask: &Get<'_>) -> Result<Answer, NetError> {
    let mut url = Target::parse(ask.url)?;
    for _ in 0..=MAX_HOPS {
        url.listed(ask.hosts)?;
        let hop = Hop {
            target: &url,
            from: ask.from,
            stall_s: ask.stall_s,
        };
        let reply = http.send(&hop)?;
        if !MOVED.contains(&reply.head.status) {
            let (start, total) = start(&reply.head, ask.from)?;
            let body = reply.body;
            return Ok(Answer {
                url,
                start,
                total,
                body,
            });
        }
        let to = reply.head.location.as_deref().ok_or(NetError::Server)?;
        url = url.join(to)?;
    }
    Err(NetError::Hops)
}

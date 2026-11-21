//! What Gemini's error answers mean [N5][N7]: a detail's exact reason or the canonical status first, then the status
//! table; a 429 waits as asked, by `Retry-After` or the error's `RetryInfo` delay, unless the free tier's day ran out.

use super::super::reason::Reason;
use super::super::reply::{self, Coded, SAYING};
use super::super::shape::Verdict;
use super::super::status::{self, LANGUAGE, RETRY, Row, Then, row};
use kx_ipc::NetFault;
use kx_platform_net::Head;

/// Google's error object [N7].
const ERROR: &str = "error";
/// Where the error names its canonical status.
const STATUS: Coded = Coded {
    object: ERROR,
    names: &["status"],
};
/// Reasons that ask for a step on the account's site, which no key fixes: a free tier to verify [N7] and an unpaid
/// balance (assumed).
const PAY: &[&str] = &["FREE_TIER_VERIFICATION_REQUIRED", "BILLING_DISABLED"];
/// The reason of a key that is not valid, which [N7] gives only as a 400 (assumed).
const BAD_KEY: &[&str] = &["API_KEY_INVALID"];
/// A 403's status: a leaked key, or one without the right [N7].
const DENIED: &[&str] = &["PERMISSION_DENIED"];
/// Words of a location the service refuses [N7].
const LOCATION: &[&str] = &["location is not supported"];
/// What the free tier's daily quota id names, as in `GenerateRequestsPerDayPerProjectPerModel-FreeTier` (assumed
/// from Google's quota ids, checked at task 24); a paid tier's day stays a wait.
const PER_DAY: &str = "PerDay";
const FREE_TIER: &str = "FreeTier";

/// The details' fields read inside the error [N7]: an `ErrorInfo`'s reason, a `RetryInfo`'s wait such as `37s`, and a
/// `QuotaFailure`'s quota ids run out. A detail of another shape keeps nothing and hides no other.
const REASON: &str = "reason";
const DELAY: &str = "retryDelay";
const QUOTA_ID: &str = "quotaId";
const KEPT: &[&str] = &[REASON, DELAY, QUOTA_ID];

/// What a row asks of the error beside its status.
#[derive(Copy, Clone)]
enum Says {
    /// Nothing.
    Any,
    /// A detail's reason is exactly one of these.
    Reason(&'static [&'static str]),
    /// Its canonical status is exactly one of these.
    Status(&'static [&'static str]),
    /// Its words hold one of these.
    Words(&'static [&'static str]),
}

/// The status table from its error pages [N5][N7]: the first row that matches wins, and anything else is strange, a
/// plain 400 for a request it cannot read too.
const TABLE: &[Row<Says>] = &[
    // No redirect is followed, so the key and the sound reach one address only.
    row(300..=399, Says::Any, Then::Fault(NetFault::Strange)),
    row(400..=499, Says::Reason(PAY), Then::Fault(NetFault::Account)),
    row(400..=499, Says::Reason(BAD_KEY), Then::Fault(NetFault::Key)),
    row(
        400..=499,
        Says::Words(LOCATION),
        Then::Fault(NetFault::Region),
    ),
    row(403..=403, Says::Status(DENIED), Then::Fault(NetFault::Key)),
    row(
        400..=400,
        Says::Words(LANGUAGE),
        Then::Fault(NetFault::Language),
    ),
    // Too many requests, or a day's quota used, which renews at midnight Pacific time [N5].
    row(429..=429, Says::Any, Then::Busy),
    // The model overloaded, or the service down: tried again.
    row(500..=599, Says::Any, Then::Fault(NetFault::Network)),
];

/// What an answer that is not done means, by the first table row that takes it; the body is read once.
pub fn judge<T>(head: &Head, body: &[u8]) -> Verdict<T> {
    let reason = Reason::keeping(body, &STATUS, SAYING, KEPT);
    let said = |says: Says| match says {
        Says::Any => true,
        Says::Reason(reasons) => reasoned(&reason, reasons),
        Says::Status(names) => reason.coded(names),
        Says::Words(words) => reason.says(words),
    };
    status::judge(TABLE, head, said, |head| busy(head, &reason))
}

/// True when a detail gives one of `reasons` exactly, in any case.
fn reasoned(reason: &Reason, reasons: &[&str]) -> bool {
    let is = |r: &str| reasons.iter().any(|c| c.eq_ignore_ascii_case(r));
    reason.kept(REASON).any(is)
}

/// A 429: the used allowance with no date when the free tier's daily quota ran out, as its delay is no renewal; else
/// a wait for its `Retry-After`, or for its `RetryInfo` delay.
fn busy<T>(head: &Head, reason: &Reason) -> Verdict<T> {
    let daily = |id: &str| id.contains(PER_DAY) && id.contains(FREE_TIER);
    if reason.kept(QUOTA_ID).any(daily) {
        return Verdict::Fault(NetFault::Allowance { renews_s: None });
    }
    let delay = reason.kept(DELAY).find_map(reply::span_s);
    let asked = head.header(RETRY).and_then(reply::seconds);
    Verdict::Busy(asked.or_else(|| delay.and_then(|s| u32::try_from(s).ok())))
}

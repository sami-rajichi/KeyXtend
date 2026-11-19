//! What Gemini's error answers mean [N5][N7]: a detail's exact reason or the canonical status first, then the status
//! table; a 429 waits as asked, by `Retry-After` or the error's `RetryInfo` delay, unless the free tier's day ran out.

use super::super::reply::{self, Coded};
use super::super::shape::Verdict;
use super::super::status::{self, LANGUAGE, RETRY, Row, Then, row};
use kx_ipc::NetFault;
use kx_platform_net::Head;
use serde_json::Value;

/// Google's error object, its list of details and a detail's reason [N7].
const ERROR: &str = "error";
const DETAILS: &str = "details";
const REASON: &str = "reason";
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
/// A `RetryInfo` detail's wait, as a span such as `37s`.
const RETRY_DELAY: &str = "retryDelay";
/// A `QuotaFailure` detail's quotas run out, each by its id.
const VIOLATIONS: &str = "violations";
const QUOTA_ID: &str = "quotaId";
/// What the free tier's daily quota id names, as in `GenerateRequestsPerDayPerProjectPerModel-FreeTier` (assumed
/// from Google's quota ids, checked at task 24); a paid tier's day stays a wait.
const PER_DAY: &str = "PerDay";
const FREE_TIER: &str = "FreeTier";

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

/// What an answer that is not done means, by the first table row that takes it.
pub fn judge<T>(head: &Head, body: &[u8]) -> Verdict<T> {
    let details = details(body);
    let said = |says: Says| match says {
        Says::Any => true,
        Says::Reason(reasons) => reasoned(&details, reasons),
        Says::Status(names) => reply::coded(body, &STATUS, names),
        Says::Words(words) => reply::says(body, words),
    };
    status::judge(TABLE, head, said, |head| busy(head, &details))
}

/// The details of a Google error [N7]; none for any other body.
fn details(body: &[u8]) -> Vec<Value> {
    let mut reply = serde_json::from_slice::<Value>(body).unwrap_or_default();
    let details = reply.get_mut(ERROR).and_then(|e| e.get_mut(DETAILS));
    match details.map(Value::take) {
        Some(Value::Array(details)) => details,
        _ => Vec::new(),
    }
}

/// True when a detail gives one of `reasons` exactly, in any case.
fn reasoned(details: &[Value], reasons: &[&str]) -> bool {
    let is = |r: &str| reasons.iter().any(|c| c.eq_ignore_ascii_case(r));
    let mut given = details.iter().filter_map(|d| d.get(REASON)?.as_str());
    given.any(is)
}

/// A 429: the used allowance with no date when the free tier's daily quota ran out, as its delay is no renewal; else
/// a wait for its `Retry-After`, or for its `RetryInfo` delay.
fn busy<T>(head: &Head, details: &[Value]) -> Verdict<T> {
    if details.iter().any(daily) {
        return Verdict::Fault(NetFault::Allowance { renews_s: None });
    }
    let delay = details
        .iter()
        .filter_map(|d| d.get(RETRY_DELAY)?.as_str())
        .find_map(reply::span_s);
    let asked = head.header(RETRY).and_then(reply::seconds);
    Verdict::Busy(asked.or_else(|| delay.and_then(|s| u32::try_from(s).ok())))
}

/// True when `detail` names the free tier's daily quota among the quotas run out.
fn daily(detail: &Value) -> bool {
    let quotas = detail.get(VIOLATIONS).and_then(Value::as_array);
    let mut ids = quotas
        .into_iter()
        .flatten()
        .filter_map(|v| v.get(QUOTA_ID)?.as_str());
    ids.any(|id| id.contains(PER_DAY) && id.contains(FREE_TIER))
}

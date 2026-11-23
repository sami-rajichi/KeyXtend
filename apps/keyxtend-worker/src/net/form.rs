//! The `OpenAI`-style upload form (ADR-0047 decision 5), moved from the spike's `cloud.rs`: the model, the language,
//! the row's fixed fields, `response_format=json` and the piece as a WAV file, with the key in its header only. A reply
//! is `{"text": …}`; anything else is read through one status table from Groq's error page, where a row whose form
//! names a 403 its paywall reads it as the account's step, and one that names a region code reads that refusal by it.
//! A key test whose reply flags a management key at the row's paths fails, as that key never transcribes.

#[cfg(test)]
pub use super::multipart::escape;

use super::flag;
use super::multipart::{self, FILE, Text};
use super::reason::Reason;
use super::reply::{self, ERROR, SAYING};
use super::shape::{self, Piece, Request, Shape, Verdict};
use super::status::{self, LANGUAGE, RETRY, Row, Then, row};
use kx_ipc::{NET_PACE_S, NetCall, NetFault, NetField, NetForm, NetShape};
use kx_platform_net::{Head, Verb};
use std::borrow::Cow;
use std::cell::LazyCell;
use zeroize::Zeroizing;

/// The upload form.
pub struct Form;

/// The form's own field names, which a row's fixed field may not repeat, with the file's.
const MODEL: &str = "model";
const FORMAT: &str = "response_format";
/// The reply asked for: JSON with the text and no word or character timings, so replies stay small.
const JSON: &str = "json";
/// What follows a list field's name.
const LIST: &str = "[]";
/// The answer headers a 429 is read by (Groq's rate-limit page): the wait, the requests left and their reset.
const LEFT: &str = "x-ratelimit-remaining-requests";
const RESET: &str = "x-ratelimit-reset-requests";
const KEEP: &[&str] = &[RETRY, LEFT, RESET];
/// `LEFT` once the requests are used until their reset.
const NONE_LEFT: &str = "0";
/// The exact error codes or types that mean no credit (`OpenAI`'s error page): a rate limit's message may name billing,
/// so words never decide it.
const CREDIT: &[&str] = &[
    "insufficient_quota",
    "credit_balance_exhausted",
    "organization_usage_limit_exceeded",
];
/// The exact codes or types of a spending cap the user set, which they raise on the service's site.
const CAP: &[&str] = &[
    "organization_spend_limit_exceeded",
    "project_spend_limit_exceeded",
];

/// What a row asks of the error beside its status.
#[derive(Copy, Clone)]
enum Says {
    /// Nothing.
    Any,
    /// Nothing, but the call's form must name this status its paywall.
    Paywall,
    /// Its code or type is exactly the region code the call's form names.
    Region,
    /// Its code or type is exactly one of these.
    Code(&'static [&'static str]),
    /// Its words hold one of these.
    Words(&'static [&'static str]),
}

/// The status table, from Groq's error page and the rows' own: the first row that matches wins, and a status no row
/// takes is strange.
const TABLE: &[Row<Says>] = &[
    // No redirect is followed, so the key and the sound reach one address only.
    row(300..=399, Says::Any, Then::Fault(NetFault::Strange)),
    row(401..=401, Says::Any, Then::Fault(NetFault::Key)),
    // OpenAI's country refusal is a 403 with its own code, which no key fixes.
    row(400..=499, Says::Region, Then::Fault(NetFault::Region)),
    // Together's missing payment method and OpenRouter's spending cap are a 403: a step on the account's site.
    row(403..=403, Says::Paywall, Then::Fault(NetFault::Account)),
    row(403..=403, Says::Any, Then::Fault(NetFault::Key)),
    row(402..=402, Says::Any, Then::Fault(NetFault::Credit)),
    row(400..=499, Says::Code(CAP), Then::Fault(NetFault::Account)),
    row(400..=499, Says::Code(CREDIT), Then::Fault(NetFault::Credit)),
    row(413..=413, Says::Any, Then::Fault(NetFault::TooBig)),
    row(
        400..=400,
        Says::Words(LANGUAGE),
        Then::Fault(NetFault::Language),
    ),
    row(
        422..=422,
        Says::Words(LANGUAGE),
        Then::Fault(NetFault::Language),
    ),
    row(429..=429, Says::Any, Then::Busy),
    // Groq's flex tier is full, or the service is down.
    row(498..=498, Says::Any, Then::Fault(NetFault::Network)),
    row(500..=599, Says::Any, Then::Fault(NetFault::Network)),
];

impl Shape for Form {
    fn build(&self, piece: &Piece<'_>) -> Result<Request, NetFault> {
        let NetShape::Form(form) = &piece.call.shape else {
            return Err(NetFault::Range);
        };
        let model = piece.model.map(|m| Text::of(MODEL, m));
        let texts: Vec<Text<'_>> = model
            .into_iter()
            .chain(fields(
                (&form.lang, form.list),
                &form.fixed,
                piece.lang,
                &[MODEL, FORMAT],
            )?)
            .chain([Text::of(FORMAT, JSON)])
            .collect();
        let sent = multipart::sent(&texts, &multipart::wav(piece.wav))?;
        Ok(Request {
            verb: Verb::Post,
            fields: vec![shape::key(piece.call)?, sent.kind],
            body: sent.body,
            keep: KEEP,
        })
    }

    fn test(&self, call: &NetCall<'_>) -> Result<Request, NetFault> {
        let NetShape::Form(_) = &call.shape else {
            return Err(NetFault::Range);
        };
        Ok(Request {
            verb: Verb::Get,
            fields: vec![shape::key(call)?],
            body: Zeroizing::default(),
            keep: KEEP,
        })
    }

    fn read(&self, call: &NetCall<'_>, head: &Head, body: &[u8]) -> Verdict<Zeroizing<String>> {
        status::text(head, body, reply::text, || judge(form(call), head, body))
    }

    fn tested(&self, call: &NetCall<'_>, head: &Head, body: &[u8]) -> Verdict<()> {
        let admin = form(call).map_or(&[][..], |f| f.admin.as_slice());
        match status::tested(head, body, || judge(form(call), head, body)) {
            Verdict::Said(()) if flag::any(body, admin) => Verdict::Fault(NetFault::Admin),
            other => other,
        }
    }
}

/// `call`'s form, whose paywall and region code the table reads the replies by, and whose paths flag a management key.
fn form<'a>(call: &'a NetCall<'_>) -> Option<&'a NetForm<'a>> {
    match &call.shape {
        NetShape::Form(form) => Some(form),
        _ => None,
    }
}

/// The language field `field` holding `lang`, a list's name followed by `LIST`, then the `fixed` fields; `Range` when
/// a fixed field repeats a name of `own`, the file's or the language's, in any case.
pub(super) fn fields<'a>(
    (field, list): (&'a str, bool),
    fixed: &'a [NetField<'_>],
    lang: &'a str,
    own: &[&str],
) -> Result<Vec<Text<'a>>, NetFault> {
    let name = if list {
        Cow::Owned(format!("{field}{LIST}"))
    } else {
        Cow::Borrowed(field)
    };
    let names = [FILE, field, name.as_ref()];
    let repeats = |label: &str| {
        own.iter()
            .chain(&names)
            .any(|o| o.eq_ignore_ascii_case(label))
    };
    if fixed.iter().any(|f| repeats(&f.name)) {
        return Err(NetFault::Range);
    }
    let fixed = fixed.iter().map(|f| Text::of(&f.name, &f.value));
    let lang = Text {
        name,
        value: lang,
        kind: None,
    };
    Ok([lang].into_iter().chain(fixed).collect())
}

/// What an answer that is not done means, by the first table row that takes it, under `form`'s paywall and region code.
fn judge<T>(form: Option<&NetForm<'_>>, head: &Head, body: &[u8]) -> Verdict<T> {
    let region = form.and_then(|f| f.region.as_deref());
    let reason = LazyCell::new(|| Reason::read(body, &ERROR, SAYING));
    let said = |says: Says| match says {
        Says::Any => true,
        Says::Paywall => form.is_some_and(|f| f.paywall),
        Says::Region => region.is_some_and(|code| reason.coded(&[code])),
        Says::Code(codes) => reason.coded(codes),
        Says::Words(words) => reason.says(words),
    };
    status::judge(TABLE, head, said, busy)
}

/// A 429: the wait the service asks for, or, when no request is left, a wait for a reset within `NET_PACE_S`, as a
/// minute's limit, else the used allowance with its renewal, as Groq's day.
fn busy<T>(head: &Head) -> Verdict<T> {
    if head.header(LEFT).map(str::trim) != Some(NONE_LEFT) {
        return status::wait(head);
    }
    let reset_s = head.header(RESET).and_then(reply::span_s);
    match reset_s {
        Some(s) if s <= u64::from(NET_PACE_S) => Verdict::Busy(u32::try_from(s).ok()),
        _ => {
            let renews_s = reset_s.and_then(status::renewal);
            Verdict::Fault(NetFault::Allowance { renews_s })
        }
    }
}

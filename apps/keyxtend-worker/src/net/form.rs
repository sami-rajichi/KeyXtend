//! The `OpenAI`-style upload form (ADR-0047 decision 5), moved from the spike's `cloud.rs`: the model, the language,
//! the row's fixed fields, `response_format=json` and the piece as a WAV file, with the key in its header only. A reply
//! is `{"text": …}`; anything else is read through one status table from Groq's error page, where a row whose form
//! names a 403 its paywall reads it as no credit, and one that names a region code reads that refusal by it.

mod body;

#[cfg(test)]
pub use body::escape;

use super::reply;
use super::shape::{self, Piece, Request, Shape, Verdict};
use body::{File, Text};
use kx_ipc::{NET_RENEW_S, NetCall, NetFault, NetForm, NetShape};
use kx_platform_net::{Head, Verb};
use std::borrow::Cow;
use std::ops::RangeInclusive;
use zeroize::Zeroizing;

/// The upload form.
pub struct Form;

/// The form's own field names, which a row's fixed field may not repeat.
const MODEL: &str = "model";
const FORMAT: &str = "response_format";
const FILE: &str = "file";
/// The reply asked for: JSON with the text and no word or character timings, so replies stay small.
const JSON: &str = "json";
/// What follows a list field's name.
const LIST: &str = "[]";
/// The file part's name and type.
const FILE_NAME: &str = "piece.wav";
const WAV_TYPE: &str = "audio/wav";
/// The body's header and its value before the boundary.
const CONTENT_TYPE: &str = "Content-Type";
const FORM_TYPE: &str = "multipart/form-data; boundary=";
/// The answer headers a 429 is read by (Groq's rate-limit page): the wait, the day's requests left and their renewal.
const RETRY: &str = "retry-after";
const LEFT: &str = "x-ratelimit-remaining-requests";
const RESET: &str = "x-ratelimit-reset-requests";
const KEEP: &[&str] = &[RETRY, LEFT, RESET];
/// `LEFT` once the day's requests are used.
const NONE_LEFT: &str = "0";
/// The exact error codes or types that mean no credit (`OpenAI`'s error page): a rate limit's message may name billing,
/// so words never decide it.
const CREDIT: &[&str] = &[
    "insufficient_quota",
    "credit_balance_exhausted",
    "organization_spend_limit_exceeded",
    "project_spend_limit_exceeded",
    "organization_usage_limit_exceeded",
];
/// Words of an error that means the language, read only on a 400 or a 422.
const LANGUAGE: &[&str] = &["language"];
/// Statuses whose reply holds what was asked.
const DONE: RangeInclusive<u16> = 200..=299;

/// What a status means.
#[derive(Copy, Clone)]
enum Then {
    /// This fault.
    Fault(NetFault),
    /// Too many requests: a used allowance, or a wait.
    Busy,
}

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

/// One row of the status table: the statuses it covers, what the error must say, and what it means.
struct Row {
    codes: RangeInclusive<u16>,
    says: Says,
    then: Then,
}

/// The status table, from Groq's error page and the rows' own: the first row that matches wins, and a status no row
/// takes is strange.
const TABLE: &[Row] = &[
    // No redirect is followed, so the key and the sound reach one address only.
    row(300..=399, Says::Any, Then::Fault(NetFault::Strange)),
    row(401..=401, Says::Any, Then::Fault(NetFault::Key)),
    // OpenAI's country refusal is a 403 with its own code, which no key fixes.
    row(400..=499, Says::Region, Then::Fault(NetFault::Region)),
    // Together's missing payment method and OpenRouter's spending cap are a 403, which no key fixes.
    row(403..=403, Says::Paywall, Then::Fault(NetFault::Credit)),
    row(403..=403, Says::Any, Then::Fault(NetFault::Key)),
    row(402..=402, Says::Any, Then::Fault(NetFault::Credit)),
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

/// A table row.
const fn row(codes: RangeInclusive<u16>, says: Says, then: Then) -> Row {
    Row { codes, says, then }
}

impl Shape for Form {
    fn build(&self, piece: &Piece<'_>) -> Result<Request, NetFault> {
        let NetShape::Form(form) = &piece.call.shape else {
            return Err(NetFault::Range);
        };
        let lang = if form.list {
            Cow::Owned(format!("{}{LIST}", form.lang))
        } else {
            Cow::Borrowed(form.lang.as_ref())
        };
        if repeats(form, &lang) {
            return Err(NetFault::Range);
        }
        let model = piece.model.map(|m| Text::of(MODEL, m));
        let fixed = form.fixed.iter().map(|f| Text::of(&f.name, &f.value));
        let texts: Vec<Text<'_>> = model
            .into_iter()
            .chain([Text {
                name: lang,
                value: piece.lang,
            }])
            .chain(fixed)
            .chain([Text::of(FORMAT, JSON)])
            .collect();
        let file = File {
            name: FILE,
            called: FILE_NAME,
            kind: WAV_TYPE,
            bytes: piece.wav,
        };
        let boundary = body::boundary(&texts, &file);
        let kind = Zeroizing::new(format!("{FORM_TYPE}{boundary}").into_bytes());
        Ok(Request {
            verb: Verb::Post,
            fields: vec![shape::key(piece.call)?, (CONTENT_TYPE.to_owned(), kind)],
            body: body::body(&boundary, &texts, &file)?,
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
        if DONE.contains(&head.status) {
            return reply::text(body).map_or(Verdict::Fault(NetFault::Strange), Verdict::Said);
        }
        judge(form(call), head, body)
    }

    fn tested(&self, call: &NetCall<'_>, head: &Head, body: &[u8]) -> Verdict<()> {
        match (DONE.contains(&head.status), reply::object(body)) {
            (true, true) => Verdict::Said(()),
            (true, false) => Verdict::Fault(NetFault::Strange),
            (false, _) => judge(form(call), head, body),
        }
    }
}

/// `call`'s form, whose paywall and region code the table reads the replies by.
fn form<'a>(call: &'a NetCall<'_>) -> Option<&'a NetForm<'a>> {
    match &call.shape {
        NetShape::Form(form) => Some(form),
        _ => None,
    }
}

/// True when a fixed field repeats one of the form's own names, `lang` as sent included, in any case.
fn repeats(form: &NetForm<'_>, lang: &str) -> bool {
    let own = [MODEL, FORMAT, FILE, form.lang.as_ref(), lang];
    let taken = |name: &str| own.iter().any(|o| o.eq_ignore_ascii_case(name));
    form.fixed.iter().any(|f| taken(&f.name))
}

/// What an answer that is not done means, by the first table row that takes it, under `form`'s paywall and region code.
fn judge<T>(form: Option<&NetForm<'_>>, head: &Head, body: &[u8]) -> Verdict<T> {
    let region = form.and_then(|f| f.region.as_deref());
    let said = |says: Says| match says {
        Says::Any => true,
        Says::Paywall => form.is_some_and(|f| f.paywall),
        Says::Region => region.is_some_and(|code| reply::coded(body, &[code])),
        Says::Code(codes) => reply::coded(body, codes),
        Says::Words(words) => reply::says(body, words),
    };
    let takes = |r: &&Row| r.codes.contains(&head.status) && said(r.says);
    match TABLE.iter().find(takes).map(|r| r.then) {
        Some(Then::Fault(fault)) => Verdict::Fault(fault),
        Some(Then::Busy) => busy(head),
        None => Verdict::Fault(NetFault::Strange),
    }
}

/// A 429: the used allowance with its renewal when no request is left today, else the wait the service asks for.
fn busy<T>(head: &Head) -> Verdict<T> {
    if head.header(LEFT).map(str::trim) == Some(NONE_LEFT) {
        let renews_s = head.header(RESET).and_then(reply::span_s).and_then(renewal);
        return Verdict::Fault(NetFault::Allowance { renews_s });
    }
    Verdict::Busy(head.header(RETRY).and_then(reply::seconds))
}

/// `s` held up to `NET_RENEW_S`'s start; none past its end, as a wrong date is worse than none.
fn renewal(s: u64) -> Option<u32> {
    let s = u32::try_from(s).ok()?.max(*NET_RENEW_S.start());
    NET_RENEW_S.contains(&s).then_some(s)
}

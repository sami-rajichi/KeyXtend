//! The `OpenAI`-style upload form (ADR-0047 decision 5), moved from the spike's `cloud.rs`: the model, the language,
//! the row's fixed fields, `response_format=json` and the piece as a WAV file, with the key in its header only. A reply
//! is `{"text": …}`; anything else is read through one status table from Groq's error page.

mod body;

#[cfg(test)]
pub use body::escape;

use super::reply;
use super::shape::{self, Piece, Request, Shape, Verdict};
use body::{File, Text};
use kx_ipc::{NET_RENEW_S, NetCall, NetFault, NetShape};
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
/// Words of an error that means no credit, and of one that means the language.
const CREDIT: &[&str] = &[
    "credit",
    "insufficient_quota",
    "billing",
    "balance",
    "payment",
];
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

/// One row of the status table: the statuses it covers, words the error must hold (none: any), and what it means.
struct Row {
    codes: RangeInclusive<u16>,
    says: &'static [&'static str],
    then: Then,
}

/// The status table, from Groq's error page: the first row that matches wins, and a status no row takes is strange.
const TABLE: &[Row] = &[
    // No redirect is followed, so the key and the sound reach one address only.
    row(300..=399, &[], Then::Fault(NetFault::Strange)),
    row(401..=401, &[], Then::Fault(NetFault::Key)),
    row(403..=403, &[], Then::Fault(NetFault::Key)),
    row(402..=402, &[], Then::Fault(NetFault::Credit)),
    row(400..=499, CREDIT, Then::Fault(NetFault::Credit)),
    row(413..=413, &[], Then::Fault(NetFault::TooBig)),
    row(400..=400, LANGUAGE, Then::Fault(NetFault::Language)),
    row(422..=422, LANGUAGE, Then::Fault(NetFault::Language)),
    row(429..=429, &[], Then::Busy),
    // Groq's flex tier is full, or the service is down.
    row(498..=498, &[], Then::Fault(NetFault::Network)),
    row(500..=599, &[], Then::Fault(NetFault::Network)),
];

/// A table row.
const fn row(codes: RangeInclusive<u16>, says: &'static [&'static str], then: Then) -> Row {
    Row { codes, says, then }
}

impl Shape for Form {
    fn build(&self, piece: &Piece<'_>) -> Result<Request, NetFault> {
        let NetShape::Form(form) = &piece.call.shape else {
            return Err(NetFault::Range);
        };
        let own = [MODEL, FORMAT, FILE, form.lang.as_ref()];
        let repeats = |name: &str| own.iter().any(|o| o.eq_ignore_ascii_case(name));
        if form.fixed.iter().any(|f| repeats(&f.name)) {
            return Err(NetFault::Range);
        }
        let lang = if form.list {
            Cow::Owned(format!("{}{LIST}", form.lang))
        } else {
            Cow::Borrowed(form.lang.as_ref())
        };
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

    fn read(&self, head: &Head, body: &[u8]) -> Verdict<Zeroizing<String>> {
        if DONE.contains(&head.status) {
            return reply::text(body).map_or(Verdict::Fault(NetFault::Strange), Verdict::Said);
        }
        judge(head, body)
    }

    fn tested(&self, head: &Head, body: &[u8]) -> Verdict<()> {
        match (DONE.contains(&head.status), reply::object(body)) {
            (true, true) => Verdict::Said(()),
            (true, false) => Verdict::Fault(NetFault::Strange),
            (false, _) => judge(head, body),
        }
    }
}

/// What an answer that is not done means, by the first table row that takes it.
fn judge<T>(head: &Head, body: &[u8]) -> Verdict<T> {
    let takes = |r: &&Row| {
        r.codes.contains(&head.status) && (r.says.is_empty() || reply::says(body, r.says))
    };
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

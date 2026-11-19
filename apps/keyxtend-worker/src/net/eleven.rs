//! `ElevenLabs`' own form (ADR-0047 decision 5, research [E3]): `model_id`, then the row's language field and fixed
//! fields, such as no sound tags, then the piece as `file`, with the key in the header the call names. A reply is
//! `{"text": …}` beside its word timings; anything else is read through its status table [E8], where a used quota or
//! no credit is told only by the exact code its `detail` names, so a plain 401 stays the key.

use super::form;
use super::multipart::{self, Text};
use super::reply::{self, Coded};
use super::shape::{self, Piece, Request, Shape, Verdict};
use super::status::{self, RETRY, Row, Then, row};
use kx_ipc::{NetCall, NetFault, NetShape};
use kx_platform_net::{Head, Verb};
use zeroize::Zeroizing;

/// `ElevenLabs`' form.
pub struct Eleven;

/// The form's own model field beside the row's language field and the file's [E3].
const MODEL: &str = "model_id";
/// The answer header a 429 is read by.
const KEEP: &[&str] = &[RETRY];
/// Where an error names its code: `detail.status`, or `detail.code` in the newer shape [E8].
const DETAIL: Coded = Coded {
    object: "detail",
    names: &["status", "code"],
};
/// The exact codes of a used quota or no credit [E8], on a 401 as on a 402.
const CREDIT: &[&str] = &["quota_exceeded", "payment_required", "insufficient_credits"];
/// Words of an error that means the language, read only on a 400 or a 422.
const LANGUAGE: &[&str] = &["language"];

/// What a row asks of the error beside its status.
#[derive(Copy, Clone)]
enum Says {
    /// Nothing.
    Any,
    /// Its `detail` code is exactly one of these.
    Code(&'static [&'static str]),
    /// Its words hold one of these.
    Words(&'static [&'static str]),
}

/// The status table from its error page [E8]: the first row that matches wins, and a status no row takes, such as a
/// 403 its pages never name, is strange.
const TABLE: &[Row<Says>] = &[
    // No redirect is followed, so the key and the sound reach one address only.
    row(300..=399, Says::Any, Then::Fault(NetFault::Strange)),
    row(400..=499, Says::Code(CREDIT), Then::Fault(NetFault::Credit)),
    row(401..=401, Says::Any, Then::Fault(NetFault::Key)),
    row(402..=402, Says::Any, Then::Fault(NetFault::Credit)),
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
    // Too many requests at once for the plan, or the system busy.
    row(429..=429, Says::Any, Then::Busy),
    row(500..=599, Says::Any, Then::Fault(NetFault::Network)),
];

impl Shape for Eleven {
    fn build(&self, piece: &Piece<'_>) -> Result<Request, NetFault> {
        let (NetShape::Eleven(own), Some(model)) = (&piece.call.shape, piece.model) else {
            return Err(NetFault::Range);
        };
        let rest = form::fields((&own.lang, false), &own.fixed, piece.lang, &[MODEL])?;
        let texts: Vec<Text<'_>> = [Text::of(MODEL, model)].into_iter().chain(rest).collect();
        let sent = multipart::sent(&texts, &multipart::wav(piece.wav))?;
        Ok(Request {
            verb: Verb::Post,
            fields: vec![shape::key(piece.call)?, sent.kind],
            body: sent.body,
            keep: KEEP,
        })
    }

    fn test(&self, call: &NetCall<'_>) -> Result<Request, NetFault> {
        let NetShape::Eleven(_) = &call.shape else {
            return Err(NetFault::Range);
        };
        Ok(Request {
            verb: Verb::Get,
            fields: vec![shape::key(call)?],
            body: Zeroizing::default(),
            keep: KEEP,
        })
    }

    fn read(&self, _: &NetCall<'_>, head: &Head, body: &[u8]) -> Verdict<Zeroizing<String>> {
        status::text(head, body, reply::text, || judge(head, body))
    }

    fn tested(&self, _: &NetCall<'_>, head: &Head, body: &[u8]) -> Verdict<()> {
        status::tested(head, body, || judge(head, body))
    }
}

/// What an answer that is not done means, by the first table row that takes it.
fn judge<T>(head: &Head, body: &[u8]) -> Verdict<T> {
    let said = |says: Says| match says {
        Says::Any => true,
        Says::Code(codes) => reply::coded(body, &DETAIL, codes),
        Says::Words(words) => reply::says(body, words),
    };
    status::judge(TABLE, head, said, status::wait)
}

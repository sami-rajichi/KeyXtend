//! `AssemblyAI`'s Sync request (ADR-0051, research [A1][A2]): one multipart POST with the key in `Authorization`, the
//! model in the row's own header, a JSON `config` part first holding only the keyboard's one language code, then the
//! piece as the WAV `audio` part. A reply is `{"text": …}`; a problem (RFC 9457) is read by its status and its words.

use super::multipart::{self, Text};
use super::reply;
use super::shape::{self, Piece, Request, Shape, Verdict};
use super::status::{self, LANGUAGE, RETRY, Row, Then, row};
use kx_ipc::{NetCall, NetFault, NetShape, rules};
use kx_platform_net::{Head, Verb};
use serde::Serialize;
use zeroize::Zeroizing;

/// `AssemblyAI`'s Sync request.
pub struct Assembly;

/// The settings part, which must come before the sound [A1].
const CONFIG: &str = "config";
/// The settings part's media type [A1].
const JSON_TYPE: &str = "application/json";
/// The sound part [A1].
const AUDIO: &str = "audio";
/// The answer header a 429's or a 503's wait is read by [A2].
const KEEP: &[&str] = &[RETRY];
/// Where a problem reply says what went wrong [A1].
const PROBLEM: &[&str] = &["title", "detail"];
/// Words of a 402 that asks for a payment method: a step on the account's site, not no credit.
const PAYMENT: &[&str] = &["payment method"];

/// The settings [A1]: the keyboard's one language code and nothing else, so no timestamps, prompt or key terms.
#[derive(Serialize)]
struct Config<'a> {
    language_codes: [&'a str; 1],
}

/// What a row asks of the answer beside its status.
#[derive(Copy, Clone)]
enum Says {
    /// Nothing.
    Any,
    /// Its `Retry-After` names whole seconds.
    Waits,
    /// Its problem's title or detail holds one of these.
    Words(&'static [&'static str]),
}

/// The status table from its pages [A1][A2]: the first row that matches wins. No redirect is followed, and a 3xx, a
/// 415 and any status no row takes are strange, so a plain 400 never reads as the key.
const TABLE: &[Row<Says>] = &[
    row(401..=401, Says::Any, Then::Fault(NetFault::Key)),
    // Its reply when the credit runs out is not documented, so a 402 is assumed until the live check.
    row(
        402..=402,
        Says::Words(PAYMENT),
        Then::Fault(NetFault::Account),
    ),
    row(402..=402, Says::Any, Then::Fault(NetFault::Credit)),
    row(413..=413, Says::Any, Then::Fault(NetFault::TooBig)),
    row(
        400..=400,
        Says::Words(LANGUAGE),
        Then::Fault(NetFault::Language),
    ),
    row(429..=429, Says::Any, Then::Busy),
    // A cold start or the concurrency cap waits when it names a time; any other 5xx is the service down, tried again.
    row(503..=503, Says::Waits, Then::Busy),
    row(500..=599, Says::Any, Then::Fault(NetFault::Network)),
];

impl Shape for Assembly {
    fn build(&self, piece: &Piece<'_>) -> Result<Request, NetFault> {
        let (NetShape::Assembly { model_header }, Some(model)) = (&piece.call.shape, piece.model)
        else {
            return Err(NetFault::Range);
        };
        let key = shape::key(piece.call)?;
        // The model's header names none the request sets, its value is visible ASCII; the code is JSON, never empty.
        let plain = model.bytes().all(|b| b.is_ascii_graphic());
        let own = rules::unset_header(model_header, &key.0).is_ok();
        if piece.lang.is_empty() || !plain || !own {
            return Err(NetFault::Range);
        }
        let config = Config {
            language_codes: [piece.lang],
        };
        let config = serde_json::to_string(&config).map_err(|_| NetFault::Range)?;
        let texts = [Text::typed(CONFIG, &config, JSON_TYPE)];
        let sent = multipart::sent(&texts, &multipart::wav_named(AUDIO, piece.wav))?;
        let named = Zeroizing::new(model.as_bytes().to_vec());
        Ok(Request {
            verb: Verb::Post,
            fields: vec![key, (model_header.to_string(), named), sent.kind],
            body: sent.body,
            keep: KEEP,
        })
    }

    /// No request: the Sync host documents no call without sound, so the first dictation tests the key [A1].
    fn test(&self, _: &NetCall<'_>) -> Result<Request, NetFault> {
        Err(NetFault::Range)
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
        Says::Waits => head.header(RETRY).and_then(reply::seconds).is_some(),
        Says::Words(words) => reply::says_in(body, PROBLEM, words),
    };
    status::judge(TABLE, head, said, status::wait)
}

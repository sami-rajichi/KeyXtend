//! Gemini's single request (ADR-0047 decision 5, research [N1][N6]): one JSON POST with the key in the call's header, the
//! model, the piece inline as Base64 and the keyboard's one language code, never an empty list, which would detect it.
//! A reply's transcript is read by `text`, and an error by the exact reason or status Google names [N7], else by status.

mod text;

use super::base64::Base64;
use super::multipart::CONTENT_TYPE;
use super::reply::{self, Coded};
use super::shape::{self, Piece, Request, Shape, Verdict};
use super::status::{self, RETRY, Row, Then, row};
use super::wav;
use kx_ipc::{NetCall, NetFault, NetShape};
use kx_platform_net::{Head, Verb};
use serde::{Serialize, Serializer};
use std::io;
use zeroize::Zeroizing;

/// Gemini's request.
pub struct Gemini;

/// The body's media type.
const JSON_TYPE: &[u8] = b"application/json";
/// The input item's kind for a piece [N6].
const AUDIO: &str = "audio";
/// The answer header a 429 is read by.
const KEEP: &[&str] = &[RETRY];
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
/// Words of a location the service refuses [N7], and of a language code it refuses.
const LOCATION: &[&str] = &["location is not supported"];
const LANGUAGE: &[&str] = &["language"];

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

/// The request's JSON [N1]: the model, the piece as its one input, and the keyboard's language.
#[derive(Serialize)]
struct Body<'a> {
    model: &'a str,
    input: [Input<'a>; 1],
    generation_config: Config<'a>,
}

/// The piece as an input item [N6].
#[derive(Serialize)]
struct Input<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    data: Inline<'a>,
    mime_type: &'static str,
}

/// The piece's bytes, written as Base64 straight into the body.
struct Inline<'a>(&'a [u8]);

impl Serialize for Inline<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(&Base64(self.0))
    }
}

/// The settings a request gives: its transcription's.
#[derive(Serialize)]
struct Config<'a> {
    transcription_config: Transcription<'a>,
}

/// The transcription's settings: one language code.
#[derive(Serialize)]
struct Transcription<'a> {
    language_codes: [&'a str; 1],
}

impl Shape for Gemini {
    fn build(&self, piece: &Piece<'_>) -> Result<Request, NetFault> {
        let (NetShape::Gemini, Some(model), false) =
            (&piece.call.shape, piece.model, piece.lang.is_empty())
        else {
            return Err(NetFault::Range);
        };
        let input = Input {
            kind: AUDIO,
            data: Inline(piece.wav),
            mime_type: wav::TYPE,
        };
        let language_codes = [piece.lang];
        let body = Body {
            model,
            input: [input],
            generation_config: Config {
                transcription_config: Transcription { language_codes },
            },
        };
        let kind = (CONTENT_TYPE.to_owned(), Zeroizing::new(JSON_TYPE.to_vec()));
        Ok(Request {
            verb: Verb::Post,
            fields: vec![shape::key(piece.call)?, kind],
            body: json(&body)?,
            keep: KEEP,
        })
    }

    fn test(&self, call: &NetCall<'_>) -> Result<Request, NetFault> {
        let NetShape::Gemini = &call.shape else {
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
        status::text(head, body, text::said, || judge(head, body))
    }

    fn tested(&self, _: &NetCall<'_>, head: &Head, body: &[u8]) -> Verdict<()> {
        status::tested(head, body, || judge(head, body))
    }
}

/// A writer that only counts, so the body's room is taken once.
#[derive(Default)]
struct Count(usize);

impl io::Write for Count {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0 += buf.len();
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// `body` written into one wiped buffer sized once, so no growth leaves an unwiped copy of the sound.
///
/// # Errors
/// `Gone` when the room cannot be had or the writer fails.
fn json(body: &Body<'_>) -> Result<Zeroizing<Vec<u8>>, NetFault> {
    let mut count = Count::default();
    serde_json::to_writer(&mut count, body).map_err(|_| NetFault::Gone)?;
    let mut out = Zeroizing::new(Vec::new());
    out.try_reserve_exact(count.0).map_err(|_| NetFault::Gone)?;
    serde_json::to_writer(&mut *out, body).map_err(|_| NetFault::Gone)?;
    Ok(out)
}

/// What an answer that is not done means, by the first table row that takes it.
fn judge<T>(head: &Head, body: &[u8]) -> Verdict<T> {
    let said = |says: Says| match says {
        Says::Any => true,
        Says::Reason(reasons) => reasoned(body, reasons),
        Says::Status(names) => reply::coded(body, &STATUS, names),
        Says::Words(words) => reply::says(body, words),
    };
    status::judge(TABLE, head, said, status::wait)
}

/// True when a detail of a Google error gives one of `reasons` exactly, in any case; never for another body.
fn reasoned(body: &[u8], reasons: &[&str]) -> bool {
    let reply = serde_json::from_slice::<serde_json::Value>(body).unwrap_or_default();
    let details = reply.get(ERROR).and_then(|e| e.get(DETAILS));
    let is = |r: &str| reasons.iter().any(|c| c.eq_ignore_ascii_case(r));
    let given = details
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten();
    given
        .filter_map(|d| d.get(REASON).and_then(serde_json::Value::as_str))
        .any(is)
}

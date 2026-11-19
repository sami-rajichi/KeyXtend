//! Gemini's single request (ADR-0047 decision 5, research [N1][N6][N10]): one JSON POST with the key in the call's
//! header, the model, the piece inline as Base64, the keyboard's one language code and `store` off. A reply's
//! transcript is read by `text`, and an error answer by `error`.

mod error;
mod lax;
mod text;

use super::base64::Base64;
use super::shape::{self, CONTENT_TYPE, Piece, Request, Shape, Verdict};
use super::status::{self, RETRY};
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
/// Google keeps each interaction a day free, 55 days paid, unless told not to; we never chain turns [N10].
const STORE: bool = false;

/// The request's JSON [N1]: the model, the piece as its one input, the keyboard's language, and nothing kept [N10].
#[derive(Serialize)]
struct Body<'a> {
    model: &'a str,
    input: [Input<'a>; 1],
    generation_config: Config<'a>,
    store: bool,
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

/// The transcription's settings: one language code, never an empty list, which would detect the language.
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
            store: STORE,
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
        status::text(head, body, text::said, || error::judge(head, body))
    }

    fn tested(&self, _: &NetCall<'_>, head: &Head, body: &[u8]) -> Verdict<()> {
        status::tested(head, body, || error::judge(head, body))
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

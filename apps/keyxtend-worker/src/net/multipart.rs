//! A form's body (`multipart/form-data`, RFC 7578), moved from the spike's `multipart.rs`, which the upload form,
//! `ElevenLabs`' and `AssemblyAI`'s share: text fields, then the piece as a WAV file, under a boundary drawn from the
//! parts' own SHA-256, so no part can hold it, in one wiped buffer sized once.

use super::shape::CONTENT_TYPE;
use super::wav;
use kx_ipc::NetFault;
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::fmt::Write as _;
use zeroize::Zeroizing;

/// The file field's name, which both forms give the piece.
pub const FILE: &str = "file";
/// What the piece is called.
const PIECE_NAME: &str = "piece.wav";
/// The request's body type before its boundary.
const FORM_TYPE: &str = "multipart/form-data; boundary=";
/// What starts every boundary.
const MARK: &str = "kx-";
/// Bytes of the parts' fingerprint a boundary holds, written as hex.
const PRINT_BYTES: usize = 16;
/// Ends each line of a part's head, and each part.
const CRLF: &str = "\r\n";
/// What starts a boundary line, and ends the last.
const DASHES: &str = "--";
/// A part's head up to its name, and the quote that ends a name.
const DISPOSITION: &str = "Content-Disposition: form-data; name=\"";
const QUOTE: &str = "\"";
/// A file part's name before its value, and what parts a header's name from its value.
const FILE_NAME: &str = "; filename=\"";
const TYPE_SEP: &str = ": ";
/// Bytes a value may not hold: a line break or a NUL would end its part early.
const BREAKS: &[u8] = b"\r\n\0";
/// What a name escapes, as the HTML standard's form encoding does: a quote and line breaks.
const ESCAPES: [(char, &str); 3] = [('"', "%22"), ('\r', "%0D"), ('\n', "%0A")];

/// A text field.
pub struct Text<'a> {
    /// Its name, escaped when written.
    pub name: Cow<'a, str>,
    /// Its value, sent as it is.
    pub value: &'a str,
    /// Its media type, when its part names one, as a JSON part does.
    pub kind: Option<&'a str>,
}

impl<'a> Text<'a> {
    /// A field named `name`, holding `value`.
    pub fn of(name: &'a str, value: &'a str) -> Self {
        Self {
            name: Cow::Borrowed(name),
            value,
            kind: None,
        }
    }

    /// A field named `name`, holding `value` of media type `kind`.
    pub fn typed(name: &'a str, value: &'a str, kind: &'a str) -> Self {
        Self {
            kind: Some(kind),
            ..Self::of(name, value)
        }
    }
}

/// The file field.
pub struct File<'a> {
    /// Its name.
    pub name: &'a str,
    /// What the file is called.
    pub called: &'a str,
    /// Its media type.
    pub kind: &'a str,
    /// Its bytes.
    pub bytes: &'a [u8],
}

/// A form ready to send: the header that names its boundary, and its body.
pub struct Sent {
    /// The `Content-Type` header and its value.
    pub kind: (String, Zeroizing<Vec<u8>>),
    /// The body, wiped when dropped, as it holds the sound.
    pub body: Zeroizing<Vec<u8>>,
}

/// The piece `wav` as the form's WAV file.
pub const fn wav(wav: &[u8]) -> File<'_> {
    wav_named(FILE, wav)
}

/// The piece `wav` as a WAV file in the field `name`.
pub const fn wav_named<'a>(name: &'a str, wav: &'a [u8]) -> File<'a> {
    File {
        name,
        called: PIECE_NAME,
        kind: wav::TYPE,
        bytes: wav,
    }
}

/// The form of `texts`, then `file`, with the header that names its boundary.
///
/// # Errors
/// As `body`.
pub fn sent(texts: &[Text<'_>], file: &File<'_>) -> Result<Sent, NetFault> {
    let boundary = boundary(texts, file);
    let kind = Zeroizing::new(format!("{FORM_TYPE}{boundary}").into_bytes());
    Ok(Sent {
        kind: (CONTENT_TYPE.to_owned(), kind),
        body: body(&boundary, texts, file)?,
    })
}

/// `name` as a form writes it: `ESCAPES` replaced.
pub fn escape(name: &str) -> Cow<'_, str> {
    if !name.contains(|c| ESCAPES.iter().any(|(e, _)| *e == c)) {
        return Cow::Borrowed(name);
    }
    let mut out = String::with_capacity(name.len());
    for c in name.chars() {
        match ESCAPES.iter().find(|(e, _)| *e == c) {
            Some((_, with)) => out.push_str(with),
            None => out.push(c),
        }
    }
    Cow::Owned(out)
}

/// The boundary for these parts: `MARK` and the hex of their SHA-256, which no part can hold.
fn boundary(texts: &[Text<'_>], file: &File<'_>) -> String {
    let mut print = Sha256::new();
    for t in texts {
        print.update(t.name.as_bytes());
        print.update(t.value.as_bytes());
        print.update(t.kind.unwrap_or_default().as_bytes());
    }
    print.update(file.bytes);
    let mut out = String::from(MARK);
    for b in &print.finalize()[..PRINT_BYTES] {
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// The body of these parts under `boundary`.
///
/// # Errors
/// `Range` for a value or a media type holding a `BREAKS` byte, `Gone` when the room cannot be had.
fn body(
    boundary: &str,
    texts: &[Text<'_>],
    file: &File<'_>,
) -> Result<Zeroizing<Vec<u8>>, NetFault> {
    let breaks = |s: &str| s.bytes().any(|b| BREAKS.contains(&b));
    if texts
        .iter()
        .any(|t| breaks(t.value) || t.kind.is_some_and(breaks))
    {
        return Err(NetFault::Range);
    }
    let (heads, file_head) = heads_of(boundary, texts, file);
    let last = format!("{DASHES}{boundary}{DASHES}{CRLF}");
    let texts_len: usize = texts
        .iter()
        .zip(&heads)
        .map(|(t, h)| h.len() + t.value.len() + CRLF.len())
        .sum();
    let size = texts_len + file_head.len() + file.bytes.len() + CRLF.len() + last.len();
    let mut out = Zeroizing::new(Vec::new());
    out.try_reserve_exact(size).map_err(|_| NetFault::Gone)?;
    for (t, h) in texts.iter().zip(&heads) {
        for part in [h.as_bytes(), t.value.as_bytes(), CRLF.as_bytes()] {
            out.extend_from_slice(part);
        }
    }
    for part in [
        file_head.as_bytes(),
        file.bytes,
        CRLF.as_bytes(),
        last.as_bytes(),
    ] {
        out.extend_from_slice(part);
    }
    Ok(out)
}

/// Each part's head under `boundary`: the texts' in order, each with its media type when it names one, then the file's.
fn heads_of(boundary: &str, texts: &[Text<'_>], file: &File<'_>) -> (Vec<String>, String) {
    let line = format!("{DASHES}{boundary}{CRLF}");
    let head = |name: &str| format!("{line}{DISPOSITION}{}{QUOTE}", escape(name));
    let typed = |kind: &str| format!("{CRLF}{CONTENT_TYPE}{TYPE_SEP}{kind}");
    let heads = texts
        .iter()
        .map(|t| {
            let kind = t.kind.map(typed).unwrap_or_default();
            format!("{}{kind}{CRLF}{CRLF}", head(&t.name))
        })
        .collect();
    let file_name = escape(file.called);
    let file_head = format!(
        "{}{FILE_NAME}{file_name}{QUOTE}{}{CRLF}{CRLF}",
        head(file.name),
        typed(file.kind)
    );
    (heads, file_head)
}

//! Which formats a copy is read from and the bytes a write sets, pure over the format ids in `Formats`.

use std::borrow::Cow;

use kx_module_api::ports::{ClipPart, ImageCodec};

use super::hdrop;

/// The clipboard formats a read and a write use, and the drop effect that means "copy".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Formats {
    /// `CF_UNICODETEXT`.
    pub(crate) text: u32,
    /// The registered HTML format.
    pub(crate) html: u32,
    /// The registered RTF format.
    pub(crate) rtf: u32,
    /// The registered PNG format.
    pub(crate) png: u32,
    /// `CF_DIBV5`.
    pub(crate) dib_v5: u32,
    /// `CF_DIB`.
    pub(crate) dib: u32,
    /// `CF_HDROP`.
    pub(crate) files: u32,
    /// The registered drop-effect format.
    pub(crate) effect: u32,
    /// `DROPEFFECT_COPY`.
    pub(crate) copy: u32,
}

/// What a part is read as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    /// Plain text.
    Text,
    /// HTML.
    Html,
    /// RTF.
    Rtf,
    /// An image in this codec.
    Image(ImageCodec),
    /// A file list.
    Files,
}

/// A part to read: what it is and the format that holds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Want {
    /// What it is read as.
    pub(crate) kind: Kind,
    /// The format to read it from.
    pub(crate) format: u32,
}

impl Formats {
    /// The parts to read from a clipboard holding `held`, in its order (the richest first), each kind once. The one
    /// image is PNG when there is one, else DIBV5, else DIB, in the place of the first image format; an insert of a PNG
    /// adds the worker's DIB.
    pub(crate) fn pick(&self, held: &[u32]) -> Vec<Want> {
        let best = [self.png, self.dib_v5, self.dib]
            .into_iter()
            .find(|f| held.contains(f));
        let mut out: Vec<Want> = Vec::new();
        for &format in held {
            let format = if self.is_image(format) {
                best.unwrap_or(format)
            } else {
                format
            };
            let Some(kind) = self.kind(format) else {
                continue;
            };
            if !out.iter().any(|w| w.kind == kind) {
                out.push(Want { kind, format });
            }
        }
        out
    }

    /// The formats and bytes that set every one of `parts`, in order, as a write sets them.
    pub(crate) fn encode_all<'a>(&self, parts: &'a [ClipPart]) -> Vec<(u32, Cow<'a, [u8]>)> {
        parts.iter().flat_map(|p| self.encode(p)).collect()
    }

    /// The formats and bytes that set `part`; a file list says "copy", never "move".
    pub(crate) fn encode<'a>(&self, part: &'a ClipPart) -> Vec<(u32, Cow<'a, [u8]>)> {
        match part {
            ClipPart::Text(t) => vec![(self.text, wide_z(&crlf(t.expose())).into())],
            ClipPart::Html(b) => vec![(self.html, with_nul(b.expose()).into())],
            ClipPart::Rtf(b) => vec![(self.rtf, with_nul(b.expose()).into())],
            ClipPart::Image { codec, bytes } => {
                let bytes = bytes.expose().as_slice();
                let format = match codec {
                    ImageCodec::Png => self.png,
                    ImageCodec::Dib if header(bytes) >= Some(V4_HEADER) => self.dib_v5,
                    ImageCodec::Dib => self.dib,
                };
                vec![(format, Cow::Borrowed(bytes))]
            }
            ClipPart::Files(paths) => vec![
                (self.files, hdrop::encode(paths.expose()).into()),
                (self.effect, self.copy.to_ne_bytes().to_vec().into()),
            ],
        }
    }

    /// Whether `format` holds an image we keep.
    fn is_image(&self, format: u32) -> bool {
        [self.png, self.dib_v5, self.dib].contains(&format)
    }

    /// What `format` is read as; `None` for a format we do not keep.
    fn kind(&self, format: u32) -> Option<Kind> {
        Some(match format {
            f if f == self.text => Kind::Text,
            f if f == self.html => Kind::Html,
            f if f == self.rtf => Kind::Rtf,
            f if f == self.png => Kind::Image(ImageCodec::Png),
            f if f == self.dib_v5 || f == self.dib => Kind::Image(ImageCodec::Dib),
            f if f == self.files => Kind::Files,
            _ => return None,
        })
    }
}

/// Bytes in a `BITMAPV4HEADER`: a DIB whose header is this long or longer goes as `CF_DIBV5`, and Windows makes the
/// `CF_DIB` from it.
pub(crate) const V4_HEADER: u32 = 108;

/// The header size a DIB starts with; `None` when it is too short to say.
fn header(dib: &[u8]) -> Option<u32> {
    dib.first_chunk::<4>().copied().map(u32::from_le_bytes)
}

/// `text` with each line feed that follows no carriage return made CR LF, as Windows text breaks lines.
pub(crate) fn crlf(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last = None;
    for c in text.chars() {
        if c == '\n' && last != Some('\r') {
            out.push('\r');
        }
        out.push(c);
        last = Some(c);
    }
    out
}

/// Whether `text` holds only white space and NULs, as a clear that blanks the clipboard leaves.
pub(crate) fn blank(text: &str) -> bool {
    text.chars().all(|c| c.is_whitespace() || c == '\0')
}

/// The text before the first NUL of `units`, or `None` past `max_chars` characters. A counting pass sizes the buffer
/// exactly, so the decode neither grows nor copies it and the caller wipes the one buffer; a text past the cap is never
/// decoded.
pub(crate) fn text_of(units: &[u16], max_chars: usize) -> Option<String> {
    let limit = max_chars.saturating_mul(UNITS_PER_CHAR);
    let scan = units.get(..units.len().min(limit.saturating_add(1)))?;
    let len = scan.iter().position(|&u| u == 0).unwrap_or(scan.len());
    let units = scan.get(..len).filter(|_| len <= limit)?;
    // A lone surrogate decodes to U+FFFD, three bytes, so both passes agree.
    let chars = || {
        char::decode_utf16(units.iter().copied()).map(|c| c.unwrap_or(char::REPLACEMENT_CHARACTER))
    };
    let (count, bytes) = chars().fold((0, 0), |(n, b), c| (n + 1, b + c.len_utf8()));
    if count > max_chars {
        return None;
    }
    let mut text = String::with_capacity(bytes);
    text.extend(chars());
    Some(text)
}

/// The most UTF-16 units one character takes.
pub(crate) const UNITS_PER_CHAR: usize = 2;

/// `bytes` before their first NUL, as text formats end.
pub(crate) fn until_nul(bytes: &[u8]) -> &[u8] {
    bytes.split(|&b| b == 0).next().unwrap_or(bytes)
}

/// The signature every PNG starts with.
pub(crate) const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";
/// The type of the chunk that ends a PNG.
pub(crate) const PNG_END: &[u8] = b"IEND";
/// Bytes in a PNG chunk's length, before its type.
const CHUNK_LEN: usize = 4;
/// Bytes in a PNG chunk's type, before its data.
const CHUNK_TYPE: usize = 4;
/// Bytes in a PNG chunk's check, after its data.
pub(crate) const CHUNK_CHECK: usize = 4;

/// `bytes` up to the end of their PNG's end chunk, as the clipboard's memory may run past it; all of them when they
/// do not read as a whole PNG.
pub(crate) fn png_end(bytes: &[u8]) -> &[u8] {
    png_len(bytes).map_or(bytes, |len| &bytes[..len])
}

/// Where the PNG in `bytes` ends, walking its chunks; `None` without its signature, or when a chunk runs past the
/// bytes before the end chunk.
fn png_len(bytes: &[u8]) -> Option<usize> {
    let mut at = PNG_SIGNATURE.len();
    if !bytes.starts_with(PNG_SIGNATURE) {
        return None;
    }
    loop {
        let type_at = at + CHUNK_LEN;
        let len: [u8; CHUNK_LEN] = bytes.get(at..type_at)?.try_into().ok()?;
        let data = usize::try_from(u32::from_be_bytes(len)).ok()?;
        let next = data.checked_add(type_at + CHUNK_TYPE + CHUNK_CHECK)?;
        if next > bytes.len() {
            return None;
        }
        if bytes.get(type_at..type_at + CHUNK_TYPE)? == PNG_END {
            return Some(next);
        }
        at = next;
    }
}

/// `text` in UTF-16 with a closing NUL, as native-order bytes.
pub(crate) fn wide_z(text: &str) -> Vec<u8> {
    text.encode_utf16()
        .chain([0])
        .flat_map(u16::to_ne_bytes)
        .collect()
}

/// `bytes` with a closing NUL, as text formats end.
fn with_nul(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len() + 1);
    out.extend_from_slice(bytes);
    out.push(0);
    out
}

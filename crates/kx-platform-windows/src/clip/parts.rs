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
    /// The parts to read from a clipboard holding `held`, in its order (the richest first), each kind once. The images
    /// are PNG and the richest DIB (DIBV5, else DIB), both in the place of the first image format, so an insert sets
    /// every format apps read.
    pub(crate) fn pick(&self, held: &[u32]) -> Vec<Want> {
        let mut out: Vec<Want> = Vec::new();
        for &format in held {
            let formats = if self.is_image(format) {
                self.images(held)
            } else {
                vec![format]
            };
            for format in formats {
                let Some(kind) = self.kind(format) else {
                    continue;
                };
                if !out.iter().any(|w| w.kind == kind) {
                    out.push(Want { kind, format });
                }
            }
        }
        out
    }

    /// The image formats to read from `held`: PNG when held, then DIBV5 when held, else DIB.
    fn images(&self, held: &[u32]) -> Vec<u32> {
        let dib = [self.dib_v5, self.dib]
            .into_iter()
            .find(|f| held.contains(f));
        let png = held.contains(&self.png).then_some(self.png);
        png.into_iter().chain(dib).collect()
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

/// The text before the first NUL of `units`; `None` past `max_chars` characters.
pub(crate) fn text_of(units: &[u16], max_chars: usize) -> Option<String> {
    let limit = max_chars.saturating_mul(UNITS_PER_CHAR);
    let scan = units.get(..units.len().min(limit.saturating_add(1)))?;
    let len = scan.iter().position(|&u| u == 0).unwrap_or(scan.len());
    let text = String::from_utf16_lossy(scan.get(..len)?);
    (len <= limit && text.chars().count() <= max_chars).then_some(text)
}

/// The most UTF-16 units one character takes.
pub(crate) const UNITS_PER_CHAR: usize = 2;

/// `bytes` before their first NUL, as text formats end.
pub(crate) fn until_nul(bytes: &[u8]) -> &[u8] {
    bytes.split(|&b| b == 0).next().unwrap_or(bytes)
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

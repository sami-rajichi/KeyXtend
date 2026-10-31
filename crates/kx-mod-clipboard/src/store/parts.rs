//! An entry's content as the store keeps it: its parts in one versioned byte form, which the store splits into blobs
//! `<id hex>-<n>`, and its thumbnail as `<id hex>-t`. Saved blobs hold the tags, so they never change.

use kx_module_api::Redacted;
use kx_module_api::ports::{BlobName, ClipPart, ImageCodec, Pixels, SealedError};

use super::codec::{Bad, Reader, Writer};

/// The content form's version, its first byte.
const VERSION: u8 = 1;
/// The thumbnail form's version, its first byte.
const THUMB_VERSION: u8 = 1;
/// What stands between an entry's id and the rest of its blob names.
const SEP: char = '-';
/// What follows the id in a thumbnail's name.
const THUMB: &str = "t";
/// The radix of the id in blob names.
const HEX: u32 = 16;

/// The plain text part's tag.
const TEXT: u8 = 1;
/// The HTML part's tag.
const HTML: u8 = 2;
/// The RTF part's tag.
const RTF: u8 = 3;
/// The image part's tag.
const IMAGE: u8 = 4;
/// The file list part's tag.
const FILES: u8 = 5;
/// The PNG codec's tag.
const PNG: u8 = 1;
/// The DIB codec's tag.
const DIB: u8 = 2;

/// Which of an entry's blobs a name is.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Blob {
    /// Content blob `n`.
    Part(usize),
    /// The thumbnail.
    Thumb,
}

/// The byte form of `parts`, which `decode` reads back equal.
pub(crate) fn encode(parts: &[ClipPart]) -> Vec<u8> {
    let mut w = Writer::default();
    w.u8(VERSION);
    w.count(parts.len());
    for part in parts {
        put(&mut w, part);
    }
    w.done()
}

/// The parts in `bytes`.
pub(crate) fn decode(bytes: &[u8]) -> Result<Vec<ClipPart>, Bad> {
    let mut r = Reader::new(bytes);
    if r.u8()? != VERSION {
        return Err(Bad);
    }
    let n = r.count()?;
    let parts = (0..n).map(|_| part(&mut r)).collect::<Result<_, _>>()?;
    if r.done() { Ok(parts) } else { Err(Bad) }
}

/// The byte form of a thumbnail, which `decode_thumb` reads back equal.
pub(crate) fn encode_thumb(thumb: &Pixels) -> Vec<u8> {
    let mut w = Writer::default();
    w.u8(THUMB_VERSION);
    w.u32(thumb.width());
    w.u32(thumb.height());
    w.bytes(thumb.rgba().expose());
    w.done()
}

/// The thumbnail in `bytes`; `Bad` when a side is over `side`.
pub(crate) fn decode_thumb(bytes: &[u8], side: u32) -> Result<Pixels, Bad> {
    let mut r = Reader::new(bytes);
    if r.u8()? != THUMB_VERSION {
        return Err(Bad);
    }
    let (width, height) = (r.u32()?, r.u32()?);
    let rgba = r.bytes()?.to_vec();
    if !r.done() {
        return Err(Bad);
    }
    Pixels::new(width, height, rgba, side).ok_or(Bad)
}

/// The name of content blob `n` of entry `id`.
pub(crate) fn part_name(id: u64, n: usize) -> Result<BlobName, SealedError> {
    BlobName::new(&format!("{id:x}{SEP}{n}"))
}

/// The name of entry `id`'s thumbnail.
pub(crate) fn thumb_name(id: u64) -> Result<BlobName, SealedError> {
    BlobName::new(&format!("{id:x}{SEP}{THUMB}"))
}

/// The entry and the blob a name gives, as `part_name` and `thumb_name` write them; `None` for any other name.
pub(crate) fn owner(name: &BlobName) -> Option<(u64, Blob)> {
    let (hex, rest) = name.as_str().split_once(SEP)?;
    let id = u64::from_str_radix(hex, HEX).ok()?;
    let blob = if rest == THUMB {
        Blob::Thumb
    } else {
        Blob::Part(rest.parse().ok()?)
    };
    let back = match blob {
        Blob::Part(n) => part_name(id, n),
        Blob::Thumb => thumb_name(id),
    };
    (back.ok()? == *name).then_some((id, blob))
}

/// Writes one part: its tag, then its bytes; an image, its codec first.
fn put(w: &mut Writer, part: &ClipPart) {
    match part {
        ClipPart::Text(text) => {
            w.u8(TEXT);
            w.text(text.expose());
        }
        ClipPart::Html(bytes) => {
            w.u8(HTML);
            w.bytes(bytes.expose());
        }
        ClipPart::Rtf(bytes) => {
            w.u8(RTF);
            w.bytes(bytes.expose());
        }
        ClipPart::Image { codec, bytes } => {
            w.u8(IMAGE);
            w.u8(codec_tag(*codec));
            w.bytes(bytes.expose());
        }
        ClipPart::Files(paths) => {
            w.u8(FILES);
            w.paths(paths.expose());
        }
    }
}

/// Reads one part.
fn part(r: &mut Reader<'_>) -> Result<ClipPart, Bad> {
    let bytes = |r: &mut Reader<'_>| r.bytes().map(|b| Redacted::new(b.to_vec()));
    Ok(match r.u8()? {
        TEXT => ClipPart::Text(Redacted::new(r.text()?.to_owned())),
        HTML => ClipPart::Html(bytes(r)?),
        RTF => ClipPart::Rtf(bytes(r)?),
        IMAGE => {
            let codec = codec_of(r.u8()?)?;
            ClipPart::Image {
                codec,
                bytes: bytes(r)?,
            }
        }
        FILES => ClipPart::Files(Redacted::new(r.paths()?)),
        _ => return Err(Bad),
    })
}

/// The codec's tag.
fn codec_tag(codec: ImageCodec) -> u8 {
    match codec {
        ImageCodec::Png => PNG,
        ImageCodec::Dib => DIB,
    }
}

/// The codec a tag names.
fn codec_of(tag: u8) -> Result<ImageCodec, Bad> {
    match tag {
        PNG => Ok(ImageCodec::Png),
        DIB => Ok(ImageCodec::Dib),
        _ => Err(Bad),
    }
}

#[cfg(test)]
mod tests;

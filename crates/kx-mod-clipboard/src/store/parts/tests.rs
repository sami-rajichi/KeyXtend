//! The content form: every part kind round-trips, a wrong version or a trailing byte is refused, blob names read back
//! to their entry, and thumbnails keep within their side.

use kx_module_api::Redacted;
use kx_module_api::ports::{ClipPart, ImageCodec, Pixels, RGBA_BYTES};

use super::*;
use crate::fixture::{files, text};

/// The longest thumbnail side the tests allow.
const SIDE: u32 = 4;

/// One part of every kind, with text and paths beyond ASCII.
fn every_kind() -> Vec<ClipPart> {
    let bytes = |b: &[u8]| Redacted::new(b.to_vec());
    vec![
        text("École أرسلت 😀"),
        ClipPart::Html(bytes(b"<b>x</b>")),
        ClipPart::Rtf(bytes(b"{\\rtf1}")),
        ClipPart::Image {
            codec: ImageCodec::Png,
            bytes: bytes(&[0, 1, 2]),
        },
        ClipPart::Image {
            codec: ImageCodec::Dib,
            bytes: bytes(&[]),
        },
        files(&["C:\\Users\\Sami\\Desktop\\تقرير.pdf", "D:\\a b"]),
    ]
}

/// A `width` by `height` thumbnail of grey pixels.
fn grey(width: u32, height: u32, side: u32) -> Pixels {
    let n = usize::try_from(width * height).unwrap() * RGBA_BYTES;
    Pixels::new(width, height, vec![128; n], side).unwrap()
}

#[test]
fn every_part_kind_round_trips() {
    let parts = every_kind();
    assert_eq!(decode(&encode(&parts)), Ok(parts));
    assert_eq!(decode(&encode(&[])), Ok(vec![]), "no part at all");
}

#[test]
fn another_version_a_bad_tag_or_a_trailing_byte_is_refused() {
    let mut bytes = encode(&[text("a")]);
    bytes[0] = VERSION + 1;
    assert_eq!(decode(&bytes), Err(Bad));
    let mut bytes = encode(&[text("a")]);
    bytes.push(0);
    assert_eq!(decode(&bytes), Err(Bad));
    let mut w = Writer::default();
    w.u8(VERSION);
    w.count(1);
    w.u8(FILES + 1);
    assert_eq!(decode(&w.done()), Err(Bad), "an unknown part tag");
    let image = encode(&[ClipPart::Image {
        codec: ImageCodec::Png,
        bytes: Redacted::new(vec![]),
    }]);
    let mut bad = image.clone();
    let at = image.iter().rposition(|&b| b == PNG).unwrap();
    bad[at] = DIB + 1;
    assert_eq!(decode(&bad), Err(Bad), "an unknown codec tag");
}

#[test]
fn blob_names_read_back_to_their_entry() {
    let name = part_name(0xab, 2).unwrap();
    assert_eq!(name.as_str(), "ab-2");
    assert_eq!(owner(&name), Some((0xab, Blob::Part(2))));
    let thumb = thumb_name(u64::MAX).unwrap();
    assert_eq!(thumb.as_str(), "ffffffffffffffff-t");
    assert_eq!(owner(&thumb), Some((u64::MAX, Blob::Thumb)));
    for other in [
        "list",
        "list-0-1",
        "0ab-2",
        "ab-02",
        "ab-x",
        "ab",
        "2026-10-31",
    ] {
        let name = BlobName::new(other).unwrap();
        assert_eq!(owner(&name), None, "{other}");
    }
}

#[test]
fn a_thumbnail_round_trips_within_its_side() {
    let thumb = grey(SIDE, 2, SIDE);
    assert_eq!(decode_thumb(&encode_thumb(&thumb), SIDE), Ok(thumb.clone()));
    assert_eq!(
        decode_thumb(&encode_thumb(&thumb), SIDE - 1),
        Err(Bad),
        "a thumbnail from a larger side setting is drawn again"
    );
    let mut cut = encode_thumb(&thumb);
    cut.pop();
    assert_eq!(decode_thumb(&cut, SIDE), Err(Bad));
}

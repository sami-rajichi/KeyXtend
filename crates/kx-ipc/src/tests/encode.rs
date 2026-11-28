//! The media job's `Encode` and its PNG, both directions: the wire layout, round trips, caps derived from
//! `MAX_ENCODE_BYTES`, refusals before any pixel is read, the host's bounded read, and `Debug` that shows lengths only.

use super::*;
use crate::limits::{DRAW_HEAD, ENCODE_HEAD, MAX_PIXELS_LEN, PIXELS_HEAD};
use crate::{BGRX_BYTES, MAX_ENCODE_BYTES, MAX_PNG_BYTES, MAX_RGBA_BYTES, MIB, png_most};
use IpcError::{BadField, BadKind, Inconsistent, TooLong};

/// Bytes a PNG adds before its pixels and after them: the signature, the header chunk and the end chunk.
const PNG_FRAME: usize = 8 + 25 + 12;
/// Bytes in a PNG row's filter mark, one a row.
const FILTER: usize = 1;
/// Bytes a PNG keeps of one raw pixel once its unused byte is dropped.
const RGB: usize = 3;
/// The most bytes one stored deflate block holds.
const STORED: usize = 65_535;
/// What each stored block adds before its bytes.
const STORED_HEAD: usize = 5;
/// The zlib header and checksum around the deflated rows.
const ZLIB: usize = 6;
/// What each data chunk adds: its length, type and checksum.
const CHUNK_FRAME: usize = 12;
/// The smallest data chunk an encoder writes, so its frames stay a small share.
const SMALLEST_CHUNK: usize = 1024;
/// A wide picture's pixels in a row, for one that passes the cap.
const WIDE: u32 = 16_384;

/// `width` by `height` raw pixels, `stride` bytes a row, whose bytes count up.
fn encode(id: u32, width: u32, height: u32, stride: u32) -> Ask<'static> {
    let len = stride as usize * height as usize;
    let bgrx = (0..len).map(|i| i.to_le_bytes()[0]).collect();
    Ask::Encode {
        id,
        width,
        height,
        stride,
        bgrx: Cow::Owned(bgrx),
    }
}

/// A PNG reply of `bytes`.
fn png(id: u32, bytes: &[u8]) -> Reply {
    Reply::Png {
        id,
        bytes: bytes.to_vec(),
    }
}

/// An encode's fields as a payload, before `pixels` bytes.
fn fields(width: u32, height: u32, stride: u32, pixels: usize) -> Vec<u8> {
    let sides = [width, height, stride].map(u32::to_le_bytes).concat();
    [sides, vec![0; pixels]].concat()
}

/// Checks that writing `ask` is refused with `err` and writes nothing.
fn refused(ask: &Ask, err: IpcError) {
    let mut out = Vec::new();
    assert_eq!(ask.write(&mut out), Err(err), "{ask:?}");
    assert!(out.is_empty());
}

#[test]
fn an_encode_and_its_png_are_little_endian_after_their_kinds() {
    let ask = Ask::Encode {
        id: 7,
        width: 1,
        height: 1,
        stride: 4,
        bgrx: Cow::Borrowed(&[1, 2, 3, 4]),
    };
    let want = b"KXIP\x01\x3a\x07\0\0\0\x10\0\0\0\x01\0\0\0\x01\0\0\0\x04\0\0\0\x01\x02\x03\x04";
    assert_eq!(ask_bytes(&ask), want);
    let reply = b"KXIP\x01\x3b\x07\0\0\0\x02\0\0\0\x89P";
    assert_eq!(reply_bytes(&png(7, b"\x89P")), reply);
}

#[test]
fn each_encode_and_png_round_trips_and_names_its_ask() {
    let asks = [
        encode(1, 1, 1, 4),
        encode(2, 3, 2, 12),
        encode(u32::MAX, 3, 2, 17),
    ];
    for ask in asks {
        assert_eq!(Ask::read(&mut &ask_bytes(&ask)[..]), Ok(ask));
    }
    for reply in [png(3, b"p"), png(4, &[9; 300])] {
        assert_eq!(Reply::read(&mut &reply_bytes(&reply)[..]), Ok(reply));
    }
    assert_eq!(png(5, b"p").id(), 5);
}

#[test]
fn the_media_caps_are_the_largest_encode_and_its_png() {
    assert_eq!(MAX_ASK_LEN, ENCODE_HEAD + MAX_ENCODE_BYTES);
    assert_eq!(MAX_PNG_BYTES, png_most(MAX_ENCODE_BYTES));
    assert_eq!(MAX_REPLY_LEN, MAX_PNG_BYTES);
    let len = u32::try_from(MAX_ASK_LEN).unwrap();
    let raw = head(MAGIC, VERSION, Kind::Encode.code(), 1, len);
    let want = Head {
        kind: Kind::Encode,
        id: 1,
        len: MAX_ASK_LEN,
    };
    assert_eq!(Head::read(&mut &raw[..], &[Kind::Encode]), Ok(want));
    let over = head(MAGIC, VERSION, Kind::Encode.code(), 1, len + 1);
    assert_eq!(Head::read(&mut &over[..], &[Kind::Encode]), Err(TooLong));
}

#[test]
fn a_draw_and_its_pixels_keep_their_own_caps() {
    let over = |n: usize| u32::try_from(n + 1).unwrap();
    let cases = [
        (
            Kind::Draw,
            over(DRAW_HEAD + MAX_IMAGE_BYTES),
            read_ask as Reader,
        ),
        (Kind::Pixels, over(PIXELS_HEAD + MAX_RGBA_BYTES), read_reply),
        (Kind::Png, over(MAX_PNG_BYTES), read_reply),
    ];
    for (kind, len, read) in cases {
        let raw = head(MAGIC, VERSION, kind.code(), 0, len);
        let mut r = Counted::new((&raw[..]).chain(io::repeat(0)));
        assert_eq!(read(&mut r), Err(TooLong), "{kind:?}");
        assert_eq!(r.taken, HEADER_LEN);
    }
    assert_eq!(MAX_PIXELS_LEN, PIXELS_HEAD + MAX_RGBA_BYTES);
    const { assert!(MAX_IMAGE_BYTES <= MAX_PIXELS_LEN) };
}

#[test]
fn a_png_within_its_bound_holds_its_pixels_stored_whole() {
    for raw in [BGRX_BYTES, MIB, MAX_ENCODE_BYTES] {
        let rows = raw / BGRX_BYTES;
        let filtered = rows * (RGB + FILTER);
        let deflated = filtered + filtered.div_ceil(STORED) * STORED_HEAD + ZLIB;
        let chunks = deflated.div_ceil(SMALLEST_CHUNK) * CHUNK_FRAME;
        assert!(deflated + chunks + PNG_FRAME <= png_most(raw), "{raw}");
    }
    assert!(u32::try_from(png_most(MAX_ENCODE_BYTES)).is_ok());
}

#[test]
fn pixels_outside_their_sides_stride_or_cap_are_refused_both_ways() {
    let row = WIDE * u32::try_from(BGRX_BYTES).unwrap();
    let past = u32::try_from(MAX_ENCODE_BYTES).unwrap() / row + 1;
    let cases = [
        (0, 1, 4),
        (1, 0, 4),
        (2, 1, 7),
        (u32::MAX, 1, u32::MAX),
        (WIDE, past, row),
    ];
    for (width, height, stride) in cases {
        let ask = Ask::Encode {
            id: 1,
            width,
            height,
            stride,
            bgrx: Cow::Borrowed(&[0; BGRX_BYTES]),
        };
        refused(&ask, BadField);
        let bytes = frame(Kind::Encode, 1, &fields(width, height, stride, BGRX_BYTES));
        assert_eq!(read_ask(&mut &bytes[..]), Err(BadField), "{ask:?}");
    }
}

#[test]
fn pixels_that_are_not_stride_by_height_are_inconsistent_both_ways() {
    let ask = Ask::Encode {
        id: 1,
        width: 1,
        height: 1,
        stride: 4,
        bgrx: Cow::Borrowed(&[0; 5]),
    };
    refused(&ask, Inconsistent);
    for pixels in [5, 8] {
        let bytes = frame(Kind::Encode, 1, &fields(1, 1, 4, pixels));
        assert_eq!(read_ask(&mut &bytes[..]), Err(Inconsistent), "{pixels}");
    }
    let short = frame(Kind::Encode, 1, &fields(1, 1, 4, 0)[..ENCODE_HEAD]);
    assert_eq!(read_ask(&mut &short[..]), Err(Inconsistent));
}

#[test]
fn a_lying_encode_length_is_refused_before_the_pixels() {
    let len = u32::try_from(MAX_ASK_LEN).unwrap();
    let raw = head(MAGIC, VERSION, Kind::Encode.code(), 1, len);
    let sides = fields(1, 1, 4, 0);
    let mut r = Counted::new((&raw[..]).chain(&sides[..]).chain(io::repeat(0)));
    assert_eq!(read_ask(&mut r), Err(Inconsistent));
    assert_eq!(r.taken, HEADER_LEN + ENCODE_HEAD);
}

#[test]
fn the_hosts_read_holds_a_png_only_within_its_bound() {
    let len = u32::try_from(MAX_PIXELS_LEN + 1).unwrap();
    let raw = head(MAGIC, VERSION, Kind::Png.code(), 1, len);
    let mut r = Counted::new((&raw[..]).chain(io::repeat(0)));
    assert_eq!(Reply::read(&mut r), Err(TooLong));
    assert_eq!(r.taken, HEADER_LEN);
    let ten = reply_bytes(&png(1, &[7; 10]));
    assert_eq!(Reply::read_within(&mut &ten[..], 10), Ok(png(1, &[7; 10])));
    let mut r = Counted::new(&ten[..]);
    assert_eq!(Reply::read_within(&mut r, 9), Err(TooLong));
    assert_eq!(r.taken, HEADER_LEN);
    let failed = reply_bytes(&Reply::Failed {
        id: 2,
        why: Why::TooBig,
    });
    assert_eq!(Reply::read_within(&mut &failed[..], 0), Err(TooLong));
}

#[test]
fn each_side_reads_only_the_other_sides_encode_kind() {
    let ask = ask_bytes(&encode(1, 1, 1, 4));
    assert_eq!(read_reply(&mut &ask[..]), Err(BadKind));
    let reply = reply_bytes(&png(1, b"p"));
    assert_eq!(read_ask(&mut &reply[..]), Err(BadKind));
}

#[test]
fn a_cut_encode_or_png_is_short_and_an_empty_stream_closed() {
    let cases: [(Vec<u8>, Reader); 2] = [
        (ask_bytes(&encode(5, 2, 2, 8)), read_ask),
        (reply_bytes(&png(6, b"png")), read_reply),
    ];
    for (bytes, read) in cases {
        assert_eq!(read(&mut &bytes[..0]), Err(IpcError::Closed));
        for cut in 1..bytes.len() {
            assert_eq!(
                read(&mut &bytes[..cut]),
                Err(IpcError::Short),
                "cut at {cut}"
            );
        }
    }
}

#[test]
fn debug_shows_the_encode_frames_lengths_never_pixels() {
    let ask = format!("{:?}", encode(3, 1, 2, 8));
    assert_eq!(
        ask,
        "Encode { id: 3, width: 1, height: 2, stride: 8, len: 16 }"
    );
    assert_eq!(format!("{:?}", png(4, b"secret")), "Png { id: 4, len: 6 }");
}

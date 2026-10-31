//! The DIB ask and its reply, both directions: the wire layout, round trips, each side's kinds, the caps and their
//! refusals before any payload is read, cut frames, and `Debug` that shows lengths only.

use super::*;
use IpcError::{BadField, BadKind, Inconsistent, TooLong};

/// An ask to make a DIB of `png`, at most `max_bytes` long.
fn make(id: u32, max_bytes: u32, png: &[u8]) -> Ask<'_> {
    Ask::Dib {
        id,
        max_bytes,
        bytes: Cow::Borrowed(png),
    }
}

/// A DIB reply of `bytes`.
fn dib(id: u32, bytes: &[u8]) -> Reply {
    Reply::Dib {
        id,
        bytes: bytes.to_vec(),
    }
}

/// The largest DIB a reply may hold, as an ask's field.
fn cap() -> u32 {
    u32::try_from(MAX_IMAGE_BYTES).unwrap()
}

#[test]
fn the_ask_and_reply_are_little_endian_after_their_kinds() {
    let ask = b"KXIP\x01\x05\x07\0\0\0\x06\0\0\0\x2c\x01\0\0\x09\x08";
    assert_eq!(ask_bytes(&make(7, 300, &[9, 8])), ask);
    let reply = b"KXIP\x01\x06\x07\0\0\0\x03\0\0\0\x28\0\0";
    assert_eq!(reply_bytes(&dib(7, &[40, 0, 0])), reply);
}

#[test]
fn the_ask_and_reply_round_trip_at_their_caps_ends() {
    for ask in [make(1, 1, b"p"), make(u32::MAX, cap(), b"png")] {
        assert_eq!(Ask::read(&mut &ask_bytes(&ask)[..]), Ok(ask));
    }
    let largest = dib(2, &vec![7; MAX_IMAGE_BYTES]);
    let bytes = reply_bytes(&largest);
    assert_eq!(bytes.len(), HEADER_LEN + MAX_IMAGE_BYTES);
    assert_eq!(Reply::read(&mut &bytes[..]), Ok(largest));
    assert_eq!(dib(9, b"x").id(), 9);
}

#[test]
fn each_side_reads_only_the_other_sides_dib_kind() {
    let ask = ask_bytes(&make(1, 10, b"p"));
    assert_eq!(read_reply(&mut &ask[..]), Err(BadKind));
    let reply = reply_bytes(&dib(1, b"d"));
    assert_eq!(read_ask(&mut &reply[..]), Err(BadKind));
}

#[test]
fn a_cap_outside_one_to_the_largest_image_is_refused_both_ways() {
    for max in [0, cap() + 1, u32::MAX] {
        let mut out = Vec::new();
        assert_eq!(make(1, max, b"p").write(&mut out), Err(BadField), "{max}");
        assert!(out.is_empty());
        let payload = [&max.to_le_bytes()[..], b"p"].concat();
        let raw = frame(Kind::MakeDib, 1, &payload);
        assert_eq!(read_ask(&mut &raw[..]), Err(BadField), "{max}");
    }
}

#[test]
fn an_empty_or_oversized_png_or_dib_is_refused_both_ways() {
    let big = vec![0; MAX_IMAGE_BYTES + 1];
    let asks = [
        (make(1, 10, b""), Inconsistent),
        (make(1, 10, &big), TooLong),
    ];
    for (ask, err) in asks {
        let mut out = Vec::new();
        assert_eq!(ask.write(&mut out), Err(err));
        assert!(out.is_empty());
    }
    for (bytes, err) in [(&[][..], Inconsistent), (&big[..], TooLong)] {
        let mut out = Vec::new();
        assert_eq!(dib(1, bytes).write(&mut out), Err(err));
        assert!(out.is_empty());
    }
    let short = frame(Kind::MakeDib, 1, &10u32.to_le_bytes());
    assert_eq!(read_ask(&mut &short[..]), Err(Inconsistent), "no PNG");
}

#[test]
fn a_length_past_the_largest_image_is_refused_right_after_the_header() {
    let over = u32::try_from(MAX_IMAGE_BYTES + 1).unwrap();
    let plenty = u64::try_from(READ_CHUNK).unwrap();
    let cases: [(Kind, u32, Reader); 2] = [
        (Kind::MakeDib, over + 4, read_ask),
        (Kind::Dib, over, read_reply),
    ];
    for (kind, len, read) in cases {
        let raw = head(MAGIC, VERSION, kind.code(), 0, len);
        let mut r = Counted::new((&raw[..]).chain(io::repeat(0).take(plenty)));
        assert_eq!(read(&mut r), Err(TooLong), "{kind:?}");
        assert_eq!(r.taken, HEADER_LEN);
    }
}

#[test]
fn a_cut_dib_frame_is_short_and_an_empty_stream_closed() {
    let cases: [(Vec<u8>, Reader); 2] = [
        (ask_bytes(&make(5, 10, b"png")), read_ask),
        (reply_bytes(&dib(6, b"dib")), read_reply),
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
fn debug_shows_the_dib_frames_lengths_never_content() {
    let ask = format!("{:?}", make(3, 300, b"secret"));
    assert_eq!(ask, "Dib { id: 3, max_bytes: 300, len: 6 }");
    assert_eq!(format!("{:?}", dib(4, b"secret!")), "Dib { id: 4, len: 7 }");
}

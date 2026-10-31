//! Round trips, the wire layout and clean truncation; refusals and properties have their own files.

mod props;
mod refuse;

use crate::code::{Code, Kind};
use crate::frame::{Head, grow, read_tail};
use crate::limits::{HEADER_LEN, MAGIC, MAX_SIDE, READ_CHUNK, RGBA_BYTES, VERSION};
use crate::{Ask, Codec, IpcError, MAX_ASK_LEN, MAX_IMAGE_BYTES, MAX_REPLY_LEN, Reply, Why};
use std::borrow::Cow;
use std::io::{self, Read};

/// The longest side the tests' draws ask for.
const SIDE: u32 = 128;

/// A header with any field, for frames a writer never makes.
fn head(magic: [u8; 4], version: u8, kind: u8, id: u32, len: u32) -> Vec<u8> {
    [
        &magic[..],
        &[version, kind],
        &id.to_le_bytes(),
        &len.to_le_bytes(),
    ]
    .concat()
}

/// A frame of `kind` with a good magic and version, and `payload` under its true length.
fn frame(kind: Kind, id: u32, payload: &[u8]) -> Vec<u8> {
    let len = u32::try_from(payload.len()).unwrap();
    [head(MAGIC, VERSION, kind.code(), id, len), payload.to_vec()].concat()
}

/// A reader that counts the bytes taken from it.
struct Counted<R> {
    inner: R,
    taken: usize,
}

impl<R> Counted<R> {
    fn new(inner: R) -> Self {
        Self { inner, taken: 0 }
    }
}

impl<R: Read> Read for Counted<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.taken += n;
        Ok(n)
    }
}

/// A PNG draw of `bytes` within `SIDE`.
fn draw(id: u32, bytes: &[u8]) -> Ask<'_> {
    Ask::Draw {
        id,
        codec: Codec::Png,
        side: SIDE,
        bytes: Cow::Borrowed(bytes),
    }
}

/// `width` by `height` pixels whose bytes count up.
fn pixels(id: u32, width: u32, height: u32) -> Reply {
    let len = width as usize * height as usize * RGBA_BYTES;
    let rgba = (0..len).map(|i| i.to_le_bytes()[0]).collect();
    Reply::Pixels {
        id,
        width,
        height,
        rgba,
    }
}

fn ask_bytes(ask: &Ask) -> Vec<u8> {
    let mut out = Vec::new();
    ask.write(&mut out).unwrap();
    out
}

fn reply_bytes(reply: &Reply) -> Vec<u8> {
    let mut out = Vec::new();
    reply.write(&mut out).unwrap();
    out
}

#[test]
fn frames_are_little_endian_after_the_magic_version_and_kind() {
    let bye = b"KXIP\x01\x02\0\0\0\0\0\0\0\0";
    assert_eq!(ask_bytes(&Ask::Bye), bye);
    let failed = Reply::Failed {
        id: 0x0102_0304,
        why: Why::TooBig,
    };
    assert_eq!(
        reply_bytes(&failed),
        b"KXIP\x01\x04\x04\x03\x02\x01\x01\0\0\0\x02"
    );
    let png = b"KXIP\x01\x01\x07\0\0\0\x07\0\0\0\x01\x80\0\0\0\x09\x08";
    assert_eq!(ask_bytes(&draw(7, &[9, 8])), png);
    assert_eq!(HEADER_LEN, 14);
}

#[test]
fn each_message_round_trips() {
    let asks = [draw(1, b"png"), Ask::Bye];
    for ask in asks {
        assert_eq!(Ask::read(&mut &ask_bytes(&ask)[..]), Ok(ask));
    }
    let dib = Ask::Draw {
        id: u32::MAX,
        codec: Codec::Dib,
        side: MAX_SIDE,
        bytes: Cow::Borrowed(b"dib"),
    };
    assert_eq!(Ask::read(&mut &ask_bytes(&dib)[..]), Ok(dib));
    let mut replies = vec![pixels(2, 3, 5), pixels(3, 1, 1)];
    replies.extend(Why::ALL.iter().map(|&why| Reply::Failed { id: 4, why }));
    for reply in replies {
        assert_eq!(Reply::read(&mut &reply_bytes(&reply)[..]), Ok(reply));
    }
}

#[test]
fn frames_in_one_stream_read_back_in_order_and_then_close() {
    let first = draw(1, b"one");
    let second = draw(2, b"two");
    let stream = [ask_bytes(&first), ask_bytes(&second), ask_bytes(&Ask::Bye)].concat();
    let mut r = &stream[..];
    assert_eq!(Ask::read(&mut r), Ok(first));
    assert_eq!(Ask::read(&mut r), Ok(second));
    assert_eq!(Ask::read(&mut r), Ok(Ask::Bye));
    assert_eq!(Ask::read(&mut r), Err(IpcError::Closed));
}

#[test]
fn the_largest_pixels_round_trip() {
    let largest = pixels(9, MAX_SIDE, MAX_SIDE);
    let bytes = reply_bytes(&largest);
    assert_eq!(bytes.len(), HEADER_LEN + MAX_REPLY_LEN);
    assert_eq!(Reply::read(&mut &bytes[..]), Ok(largest));
}

#[test]
fn the_largest_image_is_written_and_its_header_is_read() {
    let image = vec![0; MAX_IMAGE_BYTES];
    assert_eq!(draw(1, &image).write(&mut io::sink()), Ok(()));
    let len = u32::try_from(MAX_ASK_LEN).unwrap();
    let raw = head(MAGIC, VERSION, Kind::Draw.code(), 1, len);
    let want = Head {
        kind: Kind::Draw,
        id: 1,
        len: MAX_ASK_LEN,
    };
    assert_eq!(Head::read(&mut &raw[..], &[Kind::Draw]), Ok(want));
}

/// One side's reader, keeping only the outcome.
type Reader = fn(&mut dyn Read) -> Result<(), IpcError>;

fn read_ask(r: &mut dyn Read) -> Result<(), IpcError> {
    Ask::read(r).map(drop)
}

fn read_reply(r: &mut dyn Read) -> Result<(), IpcError> {
    Reply::read(r).map(drop)
}

#[test]
fn a_cut_frame_is_short_and_an_empty_stream_closed() {
    let failed = Reply::Failed {
        id: 7,
        why: Why::Internal,
    };
    let cases: [(Vec<u8>, Reader); 4] = [
        (ask_bytes(&draw(5, b"image")), read_ask),
        (ask_bytes(&Ask::Bye), read_ask),
        (reply_bytes(&pixels(6, 2, 2)), read_reply),
        (reply_bytes(&failed), read_reply),
    ];
    for (bytes, read) in cases {
        assert_eq!(read(&mut &bytes[..0]), Err(IpcError::Closed));
        for cut in 1..bytes.len() {
            let got = read(&mut &bytes[..cut]);
            assert_eq!(got, Err(IpcError::Short), "cut at {cut}");
        }
        assert_eq!(read(&mut &bytes[..]), Ok(()));
    }
}

#[test]
fn a_payload_read_grows_from_one_chunk_and_ends_at_its_length() {
    assert_eq!(grow(0, 10), 10);
    assert_eq!(grow(0, READ_CHUNK * 4), READ_CHUNK);
    assert_eq!(grow(READ_CHUNK, READ_CHUNK * 4), READ_CHUNK * 2);
    assert_eq!(grow(READ_CHUNK * 3, READ_CHUNK * 4), READ_CHUNK * 4);
    let data = vec![3; READ_CHUNK * 3 + 1];
    let tail = read_tail(&mut &data[..], data.len()).unwrap();
    assert_eq!(tail, data);
    // Today's `Vec` keeps exactly the capacity `reserve_exact` asks for, so this checks no read reserves past the
    // length; its docs would let an allocator give more, which would fail this line without a bug.
    assert!(tail.capacity() <= data.len());
}

#[test]
fn the_reply_names_its_draw() {
    assert_eq!(pixels(11, 1, 1).id(), 11);
    let failed = Reply::Failed {
        id: 12,
        why: Why::Refused,
    };
    assert_eq!(failed.id(), 12);
}

#[test]
fn debug_shows_lengths_never_content() {
    let ask = format!("{:?}", draw(3, b"secret"));
    assert_eq!(ask, "Draw { id: 3, codec: Png, side: 128, len: 6 }");
    let reply = format!("{:?}", pixels(4, 1, 2));
    assert_eq!(reply, "Pixels { id: 4, width: 1, height: 2, len: 8 }");
    assert_eq!(format!("{:?}", Ask::Bye), "Bye");
}

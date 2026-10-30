//! Each refusal, on reading and on writing: magic, version, kind, length, consistency, field range and stream errors.

use super::*;
use crate::limits::FAILED_LEN;
use IpcError::{BadField, BadKind, BadMagic, BadVersion, Inconsistent, TooLong};

/// What reading `bytes` as an ask and as a reply gives.
fn both(bytes: &[u8]) -> (Result<(), IpcError>, Result<(), IpcError>) {
    (read_ask(&mut &bytes[..]), read_reply(&mut &bytes[..]))
}

/// A draw's payload with any codec byte and side, and a one-byte image.
fn draw_payload(codec: u8, side: u32) -> Vec<u8> {
    [&[codec][..], &side.to_le_bytes(), b"x"].concat()
}

/// A pixel reply's payload with any sides and `rgba_len` bytes of pixels.
fn pixel_payload(width: u32, height: u32, rgba_len: usize) -> Vec<u8> {
    [
        &width.to_le_bytes()[..],
        &height.to_le_bytes(),
        &vec![0; rgba_len],
    ]
    .concat()
}

#[test]
fn a_wrong_magic_is_refused() {
    let good = ask_bytes(&Ask::Bye);
    for at in 0..MAGIC.len() {
        let mut bad = good.clone();
        bad[at] ^= 0x20;
        assert_eq!(both(&bad), (Err(BadMagic), Err(BadMagic)));
    }
}

#[test]
fn another_version_is_refused() {
    for version in [0, VERSION + 1, u8::MAX] {
        let bad = head(MAGIC, version, Kind::Bye.code(), 0, 0);
        assert_eq!(both(&bad), (Err(BadVersion), Err(BadVersion)));
    }
}

#[test]
fn an_unknown_kind_is_refused() {
    for kind in [0, 5, u8::MAX] {
        let bad = head(MAGIC, VERSION, kind, 0, 0);
        assert_eq!(both(&bad), (Err(BadKind), Err(BadKind)));
    }
}

#[test]
fn each_side_reads_only_the_other_sides_kinds() {
    let failed = Reply::Failed {
        id: 1,
        why: Why::Refused,
    };
    for ask in [draw(1, b"x"), Ask::Bye] {
        assert_eq!(read_reply(&mut &ask_bytes(&ask)[..]), Err(BadKind));
    }
    for reply in [pixels(1, 1, 1), failed] {
        assert_eq!(read_ask(&mut &reply_bytes(&reply)[..]), Err(BadKind));
    }
}

#[test]
fn a_length_past_its_kind_is_refused_before_the_payload() {
    let over = |n: usize| u32::try_from(n + 1).unwrap();
    let cases: [(Kind, u32, Reader); 6] = [
        (Kind::Draw, over(MAX_ASK_LEN), read_ask),
        (Kind::Draw, u32::MAX, read_ask),
        (Kind::Bye, 1, read_ask),
        (Kind::Pixels, over(MAX_REPLY_LEN), read_reply),
        (Kind::Pixels, u32::MAX, read_reply),
        (Kind::Failed, over(FAILED_LEN), read_reply),
    ];
    let plenty = u64::try_from(READ_CHUNK).unwrap();
    for (kind, len, read) in cases {
        let raw = head(MAGIC, VERSION, kind.code(), 0, len);
        let mut r = Counted::new((&raw[..]).chain(io::repeat(0).take(plenty)));
        assert_eq!(read(&mut r), Err(TooLong), "{kind:?} {len}");
        assert_eq!(r.taken, HEADER_LEN);
    }
}

#[test]
fn a_length_short_of_its_fields_is_inconsistent() {
    let png = Codec::Png.code();
    let cases: [(Vec<u8>, Reader); 5] = [
        (frame(Kind::Draw, 1, &[png, 0x80, 0, 0, 0]), read_ask),
        (frame(Kind::Draw, 1, &[png, 0x80]), read_ask),
        (frame(Kind::Pixels, 1, &[1, 0, 0, 0]), read_reply),
        (frame(Kind::Pixels, 1, &pixel_payload(1, 1, 0)), read_reply),
        (frame(Kind::Failed, 1, &[]), read_reply),
    ];
    for (bytes, read) in cases {
        assert_eq!(read(&mut &bytes[..]), Err(Inconsistent));
    }
    assert_eq!(
        read_ask(&mut &frame(Kind::Bye, 5, &[])[..]),
        Err(Inconsistent)
    );
}

#[test]
fn pixels_that_are_not_width_by_height_are_inconsistent() {
    for n in [15, 17, 4, 64] {
        let bytes = frame(Kind::Pixels, 1, &pixel_payload(2, 2, n));
        assert_eq!(read_reply(&mut &bytes[..]), Err(Inconsistent), "{n} bytes");
    }
    let huge = frame(
        Kind::Pixels,
        1,
        &pixel_payload(MAX_SIDE, MAX_SIDE, RGBA_BYTES),
    );
    assert_eq!(read_reply(&mut &huge[..]), Err(Inconsistent));
}

#[test]
fn a_lying_pixel_length_is_refused_before_the_pixels() {
    let len = u32::try_from(MAX_REPLY_LEN).unwrap();
    let raw = head(MAGIC, VERSION, Kind::Pixels.code(), 1, len);
    let fields = pixel_payload(1, 1, 0);
    let mut r = Counted::new((&raw[..]).chain(&fields[..]).chain(io::repeat(0)));
    assert_eq!(read_reply(&mut r), Err(Inconsistent));
    assert_eq!(r.taken, HEADER_LEN + fields.len());
}

#[test]
fn a_field_out_of_range_is_refused() {
    let png = Codec::Png.code();
    let asks = [
        draw_payload(0, SIDE),
        draw_payload(u8::MAX, SIDE),
        draw_payload(png, 0),
        draw_payload(png, MAX_SIDE + 1),
    ];
    for payload in asks {
        assert_eq!(
            read_ask(&mut &frame(Kind::Draw, 1, &payload)[..]),
            Err(BadField)
        );
    }
    let replies = [
        frame(Kind::Pixels, 1, &pixel_payload(0, 1, RGBA_BYTES)),
        frame(Kind::Pixels, 1, &pixel_payload(1, 0, RGBA_BYTES)),
        frame(Kind::Pixels, 1, &pixel_payload(MAX_SIDE + 1, 1, RGBA_BYTES)),
        frame(Kind::Failed, 1, &[0]),
        frame(Kind::Failed, 1, &[u8::MAX]),
    ];
    for bytes in replies {
        assert_eq!(read_reply(&mut &bytes[..]), Err(BadField));
    }
}

/// A draw within `side` pixels.
fn draw_side(side: u32) -> Ask<'static> {
    Ask::Draw {
        id: 1,
        codec: Codec::Dib,
        side,
        bytes: Cow::Borrowed(b"x"),
    }
}

#[test]
fn a_writer_refuses_what_the_reader_would_and_writes_nothing() {
    let big = vec![0; MAX_IMAGE_BYTES + 1];
    let asks = [
        (draw(1, b""), Inconsistent),
        (draw(1, &big), TooLong),
        (draw_side(0), BadField),
        (draw_side(MAX_SIDE + 1), BadField),
    ];
    for (ask, err) in asks {
        let mut out = Vec::new();
        assert_eq!(ask.write(&mut out), Err(err));
        assert!(out.is_empty());
    }
    let wide = MAX_SIDE + 1;
    let replies = [
        (2, 2, 15, Inconsistent),
        (0, 1, 0, BadField),
        (wide, 1, wide as usize * RGBA_BYTES, BadField),
    ];
    for (width, height, len, err) in replies {
        let rgba = vec![0; len];
        let reply = Reply::Pixels {
            id: 1,
            width,
            height,
            rgba,
        };
        let mut out = Vec::new();
        assert_eq!(reply.write(&mut out), Err(err));
        assert!(out.is_empty());
    }
}

/// A stream whose every call fails with one error.
struct Broken(io::ErrorKind);

impl Read for Broken {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(self.0.into())
    }
}

impl io::Write for Broken {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(self.0.into())
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(self.0.into())
    }
}

#[test]
fn a_failing_stream_is_an_io_error() {
    let pipe = io::ErrorKind::BrokenPipe;
    assert_eq!(read_ask(&mut Broken(pipe)), Err(IpcError::Io(pipe)));
    assert_eq!(read_reply(&mut Broken(pipe)), Err(IpcError::Io(pipe)));
    assert_eq!(Ask::Bye.write(&mut Broken(pipe)), Err(IpcError::Io(pipe)));
    assert_eq!(
        pixels(1, 1, 1).write(&mut Broken(pipe)),
        Err(IpcError::Io(pipe))
    );
    let eof = io::Error::from(io::ErrorKind::UnexpectedEof);
    assert_eq!(IpcError::from(eof), IpcError::Short);
}

/// A reader that is interrupted before each byte it gives, as a signal may do.
struct Jumpy<'a> {
    rest: &'a [u8],
    interrupt: bool,
}

impl Read for Jumpy<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.interrupt = !self.interrupt;
        if self.interrupt {
            return Err(io::ErrorKind::Interrupted.into());
        }
        let n = buf.len().min(self.rest.len()).min(1);
        let (now, later) = self.rest.split_at(n);
        buf[..n].copy_from_slice(now);
        self.rest = later;
        Ok(n)
    }
}

#[test]
fn an_interrupted_or_partial_read_is_carried_on() {
    let ask = draw(8, b"slow");
    let bytes = ask_bytes(&ask);
    let mut r = Jumpy {
        rest: &bytes,
        interrupt: false,
    };
    assert_eq!(Ask::read(&mut r), Ok(ask));
}

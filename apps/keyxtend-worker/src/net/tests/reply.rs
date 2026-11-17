//! The reply reader: a body read whole within the cap, refused one byte past it without reading on, its breaks and
//! stalls told apart, JSON read into wiped text, and a wait's seconds and a renewal's span read strictly.

use super::*;
use crate::net::reply::{drain, seconds, span_s, text};
use kx_ipc::NetFault;
use kx_platform_net::{Body, NetError};
use std::sync::atomic::AtomicUsize;

/// A body of `left` bytes given at most `step` at a time, counting the bytes it hands out, then failing as `end`.
pub(super) struct Endless {
    pub left: usize,
    pub step: usize,
    pub given: Arc<AtomicUsize>,
    pub end: Option<NetError>,
}

impl Body for Endless {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, NetError> {
        let n = self.left.min(self.step).min(buf.len());
        if n == 0 {
            return self.end.map_or(Ok(0), Err);
        }
        buf[..n].fill(b'a');
        self.left -= n;
        self.given.fetch_add(n, Ordering::SeqCst);
        Ok(n)
    }
}

/// A body of `left` bytes in reads of `step`, and its count of bytes given.
pub(super) fn endless(left: usize, step: usize) -> (Endless, Arc<AtomicUsize>) {
    let given = Arc::new(AtomicUsize::new(0));
    let body = Endless {
        left,
        step,
        given: given.clone(),
        end: None,
    };
    (body, given)
}

#[test]
fn a_body_within_the_cap_is_read_whole() {
    let (mut body, _) = endless(10, 3);
    assert_eq!(drain(&mut body, 10).map(|b| b.to_vec()), Ok(vec![b'a'; 10]));
    let (mut body, _) = endless(0, 3);
    assert_eq!(drain(&mut body, 10).map(|b| b.len()), Ok(0));
}

#[test]
fn a_body_past_the_cap_is_strange_and_is_not_read_on() {
    let (mut body, given) = endless(usize::MAX, 4096);
    assert_eq!(drain(&mut body, 10_000).err(), Some(NetFault::Strange));
    assert_eq!(
        given.load(Ordering::SeqCst),
        10_001,
        "one byte past the cap, then it stops"
    );
    let (mut body, _) = endless(11, 11);
    assert_eq!(drain(&mut body, 10).err(), Some(NetFault::Strange));
}

#[test]
fn a_broken_or_quiet_body_is_told_apart() {
    let cases = [
        (NetError::Net, NetFault::Network),
        (NetError::Stall, NetFault::Timeout),
        (NetError::Internal, NetFault::Gone),
    ];
    for (e, fault) in cases {
        let (mut body, _) = endless(5, 2);
        body.end = Some(e);
        assert_eq!(drain(&mut body, 10).err(), Some(fault), "{e:?}");
    }
}

#[test]
fn the_text_is_read_from_json_with_its_escapes() {
    let read = |b: &[u8]| text(b).map(|t| t.to_string());
    let escaped = b"{\"text\":\"a\\\"b\\n\\u00e9\"}";
    assert_eq!(read(escaped), Some("a\"b\n\u{e9}".into()));
    assert_eq!(read(br#"{"words":[1],"text":"x"}"#), Some("x".into()));
    for odd in [&b"{}"[..], b"nope", br#"{"text":["x"]}"#, b"\xff\xfe"] {
        assert_eq!(read(odd), None, "{}", String::from_utf8_lossy(odd));
    }
}

#[test]
fn a_wait_reads_whole_seconds_and_a_span_reads_its_units() {
    assert_eq!(seconds("7"), Some(7));
    assert_eq!(seconds(" 7 "), Some(7));
    for odd in ["", "7.5", "+7", "-1", "4294967296", "7s"] {
        assert_eq!(seconds(odd), None, "{odd}");
    }
    let cases = [
        ("2m59.56s", 180),
        ("7.66s", 8),
        ("1h2m3s", 3723),
        ("250ms", 1),
        ("1.5h", 5400),
        ("0s", 0),
    ];
    for (span, s) in cases {
        assert_eq!(span_s(span), Some(s), "{span}");
    }
    for odd in [
        "",
        "5",
        "s",
        "1x",
        "1.2.3s",
        ".s",
        "1h1",
        "99999999999999999999h",
        "-1s",
    ] {
        assert_eq!(span_s(odd), None, "{odd}");
    }
}

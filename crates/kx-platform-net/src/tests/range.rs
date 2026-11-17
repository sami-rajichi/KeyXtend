//! The range a resume asks for, the `Content-Range` and `Content-Length` a server answers, and where the body starts.

use crate::range::{length, range_header, span, start};
use crate::{Head, NetError, Span};

/// A head with `status` and the given length and range headers.
fn head(status: u16, len: Option<&str>, range: Option<&str>) -> Head {
    Head {
        status,
        location: None,
        length: len.map(str::to_owned),
        range: range.map(str::to_owned),
        extra: Vec::new(),
    }
}

#[test]
fn a_resume_asks_for_the_rest_from_the_part_length() {
    assert_eq!(
        range_header(0),
        None,
        "a fresh download asks for the whole file"
    );
    assert_eq!(
        range_header(4096).as_deref(),
        Some("Range: bytes=4096-\r\n")
    );
}

#[test]
fn a_content_range_reads_as_its_first_and_last_bytes_and_the_total() {
    let want = |first, last, total| Some(Span { first, last, total });
    assert_eq!(span("bytes 4-9/10"), want(4, 9, Some(10)));
    assert_eq!(span("bytes 4-9/*"), want(4, 9, None));
    assert_eq!(span(" Bytes 0-0/1 "), want(0, 0, Some(1)));
    let big = u64::MAX - 1;
    assert_eq!(
        span(&format!("bytes 0-{big}/{}", u64::MAX)),
        want(0, big, Some(u64::MAX))
    );
    let top = u64::MAX;
    assert_eq!(span(&format!("bytes {top}-{top}/*")), want(top, top, None));
}

#[test]
fn a_content_range_that_is_not_a_byte_span_is_refused() {
    for text in [
        "bytes */10",
        "bytes 9-4/10",
        "bytes 4-10/10",
        "items 4-9/10",
        "bytes 4-9",
        "bytes -4-9/10",
        "bytes 4 - 9/10",
        "bytes 4-9/10/11",
        "bytes +4-9/10",
        "bytes 4-9/0",
        "bytes 0-1/99999999999999999999",
        "",
    ] {
        assert_eq!(span(text), None, "{text:?}");
    }
}

#[test]
fn a_content_length_is_digits_only() {
    assert_eq!(length(None), Ok(None));
    assert_eq!(length(Some("1234")), Ok(Some(1234)));
    assert_eq!(length(Some(&u64::MAX.to_string())), Ok(Some(u64::MAX)));
    assert_eq!(
        length(Some(" 68719476736 ")),
        Ok(Some(68_719_476_736)),
        "a 64 GiB model fits"
    );
    for text in ["", "12a", "-1", "+1", "1 2", "99999999999999999999"] {
        assert_eq!(length(Some(text)), Err(NetError::Server), "{text:?}");
    }
}

#[test]
fn a_whole_file_starts_at_zero_with_its_length_as_total() {
    assert_eq!(start(&head(200, Some("10"), None), 0), Ok((0, Some(10))));
    assert_eq!(
        start(&head(200, None, None), 0),
        Ok((0, None)),
        "a chunked answer"
    );
}

#[test]
fn a_whole_file_to_a_range_request_restarts_from_zero() {
    assert_eq!(start(&head(200, Some("10"), None), 4), Ok((0, Some(10))));
}

#[test]
fn a_partial_answer_must_start_where_asked() {
    let partial = |range| head(206, Some("6"), range);
    assert_eq!(start(&partial(Some("bytes 4-9/10")), 4), Ok((4, Some(10))));
    assert_eq!(start(&partial(Some("bytes 4-9/*")), 4), Ok((4, None)));
    assert_eq!(
        start(&partial(Some("bytes 0-9/10")), 4),
        Err(NetError::Range)
    );
    assert_eq!(
        start(&partial(Some("bytes 5-9/10")), 4),
        Err(NetError::Range)
    );
    assert_eq!(start(&partial(None), 4), Err(NetError::Range));
    assert_eq!(start(&partial(Some("bytes */10")), 4), Err(NetError::Range));
    assert_eq!(
        start(&partial(Some("bytes 0-9/10")), 0),
        Ok((0, Some(10))),
        "a part from 0 to a fresh request still starts where asked"
    );
}

#[test]
fn any_other_status_is_refused_with_its_code() {
    for code in [204, 304, 400, 403, 404, 416, 500, 503] {
        assert_eq!(
            start(&head(code, Some("10"), None), 0),
            Err(NetError::Status(code))
        );
    }
}

#[test]
fn a_bad_length_on_a_whole_file_is_a_server_fault() {
    assert_eq!(
        start(&head(200, Some("ten"), None), 0),
        Err(NetError::Server)
    );
}

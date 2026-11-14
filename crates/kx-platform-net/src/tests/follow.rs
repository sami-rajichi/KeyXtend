//! `Http::get` over the fake: resume, restart, redirects to listed HTTPS hosts only, the hop limit, stalls and errors.

use super::{HOSTS, drain};
use crate::fake::{Act, FakeHttp, Seen, Step};
use crate::{Get, Head, Http, MAX_HOPS, NetError, wait_ms};
use kx_ipc::{MAX_URL_BYTES, STALL_S as STALL_RANGE};
use std::borrow::Cow;

/// The file the tests download.
const FILE: &[u8] = b"0123456789";
/// The address the tests ask for first.
const URL: &str = "https://models.example/m.bin";
/// The stall time the tests ask with.
const STALL_S: u16 = 30;

/// A download of `URL` from byte `from`.
fn ask(from: u64) -> Get<'static> {
    Get {
        url: URL,
        hosts: &HOSTS,
        from,
        stall_s: STALL_S,
    }
}

/// What the fake should have been asked: `url` from byte `from`.
fn seen(url: &str, from: u64) -> Seen {
    Seen {
        url: url.to_owned(),
        from,
        stall_s: STALL_S,
    }
}

#[test]
fn a_whole_file_comes_from_the_first_address() {
    let http = FakeHttp::new([Act::whole(FILE)]);
    let mut got = http.get(&ask(0)).unwrap();
    assert_eq!((got.start, got.total), (0, Some(10)));
    assert_eq!(got.url.to_string(), URL);
    assert_eq!(drain(&mut got).unwrap(), FILE);
    assert_eq!(http.seen(), [seen(URL, 0)]);
}

#[test]
fn a_cut_download_resumes_from_the_part_length() {
    let cut = Act::Answer(
        Head::whole(FILE.len()),
        vec![Step::Bytes(FILE[..4].to_vec()), Step::Fail(NetError::Net)],
    );
    let http = FakeHttp::new([cut]);
    let mut first = http.get(&ask(0)).unwrap();
    assert_eq!(
        drain(&mut first),
        Err(NetError::Net),
        "the cut reaches the reader"
    );
    http.push(Act::part(&FILE[4..], 4, 10));
    let mut rest = http.get(&ask(4)).unwrap();
    assert_eq!((rest.start, rest.total), (4, Some(10)));
    assert_eq!(drain(&mut rest).unwrap(), &FILE[4..]);
    assert_eq!(http.seen(), [seen(URL, 0), seen(URL, 4)]);
}

#[test]
fn a_whole_file_to_a_range_request_restarts_from_zero() {
    let http = FakeHttp::new([Act::whole(FILE)]);
    let mut got = http.get(&ask(4)).unwrap();
    assert_eq!(
        (got.start, got.total),
        (0, Some(10)),
        "the caller empties its part"
    );
    assert_eq!(drain(&mut got).unwrap(), FILE);
}

#[test]
fn a_partial_answer_for_another_range_is_a_server_fault() {
    let http = FakeHttp::new([Act::part(&FILE[2..], 2, 10)]);
    assert_eq!(http.get(&ask(4)).err(), Some(NetError::Range));
}

#[test]
fn a_redirect_to_a_listed_host_is_followed() {
    let cdn = "https://cdn.example/blob/m.bin?sig=1";
    let http = FakeHttp::new([Act::moved(cdn), Act::whole(FILE)]);
    let got = http.get(&ask(0)).unwrap();
    assert_eq!(got.url.to_string(), cdn);
    assert_eq!(got.url.host(), "cdn.example");
    assert_eq!(http.seen(), [seen(URL, 0), seen(cdn, 0)]);
}

#[test]
fn every_redirect_status_is_followed_and_a_resume_keeps_its_range() {
    for code in [301, 302, 303, 307, 308] {
        let mut moved = Head::moved("/m2.bin");
        moved.status = code;
        let http = FakeHttp::new([Act::Answer(moved, Vec::new()), Act::part(&FILE[4..], 4, 10)]);
        assert!(http.get(&ask(4)).is_ok(), "{code}");
        let url2 = "https://models.example/m2.bin";
        assert_eq!(http.seen(), [seen(URL, 4), seen(url2, 4)], "{code}");
    }
}

#[test]
fn a_redirect_to_an_unlisted_host_is_refused_before_it_is_asked() {
    let http = FakeHttp::new([Act::moved("https://evil.example/m.bin"), Act::whole(FILE)]);
    assert_eq!(http.get(&ask(0)).err(), Some(NetError::Unlisted));
    assert_eq!(http.seen(), [seen(URL, 0)], "the other host is never asked");
}

#[test]
fn a_redirect_that_is_not_https_is_refused_before_it_is_asked() {
    let http = FakeHttp::new([Act::moved("http://cdn.example/m.bin"), Act::whole(FILE)]);
    assert_eq!(http.get(&ask(0)).err(), Some(NetError::NotHttps));
    assert_eq!(http.seen(), [seen(URL, 0)]);
}

#[test]
fn every_hop_of_a_chain_is_checked_before_it_is_asked() {
    let cdn = "https://cdn.example/1";
    let http = FakeHttp::new([
        Act::moved(cdn),
        Act::moved("//evil.example/2"),
        Act::whole(FILE),
    ]);
    assert_eq!(http.get(&ask(0)).err(), Some(NetError::Unlisted));
    assert_eq!(http.seen(), [seen(URL, 0), seen(cdn, 0)]);
}

#[test]
fn a_location_that_joins_past_the_cap_is_refused_before_it_is_asked() {
    let folder = &URL[..=URL.rfind('/').unwrap()];
    let long = "a".repeat(MAX_URL_BYTES + 1 - folder.len());
    assert!(long.len() <= MAX_URL_BYTES, "the Location alone fits");
    let http = FakeHttp::new([Act::moved(&long), Act::whole(FILE)]);
    assert_eq!(http.get(&ask(0)).err(), Some(NetError::NotHttps));
    assert_eq!(http.seen(), [seen(URL, 0)]);
}

#[test]
fn an_address_that_is_not_https_or_not_listed_asks_nothing() {
    let http = FakeHttp::new([Act::whole(FILE)]);
    let mut plain = ask(0);
    plain.url = "http://models.example/m.bin";
    assert_eq!(http.get(&plain).err(), Some(NetError::NotHttps));
    plain.url = "https://other.example/m.bin";
    assert_eq!(http.get(&plain).err(), Some(NetError::Unlisted));
    assert_eq!(http.seen(), []);
}

#[test]
fn a_redirect_without_a_location_is_a_server_fault() {
    let mut moved = Head::moved("");
    moved.location = None;
    let http = FakeHttp::new([Act::Answer(moved, Vec::new())]);
    assert_eq!(http.get(&ask(0)).err(), Some(NetError::Server));
}

#[test]
fn redirects_past_the_hop_limit_are_refused() {
    let hops = |n| {
        (0..n)
            .map(|_| Act::moved("/again"))
            .chain([Act::whole(FILE)])
    };
    let http = FakeHttp::new(hops(MAX_HOPS));
    assert!(http.get(&ask(0)).is_ok(), "the limit itself is followed");
    let http = FakeHttp::new(hops(MAX_HOPS + 1));
    assert_eq!(http.get(&ask(0)).err(), Some(NetError::Hops));
    assert_eq!(
        http.seen().len(),
        MAX_HOPS + 1,
        "the last redirect is never asked"
    );
}

#[test]
fn an_error_status_is_refused_with_its_code() {
    for code in [403, 404, 416, 500, 503] {
        let http = FakeHttp::new([Act::status(code)]);
        assert_eq!(http.get(&ask(0)).err(), Some(NetError::Status(code)));
    }
}

#[test]
fn a_failed_connection_comes_back_as_it_failed() {
    for e in [NetError::Net, NetError::Stall, NetError::Internal] {
        let http = FakeHttp::new([Act::Fail(e)]);
        assert_eq!(http.get(&ask(0)).err(), Some(e));
    }
    assert_eq!(
        FakeHttp::new([]).get(&ask(0)).err(),
        Some(NetError::Net),
        "no server"
    );
}

#[test]
fn a_quiet_body_past_the_stall_time_is_a_stall() {
    let wait = wait_ms(STALL_S);
    let quiet = |ms| {
        Act::Answer(
            Head::whole(2),
            vec![
                Step::Bytes(b"a".to_vec()),
                Step::Quiet(ms),
                Step::Bytes(b"b".to_vec()),
            ],
        )
    };
    let http = FakeHttp::new([quiet(wait - 1), quiet(wait)]);
    assert_eq!(
        drain(&mut http.get(&ask(0)).unwrap()).unwrap(),
        b"ab",
        "a shorter quiet goes on"
    );
    assert_eq!(drain(&mut http.get(&ask(0)).unwrap()), Err(NetError::Stall));
}

#[test]
fn the_stall_time_is_held_within_its_range_in_milliseconds() {
    let ms = |s: u16| u32::from(s) * 1000;
    assert_eq!(wait_ms(STALL_S), ms(STALL_S));
    assert_eq!(
        wait_ms(0),
        ms(*STALL_RANGE.start()),
        "0 would be an endless wait"
    );
    assert_eq!(wait_ms(u16::MAX), ms(*STALL_RANGE.end()));
}

#[test]
fn hosts_from_a_read_ask_are_taken_as_they_are() {
    let owned: Vec<Cow<'static, str>> = vec![Cow::Owned("models.example".to_owned())];
    let http = FakeHttp::new([Act::whole(FILE)]);
    let ask = Get {
        url: URL,
        hosts: &owned,
        from: 0,
        stall_s: STALL_S,
    };
    assert!(http.get(&ask).is_ok());
}

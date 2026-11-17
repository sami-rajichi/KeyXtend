//! Each net ask refusal, writing and reading: an address that is not plain HTTPS, a name, value or key that could
//! break a request, a wait, try or limit outside its range, piece sizes past their caps, bad sound, extra bytes and a
//! goodbye with an id.

use super::*;
use crate::{MAX_RECORD_BYTES, NET_MAX_MB, piece_bytes};
use IpcError::{BadField, Inconsistent, TooLong};

/// One change to a field of a good value.
type Change<T> = fn(&mut T);

/// A call whose every name is one letter: a form with one fixed field, to `https://a/t`.
fn small_call() -> NetCall<'static> {
    NetCall {
        shape: form("l", false, vec![field("n", "v")]),
        url: Cow::Borrowed("https://a/t"),
        header: NetHeader::Bearer,
        key: Wiped::Lent(b"K"),
        wait_s: 60,
    }
}

/// A run of one two-byte piece for model `m`.
fn small_run() -> NetRun<'static> {
    NetRun {
        model: Some(Cow::Borrowed("m")),
        ..run(vec![2])
    }
}

/// A small send, whose payload offsets the read refusals name.
fn small() -> NetAsk<'static> {
    NetAsk::Send {
        id: 1,
        call: small_call(),
        run: small_run(),
    }
}

/// Checks that writing `ask` is refused with `err` and writes nothing.
fn refused(ask: &NetAsk, err: IpcError) {
    let mut out = Vec::new();
    assert_eq!(ask.write(&mut out), Err(err), "{ask:?}");
    assert!(out.is_empty());
}

/// Checks that each `(at, value)` patch of `good` is refused with `err` when read.
fn refused_reads(good: &[u8], reads: &[(usize, &[u8])], err: IpcError) {
    for &(at, value) in reads {
        let bytes = patch(good.to_vec(), at, value);
        assert_eq!(read_net_ask(&mut &bytes[..]), Err(err), "{at} {value:?}");
    }
}

/// The largest piece Groq's limits take, in bytes.
fn groq_cap() -> u32 {
    let limits = limits();
    u32::try_from(piece_bytes(limits.max_mb, limits.max_s)).unwrap()
}

#[test]
fn the_small_send_has_the_payload_its_offsets_name() {
    let good = net_ask_bytes(&small());
    assert_eq!(good.len(), HEADER_LEN + 50);
    assert_eq!(NetAsk::read(&mut &good[..]), Ok(small()));
}

#[test]
fn an_address_that_is_not_plain_https_is_refused_both_ways() {
    let bad = [
        "http://a/t",
        "HTTPS://a/t",
        "https://A/t",
        "https://a@b/t",
        "https://a/t#x",
        "https://a/t\r\nX-Key: 1",
        "https://a /t",
        "https://.a/t",
        "https://a..b/t",
        "https://a:0/t",
        "https://a:99999/t",
        "https://a:+1/t",
        "https://a:/t",
        "https://[::1]/t",
        "https://",
        "https:///t",
        "https://مثال/t",
    ];
    for url in bad {
        let mut call = small_call();
        call.url = Cow::Borrowed(url);
        refused(&NetAsk::Test { id: 1, call }, BadField);
    }
    let mut call = small_call();
    call.url = Cow::Owned(format!("{HTTPS}a/{}", "p".repeat(MAX_URL_BYTES)));
    refused(&NetAsk::Test { id: 1, call }, TooLong);
    let good = net_ask_bytes(&small());
    refused_reads(
        &good,
        &[(11, b"H"), (19, b"A"), (19, b"@"), (20, b"#"), (21, b"\r")],
        BadField,
    );
}

/// The key alone under the header `name`.
fn named(name: &'static str) -> NetHeader<'static> {
    NetHeader::Named(Cow::Borrowed(name))
}

/// A form with one fixed field.
fn one_field(name: &'static str, value: &'static str) -> NetShape<'static> {
    form("l", false, vec![field(name, value)])
}

/// Checks that each change to a small call is refused with its error, in a test and in a send.
fn refused_calls(changes: &[(Change<NetCall>, IpcError)]) {
    for &(change, err) in changes {
        let mut call = small_call();
        change(&mut call);
        let test = NetAsk::Test {
            id: 1,
            call: call.clone(),
        };
        refused(&test, err);
        let run = small_run();
        refused(&NetAsk::Send { id: 1, call, run }, err);
    }
}

#[test]
fn a_name_or_value_that_could_break_a_request_is_refused_when_written() {
    refused_calls(&[
        (|c| c.header = named("x-api key"), BadField),
        (|c| c.header = named("x\r\nkey"), BadField),
        (|c| c.header = named("clé"), BadField),
        (|c| c.header = named(""), Inconsistent),
        (|c| c.shape = form("lang uage", false, Vec::new()), BadField),
        (|c| c.shape = form("", false, Vec::new()), Inconsistent),
        (|c| c.shape = one_field("a.b", "1"), BadField),
        (|c| c.shape = one_field("a", "1\r\n2"), BadField),
        (|c| c.shape = one_field("a", "1\t2"), BadField),
        (
            |c| c.shape = form("l", false, vec![field("a", "1"); MAX_FIXED + 1]),
            TooLong,
        ),
    ]);
    let long = NetField {
        name: Cow::Borrowed("a"),
        value: Cow::Owned("v".repeat(MAX_NAME_BYTES + 1)),
    };
    let call = NetCall {
        shape: form("l", false, vec![long]),
        ..small_call()
    };
    refused(&NetAsk::Test { id: 1, call }, TooLong);
}

#[test]
fn a_key_that_could_break_a_request_or_a_wait_out_of_range_is_refused_when_written() {
    refused_calls(&[
        (|c| c.key = Wiped::Lent(b""), Inconsistent),
        (|c| c.key = Wiped::Lent(b"a b"), BadField),
        (|c| c.key = Wiped::Lent(b"key\r\nX: 1"), BadField),
        (|c| c.key = Wiped::Lent("clé".as_bytes()), BadField),
        (|c| c.wait_s = *NET_WAIT_S.start() - 1, BadField),
        (|c| c.wait_s = *NET_WAIT_S.end() + 1, BadField),
    ]);
    let call = NetCall {
        key: Wiped::Kept(Zeroizing::new(vec![b'k'; MAX_KEY_BYTES + 1])),
        ..small_call()
    };
    refused(&NetAsk::Test { id: 1, call }, TooLong);
}

#[test]
fn a_run_value_outside_its_range_is_refused_when_written() {
    let changes: [(Change<NetRun>, IpcError); 21] = [
        (|r| r.model = Some(Cow::Borrowed("")), Inconsistent),
        (|r| r.model = Some(Cow::Borrowed("a b")), BadField),
        (|r| r.lang = Cow::Borrowed(""), Inconsistent),
        (|r| r.lang = Cow::Borrowed("fr\n"), BadField),
        (|r| r.limits.max_mb = *NET_MAX_MB.start() - 1, BadField),
        (|r| r.limits.max_mb = *NET_MAX_MB.end() + 1, BadField),
        (|r| r.limits.max_s = *NET_MAX_S.start() - 1, BadField),
        (|r| r.limits.max_s = *NET_MAX_S.end() + 1, BadField),
        (|r| r.limits.per_min = *NET_PER_MIN.start() - 1, BadField),
        (|r| r.limits.per_min = *NET_PER_MIN.end() + 1, BadField),
        (|r| r.limits.at_once = *NET_AT_ONCE.start() - 1, BadField),
        (|r| r.limits.at_once = *NET_AT_ONCE.end() + 1, BadField),
        (|r| r.tries = *NET_TRIES.start() - 1, BadField),
        (|r| r.tries = *NET_TRIES.end() + 1, BadField),
        (|r| r.sizes = Vec::new(), Inconsistent),
        (|r| r.sizes = vec![2; MAX_PIECES + 1], TooLong),
        (|r| r.sizes = vec![2, 0], BadField),
        (|r| r.sizes = vec![3], BadField),
        (|r| r.sizes = vec![groq_cap() + 2], BadField),
        (|r| r.sizes = vec![2, u32::MAX - 1], BadField),
        (
            |r| {
                r.limits.max_mb = *NET_MAX_MB.end();
                let half = u32::try_from(MAX_RECORD_BYTES / 2 + SAMPLE_BYTES).unwrap();
                r.sizes = vec![half, half];
            },
            BadField,
        ),
    ];
    for (change, err) in changes {
        let mut run = small_run();
        change(&mut run);
        let call = small_call();
        refused(&NetAsk::Send { id: 1, call, run }, err);
    }
}

#[test]
fn a_piece_at_its_caps_passes() {
    let cap = groq_cap();
    let at_cap = NetAsk::Send {
        id: 1,
        call: small_call(),
        run: run(vec![cap, 2]),
    };
    assert_eq!(NetAsk::read(&mut &net_ask_bytes(&at_cap)[..]), Ok(at_cap));
}

#[test]
fn a_bad_call_or_run_is_refused_when_read() {
    let good = net_ask_bytes(&small());
    let over = (groq_cap() + 2).to_le_bytes();
    let reads: [(usize, &[u8]); 24] = [
        (0, &[0]),
        (0, &[5]),
        (2, b" "),
        (3, &[2]),
        (6, b"."),
        (8, b"\n"),
        (22, &[0]),
        (22, &[3]),
        (25, b" "),
        (25, b"\x7f"),
        (26, &(*NET_WAIT_S.start() - 1).to_le_bytes()),
        (28, &[2]),
        (30, b" "),
        (32, b"\0"),
        (34, &0u32.to_le_bytes()),
        (38, &(*NET_MAX_S.start() - 1).to_le_bytes()),
        (40, &0u16.to_le_bytes()),
        (42, &[*NET_AT_ONCE.end() + 1]),
        (43, &[0]),
        (46, &0u32.to_le_bytes()),
        (46, &3u32.to_le_bytes()),
        (46, &over),
        (1, &[0]),
        (44, &0u16.to_le_bytes()),
    ];
    let (bad, empty) = reads.split_at(reads.len() - 2);
    refused_reads(&good, bad, BadField);
    refused_reads(&good, empty, Inconsistent);
    let fixed = [u8::try_from(MAX_FIXED + 1).unwrap()];
    let pieces = u16::try_from(MAX_PIECES + 1).unwrap().to_le_bytes();
    refused_reads(&good, &[(4, &fixed), (44, &pieces)], TooLong);
}

#[test]
fn sound_that_is_odd_empty_too_long_or_past_the_last_piece_is_refused_both_ways() {
    let past = u16::try_from(MAX_PIECES).unwrap();
    let sound = |piece, bytes| NetAsk::Sound {
        id: 1,
        piece,
        bytes,
    };
    refused(&sound(past, Wiped::Lent(b"ab")), BadField);
    refused(&sound(0, Wiped::Lent(b"odd")), BadField);
    refused(&sound(0, Wiped::Lent(b"")), Inconsistent);
    let long = Wiped::Kept(Zeroizing::new(vec![0; MAX_SOUND_BYTES + SAMPLE_BYTES]));
    refused(&sound(0, long), TooLong);
    let good = net_ask_bytes(&sound(0, Wiped::Lent(b"ab")));
    refused_reads(&good, &[(0, &past.to_le_bytes())], BadField);
    let mut odd = patch(good, 2, &[1]);
    odd.truncate(odd.len() - 1);
    odd[HEADER_LEN - 4] -= 1;
    assert_eq!(read_net_ask(&mut &odd[..]), Err(BadField));
}

#[test]
fn bytes_after_the_last_field_or_a_goodbye_with_an_id_are_inconsistent() {
    let mut payload = net_ask_bytes(&small())[HEADER_LEN..].to_vec();
    payload.push(0);
    let bytes = frame(Kind::NetSend, 1, &payload);
    assert_eq!(read_net_ask(&mut &bytes[..]), Err(Inconsistent));
    let bye = frame(Kind::NetBye, 5, &[]);
    assert_eq!(read_net_ask(&mut &bye[..]), Err(Inconsistent));
}

//! Each fetch refusal, reading and writing: a length past its kind, an address that is not HTTPS, a host that is not a
//! plain lower-case name, an empty or long list, a size, stall or progress out of range, and a bad path. The host and
//! tag rules are public, so the ports check the same way.

use super::*;
use crate::{MAX_MODEL_BYTES, STALL_S};
use IpcError::{BadField, Inconsistent, TooLong};

/// A download of `size` bytes from `urls`, allowed on `hosts`, into `path`, cut after `stall_s`.
fn ask<'a>(
    urls: &[&'a str],
    hosts: &[&'a str],
    path: &'a str,
    size: u64,
    stall_s: u16,
) -> FetchAsk<'a> {
    FetchAsk::Get {
        id: 1,
        urls: urls.iter().map(|&u| Cow::Borrowed(u)).collect(),
        hosts: hosts.iter().map(|&h| Cow::Borrowed(h)).collect(),
        path: Cow::Borrowed(path),
        size,
        sha256: SHA,
        stall_s,
    }
}

/// A small download with these addresses, hosts and path.
fn with<'a>(urls: &[&'a str], hosts: &[&'a str], path: &'a str) -> FetchAsk<'a> {
    ask(urls, hosts, path, 7, 30)
}

/// A small download of `size` bytes, cut after `stall_s`.
fn sized(size: u64, stall_s: u16) -> FetchAsk<'static> {
    ask(&["https://a/m"], &["a"], "p", size, stall_s)
}

/// Checks that writing `ask` is refused with `err` and writes nothing.
fn refused(ask: &FetchAsk, err: IpcError) {
    let mut out = Vec::new();
    assert_eq!(ask.write(&mut out), Err(err), "{ask:?}");
    assert!(out.is_empty());
}

#[test]
fn a_length_past_its_kind_is_refused_before_the_payload() {
    let plenty = u64::try_from(READ_CHUNK).unwrap();
    for &kind in fetch_ask::KINDS.iter().chain(fetch_note::KINDS) {
        let over = u32::try_from(*kind.lens().end() + 1).unwrap();
        let read: Reader = if fetch_ask::KINDS.contains(&kind) {
            read_fetch_ask
        } else {
            read_fetch_note
        };
        for len in [over, u32::MAX] {
            let raw = head(MAGIC, VERSION, kind.code(), 0, len);
            let mut r = Counted::new((&raw[..]).chain(io::repeat(0).take(plenty)));
            assert_eq!(read(&mut r), Err(TooLong), "{kind:?} {len}");
            assert_eq!(r.taken, HEADER_LEN);
        }
    }
}

#[test]
fn a_wrong_version_or_a_length_off_by_one_is_refused_for_every_fetch_frame() {
    let asks = every_fetch_ask();
    let asks = asks
        .iter()
        .map(|a| (fetch_ask_bytes(a), read_fetch_ask as Reader));
    let notes = every_fetch_note();
    let notes = notes
        .iter()
        .map(|n| (fetch_note_bytes(n), read_fetch_note as Reader));
    for (bytes, read) in asks.chain(notes) {
        let mut old = bytes.clone();
        old[MAGIC.len()] = VERSION + 1;
        assert_eq!(read(&mut &old[..]), Err(IpcError::BadVersion));
        let mut long = bytes.clone();
        long[HEADER_LEN - 4] = long[HEADER_LEN - 4].wrapping_add(1);
        long.push(0);
        let got = read(&mut &long[..]);
        assert!(matches!(got, Err(Inconsistent | TooLong)), "{got:?}");
        let mut short = bytes.clone();
        short[HEADER_LEN - 4] = short[HEADER_LEN - 4].wrapping_sub(1);
        assert!(read(&mut &short[..]).is_err());
    }
}

#[test]
fn an_address_that_is_not_https_or_holds_a_space_is_refused_both_ways() {
    let h = &["a"][..];
    for url in [
        "http://a/m",
        "HTTPS://a/m",
        "https://a/m x",
        "ftp://a",
        "https://a/é",
    ] {
        refused(&with(&[url], h, "p"), BadField);
    }
    let good = fetch_ask_bytes(&with(&["https://a/m"], h, "p"));
    for (at, value) in [(3, b'H'), (7, b':'), (11, b' ')] {
        let mut bytes = good.clone();
        bytes[HEADER_LEN + at] = value;
        assert_eq!(read_fetch_ask(&mut &bytes[..]), Err(BadField), "{at}");
    }
}

#[test]
fn a_host_that_is_not_a_plain_lower_case_name_is_refused_both_ways() {
    let u = &["https://a/m"][..];
    for host in ["A.example", "a/b", "a b", "a:443", "é.example", "a_b"] {
        refused(&with(u, &[host], "p"), BadField);
    }
    let good = fetch_ask_bytes(&with(u, &["ab"], "p"));
    let at = HEADER_LEN + 1 + 2 + 11 + 1 + 1;
    let mut bytes = good.clone();
    bytes[at] = b'/';
    assert_eq!(read_fetch_ask(&mut &bytes[..]), Err(BadField));
}

#[test]
fn the_public_host_and_tag_rules_are_the_frames_own() {
    use crate::rules::{host, tag};
    for good in ["huggingface.co", "cdn-lfs.hf.co", "a1"] {
        assert_eq!(host(good), Ok(()), "{good}");
    }
    for bad in ["A.example", "a/b", "a b", "a:443", "é.example", "a_b"] {
        assert_eq!(host(bad), Err(BadField), "{bad}");
    }
    for good in ["fr-FR", "whisper", "Vulkan0"] {
        assert_eq!(tag(good), Ok(()), "{good}");
    }
    for bad in ["fr FR", "مر", "a\tb", "a\0"] {
        assert_eq!(tag(bad), Err(BadField), "{bad}");
    }
}

#[test]
fn an_empty_or_long_list_or_name_is_refused() {
    let (u, h) = (&["https://a/m"][..], &["a"][..]);
    refused(&with(&[], h, "p"), Inconsistent);
    refused(&with(u, &[], "p"), Inconsistent);
    refused(&with(u, h, ""), Inconsistent);
    refused(&with(&[""], h, "p"), Inconsistent);
    let many: Vec<&str> = vec!["https://a/m"; crate::MAX_URLS + 1];
    refused(&with(&many, h, "p"), TooLong);
    let many: Vec<&str> = vec!["a"; crate::MAX_HOSTS + 1];
    refused(&with(u, &many, "p"), TooLong);
    let path = "p".repeat(MAX_PATH_BYTES - PART_EXT.len() + 1);
    refused(&with(u, h, &path), TooLong);
    refused(&with(u, h, "a\0b"), BadField);
}

#[test]
fn a_size_or_stall_out_of_range_is_refused_both_ways() {
    let cases = [
        sized(0, 30),
        sized(MAX_MODEL_BYTES + 1, 30),
        sized(1, *STALL_S.start() - 1),
        sized(1, *STALL_S.end() + 1),
    ];
    for ask in cases {
        refused(&ask, BadField);
    }
    let good = fetch_ask_bytes(&with(&["https://a/m"], &["a"], "p"));
    let size_at = good.len() - 2 - SHA256_BYTES - 8;
    let stall_at = good.len() - 2;
    let reads = [
        (size_at, &0u64.to_le_bytes()[..]),
        (size_at, &(MAX_MODEL_BYTES + 1).to_le_bytes()),
        (stall_at, &(*STALL_S.end() + 1).to_le_bytes()),
    ];
    for (at, value) in reads {
        let mut bytes = good.clone();
        bytes[at..at + value.len()].copy_from_slice(value);
        assert_eq!(read_fetch_ask(&mut &bytes[..]), Err(BadField), "{at}");
    }
}

#[test]
fn progress_past_its_total_or_a_total_outside_a_models_size_is_refused_both_ways() {
    let cases = [(2, 1), (0, 0), (0, MAX_MODEL_BYTES + 1)];
    for (got, total) in cases {
        let note = FetchNote::Progress { id: 1, got, total };
        let mut out = Vec::new();
        assert_eq!(note.write(&mut out), Err(BadField));
        assert!(out.is_empty());
        let payload = [got.to_le_bytes(), total.to_le_bytes()].concat();
        let bytes = frame(Kind::Progress, 1, &payload);
        assert_eq!(read_fetch_note(&mut &bytes[..]), Err(BadField));
    }
}

#[test]
fn a_fault_out_of_range_or_a_goodbye_with_an_id_is_refused() {
    let next = u8::try_from(FetchFault::ALL.len() + 1).unwrap();
    for byte in [0, next, u8::MAX] {
        let bytes = frame(Kind::FetchFailed, 1, &[byte]);
        assert_eq!(read_fetch_note(&mut &bytes[..]), Err(BadField));
    }
    let bye = frame(Kind::FetchBye, 5, &[]);
    assert_eq!(read_fetch_ask(&mut &bye[..]), Err(Inconsistent));
}

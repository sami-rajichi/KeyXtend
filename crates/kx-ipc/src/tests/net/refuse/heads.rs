//! A key's header that names one the request already sets is refused, writing and reading, in any case; only
//! `Authorization` may carry a key without `Bearer`, as `AssemblyAI`'s does.

use super::*;

/// Headers each request sets itself, in odd cases.
const TAKEN: [&str; 5] = [
    "content-TYPE",
    "Content-Length",
    "HOST",
    "user-agent",
    "Connection",
];

/// A test ask whose key goes under `name`.
fn under(name: &str) -> NetAsk<'static> {
    let call = NetCall {
        header: NetHeader::Named(Cow::Owned(name.to_owned())),
        ..small_call()
    };
    NetAsk::Test { id: 1, call }
}

#[test]
fn a_key_header_the_request_sets_is_refused_when_written() {
    for name in TAKEN {
        refused(&under(name), BadField);
    }
    let raw = under("authorization");
    assert_eq!(NetAsk::read(&mut &net_ask_bytes(&raw)[..]), Ok(raw));
}

#[test]
fn a_key_header_the_request_sets_is_refused_when_read() {
    for name in TAKEN {
        let free = "x".repeat(name.len());
        let good = net_ask_bytes(&under(&free));
        let at = good.windows(free.len()).position(|w| w == free.as_bytes());
        let at = at.unwrap() - HEADER_LEN;
        refused_reads(&good, &[(at, name.as_bytes())], BadField);
    }
}

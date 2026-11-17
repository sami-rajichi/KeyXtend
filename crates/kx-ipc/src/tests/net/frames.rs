//! The refusals every net frame shares: a length past its kind, read no further than the header, and a wrong version
//! or a length off by one.

use super::*;
use IpcError::{BadVersion, Inconsistent, TooLong};

#[test]
fn a_length_past_its_kind_is_refused_before_the_payload() {
    let plenty = u64::try_from(READ_CHUNK).unwrap();
    for &kind in net_ask::KINDS.iter().chain(net_note::KINDS) {
        let over = u32::try_from(*kind.lens().end() + 1).unwrap();
        let read: Reader = if net_ask::KINDS.contains(&kind) {
            read_net_ask
        } else {
            read_net_note
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
fn a_wrong_version_or_a_length_off_by_one_is_refused_for_every_net_frame() {
    let asks = every_net_ask();
    let asks = asks
        .iter()
        .map(|a| (net_ask_bytes(a), read_net_ask as Reader));
    let notes = every_net_note();
    let notes = notes
        .iter()
        .map(|n| (net_note_bytes(n), read_net_note as Reader));
    for (bytes, read) in asks.chain(notes) {
        let mut old = bytes.clone();
        old[MAGIC.len()] = VERSION + 1;
        assert_eq!(read(&mut &old[..]), Err(BadVersion));
        let mut long = bytes.clone();
        long[HEADER_LEN - 4] = long[HEADER_LEN - 4].wrapping_add(1);
        long.push(0);
        let got = read(&mut &long[..]);
        assert!(matches!(got, Err(Inconsistent | TooLong)), "{got:?}");
        let mut short = bytes;
        short[HEADER_LEN - 4] = short[HEADER_LEN - 4].wrapping_sub(1);
        assert!(read(&mut &short[..]).is_err());
    }
}

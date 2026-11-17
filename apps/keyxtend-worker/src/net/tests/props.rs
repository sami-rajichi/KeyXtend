//! Properties: a random reply never panics the readers, a random JSON error with a used allowance always tells a
//! renewal in range, a body past any cap is always refused one byte past it, a run takes exactly the parts that fill
//! its pieces in order, and texts are told in the pieces' order whatever order they come in.

use super::reply::{NEVER, endless};
use super::*;
use crate::net::form::Form;
use crate::net::reply::{drain, seconds, span_s, text};
use crate::net::run::Run;
use crate::net::shape::{Shape, Verdict};
use kx_ipc::{NET_RENEW_S, NetFault, NetNote};
use proptest::prelude::*;

/// What a model of the run expects of a part: the pieces' cursor and the bytes in the current one.
fn accepts(sizes: &[u32], cursor: &mut (usize, usize), piece: u16, n: usize) -> bool {
    let (at, got) = *cursor;
    let fits = usize::from(piece) == at && sizes.get(at).is_some_and(|&s| got + n <= s as usize);
    if fits {
        *cursor = if got + n == sizes[at] as usize {
            (at + 1, 0)
        } else {
            (at, got + n)
        };
    }
    fits
}

/// A value an error's field may hold: nothing, a number or any text.
fn loose() -> impl Strategy<Value = serde_json::Value> {
    prop_oneof![
        Just(serde_json::Value::Null),
        any::<i64>().prop_map(serde_json::Value::from),
        ".{0,32}".prop_map(serde_json::Value::from),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn a_random_reply_never_panics_the_readers(
        status in any::<u16>(),
        body in proptest::collection::vec(any::<u8>(), 0..512),
        value in ".{0,24}",
    ) {
        let head = head(status, &[("retry-after", value.as_str()), ("x-ratelimit-reset-requests", value.as_str())]);
        let _ = Form.read(&head, &body);
        let _ = Form.tested(&head, &body);
        let _ = text(&body);
        let _ = (seconds(&value), span_s(&value));
    }

    #[test]
    fn a_random_json_error_never_panics_and_a_used_allowance_renews_in_range(
        status in 400u16..=599,
        (message, kind, code) in (loose(), loose(), loose()),
        reset in prop_oneof![".{0,16}", r"([0-9]{1,6}(\.[0-9]{1,3})?(ms|h|m|s)){1,3}"],
    ) {
        let body = serde_json::json!({"error": {"message": message, "type": kind, "code": code}}).to_string();
        let used = [("x-ratelimit-remaining-requests", "0"), ("x-ratelimit-reset-requests", reset.as_str())];
        let head = head(status, &used);
        let _ = Form.tested(&head, body.as_bytes());
        match Form.read(&head, body.as_bytes()) {
            Verdict::Fault(NetFault::Allowance { renews_s }) => {
                prop_assert!(renews_s.is_none_or(|s| NET_RENEW_S.contains(&s)), "{renews_s:?}");
            }
            Verdict::Fault(NetFault::Credit) => {}
            _ => prop_assert!(status != BUSY, "a used allowance is told"),
        }
    }

    #[test]
    fn a_body_past_any_cap_is_refused_one_byte_past_it(
        cap in 1usize..=65_536,
        extra in 1usize..=65_536,
        step in 1usize..=8_192,
    ) {
        let (mut body, given) = endless(cap + extra, step);
        prop_assert_eq!(drain(&mut body, cap, &NEVER).err(), Some(NetFault::Strange));
        prop_assert_eq!(given.load(Ordering::SeqCst), cap + 1);
        let (mut body, _) = endless(cap, step);
        prop_assert_eq!(drain(&mut body, cap, &NEVER).map(|b| b.len()), Ok(cap));
    }

    #[test]
    fn a_run_takes_exactly_the_parts_that_fill_its_pieces_in_order(
        sizes in proptest::collection::vec((1u32..=8).prop_map(|n| n * 2), 1..5),
        parts in proptest::collection::vec((0u16..6, (1usize..=10).prop_map(|n| n * 2)), 0..24),
    ) {
        let mut run = Run::new(1, call(), run_of(&sizes, 1, 1, 20)).unwrap();
        let mut cursor = (0, 0);
        for (piece, n) in parts {
            let want = accepts(&sizes, &mut cursor, piece, n);
            let got = run.sound(piece, &sound(n, 1));
            prop_assert_eq!(got.is_ok(), want);
            if !want {
                prop_assert_eq!(got, Err(NetFault::Gone));
                break;
            }
        }
        prop_assert_eq!(run.waiting(), cursor.0);
    }

    #[test]
    fn texts_are_told_in_the_pieces_order_whatever_order_they_come_in(
        order in Just((0u16..6).collect::<Vec<_>>()).prop_shuffle(),
    ) {
        let mut run = Run::new(1, call(), run_of(&[2; 6], 1, 6, 20)).unwrap();
        let mut told = Vec::new();
        for index in order {
            let (notes, _) = run.told(1, index, Ok(Zeroizing::new(index.to_string())));
            told.extend(notes);
        }
        let mut want: Vec<NetNote> = (0u16..6).map(|i| super::bench::text(1, i, &i.to_string())).collect();
        want.push(NetNote::Done { id: 1 });
        prop_assert_eq!(told, want);
    }
}

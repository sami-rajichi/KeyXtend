//! Each net note refusal, writing and reading: a text over its cap, past the last piece or not UTF-8, a fault code,
//! flag or renewal out of range, bytes a fault does not hold, and the account's step as its own code.

use super::*;
use crate::code::Code;
use crate::net::code::FaultCode;
use IpcError::{BadField, Inconsistent, TooLong};

/// Checks that writing `note` is refused with `err` and writes nothing, as its `check` says.
fn refused_note(note: &NetNote, err: IpcError) {
    let mut out = Vec::new();
    assert_eq!(note.write(&mut out), Err(err), "{note:?}");
    assert!(out.is_empty());
    assert_eq!(note.check(), Err(err));
}

/// Checks that a fault note holding `payload` is refused with `err` when read.
fn refused_fault(payload: &[u8], err: IpcError) {
    let bytes = frame(Kind::NetFailed, 1, payload);
    assert_eq!(read_net_note(&mut &bytes[..]), Err(err), "{payload:?}");
}

#[test]
fn an_oversized_reply_is_refused_both_ways() {
    let long = text(&"x".repeat(MAX_NET_TEXT_BYTES + 1));
    let note = NetNote::Text {
        id: 1,
        index: 0,
        text: long,
    };
    refused_note(&note, TooLong);
    let count = u32::try_from(MAX_NET_TEXT_BYTES + 1).unwrap().to_le_bytes();
    let bytes = frame(Kind::NetText, 1, &[&[0, 0][..], &count].concat());
    let mut r = Counted::new(&bytes[..]);
    assert_eq!(NetNote::read(&mut r), Err(TooLong));
    assert_eq!(r.taken, bytes.len(), "refused before any text is read");
}

#[test]
fn a_text_past_the_last_piece_or_not_utf8_is_refused() {
    let past = u16::try_from(MAX_SPEECH_PIECES).unwrap();
    let note = NetNote::Text {
        id: 1,
        index: past,
        text: text("x"),
    };
    refused_note(&note, BadField);
    let index = [&past.to_le_bytes()[..], b"\x01\0\0\0x"].concat();
    let bytes = frame(Kind::NetText, 1, &index);
    assert_eq!(read_net_note(&mut &bytes[..]), Err(BadField));
    let bad = frame(Kind::NetText, 1, b"\0\0\x02\0\0\0\xff\xfe");
    assert_eq!(read_net_note(&mut &bad[..]), Err(BadField));
}

#[test]
fn a_fault_code_flag_or_renewal_out_of_range_is_refused_both_ways() {
    let next = u8::try_from(FaultCode::ALL.len() + 1).unwrap();
    let allowance = FaultCode::Allowance.code();
    let renewal = |s: u32| [&[allowance, 1][..], &s.to_le_bytes()].concat();
    refused_fault(&[0], BadField);
    refused_fault(&[next], BadField);
    refused_fault(&[allowance, 2], BadField);
    refused_fault(&renewal(*NET_RENEW_S.start() - 1), BadField);
    refused_fault(&renewal(*NET_RENEW_S.end() + 1), BadField);
    for renews_s in [*NET_RENEW_S.start() - 1, *NET_RENEW_S.end() + 1] {
        let fault = NetFault::Allowance {
            renews_s: Some(renews_s),
        };
        refused_note(&NetNote::Fault { id: 1, fault }, BadField);
    }
}

#[test]
fn bytes_a_fault_does_not_hold_are_inconsistent() {
    let key = FaultCode::Key.code();
    refused_fault(&[key, 0], Inconsistent);
    refused_fault(&[FaultCode::Allowance.code()], Inconsistent);
    let told = [FaultCode::Allowance.code(), 0, 0];
    refused_fault(&told, Inconsistent);
    let done = frame(Kind::NetDone, 1, &[0]);
    assert_eq!(read_net_note(&mut &done[..]), Err(TooLong));
}

#[test]
fn a_step_on_the_services_site_is_its_own_fault_after_the_region_and_reads_back_whole() {
    assert_eq!(FaultCode::Account.code(), FaultCode::Region.code() + 1);
    assert_eq!(NetFault::Account.code(), FaultCode::Account);
    let note = NetNote::Fault {
        id: 1,
        fault: NetFault::Account,
    };
    let bytes = net_note_bytes(&note);
    assert_eq!(NetNote::read(&mut &bytes[..]), Ok(note));
    refused_fault(&[FaultCode::Account.code(), 0], Inconsistent);
}

#[test]
fn every_fault_has_its_own_code_and_the_allowance_alone_tells_a_renewal() {
    let faults = every_fault();
    assert_eq!(faults.len(), FaultCode::ALL.len());
    let names: std::collections::HashSet<String> =
        faults.iter().map(|f| format!("{f:?}")).collect();
    assert_eq!(names.len(), faults.len(), "{names:?}");
    let told = NetFault::of(FaultCode::Allowance, Some(60));
    assert_eq!(told, NetFault::Allowance { renews_s: Some(60) });
    assert_eq!(NetFault::of(FaultCode::Key, Some(60)), NetFault::Key);
}

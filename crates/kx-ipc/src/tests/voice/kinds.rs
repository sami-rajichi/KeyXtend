//! Every kind belongs to one reader: the codes run without a gap, each kind has its lengths, and each voice frame is
//! refused by every other job's reader and by the other side's.

use super::*;
use crate::{FetchAsk, FetchNote, VaultAsk, VaultReply, ask, reply};

#[test]
fn every_kind_belongs_to_exactly_one_reader_and_the_codes_have_no_gap() {
    let lists: [&[Kind]; 8] = [
        ask::KINDS,
        reply::KINDS,
        crate::vault::ask::KINDS,
        crate::vault::reply::KINDS,
        voice_ask::KINDS,
        voice_note::KINDS,
        crate::fetch::ask::KINDS,
        crate::fetch::note::KINDS,
    ];
    for kind in Kind::ALL {
        let owners = lists.iter().filter(|l| l.contains(kind)).count();
        assert_eq!(owners, 1, "{kind:?}");
    }
    let codes: Vec<u8> = Kind::ALL.iter().map(|k| k.code()).collect();
    let want: Vec<u8> = (1..=u8::try_from(Kind::ALL.len()).unwrap()).collect();
    assert_eq!(codes, want);
    assert_eq!(Kind::VoiceStart.code(), 19);
}

#[test]
fn every_kind_has_its_lengths_and_only_a_bare_kind_takes_no_payload() {
    for &kind in Kind::ALL {
        let lens = kind.lens();
        assert!(lens.start() <= lens.end(), "{kind:?}");
        assert_eq!(kind.bare(), lens == (0..=0), "{kind:?}");
    }
}

#[test]
fn each_voice_frame_is_refused_by_every_other_reader() {
    let others: [Reader; 6] = [
        read_ask,
        read_reply,
        |r| VaultAsk::read(r).map(drop),
        |r| VaultReply::read(r).map(drop),
        |r| FetchAsk::read(r).map(drop),
        |r| FetchNote::read(r).map(drop),
    ];
    let asks = every_voice_ask()
        .iter()
        .map(voice_ask_bytes)
        .collect::<Vec<_>>();
    let notes = every_voice_note()
        .iter()
        .map(voice_note_bytes)
        .collect::<Vec<_>>();
    for bytes in asks.iter().chain(&notes) {
        for read in others {
            assert_eq!(read(&mut &bytes[..]), Err(IpcError::BadKind));
        }
    }
    for bytes in &asks {
        assert_eq!(read_voice_note(&mut &bytes[..]), Err(IpcError::BadKind));
    }
    for bytes in &notes {
        assert_eq!(read_voice_ask(&mut &bytes[..]), Err(IpcError::BadKind));
    }
}

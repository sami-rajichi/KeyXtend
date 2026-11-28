//! Which record notes end a watch, the id each record ask carries, and the pick job's words and ids.

use super::*;
use kx_ipc::{FileKind, RecordEnd, RecordFault, Switch};
use std::borrow::Cow;

/// The id the test frames carry.
const ID: u32 = 4;

/// A fault of the record job.
const fn fault(fault: RecordFault) -> RecordNote {
    RecordNote::Fault { id: ID, fault }
}

#[test]
fn a_record_watch_ends_at_a_runs_or_copys_end_and_at_faults_that_end_it() {
    let end = RecordNote::End {
        id: ID,
        length_ms: 1,
        end: RecordEnd::Stopped,
    };
    let ending = [RecordFault::NoVideo, RecordFault::Full, RecordFault::Failed];
    for note in [end].into_iter().chain(ending.map(fault)) {
        assert!(RecordFrames::last(&note), "{note:?}");
    }
    let sounds = [
        RecordFault::NoMic,
        RecordFault::MicRefused,
        RecordFault::NoSound,
    ];
    let switched = RecordNote::Switched {
        id: ID,
        switch: Switch::Mic,
        on: true,
    };
    let lost = RecordNote::Lost {
        id: ID,
        switch: Switch::Sound,
    };
    let time = RecordNote::Time { id: ID, ms: 0 };
    for note in [time, switched, lost].into_iter().chain(sounds.map(fault)) {
        assert!(!RecordFrames::last(&note), "a run goes on: {note:?}");
    }
}

#[test]
fn each_record_ask_carries_its_runs_id_and_goodbye_none() {
    let asks = [
        RecordAsk::Mic { id: ID, on: true },
        RecordAsk::Sound { id: ID, on: false },
        RecordAsk::Stop { id: ID },
        RecordAsk::Cancel { id: ID },
    ];
    for ask in &asks {
        assert_eq!(RecordFrames::ask_id(ask), ID, "{ask:?}");
    }
    assert_eq!(RecordFrames::ask_id(&RecordFrames::bye()), 0);
    assert_eq!((RecordFrames::JOB, PickFrames::JOB), (RECORD_JOB, PICK_JOB));
}

#[test]
fn a_pick_reply_names_its_dialog_and_goodbye_ends_the_job() {
    let replies = [
        PickReply::Cancelled { id: ID },
        PickReply::Refused { id: ID },
        PickReply::Failed { id: ID, code: 1 },
        PickReply::Picked {
            id: ID,
            path: Cow::Borrowed(r"C:\Snips\a.png"),
        },
    ];
    for reply in &replies {
        assert_eq!(PickFrames::id(reply), ID);
    }
    assert_eq!(PickFrames::bye(), PickAsk::Bye);
    let ask = PickAsk::Folder {
        id: ID,
        at: Cow::Borrowed(r"C:\Snips"),
    };
    let mut wire = Vec::new();
    PickFrames::write(&ask, &mut wire).unwrap();
    let save = PickAsk::Save {
        id: ID,
        folder: Cow::Borrowed(r"C:\Snips"),
        name: Cow::Borrowed("a"),
        kind: FileKind::Png,
    };
    PickFrames::write(&save, &mut wire).unwrap();
    assert!(!wire.is_empty());
}

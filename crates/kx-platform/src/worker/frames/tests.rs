//! Which voice and fetch notes end a watch, and the id each ask carries.

use super::*;
use crate::worker::fake::voice::{card_failed, devices, done, failed, level, open, piece, stop};
use kx_ipc::{FetchFault, SoundPiece, Stage, Wiped};
use kx_module_api::Zeroizing;

/// The id the test frames carry.
const ID: u32 = 1;
/// The runner a test file's facts name.
const RUNNER: &str = "transcribe";

/// A piece of the held recording, the last or not.
fn sound(last: bool) -> VoiceNote {
    let bytes = Wiped::Kept(Zeroizing::new(Vec::new()));
    let piece = SoundPiece {
        index: 0,
        last,
        bytes,
    };
    VoiceNote::Sound { id: ID, piece }
}

/// A model file's facts, naming no language.
fn facts() -> VoiceNote {
    let (runner, langs) = (RUNNER.to_owned(), Vec::new());
    VoiceNote::Facts {
        id: ID,
        runner,
        langs,
    }
}

/// A card's check with empty texts.
fn checked() -> VoiceNote {
    let (card_text, cpu_text) = (Zeroizing::new(String::new()), Zeroizing::new(String::new()));
    let (card_ms, cpu_ms) = (0, 0);
    VoiceNote::Checked {
        id: ID,
        card_ms,
        cpu_ms,
        card_text,
        cpu_text,
    }
}

/// Notes a run sends while it goes, beside its levels and pieces: speech heard, a warm-up and the model loading.
fn going() -> [VoiceNote; 3] {
    let (speech, percent, stage) = (true, 0, Stage::Loading);
    let heard = VoiceNote::Heard { id: ID, speech };
    let warm = VoiceNote::Warm { id: ID, percent };
    [heard, warm, VoiceNote::Stage { id: ID, stage }]
}

#[test]
fn a_voice_watch_ends_at_an_answer_a_runs_end_the_last_sound_or_a_fault_that_ends_it() {
    let ends = [
        done(ID),
        devices(ID),
        facts(),
        checked(),
        failed(ID),
        sound(true),
    ];
    assert!(ends.iter().all(VoiceFrames::last));
    let notes = [
        open(ID),
        level(ID),
        piece(ID),
        card_failed(ID),
        sound(false),
    ];
    let goes_on = going().into_iter().chain(notes);
    let ended = goes_on.into_iter().any(|note| VoiceFrames::last(&note));
    assert!(!ended, "a card's fault goes on too");
    assert_eq!(VoiceFrames::ask_id(&stop(ID)), ID);
}

#[test]
fn a_fetch_watch_ends_at_the_named_file_or_a_fault() {
    let (got, total, fault) = (0, 1, FetchFault::Cancelled);
    let progress = FetchNote::Progress { id: ID, got, total };
    let ends = [
        FetchNote::Done { id: ID },
        FetchNote::Fault { id: ID, fault },
    ];
    assert!(ends.iter().all(FetchFrames::last));
    assert!(!FetchFrames::last(&progress), "progress goes on");
    assert_eq!(FetchFrames::ask_id(&FetchAsk::Cancel { id: ID }), ID);
}

#[test]
fn a_voice_wait_shifts_when_the_microphone_closes_and_at_nothing_else() {
    let closed = [Stage::Shut, Stage::Limit, Stage::Silence];
    let stage = |stage| VoiceNote::Stage { id: ID, stage };
    assert!(
        closed
            .into_iter()
            .map(stage)
            .all(|n| VoiceFrames::shift(&n))
    );
    let others = going()
        .into_iter()
        .chain([open(ID), level(ID), piece(ID), done(ID)]);
    assert!(!others.into_iter().any(|n| VoiceFrames::shift(&n)));
    let progress = FetchNote::Progress {
        id: ID,
        got: 0,
        total: 1,
    };
    assert!(!FetchFrames::shift(&progress), "a download never shifts");
}

#[test]
fn only_a_runs_end_takes_the_jobs_peak_and_never_lowers_its_own() {
    let peak = |mb: u32| move || Some(mb);
    let memory = |note: &VoiceNote| match note {
        VoiceNote::Done { memory_mb, .. } => Some(*memory_mb),
        _ => None,
    };
    let mut end = done(ID);
    VoiceFrames::stamp(&mut end, &mut peak(7));
    assert_eq!(memory(&end), Some(7));
    VoiceFrames::stamp(&mut end, &mut peak(3));
    assert_eq!(memory(&end), Some(7), "the larger stays");
    let mut level = level(ID);
    VoiceFrames::stamp(&mut level, &mut || panic!("a level reads no peak"));
    assert_eq!(level, crate::worker::fake::voice::level(ID));
}

#[test]
fn a_net_watch_ends_at_done_or_a_fault_and_a_text_goes_on() {
    let fault = kx_ipc::NetFault::Network;
    let text = Zeroizing::new(String::new());
    let ends = [NetNote::Done { id: ID }, NetNote::Fault { id: ID, fault }];
    assert!(ends.iter().all(NetFrames::last));
    let said = NetNote::Text {
        id: ID,
        index: 0,
        text,
    };
    assert!(!NetFrames::last(&said), "the next text still comes");
    assert_eq!(NetFrames::ask_id(&NetAsk::Cancel { id: ID }), ID);
    assert_eq!(NetFrames::JOB, kx_ipc::NET_JOB);
}

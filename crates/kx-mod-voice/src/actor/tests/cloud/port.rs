//! The cloud port and the row's limits: with no port a cloud dictation ends with the fault note and keeps its
//! recording, and a listen-only run cuts pieces no longer than the row's size and time caps both allow.

use kx_module_api::ports::{CloudLimits, SAMPLE_BYTES, SOUND_HZ, SpeechAsk};

use super::*;
use crate::actor::offer::piece_s;

/// Limits of `max_mb` and `max_s`, the rest at their least.
fn limits(max_mb: u32, max_s: u16) -> CloudLimits {
    CloudLimits {
        max_mb,
        max_s,
        per_min: *CloudLimits::PER_MIN.start(),
        at_once: *CloudLimits::AT_ONCE.start(),
    }
}

#[test]
fn with_no_cloud_port_a_cloud_dictation_ends_with_the_fault_note_and_keeps_the_recording() {
    let mut rig = Rig::build(groq);
    rig.actor.ports.cloud = None;
    rig.dictate();
    assert!(rig.fake.cloud.calls().runs.is_empty(), "nothing sent");
    assert!(rig.asks().is_empty(), "nothing typed");
    assert_eq!(rig.bar(), cloud_note(CloudWhy::Fault));
    assert!(rig.fake.speech.runs()[0].held, "the recording stays");
}

#[test]
fn a_rows_size_cap_cuts_pieces_shorter_than_its_time_cap() {
    let per_s = SOUND_HZ * SAMPLE_BYTES;
    let small = limits(*CloudLimits::MAX_MB.start(), *CloudLimits::MAX_S.end());
    let cut = piece_s(&small);
    assert!(
        cut < *SpeechAsk::PIECE_S.end(),
        "read from the row, below the clamp"
    );
    assert!(
        usize::from(cut) * per_s <= small.piece_bytes(),
        "each piece fits"
    );
    assert!(
        usize::from(cut + 1) * per_s > small.piece_bytes(),
        "as long as fits"
    );
    let timed = limits(*CloudLimits::MAX_MB.end(), *CloudLimits::MAX_S.start());
    assert_eq!(
        u16::from(piece_s(&timed)),
        timed.max_s,
        "the time cap rules"
    );
}

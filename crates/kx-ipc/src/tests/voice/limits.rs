//! The voice, fetch and cloud limits: the settings' ranges the asks carry, and the hand-over's sizes against the
//! longest dictation (edge 31).

use crate::limits::MIB;
use crate::voice::wire as voice_wire;
use crate::{
    LEVEL_HZ, MAX_FIXED, MAX_KEY_BYTES, MAX_PIECES, MAX_RECORD_BYTES, MAX_SOUND_BYTES,
    MAX_SOUND_MS, MAX_SOUND_PIECES, MAX_TEXT_BYTES, NET_AT_ONCE, NET_MAX_MB, NET_PER_MIN,
    NET_TRIES, NET_WAIT_S, PAD_MS, PART_EXT, PIECE_S, RECORD_MAX_S, SAMPLE_BYTES, SILENCE_STOP_S,
    SOUND_HZ, STALL_S, THREADS,
};

/// `ElevenLabs`' cap on one file in MB, the largest a service row names.
const ELEVEN_MB: u32 = 5_000;

/// Seconds in ten minutes, the dictation edge 31 names.
const TEN_MIN_S: usize = 600;

#[test]
fn the_ranges_are_the_voice_settings_ranges() {
    assert_eq!(THREADS, 0..=64);
    assert_eq!(PAD_MS, 0..=2000);
    assert_eq!(PIECE_S, 10..=120);
    assert_eq!(RECORD_MAX_S, 10..=3600);
    assert_eq!(LEVEL_HZ, 5..=60);
    assert_eq!(SILENCE_STOP_S, 1..=30);
    assert!(*STALL_S.start() > 0 && STALL_S.contains(&30), "{STALL_S:?}");
}

#[test]
fn zero_is_free_to_mean_no_silence_stop() {
    assert!(!SILENCE_STOP_S.contains(&0));
}

#[test]
fn the_longest_dictation_fits_the_hand_over_in_whole_samples() {
    let longest = usize::from(*RECORD_MAX_S.end()) * SOUND_HZ * SAMPLE_BYTES;
    assert_eq!(MAX_RECORD_BYTES, longest);
    const { assert!(MAX_SOUND_PIECES * MAX_SOUND_BYTES >= MAX_RECORD_BYTES) };
    const { assert!((MAX_SOUND_PIECES - 1) * MAX_SOUND_BYTES < MAX_RECORD_BYTES) };
    assert_eq!(MAX_SOUND_BYTES % SAMPLE_BYTES, 0);
    assert_eq!(SOUND_HZ, 16_000);
    let ten_min = TEN_MIN_S * SOUND_HZ * SAMPLE_BYTES;
    assert!(ten_min.div_ceil(MAX_SOUND_BYTES) < MAX_SOUND_PIECES);
    assert_eq!(MAX_SOUND_MS, u32::from(*RECORD_MAX_S.end()) * 1000);
}

#[test]
fn a_sound_frame_stays_small_beside_a_whole_recording() {
    const { assert!(voice_wire::SOUND_MOST <= MAX_SOUND_BYTES + MIB) };
    const { assert!(voice_wire::SOUND_MOST < MAX_RECORD_BYTES) };
}

#[test]
fn the_pieces_cover_the_longest_dictation_at_the_shortest_piece() {
    let most = usize::from(*RECORD_MAX_S.end() / u16::from(*PIECE_S.start())) + 1;
    assert_eq!(MAX_PIECES, most);
    assert!(u16::try_from(MAX_PIECES).is_ok() && u16::try_from(MAX_SOUND_PIECES).is_ok());
}

#[test]
fn a_piece_holds_a_long_text_in_any_script() {
    let arabic = "مرحبا ".repeat(MAX_TEXT_BYTES / "مرحبا ".len());
    assert!(arabic.len() <= MAX_TEXT_BYTES && arabic.chars().count() > 5_000);
}

#[test]
fn the_part_name_is_a_lower_case_extension() {
    assert!(PART_EXT.starts_with('.'));
    assert!(PART_EXT[1..].bytes().all(|b| b.is_ascii_lowercase()));
}

#[test]
fn the_cloud_ranges_are_the_plans_and_fit_their_widths() {
    assert_eq!(NET_WAIT_S, 5..=300, "the plan's net_wait_s bound");
    assert_eq!(NET_TRIES, 1..=5, "the plan's net_tries bound");
    let starts: [u32; 3] = [
        *NET_MAX_MB.start(),
        (*NET_PER_MIN.start()).into(),
        (*NET_AT_ONCE.start()).into(),
    ];
    assert_eq!(starts, [1; 3], "a row's limit is never zero");
    assert!(NET_MAX_MB.contains(&ELEVEN_MB), "{NET_MAX_MB:?}");
    assert!(u16::try_from(MAX_KEY_BYTES).is_ok() && u8::try_from(MAX_FIXED).is_ok());
    const { assert!(MAX_FIXED >= 1, "OpenRouter's keep-nothing field") };
}

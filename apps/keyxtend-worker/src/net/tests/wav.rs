//! A piece wrapped as WAV: the plain 44-byte header spelled out by hand, room taken once, the voice job's own reader
//! taking it back sample for sample, and a size no header holds refused.

use crate::net::wav::{HEAD, start};
use kx_ipc::NetFault;

#[test]
fn a_piece_is_a_plain_wav_header_then_its_samples_in_room_taken_once() {
    let samples = [1u8, 0, 255, 127];
    let mut wav = start(samples.len()).unwrap();
    assert_eq!(wav.capacity(), HEAD + samples.len());
    wav.extend_from_slice(&samples);
    let mut want = b"RIFF".to_vec();
    want.extend_from_slice(&40u32.to_le_bytes());
    want.extend_from_slice(b"WAVEfmt ");
    want.extend_from_slice(&16u32.to_le_bytes());
    want.extend_from_slice(&[1, 0, 1, 0]);
    want.extend_from_slice(&16_000u32.to_le_bytes());
    want.extend_from_slice(&32_000u32.to_le_bytes());
    want.extend_from_slice(&[2, 0, 16, 0]);
    want.extend_from_slice(b"data");
    want.extend_from_slice(&4u32.to_le_bytes());
    want.extend_from_slice(&samples);
    assert_eq!((HEAD, &wav[..]), (44, &want[..]));
    assert_eq!(wav.capacity(), wav.len(), "no growth left a copy");
}

#[test]
fn the_voice_jobs_reader_takes_the_piece_back() {
    let pcm: Vec<u8> = [0i16, i16::MAX, -16_384]
        .iter()
        .flat_map(|s| s.to_le_bytes())
        .collect();
    let mut wav = start(pcm.len()).unwrap();
    wav.extend_from_slice(&pcm);
    let sound = crate::voice::wav::parse(&wav).unwrap();
    assert_eq!((sound.format.rate, sound.format.channels), (16_000, 1));
    assert_eq!(sound.samples.len(), 3);
    assert!((sound.samples[1] - 1.0).abs() < f32::EPSILON);
    assert!((sound.samples[2] + 0.5).abs() < 1e-4);
}

#[test]
fn a_piece_no_header_can_hold_is_refused() {
    let past = usize::try_from(u32::MAX).unwrap();
    assert_eq!(start(past).err(), Some(NetFault::Range));
}

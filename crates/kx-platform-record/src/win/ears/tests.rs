//! A stream's trouble and queue, with no device: a gap is counted and the stream goes on, any other error fails it, and
//! the queue drops its oldest packets past its length.

use super::*;
use std::time::Duration;

/// Packets the queue test puts, each of `FRAMES` frames, and the most frames it keeps.
const PUT: u64 = 5;
const FRAMES: usize = 10;
const KEEP: usize = 25;

#[test]
fn a_gap_is_counted_and_the_stream_goes_on() {
    let shared = Shared::default();
    shared.note(&ErrorKind::Xrun.into());
    shared.note(&ErrorKind::Xrun.into());
    assert_eq!(shared.gaps.load(Ordering::Relaxed), 2);
    assert!(!shared.failed.load(Ordering::Relaxed));
}

#[test]
fn any_other_error_fails_the_stream() {
    let shared = Shared::default();
    shared.note(&ErrorKind::StreamInvalidated.into());
    assert!(shared.failed.load(Ordering::Relaxed));
    assert_eq!(shared.gaps.load(Ordering::Relaxed), 0);
}

#[test]
fn the_queue_drops_its_oldest_packets_past_its_length() {
    let (shared, start) = (Shared::default(), Instant::now());
    for n in 0..PUT {
        let at = start + Duration::from_millis(n);
        let samples = vec![0.0; FRAMES * usize::from(CHANNELS)];
        shared.put(Packet { at, samples }, KEEP);
    }
    let queue = lock(&shared.queue);
    assert_eq!(queue.frames, 2 * FRAMES);
    assert_eq!(
        queue.packets.front().unwrap().at,
        start + Duration::from_millis(3)
    );
}

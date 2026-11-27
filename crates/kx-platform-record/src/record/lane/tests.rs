//! A lane puts each packet where it was heard: one that strays a little follows the last with no gap, a real gap is
//! silence, an overlap and anything before what was written are dropped, a cut silences from its place on, and the lane
//! never holds more than its queue.

use super::*;

/// How far a packet may stray and still follow the last, and the most a lane holds, in frames.
const DRIFT: i64 = 4;
const KEEP: usize = 64;

/// `frames` stereo frames, each `[v, -v]` with `v` counting up from `from`.
fn counting(from: i16, frames: i16) -> Vec<f32> {
    (from..from + frames)
        .flat_map(|v| [f32::from(v), -f32::from(v)])
        .collect()
}

/// The left samples of `frames` frames taken from position `from`.
fn left(lane: &mut Lane, from: i64, frames: usize) -> Vec<f32> {
    lane.take(from, frames).into_iter().step_by(2).collect()
}

#[test]
fn packets_land_where_they_were_heard_and_silence_fills_the_rest() {
    let mut lane = Lane::new(DRIFT, KEEP);
    lane.push(2, &counting(1, 3));
    lane.push(5, &counting(4, 2));
    assert_eq!(
        left(&mut lane, 0, 9),
        [0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 0.0, 0.0]
    );
}

#[test]
fn a_packet_that_strays_a_little_follows_the_last_and_a_gap_is_silence() {
    let mut lane = Lane::new(DRIFT, KEEP);
    lane.push(0, &counting(1, 2));
    lane.push(2 + DRIFT, &counting(3, 1));
    lane.push(3 + 2 * DRIFT + 1, &counting(4, 1));
    let got = left(&mut lane, 0, 13);
    let want = [
        1.0, 2.0, 3.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 4.0,
    ];
    assert_eq!(got, want);
}

#[test]
fn an_overlap_and_what_was_already_written_are_dropped() {
    let mut lane = Lane::new(DRIFT, KEEP);
    lane.push(0, &counting(1, 10));
    lane.push(10 - DRIFT - 2, &counting(20, 8));
    assert_eq!(left(&mut lane, 0, 4), [1.0, 2.0, 3.0, 4.0]);
    lane.push(0, &counting(30, 2));
    assert_eq!(
        left(&mut lane, 4, 10),
        [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 26.0, 27.0, 0.0, 0.0]
    );
}

#[test]
fn a_cut_silences_its_place_on_until_sound_comes_again() {
    let mut lane = Lane::new(DRIFT, KEEP);
    lane.push(0, &counting(1, 6));
    lane.cut(3);
    lane.push(6, &counting(7, 2));
    assert_eq!(
        left(&mut lane, 0, 8),
        [1.0, 2.0, 3.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    );
    lane.resume();
    lane.push(10, &counting(11, 2));
    assert_eq!(left(&mut lane, 8, 5), [0.0, 0.0, 11.0, 12.0, 0.0]);
}

#[test]
fn a_gap_far_past_the_queue_never_grows_the_lane_and_its_sound_still_lands() {
    let mut lane = Lane::new(DRIFT, KEEP);
    let far = 1_000_000;
    lane.push(0, &counting(1, 2));
    lane.push(far, &counting(3, 2));
    assert!(lane.buf.capacity() <= to_samples(4 * i64::try_from(KEEP).unwrap()));
    assert_eq!(left(&mut lane, 0, 2), [0.0, 0.0]);
    assert_eq!(left(&mut lane, far - 1, 4), [0.0, 3.0, 4.0, 0.0]);
}

#[test]
fn a_lane_holds_at_most_its_queue_dropping_the_oldest() {
    let mut lane = Lane::new(DRIFT, KEEP);
    let over = i16::try_from(KEEP).unwrap() + 3;
    lane.push(0, &counting(1, over));
    let got = left(&mut lane, 0, KEEP + 3);
    assert_eq!(&got[..4], [0.0, 0.0, 0.0, 4.0]);
}

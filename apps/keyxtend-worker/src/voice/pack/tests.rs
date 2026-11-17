//! The packer: stretches join in order within the cap, a longer stretch is cut at its quiet dips, and any packing keeps
//! each piece within the cap, two in a row past it, in order and covering every stretch once.

use super::*;
use crate::voice::hand::PIECE_SAMPLES;
use kx_ipc::SOUND_HZ;
use proptest::prelude::*;

/// A loud sample in the tests' sound.
const LOUD: i16 = 8_000;

/// Each piece as its stretches' starts and ends.
fn ends(pieces: &[Vec<Range<usize>>]) -> Vec<Vec<(usize, usize)>> {
    let pairs = |p: &Vec<Range<usize>>| p.iter().map(|r| (r.start, r.end)).collect();
    pieces.iter().map(pairs).collect()
}

/// Each piece's samples.
fn sizes(pieces: &[Vec<Range<usize>>]) -> Vec<usize> {
    let size = |p: &Vec<Range<usize>>| p.iter().map(ExactSizeIterator::len).sum();
    pieces.iter().map(size).collect()
}

#[test]
fn stretches_join_in_order_while_their_piece_stays_within_the_cap() {
    let mut pack = Packer::new(9);
    for span in [0..3, 5..9, 12..15, 15..20, 30..30] {
        pack.add(&[0; 30], span);
    }
    let want = [vec![(0, 3), (5, 9)], vec![(12, 15), (15, 20)]];
    assert_eq!(ends(&pack.pieces()), want);
}

#[test]
fn a_stretch_past_the_cap_is_cut_at_its_quiet_dips_not_at_the_cap() {
    let s = |secs: usize| secs * SOUND_HZ;
    let dips = [s(23)..s(24), s(46)..s(47)];
    let quiet = |k: usize| dips.iter().any(|d| d.contains(&k));
    let sound: Vec<i16> = (0..s(70))
        .map(|k| if quiet(k) { 0 } else { LOUD })
        .collect();
    let mut pack = Packer::new(PIECE_SAMPLES);
    pack.add(&sound, 0..sound.len());
    let pieces = pack.pieces();
    let cuts: Vec<usize> = pieces.iter().skip(1).map(|p| p[0].start).collect();
    assert_eq!(cuts.len(), 2, "{cuts:?}");
    assert!(
        dips.iter().zip(&cuts).all(|(d, c)| d.contains(c)),
        "{cuts:?}"
    );
    assert!(!cuts.contains(&PIECE_SAMPLES), "never at the cap");
    assert!(sizes(&pieces).iter().all(|&n| n <= PIECE_SAMPLES));
}

#[test]
fn nothing_added_is_no_piece() {
    assert!(Packer::new(0).pieces().is_empty());
}

/// Stretches in order with gaps between them, each of fewer than `most` samples.
fn stretches(most: usize) -> impl Strategy<Value = Vec<Range<usize>>> {
    let parts = prop::collection::vec((0..most, 0..most), 0..20);
    parts.prop_map(|parts| {
        let mut at = 0;
        let mut next = |(gap, len): (usize, usize)| {
            let start = at + gap;
            at = start + len;
            start..at
        };
        parts.into_iter().map(&mut next).collect()
    })
}

proptest! {
    /// Any packing keeps each piece within the cap with two in a row past it, in order, covering each stretch once.
    #[test]
    fn a_packing_stays_within_the_cap_in_order_and_covers_each_stretch_once(
        cap in 1..50_usize,
        spans in stretches(120),
        noise in prop::collection::vec(any::<i16>(), 1..64),
    ) {
        let end = spans.last().map_or(0, |r| r.end);
        let sound: Vec<i16> = (0..end).map(|k| noise[k % noise.len()]).collect();
        let mut pack = Packer::new(cap);
        for span in &spans {
            pack.add(&sound, span.clone());
        }
        let pieces = pack.pieces();
        let sizes = sizes(&pieces);
        prop_assert!(sizes.iter().all(|&n| n > 0 && n <= cap), "{sizes:?}");
        prop_assert!(sizes.windows(2).all(|w| w[0] + w[1] > cap), "{sizes:?}");
        let flat = pieces.concat();
        prop_assert!(flat.windows(2).all(|w| w[0].end <= w[1].start), "in order, none overlap");
        let covered: Vec<usize> = flat.into_iter().flatten().collect();
        let wanted: Vec<usize> = spans.into_iter().flatten().collect();
        prop_assert_eq!(covered, wanted);
    }
}

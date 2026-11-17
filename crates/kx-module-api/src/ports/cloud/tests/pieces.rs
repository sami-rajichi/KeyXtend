//! An ask's pieces: each within the row's size and seconds in whole samples, their count, and together at most the
//! longest dictation, as the frames check.

use super::*;

/// The row's limits with its fewest seconds a request, which cap a piece before its megabytes do.
fn short() -> CloudLimits {
    CloudLimits {
        max_s: *CloudLimits::MAX_S.start(),
        ..limits()
    }
}

#[test]
fn a_piece_may_hold_whole_samples_within_the_rows_size_and_seconds() {
    let tight = CloudLimits {
        max_mb: 1,
        ..limits()
    };
    assert_eq!(tight.piece_bytes(), 1 << 20, "a megabyte of whole samples");
    let by_time = usize::from(short().max_s) * crate::ports::SOUND_HZ * crate::ports::SAMPLE_BYTES;
    assert_eq!(short().piece_bytes(), by_time);
}

#[test]
fn pieces_past_the_cap_or_count_or_of_half_samples_are_refused() {
    let (short, most) = (short(), short().piece_bytes());
    let pieces = |sizes: &[usize]| sizes.iter().map(|n| sound(&vec![1; *n])).collect();
    let over = CloudAsk::PIECES.end() + 1;
    let bad = [
        Vec::new(),
        pieces(&[0]),
        pieces(&[3]),
        pieces(&[2, most + 2]),
        pieces(&vec![2; over]),
    ];
    for pieces in bad {
        let count = pieces.len();
        let ask = CloudAsk {
            limits: short,
            pieces,
            ..ask()
        };
        assert_eq!(refused(&ask), Some("pieces"), "{count} pieces");
    }
    let ask = CloudAsk {
        limits: short,
        pieces: pieces(&[most, 2]),
        ..ask()
    };
    assert_eq!(ask.check(), Ok(()), "a piece at the cap fits");
}

#[test]
fn pieces_past_the_longest_dictation_together_are_refused() {
    let limits = CloudLimits {
        max_mb: *CloudLimits::MAX_MB.end(),
        max_s: *CloudLimits::MAX_S.end(),
        ..limits()
    };
    let top = limits.piece_bytes();
    let whole = kx_ipc::MAX_RECORD_BYTES / top;
    let silence = |n| Redacted::new(Zeroizing::new(vec![0; n]));
    let mut ask = CloudAsk {
        limits,
        pieces: (0..whole).map(|_| silence(top)).collect(),
        ..ask()
    };
    assert_eq!(ask.check(), Ok(()), "the longest dictation fits");
    ask.pieces.push(silence(crate::ports::SAMPLE_BYTES));
    assert_eq!(refused(&ask), Some("pieces"), "one sample past it");
}

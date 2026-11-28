//! Shots, parts and frozen screens: a part is never empty, a shot holds exactly its pixels and shares them, a frozen
//! screen matches its shot, and none prints a pixel.

use super::{PIXELS, keeps, shot};
use crate::REDACTED_MARKER;
use crate::ports::snip::{FreezeError, Frozen, FrozenScreen, Part, SHOT_CAP_MB, Shot, ShotCap};
use crate::ports::{EncodeAsk, MB, Point, Rect};
use std::sync::Arc;

/// A rectangle at (`x`, 0) of `w` by `h`.
fn rect(x: i32, w: i32, h: i32) -> Rect {
    Rect { x, y: 0, w, h }
}

#[test]
fn a_part_cannot_be_empty() {
    for (w, h) in [(0, 10), (10, 0), (0, 0), (-5, 10), (10, -1)] {
        assert_eq!(Part::new(rect(0, w, h)), None, "{w} by {h}");
    }
    let one = rect(-1920, 1, 1);
    assert_eq!(Part::new(one).map(Part::rect), Some(one), "a left screen");
}

#[test]
fn a_part_whose_far_edge_passes_the_pixel_range_is_refused() {
    assert_eq!(Part::new(rect(i32::MAX, 1, 1)), None);
    let tall = Rect {
        x: 0,
        y: i32::MAX - 1,
        w: 1,
        h: 2,
    };
    assert_eq!(Part::new(tall), None);
    assert!(
        Part::new(rect(i32::MAX - 1, 1, 1)).is_some(),
        "the last pixel"
    );
}

#[test]
fn a_shot_holds_exactly_its_pixels() {
    let four = PIXELS.to_vec();
    assert!(Shot::new(1, 1, four.clone()).is_some());
    let short = PIXELS[..3].to_vec();
    let refused = [
        Shot::new(1, 1, short),
        Shot::new(2, 1, four.clone()),
        Shot::new(0, 1, Vec::new()),
        Shot::new(1, 0, Vec::new()),
        Shot::new(u32::MAX, u32::MAX, four),
    ];
    assert!(refused.iter().all(Option::is_none), "{refused:?}");
}

#[test]
fn a_shot_keeps_its_size_and_shares_its_pixels() {
    let one = shot();
    let two = one.clone();
    assert_eq!((one.width(), one.height()), (2, 1));
    assert_eq!(one.bgrx().expose().len(), 2 * PIXELS.len());
    assert!(Arc::ptr_eq(one.bgrx().expose(), two.bgrx().expose()));
}

#[test]
fn a_filled_shot_is_written_once_in_its_shared_pixels() {
    let mut seen = 0;
    let filled = Shot::filled(2, 1, |bytes| {
        seen = bytes.len();
        bytes.copy_from_slice(&[PIXELS, PIXELS].concat());
        true
    });
    assert_eq!(seen, 2 * PIXELS.len(), "every byte, once");
    assert_eq!(filled, Some(shot()));
    assert_eq!(Shot::filled(1, 1, |_| false), None, "a fill that failed");
    for (w, h) in [(0, 1), (1, 0), (u32::MAX, u32::MAX)] {
        assert_eq!(Shot::filled(w, h, |_| true), None, "{w} by {h}");
    }
}

#[test]
fn a_shot_prints_its_size_and_never_a_pixel() {
    let shown = format!("{:?}", shot());
    assert!(keeps(&shown), "{shown}");
    assert_eq!(
        shown,
        format!("Shot {{ width: 2, height: 1, bgrx: {REDACTED_MARKER} }}")
    );
}

#[test]
fn a_frozen_screen_is_the_size_of_its_shot() {
    let screen = FrozenScreen::new(rect(1920, 2, 1), 144, shot());
    let facts = screen
        .as_ref()
        .map(|s| (s.rect(), s.dpi(), s.shot().width()));
    assert_eq!(facts, Some((rect(1920, 2, 1), 144, 2)));
    for wrong in [rect(0, 3, 1), rect(0, 2, 2)] {
        assert!(FrozenScreen::new(wrong, 96, shot()).is_none(), "{wrong:?}");
    }
}

#[test]
fn a_freeze_prints_its_screens_and_frames_but_no_pixel() {
    let frozen = Frozen {
        screens: vec![FrozenScreen::new(rect(0, 2, 1), 96, shot()).unwrap()],
        windows: vec![rect(0, 2, 1)],
        pointer: Point { x: 1, y: 0 },
    };
    let shown = format!("{frozen:?}");
    assert!(keeps(&shown), "{shown}");
    assert!(
        shown.contains("dpi: 96") && shown.contains(REDACTED_MARKER),
        "{shown}"
    );
}

#[test]
fn a_cap_is_taken_only_within_its_range() {
    let (low, high) = (*SHOT_CAP_MB.start(), *SHOT_CAP_MB.end());
    assert_eq!(
        (ShotCap::new(low - 1), ShotCap::new(high + 1)),
        (None, None)
    );
    assert_eq!((ShotCap::new(0), ShotCap::new(u32::MAX)), (None, None));
    let least = ShotCap::new(low).unwrap();
    assert_eq!((least.mb(), least.bytes()), (low, u64::from(low) * MB));
    let most = ShotCap::new(high).map(ShotCap::bytes);
    let encode = u64::try_from(EncodeAsk::MAX_BYTES).ok();
    assert_eq!(most, encode, "no picture outgrows what Encode takes");
}

#[test]
fn freeze_errors_say_what_happened() {
    let texts = [FreezeError::TooBig, FreezeError::Refused(5)].map(|e| e.to_string());
    assert_ne!(texts[0], texts[1]);
    assert!(texts[1].contains("code 5"), "{}", texts[1]);
}

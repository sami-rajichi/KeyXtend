//! The shield: a GDI grab leaves out a window shielded before it first shows, in every frame from its show on, and
//! drops a shown window soon after it is shielded, showing our backdrop under it and never another app. Each test also
//! sees the window in the grab unshielded, so it stands alone.

use std::time::{Duration, Instant};

use windows::Win32::Foundation::RECT;

use super::hold;
use super::screen::{
    Plain, Rgb, area, centred, count, frame, grab, inset, physical, until, work_area,
};

/// The backdrop's colour.
const BACK: Rgb = Rgb(200, 0, 200);
/// The shielded window's colour.
const FRONT: Rgb = Rgb(0, 200, 0);
/// The backdrop's place inside the work area, and its size.
const BACK_BOX: (i32, i32, i32) = (200, 320, 240);
/// The shielded window's size, centred on the backdrop.
const FRONT_SIZE: (i32, i32) = (160, 120);
/// Frames the first probe watches after the window shows.
const FRAMES: u32 = 30;
/// Rounds the second probe shields a shown window, to see the spread.
const ROUNDS: u32 = 5;
/// The longest any window may take to show in, or leave, the grab.
const SETTLE: Duration = Duration::from_secs(2);

/// The backdrop's box and the shielded window's box on it.
fn boxes() -> (RECT, RECT) {
    let (margin, w, h) = BACK_BOX;
    let back = inset(work_area(), margin, w, h);
    (back, centred(back, FRONT_SIZE.0, FRONT_SIZE.1))
}

/// Whether the grab of `at` is `colour` alone.
fn all(at: RECT, colour: Rgb) -> bool {
    count(&grab(at), colour) == area(at)
}

#[test]
#[cfg_attr(
    not(kx_live_snip),
    ignore = "shows windows and reads the screen; opt-in"
)]
fn a_window_shielded_before_it_first_shows_is_never_in_a_grab() {
    let (_live, _px) = (hold(), physical());
    let (back_at, front_at) = boxes();
    let back = Plain::new(back_at, BACK);
    back.show();
    assert!(
        until(SETTLE, || all(back_at, BACK)),
        "the grab sees our backdrop"
    );
    let front = Plain::new(front_at, FRONT);
    front.shield(true);
    front.show();
    for seen in 0..=FRAMES {
        assert_eq!(count(&grab(front_at), FRONT), 0, "frame {seen}");
        frame();
    }
    assert!(all(front_at, BACK), "the backdrop shows through");
    front.shield(false);
    assert!(
        until(SETTLE, || all(front_at, FRONT)),
        "unshielded, the window shows, so it was there all along"
    );
}

#[test]
#[cfg_attr(
    not(kx_live_snip),
    ignore = "shows windows and reads the screen; opt-in"
)]
fn a_shown_window_leaves_the_grab_soon_after_it_is_shielded() {
    let (_live, _px) = (hold(), physical());
    let (back_at, front_at) = boxes();
    let back = Plain::new(back_at, BACK);
    back.show();
    let front = Plain::new(front_at, FRONT);
    front.show();
    let mut spread = Vec::new();
    for round in 0..ROUNDS {
        front.shield(false);
        assert!(
            until(SETTLE, || all(front_at, FRONT)),
            "round {round}: in the grab"
        );
        let start = Instant::now();
        front.shield(true);
        let mut frames = 0_u32;
        let gone = until(SETTLE, || {
            frames += 1;
            count(&grab(front_at), FRONT) == 0
        });
        assert!(gone, "round {round}: still in the grab after {SETTLE:?}");
        spread.push((start.elapsed(), frames - 1));
        assert!(
            all(front_at, BACK),
            "round {round}: the backdrop shows through"
        );
    }
    eprintln!("probe: a shown window left the grab after (time, frames waited): {spread:?}");
}

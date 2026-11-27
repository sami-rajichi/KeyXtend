//! The freeze on the real screens: every screen at its physical size from another thread, our window frozen to the
//! pixel but never offered (edge 3), and a window shielded by title before it shows in no freeze. Counts only.

use std::time::{Duration, Instant};

use kx_module_api::ports::{Frozen, FrozenScreen, Part, Point, SHOT_CAP_MB, ScreenFreeze, ShotCap};
use kx_platform::snip::Seen;
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Gdi::{
    DESKTOPHORZRES, DESKTOPVERTRES, GetDC, GetDeviceCaps, ReleaseDC,
};

use super::hold;
use super::screen::{Plain, Rgb, area, centred, count, frame, inset, physical, until, work_area};
use crate::screen::{contains, rect_of, whole_screens};
use crate::snip::frames::{self, shell};
use crate::{WinScreenFreeze, WindowsPlatform};

/// The backdrop's colour.
const BACK: Rgb = Rgb(0, 120, 200);
/// Our window's colour.
const FRONT: Rgb = Rgb(220, 160, 0);
/// The backdrop's place inside the work area, and its size.
const BACK_BOX: (i32, i32, i32) = (160, 280, 200);
/// Our window's size, centred on the backdrop.
const FRONT_SIZE: (i32, i32) = (120, 80);
/// Freezes the shielded window must be missing from after it shows.
const FREEZES: u32 = 10;
/// The longest any window may take to show in, or leave, a freeze.
const SETTLE: Duration = Duration::from_secs(2);

/// The smallest cap `shot_cap_mb` allows, far above one screen.
fn cap() -> ShotCap {
    ShotCap::new(*SHOT_CAP_MB.start()).unwrap()
}

/// A freeze now.
fn freeze() -> Frozen {
    WinScreenFreeze::new().freeze(cap()).unwrap()
}

/// How many pixels of the box `at`, cut from a freeze now, are `colour`.
fn seen(at: RECT, colour: Rgb) -> usize {
    let part = Part::new(rect_of([at.left, at.top, at.right, at.bottom])).unwrap();
    let cut = part.cut(&freeze().screens).unwrap();
    count(cut.bgrx().expose(), colour)
}

/// The backdrop's box and our window's box on it.
fn boxes() -> (RECT, RECT) {
    let (margin, w, h) = BACK_BOX;
    let back = inset(work_area(), margin, w, h);
    (back, centred(back, FRONT_SIZE.0, FRONT_SIZE.1))
}

#[test]
#[cfg_attr(not(kx_live_snip), ignore = "reads every screen; opt-in")]
fn a_freeze_from_another_thread_grabs_every_screen_at_its_physical_size() {
    let _live = hold();
    let port = WindowsPlatform::new().unwrap().screen_freeze();
    let timed = move || (Instant::now(), port.freeze(cap()));
    let (start, frozen) = std::thread::spawn(timed).join().unwrap();
    let (took, frozen) = (start.elapsed(), frozen.unwrap());
    let wanted = {
        let _px = physical();
        whole_screens()
    };
    let got: Vec<_> = frozen.screens.iter().map(|s| (s.rect(), s.dpi())).collect();
    assert_eq!(got, wanted, "every screen, whole, at its DPI");
    let main = frozen
        .screens
        .iter()
        .find(|s| contains(s.rect(), Point::default()));
    let size = main.map(|s| (s.rect().w, s.rect().h));
    assert_eq!(size, Some(main_physical()), "the main screen, unscaled");
    let on = |s: &FrozenScreen| contains(s.rect(), frozen.pointer);
    assert!(frozen.screens.iter().any(on), "the pointer is on a screen");
    let (screens, windows) = (frozen.screens.len(), frozen.windows.len());
    eprintln!(
        "probe: a freeze took {took:?}, held {screens} screens and offered {windows} windows"
    );
}

/// The main screen's size as the display reports it, unscaled whatever this thread's DPI awareness.
fn main_physical() -> (i32, i32) {
    // SAFETY: plain queries of the screen's context, let go once at the end.
    unsafe {
        let dc = GetDC(None);
        let size = (
            GetDeviceCaps(Some(dc), DESKTOPHORZRES),
            GetDeviceCaps(Some(dc), DESKTOPVERTRES),
        );
        ReleaseDC(None, dc);
        size
    }
}

#[test]
#[cfg_attr(
    not(kx_live_snip),
    ignore = "shows a window and reads the screen; opt-in"
)]
fn our_window_is_frozen_to_the_pixel_and_never_offered() {
    let (_live, _px) = (hold(), physical());
    let (_, at) = boxes();
    let ours = Plain::new(at, FRONT);
    ours.show();
    let exact = until(SETTLE, || seen(at, FRONT) == area(at));
    assert!(exact, "every pixel of our window, as painted");
    assert_eq!(
        frames::seen(ours.hwnd(), shell()),
        Seen::Ours,
        "shown, and ours"
    );
    let frame_of = rect_of([at.left, at.top, at.right, at.bottom]);
    let offered = freeze().windows;
    assert!(!offered.contains(&frame_of), "ours is never offered");
    assert!(offered.iter().all(|r| r.w > 0 && r.h > 0), "no empty frame");
}

#[test]
#[cfg_attr(
    not(kx_live_snip),
    ignore = "shows windows and reads the screen; opt-in"
)]
fn a_window_shielded_by_title_before_it_shows_is_in_no_freeze() {
    let (_live, _px) = (hold(), physical());
    let (back_at, front_at) = boxes();
    let back = Plain::new(back_at, BACK);
    back.show();
    let ready = until(SETTLE, || seen(back_at, BACK) == area(back_at));
    assert!(ready, "the freeze sees our backdrop");
    let front = Plain::new(front_at, FRONT);
    front.shield(true);
    front.show();
    for n in 0..FREEZES {
        assert_eq!(seen(front_at, FRONT), 0, "freeze {n}");
        frame();
    }
    assert_eq!(
        seen(front_at, BACK),
        area(front_at),
        "the backdrop shows through"
    );
    front.shield(false);
    let back_in = until(SETTLE, || seen(front_at, FRONT) == area(front_at));
    assert!(back_in, "unshielded, it shows, so it was there all along");
}

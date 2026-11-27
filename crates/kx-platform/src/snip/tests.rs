//! The freeze's steps over a scripted desk: the cap refuses before any grab (edge 4), each screen keeps its place and
//! DPI, a refused grab freezes nothing, and only shown windows are offered, front first, told by their looks (edge 3).

use super::*;
use kx_module_api::ports::{BGRX_BYTES, SHOT_CAP_MB, Shot};
use std::sync::Mutex;

/// A screen at 100 %.
const MAIN: Rect = Rect {
    x: 0,
    y: 0,
    w: 8,
    h: 5,
};
/// A screen to its left at 150 %.
const LEFT: Rect = Rect {
    x: -6,
    y: -2,
    w: 6,
    h: 4,
};
/// `MAIN`'s DPI.
const MAIN_DPI: u32 = 96;
/// `LEFT`'s DPI.
const LEFT_DPI: u32 = 144;
/// Where the pointer is.
const POINTER: Point = Point { x: 3, y: 2 };
/// A refusal's code.
const CODE: i32 = 5;
/// The width of the screens that test the cap.
const WIDE: i32 = 1024;

/// The smallest cap `shot_cap_mb` allows.
fn cap() -> ShotCap {
    ShotCap::new(*SHOT_CAP_MB.start()).unwrap()
}

/// The bytes of one row of a `WIDE` screen.
fn row() -> u64 {
    u64::try_from(WIDE).unwrap() * u64::try_from(BGRX_BYTES).unwrap()
}

/// A screen `WIDE` pixels wide with `rows` rows.
fn tall(rows: u64) -> Rect {
    Rect {
        x: 0,
        y: 0,
        w: WIDE,
        h: i32::try_from(rows).unwrap(),
    }
}

/// The frame of a window at (`x`, 0).
fn at(x: i32) -> Rect {
    Rect {
        x,
        y: 0,
        w: 2,
        h: 2,
    }
}

/// A scripted desk that counts its grabs.
#[derive(Default)]
struct Desk {
    screens: Vec<(Rect, u32)>,
    windows: Vec<(Rect, Seen)>,
    error: Option<FreezeError>,
    grabs: Mutex<Vec<Vec<(Rect, u32)>>>,
}

impl Desk {
    fn new(screens: Vec<(Rect, u32)>) -> Self {
        Self {
            screens,
            ..Self::default()
        }
    }

    /// Every grab asked, oldest first.
    fn grabs(&self) -> Vec<Vec<(Rect, u32)>> {
        self.grabs.lock().unwrap().clone()
    }
}

impl super::Desk for Desk {
    fn screens(&self) -> Vec<(Rect, u32)> {
        self.screens.clone()
    }

    fn grab(&self, screens: &[(Rect, u32)]) -> Result<Vec<FrozenScreen>, FreezeError> {
        self.grabs.lock().unwrap().push(screens.to_vec());
        self.error.map_or(Ok(()), Err)?;
        let plain = |&(rect, dpi): &(Rect, u32)| {
            let (w, h) = (
                u32::try_from(rect.w).unwrap(),
                u32::try_from(rect.h).unwrap(),
            );
            let shot = Shot::new(w, h, vec![0; (w * h) as usize * BGRX_BYTES]).unwrap();
            FrozenScreen::new(rect, dpi, shot).unwrap()
        };
        Ok(screens.iter().map(plain).collect())
    }

    fn windows(&self) -> Vec<(Rect, Seen)> {
        self.windows.clone()
    }

    fn pointer(&self) -> Point {
        POINTER
    }
}

/// Both screens, as a freeze over them.
fn both() -> Freezer<Desk> {
    Freezer::new(Desk::new(vec![(MAIN, MAIN_DPI), (LEFT, LEFT_DPI)]))
}

#[test]
fn a_freeze_keeps_every_screen_at_its_place_and_dpi_with_the_pointer() {
    let freeze = both();
    let frozen = freeze.freeze(cap()).unwrap();
    let facts: Vec<_> = frozen.screens.iter().map(|s| (s.rect(), s.dpi())).collect();
    assert_eq!(facts, [(MAIN, MAIN_DPI), (LEFT, LEFT_DPI)]);
    assert_eq!(frozen.pointer, POINTER);
    assert_eq!(freeze.desk.grabs().len(), 1, "every screen in one grab");
}

#[test]
fn screens_past_the_cap_together_are_refused_before_any_grab() {
    let rows = cap().bytes() / row();
    let one = Freezer::new(Desk::new(vec![(tall(rows + 1), MAIN_DPI)]));
    assert_eq!(one.freeze(cap()), Err(FreezeError::TooBig));
    let half = rows / 2 + 1;
    let two = Freezer::new(Desk::new(vec![(tall(half), MAIN_DPI); 2]));
    assert_eq!(two.freeze(cap()), Err(FreezeError::TooBig), "together");
    assert!(one.desk.grabs().is_empty() && two.desk.grabs().is_empty());
    let full = Freezer::new(Desk::new(vec![(tall(rows), MAIN_DPI)]));
    assert!(full.freeze(cap()).is_ok(), "exactly the cap");
}

#[test]
fn a_refused_grab_freezes_nothing() {
    let mut desk = Desk::new(vec![(MAIN, MAIN_DPI)]);
    desk.error = Some(FreezeError::Refused(CODE));
    let freeze = Freezer::new(desk);
    assert_eq!(freeze.freeze(cap()), Err(FreezeError::Refused(CODE)));
}

#[test]
fn a_screen_with_no_pixels_is_never_grabbed() {
    let empty = Rect { w: 0, ..MAIN };
    let freeze = Freezer::new(Desk::new(vec![(empty, MAIN_DPI), (LEFT, LEFT_DPI)]));
    let frozen = freeze.freeze(cap()).unwrap();
    assert_eq!(frozen.screens.len(), 1);
    assert_eq!(freeze.desk.grabs(), [vec![(LEFT, LEFT_DPI)]]);
}

#[test]
fn only_shown_windows_with_pixels_are_offered_front_first() {
    let mut desk = Desk::new(vec![(MAIN, MAIN_DPI)]);
    desk.windows = vec![
        (at(0), Seen::Ours),
        (at(1), Seen::Shown),
        (at(2), Seen::Cloaked),
        (at(3), Seen::Minimised),
        (at(4), Seen::Hidden),
        (at(5), Seen::Tool),
        (at(6), Seen::Desktop),
        (Rect { w: 0, ..at(7) }, Seen::Shown),
        (Rect { h: -1, ..at(8) }, Seen::Shown),
        (at(9), Seen::Through),
        (at(10), Seen::Shown),
    ];
    let frozen = Freezer::new(desk).freeze(cap()).unwrap();
    assert_eq!(frozen.windows, [at(1), at(10)]);
}

#[test]
fn the_bytes_of_screens_add_up_and_one_with_no_pixels_holds_none() {
    let bytes = |screens: &[Rect]| screen_bytes(screens.iter().copied());
    let four = u64::try_from(BGRX_BYTES).unwrap();
    assert_eq!(bytes(&[MAIN]), 40 * four);
    assert_eq!(bytes(&[MAIN, LEFT]), (40 + 24) * four);
    assert_eq!(bytes(&[Rect { h: 0, ..MAIN }, LEFT]), 24 * four);
    assert_eq!(bytes(&[Rect { w: -3, ..MAIN }]), 0);
    assert_eq!(bytes(&[]), 0);
}

#[test]
fn huge_screens_saturate_and_never_fit() {
    let huge = Rect {
        x: 0,
        y: 0,
        w: i32::MAX,
        h: i32::MAX,
    };
    assert_eq!(screen_bytes([huge; 4]), u64::MAX);
    assert!(!fits([huge], cap()));
}

/// A window's scripted looks, each a bit of `bits` in `Looks`' order, counting every read.
struct Bits {
    bits: u8,
    reads: std::cell::Cell<u8>,
}

impl Bits {
    /// Bit `n` of the looks, counted as one read.
    fn bit(&self, n: u8) -> bool {
        self.reads.set(self.reads.get() + 1);
        self.bits & (1 << n) != 0
    }
}

impl Looks for Bits {
    fn shown(&self) -> bool {
        self.bit(0)
    }
    fn ours(&self) -> bool {
        self.bit(1)
    }
    fn desktop(&self) -> bool {
        self.bit(2)
    }
    fn minimised(&self) -> bool {
        self.bit(3)
    }
    fn tool(&self) -> bool {
        self.bit(4)
    }
    fn through(&self) -> bool {
        self.bit(5)
    }
    fn cloaked(&self) -> bool {
        self.bit(6)
    }
}

/// The looks of `bits`, not read yet.
fn looks(bits: u8) -> Bits {
    Bits {
        bits,
        reads: 0.into(),
    }
}

#[test]
fn a_window_is_shown_only_when_shown_and_none_of_the_rest() {
    for bits in 0..1 << 7 {
        let seen = Seen::of(&looks(bits));
        assert_eq!(seen == Seen::Shown, bits == 1, "{bits:07b}: {seen:?}");
    }
}

#[test]
fn a_click_through_window_is_never_offered_as_it_would_take_the_pick() {
    let through = looks(0b10_0001);
    assert_eq!(
        (Seen::of(&through), through.reads.get()),
        (Seen::Through, 6)
    );
}

#[test]
fn facts_are_read_only_until_one_rules_the_window_out() {
    let hidden = looks(0b111_1110);
    assert_eq!((Seen::of(&hidden), hidden.reads.get()), (Seen::Hidden, 1));
    let ours = looks(0b111_1111);
    assert_eq!((Seen::of(&ours), ours.reads.get()), (Seen::Ours, 2));
    let cloaked = looks(0b100_0001);
    assert_eq!(
        (Seen::of(&cloaked), cloaked.reads.get()),
        (Seen::Cloaked, 7),
        "the costliest read last"
    );
}

//! The run on a fake desk: one screen with the keyboard, our banner, an app and the desktop behind it; our list opens
//! where a test says.

mod aims;
mod fake;
mod gates;
mod reads;

use super::*;
use crate::scroll::tests::{CONFIG, NOTCH, THREE, wheel};
use fake::Fake;
use kx_module_api::ports::Screen;

/// The keyboard near the bottom: x 400 to 1499, y 700 to 999.
const KB: Rect = Rect {
    x: 400,
    y: 700,
    w: 1100,
    h: 300,
};
/// Our open list, to the right.
const LIST: Rect = Rect {
    x: 1600,
    y: 100,
    w: 200,
    h: 300,
};
/// Our banner, under the list.
const BANNER: Rect = Rect {
    x: 1600,
    y: 450,
    w: 200,
    h: 100,
};
/// An app at the top left.
const APP: Rect = Rect {
    x: 0,
    y: 0,
    w: 1000,
    h: 600,
};
/// The whole screen, where the desktop lies behind everything.
const DESK: Rect = Rect {
    x: 0,
    y: 0,
    w: 1920,
    h: 1080,
};
/// The screen's work area, above the taskbar.
const WORK: Rect = Rect {
    x: 0,
    y: 0,
    w: 1920,
    h: 1040,
};
/// The keyboard's id.
const KB_ID: u64 = 10;
/// Our list's id.
const LIST_ID: u64 = 20;
/// Our banner's id.
const BANNER_ID: u64 = 25;
/// The app's id.
const APP_ID: u64 = 30;
/// The desktop's id.
const DESK_ID: u64 = 40;
/// A point on the app.
const ON_APP: Point = Point { x: 200, y: 200 };
/// A point on the keyboard.
const ON_KB: Point = Point { x: 500, y: 800 };
/// The last point on the app before the pointer entered the keyboard.
const BEFORE: Point = Point { x: 500, y: 590 };
/// The app's scrolling element: its box and its percents, half way down.
const READ: Read = Read {
    area: Some([10, 20, 990, 580]),
    pair: Some((0.0, 50.0)),
};
/// That box as a rect.
const BOX: Rect = Rect {
    x: 10,
    y: 20,
    w: 980,
    h: 560,
};
/// A whole notch at three lines a notch.
const NOTCH_LINES: f64 = 3.0;

/// The fake desk: the pointer on the app, the entry kept from `BEFORE`, every read giving `READ`, and no list open.
fn desk() -> Fake {
    let fake = Fake::default();
    fake.with(|s| {
        let spot = |window, whose| Spot { window, whose };
        s.windows = vec![
            (KB, spot(KB_ID, Whose::Keyboard)),
            (BANNER, spot(BANNER_ID, Whose::Ours)),
            (APP, spot(APP_ID, Whose::Theirs)),
            (DESK, spot(DESK_ID, Whose::Theirs)),
        ];
        s.screens = vec![Screen {
            rect: WORK,
            dpi: 96,
            primary: true,
        }];
        let history = [(500, 710), (BEFORE.x, BEFORE.y)];
        s.kept = Some(Kept::new(
            &history,
            Point { x: 500, y: 710 },
            KB,
            &s.screens,
        ));
        (s.cursor, s.whole, s.wheel) = (Some(ON_APP), DESK, Some(wheel(THREE, THREE)));
        (s.read, s.tick_ms) = (Some(READ), 1);
    });
    fake
}

/// A run on `fake` with the `[scroll]` defaults.
fn run(fake: &Fake) -> Run<Fake> {
    Run::new(fake.clone(), CONFIG)
}

/// Opens our list at `r` on `fake`, in front of every window.
fn show_list(fake: &Fake, r: Rect) {
    let list = Spot {
        window: LIST_ID,
        whose: Whose::Pages,
    };
    fake.with(|s| s.windows.insert(0, (r, list)));
}

/// `read_ms` as a cap.
fn cap() -> Duration {
    Duration::from_millis(u64::from(CONFIG.read_ms))
}

#[test]
fn the_pointer_off_our_windows_aims_at_the_window_under_it() {
    let fake = desk();
    let aim = run(&fake).aim().unwrap();
    assert_eq!((aim.at, aim.on_pointer, aim.ours), (ON_APP, true, false));
    assert_eq!((aim.id, aim.area), (TargetId::new(APP_ID), Some(BOX)));
    let reads = fake.with(|s| s.reads.clone());
    assert_eq!(reads, [(Seek::At(ON_APP), Dir::Down, cap())]);
}

#[test]
fn the_pointer_on_the_keyboard_aims_at_the_kept_point() {
    let fake = desk();
    fake.with(|s| s.cursor = Some(ON_KB));
    let aim = run(&fake).aim().unwrap();
    assert_eq!((aim.at, aim.on_pointer, aim.ours), (BEFORE, false, false));
    assert_eq!(aim.id, TargetId::new(APP_ID));
    let reads = fake.with(|s| s.reads.clone());
    assert_eq!(
        reads,
        [(Seek::At(BEFORE), Dir::Down, cap())],
        "the app is there"
    );
}

#[test]
fn a_missed_history_aims_past_the_keyboards_nearest_edge() {
    let fake = desk();
    let entry = Point { x: 500, y: 990 };
    let kept = fake.with(|s| Kept::new(&[], entry, KB, &s.screens));
    fake.with(|s| (s.cursor, s.kept) = (Some(ON_KB), Some(kept)));
    let aim = run(&fake).aim().unwrap();
    let below = Point {
        x: 500,
        y: KB.y + KB.h + 7,
    };
    assert_eq!((aim.at, aim.id), (below, TargetId::new(DESK_ID)));
}

#[test]
fn the_keyboard_never_left_has_no_target() {
    let fake = desk();
    fake.with(|s| (s.cursor, s.kept) = (Some(ON_KB), None));
    assert_eq!(run(&fake).aim(), Err(ScrollError::NoTarget));
    let full = Rect { h: 1040, ..DESK };
    let covered = fake.with(|s| Kept::new(&[], ON_KB, full, &s.screens));
    fake.with(|s| s.kept = Some(covered));
    assert_eq!(run(&fake).aim(), Err(ScrollError::NoTarget), "no push fits");
    assert!(fake.with(|s| s.reads.is_empty()));
}

#[test]
fn an_unknown_pointer_is_refused_and_no_window_is_gone() {
    let fake = desk();
    fake.with(|s| s.cursor = None);
    assert_eq!(run(&fake).aim(), Err(ScrollError::Refused));
    fake.with(|s| (s.cursor, s.windows) = (Some(ON_APP), Vec::new()));
    assert_eq!(run(&fake).aim(), Err(ScrollError::Gone));
}

#[test]
fn our_list_under_the_pointer_is_ours_and_gets_no_read_or_step() {
    let fake = desk();
    show_list(&fake, LIST);
    fake.with(|s| s.cursor = Some(Point { x: 1700, y: 200 }));
    let mut r = run(&fake);
    let aim = r.aim().unwrap();
    assert!(aim.ours && aim.on_pointer);
    assert_eq!((aim.id, aim.area), (TargetId::new(LIST_ID), Some(LIST)));
    assert_eq!(r.turn(&aim, Dir::Down, NOTCH_LINES), Ok(()));
    assert_eq!(r.reach(&aim, Dir::Down), Reach::Unknown);
    fake.with(|s| {
        assert!(s.reads.is_empty() && s.posted.is_empty() && s.sent.is_empty());
    });
}

#[test]
fn a_turn_posts_off_the_pointer_and_sends_on_it() {
    let fake = desk();
    let mut r = run(&fake);
    let on = r.aim().unwrap();
    r.turn(&on, Dir::Down, NOTCH_LINES).unwrap();
    fake.with(|s| s.cursor = Some(ON_KB));
    let off = r.aim().unwrap();
    r.turn(&off, Dir::Up, NOTCH_LINES).unwrap();
    let notch = i32::try_from(NOTCH).unwrap();
    fake.with(|s| {
        assert_eq!(s.sent, [(Dir::Down, -notch)]);
        assert_eq!(s.posted, [(APP_ID, BEFORE, Dir::Up, notch)]);
    });
}

#[test]
fn a_pointer_gone_from_its_target_since_the_aim_gets_a_post_not_a_send() {
    let fake = desk();
    let mut r = run(&fake);
    let aim = r.aim().unwrap();
    fake.with(|s| s.cursor = Some(ON_KB));
    r.turn(&aim, Dir::Down, NOTCH_LINES).unwrap();
    let notch = i32::try_from(NOTCH).unwrap();
    fake.with(|s| {
        assert!(s.sent.is_empty(), "nothing goes to the keyboard");
        assert_eq!(s.posted, [(APP_ID, ON_APP, Dir::Down, -notch)]);
    });
}

#[test]
fn a_hung_window_gets_no_step_and_none_pile_up_for_after() {
    let fake = desk();
    let mut r = run(&fake);
    fake.with(|s| s.cursor = Some(ON_KB));
    let aim = r.aim().unwrap();
    fake.with(|s| s.hung.push(APP_ID));
    for _ in 0..5 {
        assert_eq!(r.turn(&aim, Dir::Down, NOTCH_LINES), Ok(()));
    }
    assert!(fake.with(|s| s.posted.is_empty()));
    fake.with(|s| s.hung.clear());
    r.turn(&aim, Dir::Down, NOTCH_LINES).unwrap();
    let one = -i32::try_from(NOTCH).unwrap();
    assert_eq!(
        fake.with(|s| s.posted.clone()),
        [(APP_ID, BEFORE, Dir::Down, one)]
    );
}

#[test]
fn a_closed_target_is_gone_and_a_refusal_is_told() {
    let fake = desk();
    fake.with(|s| s.cursor = Some(ON_KB));
    let mut r = run(&fake);
    let aim = r.aim().unwrap();
    fake.with(|s| s.post_error = Some(ScrollError::Refused));
    assert_eq!(
        r.turn(&aim, Dir::Down, NOTCH_LINES),
        Err(ScrollError::Refused)
    );
    fake.with(|s| s.gone.push(APP_ID));
    assert_eq!(r.turn(&aim, Dir::Down, NOTCH_LINES), Err(ScrollError::Gone));
    assert_eq!(
        fake.with(|s| s.posted.len()),
        1,
        "nothing goes to a closed window"
    );
}

#[test]
fn ui_automation_steps_go_only_to_a_free_reader() {
    let fake = desk();
    fake.with(|s| (s.wheel, s.busy) = (Some(wheel(0, 0)), true));
    let mut r = run(&fake);
    let aim = r.aim().unwrap();
    for _ in 0..3 {
        r.turn(&aim, Dir::Right, 2.0).unwrap();
    }
    fake.with(|s| s.busy = false);
    r.turn(&aim, Dir::Right, 2.0).unwrap();
    let steps = fake.with(|s| s.steps.clone());
    assert_eq!(steps, [(Seek::At(ON_APP), Dir::Right, 2)]);
    assert!(fake.with(|s| s.sent.is_empty() && s.posted.is_empty()));
}

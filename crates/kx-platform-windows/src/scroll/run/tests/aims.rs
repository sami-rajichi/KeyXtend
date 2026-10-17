//! Where the run aims when our windows are in the way: our open list wins while it shows, a point our other windows
//! now cover aims at the app behind them, read and stepped at that app's own window, and with none behind, or with the
//! pointer on our other windows, there is no target.

use super::*;

/// Our list just above the keyboard, as it opens, with a gap of 10 pixels between them over the desktop.
const ABOVE: Rect = Rect {
    x: 400,
    y: 400,
    w: 400,
    h: 290,
};
/// A point in that gap.
const GAP: Point = Point { x: 500, y: 695 };
/// A point on our banner.
const ON_BANNER: Point = Point { x: 1700, y: 500 };

/// `fake` with our window `id`, `whose` it is, laid over the kept point `BEFORE` in front of every other.
fn cover(fake: &Fake, id: u64, whose: Whose) {
    let over = Rect {
        x: BEFORE.x - 50,
        y: BEFORE.y - 50,
        w: 100,
        h: 100,
    };
    fake.with(|s| s.windows.insert(0, (over, Spot { window: id, whose })));
}

/// The aim with the pointer on the keyboard.
fn from_keyboard(fake: &Fake) -> Result<Aim, ScrollError> {
    fake.with(|s| s.cursor = Some(ON_KB));
    run(fake).aim()
}

/// Passes over the points on `fake`'s windows of ours that do not page, as the guard does at an entry.
fn passed(fake: &Fake) -> impl Fn(Point) -> bool + '_ {
    |p| fake.spot(p).is_some_and(|s| s.whose.passed())
}

/// True when `aim` is our list's: ours, off the pointer and with no box, so a tap turns its page.
fn pages_the_list(aim: &Aim) -> bool {
    aim.ours && !aim.on_pointer && aim.area.is_none() && aim.id == TargetId::new(LIST_ID)
}

#[test]
fn an_entry_from_our_list_keeps_its_point_and_a_tap_pages_the_list() {
    let fake = desk();
    show_list(&fake, LIST);
    let (entry, on_list) = (Point { x: 1450, y: 750 }, Point { x: 1700, y: 200 });
    let history = [(entry.x, entry.y), (on_list.x, on_list.y)];
    let screens = fake.with(|s| s.screens.clone());
    let kept = Kept::past(&history, entry, KB, &screens, passed(&fake));
    assert_eq!(kept.outside, Some(on_list), "the list is kept");
    fake.with(|s| s.kept = Some(kept));
    assert!(pages_the_list(&from_keyboard(&fake).unwrap()));
}

/// The pointer comes down from the list through the gap above the keyboard: the entry keeps the gap's point, on the
/// desktop, yet the tap pages the list.
#[test]
fn a_pointer_from_our_list_through_the_gap_keeps_the_gap_yet_a_tap_pages_the_list() {
    let fake = desk();
    show_list(&fake, ABOVE);
    let entry = Point { x: 500, y: 701 };
    let history = [(entry.x, entry.y), (GAP.x, GAP.y), (500, 600)];
    let screens = fake.with(|s| s.screens.clone());
    let kept = Kept::past(&history, entry, KB, &screens, passed(&fake));
    assert_eq!(kept.outside, Some(GAP), "the gap is not ours");
    fake.with(|s| s.kept = Some(kept));
    assert!(pages_the_list(&from_keyboard(&fake).unwrap()));
}

#[test]
fn our_list_while_it_shows_wins_over_a_point_kept_on_the_app() {
    let fake = desk();
    show_list(&fake, LIST);
    assert!(pages_the_list(&from_keyboard(&fake).unwrap()));
    assert!(fake.with(|s| s.reads.is_empty() && s.pings.is_empty()));
}

#[test]
fn an_entry_over_our_banner_keeps_the_app_before_it() {
    let fake = desk();
    let entry = Point { x: 1450, y: 750 };
    let history = [
        (entry.x, entry.y),
        (ON_BANNER.x, ON_BANNER.y),
        (ON_APP.x, ON_APP.y),
    ];
    let screens = fake.with(|s| s.screens.clone());
    let kept = Kept::past(&history, entry, KB, &screens, passed(&fake));
    assert_eq!(kept.outside, Some(ON_APP), "the banner is passed over");
}

#[test]
fn a_kept_point_the_keyboard_or_the_banner_now_covers_aims_at_the_app_behind() {
    for (id, whose) in [(KB_ID, Whose::Keyboard), (BANNER_ID, Whose::Ours)] {
        let fake = desk();
        cover(&fake, id, whose);
        let aim = from_keyboard(&fake).unwrap();
        let want = (BEFORE, false, false, TargetId::new(APP_ID), Some(BOX));
        let got = (aim.at, aim.on_pointer, aim.ours, aim.id, aim.area);
        assert_eq!(got, want, "{whose:?}");
    }
}

/// UI Automation at the point would find our window there, so it starts at the app's own window.
#[test]
fn a_covered_target_is_read_and_stepped_at_its_own_window() {
    let fake = desk();
    cover(&fake, BANNER_ID, Whose::Ours);
    fake.with(|s| s.wheel = Some(wheel(0, 0)));
    let mut r = run(&fake);
    fake.with(|s| s.cursor = Some(ON_KB));
    let aim = r.aim().unwrap();
    r.turn(&aim, Dir::Down, 2.0).unwrap();
    let app = Seek::Window(APP_ID);
    fake.with(|s| {
        assert_eq!(s.reads, [(app, Dir::Down, cap())]);
        assert_eq!(s.steps, [(app, Dir::Down, 2)]);
    });
}

#[test]
fn a_kept_point_our_list_now_covers_pages_the_list() {
    let fake = desk();
    cover(&fake, LIST_ID, Whose::Pages);
    assert!(pages_the_list(&from_keyboard(&fake).unwrap()));
}

#[test]
fn a_covered_kept_point_with_no_app_behind_has_no_target() {
    let fake = desk();
    fake.with(|s| s.windows.retain(|w| w.1.whose != Whose::Theirs));
    cover(&fake, BANNER_ID, Whose::Ours);
    assert_eq!(from_keyboard(&fake), Err(ScrollError::NoTarget));
    assert!(fake.with(|s| s.reads.is_empty() && s.pings.is_empty()));
}

/// Just after the keyboard shows or the module starts, the keyboard reads as ours until its watch holds, and nothing is
/// kept yet: the user is asked to point first, never a quiet page turn.
#[test]
fn our_window_under_the_pointer_with_no_point_kept_has_no_target() {
    let fake = desk();
    let unwatched = Spot {
        window: KB_ID,
        whose: Whose::Ours,
    };
    fake.with(|s| (s.windows[0].1, s.kept) = (unwatched, None));
    assert_eq!(from_keyboard(&fake), Err(ScrollError::NoTarget));
}

/// A tap that reaches the pad while the pointer rests on our banner, as through Voice Access, never turns a page
/// quietly: it asks the user to point first, unless our list shows.
#[test]
fn our_banner_under_the_pointer_has_no_target_unless_our_list_shows() {
    let fake = desk();
    fake.with(|s| s.cursor = Some(ON_BANNER));
    assert_eq!(run(&fake).aim(), Err(ScrollError::NoTarget));
    show_list(&fake, ABOVE);
    assert!(pages_the_list(&run(&fake).aim().unwrap()));
    assert!(fake.with(|s| s.reads.is_empty()));
}

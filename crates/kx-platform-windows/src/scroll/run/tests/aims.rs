//! Where the run aims when our windows are in the way: a point kept on our list pages it, one our other windows now
//! cover aims at the app behind them, and with none behind, or no point kept yet, there is no target.

use super::*;

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

#[test]
fn an_entry_from_our_list_keeps_its_point_and_a_tap_pages_the_list() {
    let fake = desk();
    let (entry, on_list) = (Point { x: 1450, y: 750 }, Point { x: 1700, y: 200 });
    let history = [(entry.x, entry.y), (on_list.x, on_list.y)];
    let screens = fake.with(|s| s.screens.clone());
    let kept = Kept::past(&history, entry, KB, &screens, passed(&fake));
    assert_eq!(kept.outside, Some(on_list), "the list is kept");
    fake.with(|s| s.kept = Some(kept));
    let aim = from_keyboard(&fake).unwrap();
    assert!(aim.ours && !aim.on_pointer && aim.area.is_none());
    assert_eq!((aim.at, aim.id), (on_list, TargetId::new(LIST_ID)));
}

#[test]
fn an_entry_over_our_banner_keeps_the_app_before_it() {
    let fake = desk();
    let entry = Point { x: 1450, y: 750 };
    let history = [(entry.x, entry.y), (1700, 500), (ON_APP.x, ON_APP.y)];
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

#[test]
fn a_kept_point_our_list_now_covers_pages_the_list() {
    let fake = desk();
    cover(&fake, LIST_ID, Whose::Pages);
    let aim = from_keyboard(&fake).unwrap();
    assert!(aim.ours && !aim.on_pointer && aim.area.is_none());
    assert_eq!(aim.id, TargetId::new(LIST_ID));
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

/// A glide that crosses our banner keeps its target: the aim is ours, which the module passes over.
#[test]
fn our_banner_under_the_pointer_while_a_point_is_kept_is_ours() {
    let fake = desk();
    fake.with(|s| s.cursor = Some(Point { x: 1700, y: 500 }));
    let aim = run(&fake).aim().unwrap();
    assert!(aim.ours && aim.on_pointer);
    assert_eq!(aim.id, TargetId::new(BANNER_ID));
    assert!(fake.with(|s| s.reads.is_empty()));
}

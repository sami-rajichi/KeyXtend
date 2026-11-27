//! The UI fakes: screens, the guard's record of calls and errors, and backdrops.

use super::*;

const TITLE: &str = "KeyXtend";
/// The window a guard run puts in front.
const TOP: &str = "KeyXtend menu";
/// The windows that go just under the keyboard.
const OUTLINE: &str = "KeyXtend outline";
/// The open list, which pages.
const LIST: &str = "KeyXtend list";
/// The frozen screens, which go under our other windows.
const SCREEN: &str = "KeyXtend frozen screen";

fn area() -> Screen {
    let rect = Rect {
        x: 0,
        y: 0,
        w: 1920,
        h: 1040,
    };
    Screen {
        rect,
        dpi: 144,
        primary: true,
    }
}

#[test]
fn screens_report_what_the_test_set() {
    let fake = FakeScreens::new(vec![area()], Point { x: 5, y: 6 });
    assert_eq!(fake.work_areas(), [area()]);
    assert_eq!(fake.cursor(), Ok(Point { x: 5, y: 6 }));
    fake.set_cursor(Point { x: 7, y: 8 });
    fake.set_areas(Vec::new());
    assert_eq!(fake.cursor(), Ok(Point { x: 7, y: 8 }));
    assert!(fake.work_areas().is_empty());
}

#[test]
fn the_guard_records_every_call_in_order() {
    let fake = FakeWindowGuard::new(3);
    let rects = [area().rect];
    assert_eq!(fake.guard_own(&[TITLE], Some(TOP)), Ok(3));
    assert_eq!(fake.shape(TITLE, &rects), Ok(()));
    assert_eq!(fake.lift_alone(TITLE), Ok(()));
    assert_eq!(fake.watch_entries(TITLE), Ok(Watched::Held));
    fake.unwatch_entries(TITLE);
    assert_eq!(fake.lay_under(OUTLINE, TITLE), Ok(()));
    assert_eq!(fake.lay_low(SCREEN), Ok(()));
    assert_eq!(fake.mark_pages(LIST), Ok(()));
    let first = fake.remember();
    let second = fake.remember();
    assert_ne!(first, second);
    fake.give_back(first);
    let want = [
        GuardCall::GuardOwn(vec![TITLE.into()], Some(TOP.into())),
        GuardCall::Shape(TITLE.into(), rects.to_vec()),
        GuardCall::LiftAlone(TITLE.into()),
        GuardCall::WatchEntries(TITLE.into()),
        GuardCall::UnwatchEntries(TITLE.into()),
        GuardCall::LayUnder(OUTLINE.into(), TITLE.into()),
        GuardCall::LayLow(SCREEN.into()),
        GuardCall::MarkPages(LIST.into()),
        GuardCall::Remember(first),
        GuardCall::Remember(second),
        GuardCall::GiveBack(first),
    ];
    assert_eq!(fake.calls(), want);
}

#[test]
fn the_guard_count_and_errors_follow_the_test_and_every_try_is_recorded() {
    let fake = FakeWindowGuard::new(0);
    assert_eq!(fake.guard_own(&[], None), Ok(0));
    fake.set_error(Some(UiError::Refused(5)));
    assert_eq!(fake.guard_own(&[], None), Err(UiError::Refused(5)));
    assert_eq!(fake.shape(TITLE, &[]), Err(UiError::Refused(5)));
    assert_eq!(fake.lift_alone(TITLE), Err(UiError::Refused(5)));
    assert_eq!(fake.watch_entries(TITLE), Err(UiError::Refused(5)));
    assert_eq!(fake.lay_under(OUTLINE, TITLE), Err(UiError::Refused(5)));
    assert_eq!(fake.lay_low(SCREEN), Err(UiError::Refused(5)));
    assert_eq!(fake.mark_pages(LIST), Err(UiError::Refused(5)));
    fake.set_error(None);
    fake.set_count(2);
    assert_eq!(fake.guard_own(&[], None), Ok(2));
    let tries = fake
        .calls()
        .iter()
        .filter(|c| **c == GuardCall::GuardOwn(Vec::new(), None))
        .count();
    assert_eq!(tries, 3);
}

#[test]
fn a_watch_finds_the_window_unguarded_while_the_test_says_and_is_still_recorded() {
    let fake = FakeWindowGuard::new(0);
    fake.set_unguarded(true);
    assert_eq!(fake.watch_entries(TITLE), Ok(Watched::Unguarded));
    fake.set_error(Some(UiError::Refused(4)));
    assert_eq!(
        fake.watch_entries(TITLE),
        Err(UiError::Refused(4)),
        "a refusal first"
    );
    fake.set_error(None);
    fake.set_unguarded(false);
    assert_eq!(fake.watch_entries(TITLE), Ok(Watched::Held));
    assert_eq!(fake.calls(), vec![GuardCall::WatchEntries(TITLE.into()); 3]);
}

#[test]
fn shields_are_recorded_by_title_and_the_shielded_ones_are_kept() {
    let fake = FakeWindowGuard::new(0);
    assert!(fake.shielded().is_empty());
    assert_eq!(fake.shield(TITLE, true), Ok(()));
    assert_eq!(fake.shield(TOP, true), Ok(()));
    assert_eq!(fake.shield(TITLE, false), Ok(()));
    assert_eq!(fake.shielded(), [TOP]);
    let want = [
        GuardCall::Shield(TITLE.into(), true),
        GuardCall::Shield(TOP.into(), true),
        GuardCall::Shield(TITLE.into(), false),
    ];
    assert_eq!(fake.calls(), want);
}

#[test]
fn a_shield_alone_can_be_refused_and_is_still_recorded() {
    let fake = FakeWindowGuard::new(1);
    fake.set_shield_error(Some(UiError::NoWindow));
    assert_eq!(fake.shield(TITLE, true), Err(UiError::NoWindow));
    assert_eq!(fake.guard_own(&[], None), Ok(1), "the rest still work");
    fake.set_shield_error(None);
    fake.set_error(Some(UiError::Refused(3)));
    assert_eq!(fake.shield(TOP, true), Err(UiError::Refused(3)));
    fake.set_error(None);
    assert!(fake.shielded().is_empty(), "a refused shield keeps nothing");
    assert_eq!(fake.shield(TOP, true), Ok(()));
    assert_eq!(fake.shielded(), [TOP]);
    let shields = fake
        .calls()
        .iter()
        .filter(|c| matches!(c, GuardCall::Shield(..)))
        .count();
    assert_eq!(shields, 3, "refused ones count");
}

#[test]
fn a_refused_shield_lets_its_title_back_in_as_windows_does() {
    let fake = FakeWindowGuard::new(0);
    assert_eq!(fake.shield(TITLE, true), Ok(()));
    assert_eq!(fake.shield(TOP, true), Ok(()));
    fake.set_shield_error(Some(UiError::Refused(5)));
    assert_eq!(fake.shield(TITLE, true), Err(UiError::Refused(5)));
    assert_eq!(fake.shielded(), [TOP], "none of that title stays shielded");
    assert_eq!(fake.shield(TOP, false), Err(UiError::Refused(5)));
    assert!(
        fake.shielded().is_empty(),
        "an unshield refused lets it in too"
    );
    fake.set_shield_error(None);
    assert_eq!(fake.shield(TITLE, true), Ok(()));
    fake.set_shield_error(Some(UiError::NoWindow));
    assert_eq!(fake.shield(TITLE, true), Err(UiError::NoWindow));
    assert_eq!(fake.shielded(), [TITLE], "no window there, nothing changed");
}

#[test]
fn a_shield_every_call_refuses_lets_its_title_back_in_too() {
    let fake = FakeWindowGuard::new(0);
    assert_eq!(fake.shield(TITLE, true), Ok(()));
    fake.set_error(Some(UiError::Refused(5)));
    assert_eq!(fake.shield(TITLE, true), Err(UiError::Refused(5)));
    assert!(fake.shielded().is_empty(), "all or none, as Windows does");
}

#[test]
fn a_shield_finds_no_window_of_an_absent_title_and_keeps_the_rest() {
    let fake = FakeWindowGuard::new(0);
    fake.set_absent(&[OUTLINE]);
    assert_eq!(fake.shield(OUTLINE, true), Err(UiError::NoWindow));
    assert_eq!(fake.shield(TITLE, true), Ok(()));
    assert_eq!(fake.shielded(), [TITLE]);
    assert_eq!(fake.calls().len(), 2, "both recorded");
    fake.set_absent(&[]);
    assert_eq!(fake.shield(OUTLINE, true), Ok(()));
    assert_eq!(fake.shielded(), [TITLE, OUTLINE]);
}

#[test]
fn a_shield_of_one_refused_title_lets_its_windows_back_in_and_keeps_the_rest() {
    let fake = FakeWindowGuard::new(0);
    assert_eq!(fake.shield(OUTLINE, true), Ok(()));
    fake.set_refused(&[OUTLINE], UiError::Refused(5));
    assert_eq!(fake.shield(OUTLINE, true), Err(UiError::Refused(5)));
    assert_eq!(fake.shield(TITLE, true), Ok(()));
    assert_eq!(
        fake.shielded(),
        [TITLE],
        "none of that title stays shielded"
    );
    fake.set_absent(&[]);
    assert_eq!(
        fake.shield(OUTLINE, true),
        Ok(()),
        "in place of the refusal"
    );
}

#[test]
fn the_cursor_can_be_unknown() {
    let fake = FakeScreens::new(Vec::new(), Point::default());
    fake.set_cursor_error(Some(UiError::Refused(6)));
    assert_eq!(fake.cursor(), Err(UiError::Refused(6)));
    fake.set_cursor_error(None);
    assert_eq!(fake.cursor(), Ok(Point::default()));
}

#[test]
fn a_backdrop_is_recorded_or_refused() {
    let fake = FakeBackdrop::new();
    assert_eq!(fake.apply(TITLE, BackdropKind::Mica, true), Ok(()));
    fake.set_error(Some(UiError::Refused(1)));
    let refused = fake.apply(TITLE, BackdropKind::Acrylic, false);
    assert_eq!(refused, Err(UiError::Refused(1)));
    assert_eq!(fake.applied(), [(TITLE.into(), BackdropKind::Mica, true)]);
}

#[test]
fn a_focus_token_is_opaque() {
    let fake = FakeWindowGuard::new(0);
    assert_eq!(fake.remember(), FocusToken::new(1));
}

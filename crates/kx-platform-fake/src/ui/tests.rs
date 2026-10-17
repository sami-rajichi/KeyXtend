//! The UI fakes: screens, the guard's record of calls and errors, and backdrops.

use super::*;

const TITLE: &str = "KeyXtend";
/// The window a guard run puts in front.
const TOP: &str = "KeyXtend menu";
/// The windows that go just under the keyboard.
const OUTLINE: &str = "KeyXtend outline";
/// The open list, which pages.
const LIST: &str = "KeyXtend list";

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
    assert_eq!(fake.guard_own(Some(TITLE), Some(TOP)), Ok(3));
    assert_eq!(fake.shape(TITLE, &rects), Ok(()));
    assert_eq!(fake.lift_alone(TITLE), Ok(()));
    assert_eq!(fake.watch_entries(TITLE), Ok(Watched::Held));
    fake.unwatch_entries(TITLE);
    assert_eq!(fake.lay_under(OUTLINE, TITLE), Ok(()));
    assert_eq!(fake.mark_pages(LIST), Ok(()));
    let first = fake.remember();
    let second = fake.remember();
    assert_ne!(first, second);
    fake.give_back(first);
    let want = [
        GuardCall::GuardOwn(Some(TITLE.into()), Some(TOP.into())),
        GuardCall::Shape(TITLE.into(), rects.to_vec()),
        GuardCall::LiftAlone(TITLE.into()),
        GuardCall::WatchEntries(TITLE.into()),
        GuardCall::UnwatchEntries(TITLE.into()),
        GuardCall::LayUnder(OUTLINE.into(), TITLE.into()),
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
    assert_eq!(fake.guard_own(None, None), Ok(0));
    fake.set_error(Some(UiError::Refused(5)));
    assert_eq!(fake.guard_own(None, None), Err(UiError::Refused(5)));
    assert_eq!(fake.shape(TITLE, &[]), Err(UiError::Refused(5)));
    assert_eq!(fake.lift_alone(TITLE), Err(UiError::Refused(5)));
    assert_eq!(fake.watch_entries(TITLE), Err(UiError::Refused(5)));
    assert_eq!(fake.lay_under(OUTLINE, TITLE), Err(UiError::Refused(5)));
    assert_eq!(fake.mark_pages(LIST), Err(UiError::Refused(5)));
    fake.set_error(None);
    fake.set_count(2);
    assert_eq!(fake.guard_own(None, None), Ok(2));
    let tries = fake
        .calls()
        .iter()
        .filter(|c| **c == GuardCall::GuardOwn(None, None))
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

//! The page mark on hidden windows of the test thread: only a guarded window takes it, our marked window pages and our
//! others are passed over, and an entry keeps a point on our list but never one on our banner.

use std::sync::PoisonError;

use kx_module_api::ports::{Point, Rect};
use kx_platform::ui::{UiError, WindowGuard};
use windows::Win32::Foundation::HWND;

use super::super::entry::{enter, kept, mark_with};
use super::super::guard::guard;
use super::super::pages::{mark, paging, whose};
use super::entries::{ENTRIES, place, spot_on_main};
use super::{Hidden, WinGuard};
use crate::scroll::Whose;

/// A hidden guarded window titled `title`, at `r`.
fn guarded_at(title: &str, r: Rect) -> Hidden {
    let window = Hidden::new(title);
    place(&window, r);
    guard(window.0).unwrap();
    window
}

#[test]
fn only_our_guarded_window_takes_the_mark_and_then_pages() {
    let plain = Hidden::new("KeyXtend test: page unguarded");
    assert!(matches!(mark(plain.0), Err(UiError::Refused(_))));
    assert_eq!(whose(plain.0), Whose::Ours, "not marked");
    let list = guarded_at("KeyXtend test: page list", spot_on_main());
    mark(list.0).unwrap();
    assert_eq!(whose(list.0), Whose::Pages);
    assert_ne!(paging(), Some(list.0), "only a shown list wins the aim");
    let hidden = WinGuard.mark_pages("KeyXtend test: page list");
    assert_eq!(hidden, Err(UiError::NoWindow), "only a shown window");
    assert_eq!(whose(HWND::default()), Whose::Theirs, "no window");
}

#[test]
fn the_watched_keyboard_stays_the_keyboard_with_the_mark() {
    let _e = ENTRIES.lock().unwrap_or_else(PoisonError::into_inner);
    let kb = guarded_at("KeyXtend test: page keyboard", spot_on_main());
    mark_with(kb.0, false).unwrap();
    mark(kb.0).unwrap();
    assert_eq!(whose(kb.0), Whose::Keyboard);
}

/// The list and the banner sit above the keyboard; the pointer comes down through both, in either order.
#[test]
fn an_entry_keeps_a_point_on_our_list_and_passes_over_our_banner() {
    let _e = ENTRIES.lock().unwrap_or_else(PoisonError::into_inner);
    let kb = spot_on_main();
    let keyboard = guarded_at("KeyXtend test: page entry", kb);
    let (list, banner) = (
        guarded_at("KeyXtend test: page list above", kb),
        guarded_at("KeyXtend test: page banner above", kb),
    );
    mark(list.0).unwrap();
    let entry = Point {
        x: kb.x + 10,
        y: kb.y + 1,
    };
    let (on_list, on_banner) = ((entry.x, kb.y - 3), (entry.x, kb.y - 5));
    let at = move |p: Point| if p.y == on_list.1 { list.0 } else { banner.0 };
    let passed = move |p: Point| whose(at(p)).passed();
    for history in [[on_banner, on_list], [on_list, on_banner]] {
        let points = move |_| vec![(entry.x, entry.y), history[0], history[1]];
        enter(keyboard.0, entry, points, passed);
        let outside = kept().and_then(|k| k.outside).map(|p| p.y);
        assert_eq!(outside, Some(on_list.1), "{history:?}");
    }
}

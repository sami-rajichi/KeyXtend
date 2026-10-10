//! The size the user sets is saved through the module's own settings handle, once per finished change.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]
#![allow(
    clippy::float_cmp,
    reason = "a saved size must read back as exactly the size shown"
)]

use crate::host::{FakeHost, fake, saved, start_settings};
use kx_mod_keyboard::{SizeError, Sizer};
use kx_module_api::WriteError;
use kx_module_api::ports::Point;
use proptest::prelude::*;

/// The size-1 plate in the pointer's pixels.
const PLATE: (f64, f64) = (671.0, 196.0);

fn sizer(host: &mut FakeHost) -> Sizer {
    let settings = start_settings(host);
    Sizer::new(&settings, host.handle())
}

/// The sizes written so far, oldest first.
fn written(host: &FakeHost) -> Vec<f64> {
    let values = host.sink.writes().into_iter().map(|(_, values)| {
        assert_eq!(values.keys().collect::<Vec<_>>(), ["size"]);
        values["size"].as_float().unwrap()
    });
    values.collect()
}

fn at(x: i32, y: i32) -> Point {
    Point { x, y }
}

#[test]
fn each_step_saves_the_new_size() {
    let mut host = fake(None);
    let mut s = sizer(&mut host);
    assert_eq!(s.bigger(), Ok(true));
    assert_eq!(s.smaller(), Ok(true));
    assert_eq!(written(&host), [1.05, 1.0]);
    assert_eq!(saved(&host).size, 1.0);
}

#[test]
fn a_step_past_a_limit_changes_and_saves_nothing() {
    let mut host = fake(Some(("size", 1.8.into())));
    let mut s = sizer(&mut host);
    assert_eq!(s.bigger(), Ok(false));
    assert!(written(&host).is_empty());
}

#[test]
fn a_preset_saves_its_size_and_an_unknown_one_is_refused() {
    let mut host = fake(None);
    let mut s = sizer(&mut host);
    assert_eq!(s.pick(2), Ok(true));
    assert_eq!(s.preset(), Some(2));
    assert_eq!(s.pick(3), Err(SizeError::NoPreset(3)));
    assert_eq!(s.pick(2), Ok(false), "already L");
    assert_eq!(written(&host), [1.5]);
}

#[test]
fn a_resize_saves_once_when_it_ends() {
    let mut host = fake(None);
    let mut s = sizer(&mut host);
    s.begin(at(100, 100), PLATE);
    assert!(s.resizing());
    assert!(s.follow(at(400, 100)));
    assert!(s.follow(at(200, 100)));
    assert!(
        written(&host).is_empty(),
        "nothing saved while the pointer moves"
    );
    let reached = s.size();
    assert_eq!(s.end(), Ok(true));
    assert!(
        !s.resizing() && !s.follow(at(0, 0)),
        "nothing after the end"
    );
    assert_eq!(written(&host), [reached]);
    assert_eq!(s.end(), Ok(false));
}

#[test]
fn a_resize_back_to_where_it_began_saves_nothing() {
    let mut host = fake(None);
    let mut s = sizer(&mut host);
    s.begin(at(100, 100), PLATE);
    assert!(s.follow(at(500, 100)));
    assert!(s.follow(at(100, 100)));
    assert_eq!(s.end(), Ok(false));
    assert!(written(&host).is_empty());
}

#[test]
fn two_clicks_with_no_move_write_nothing_even_on_a_capped_size() {
    let mut host = fake(Some(("size", 1.5.into())));
    let mut s = sizer(&mut host);
    assert!(s.fit(1.1), "a laptop caps the big monitor's size");
    s.begin(at(100, 100), PLATE);
    assert_eq!(s.end(), Ok(false));
    assert!(written(&host).is_empty(), "the saved size stays");
    assert_eq!(saved(&host).size, 1.5);
}

#[test]
fn a_size_capped_below_the_floor_is_never_written() {
    let mut host = fake(None);
    let mut s = sizer(&mut host);
    s.begin(at(100, 100), PLATE);
    assert!(s.fit(0.5), "the screen shrank during the resize");
    assert_eq!(s.end(), Ok(true), "it holds for this run");
    s.begin(at(100, 100), PLATE);
    assert!(!s.follow(at(-400, 100)), "the room holds it");
    assert_eq!(s.end(), Ok(false));
    assert!(
        written(&host).is_empty(),
        "the settings check would refuse it"
    );
}

#[test]
fn a_resize_from_a_capped_size_saves_the_size_it_reached() {
    let mut host = fake(Some(("size", 1.8.into())));
    let mut s = sizer(&mut host);
    assert!(s.fit(1.3));
    s.begin(at(400, 100), PLATE);
    assert!(s.follow(at(300, 100)));
    s.begin(at(300, 100), PLATE);
    assert!(!s.follow(at(300, 100)));
    let reached = s.size();
    assert_eq!(s.end(), Ok(true), "a second start ends against the first");
    assert_eq!(written(&host), [reached]);
}

#[test]
fn a_step_during_a_resize_still_leaves_the_size_shown_saved() {
    let mut host = fake(None);
    let mut s = sizer(&mut host);
    s.begin(at(0, 0), PLATE);
    assert!(s.follow(at(-450, 0)));
    assert_eq!(s.smaller(), Ok(false), "at the floor");
    assert_eq!(s.end(), Ok(true));
    assert_eq!(saved(&host).size, s.size());
    s.begin(at(0, 0), PLATE);
    assert_eq!(s.pick(1), Ok(true), "saved at once");
    assert!(s.follow(at(0, 0)), "back to where the resize began");
    assert_eq!(s.end(), Ok(true));
    assert_eq!(saved(&host).size, s.size());
}

#[test]
fn a_size_that_cannot_be_saved_still_holds_for_this_run() {
    let mut host = fake(None);
    let mut s = sizer(&mut host);
    host.sink.refuse(Some(WriteError::Unsaved));
    assert_eq!(s.bigger(), Err(SizeError::Unsaved(WriteError::Unsaved)));
    assert_eq!(s.size(), 1.05);
    host.sink.refuse(None);
    assert_eq!(s.bigger(), Ok(true));
    assert_eq!(written(&host), [1.1]);
}

#[test]
fn after_an_unsaved_size_a_step_back_is_the_size_stop_saves() {
    let mut host = fake(None);
    let mut s = sizer(&mut host);
    host.sink.refuse(Some(WriteError::Unsaved));
    let unsaved = Err(SizeError::Unsaved(WriteError::Unsaved));
    assert_eq!(s.bigger(), unsaved);
    assert_eq!(saved(&host).size, 1.05, "the store holds it");
    assert_eq!(s.smaller(), unsaved, "the step back is written too");
    assert_eq!(saved(&host).size, 1.0, "the last size");
}

#[test]
fn a_fitted_size_holds_unsaved_and_no_step_grows_it_past_the_room() {
    let mut host = fake(Some(("size", 1.8.into())));
    let mut s = sizer(&mut host);
    assert!(s.fit(1.3));
    assert_eq!((s.size(), s.preset()), (1.3, None));
    assert_eq!(s.bigger(), Ok(false));
    assert_eq!(s.pick(2), Ok(false), "L does not fit");
    assert!(written(&host).is_empty(), "the saved size stays");
    assert_eq!(s.smaller(), Ok(true));
    assert_eq!(written(&host), [1.25]);
    assert!(!s.fit(2.0), "more room grows nothing");
    assert!(s.fit(0.5) && s.size() == 0.5, "the room wins over size_min");
}

#[test]
fn it_starts_at_the_saved_size() {
    let mut host = fake(Some(("size", 1.25.into())));
    let s = sizer(&mut host);
    assert_eq!((s.size(), s.preset()), (1.25, Some(1)));
    assert_eq!(s.limits().presets(), [1.0, 1.25, 1.5]);
}

proptest! {
    #[test]
    fn the_saved_size_always_matches_the_size_shown(ops in prop::collection::vec((0u8..6, 0i32..900), 0..32)) {
        let mut host = fake(None);
        let mut s = sizer(&mut host);
        for (op, n) in ops {
            match op {
                0 => { s.bigger().unwrap(); }
                1 => { s.smaller().unwrap(); }
                2 => { let _ = s.pick(usize::try_from(n % 4).unwrap()); }
                3 => s.begin(at(0, 0), PLATE),
                4 => { s.follow(at(n - 450, 0)); }
                _ => { s.end().unwrap(); }
            }
            if !s.resizing() {
                prop_assert_eq!(saved(&host).size, s.size());
            }
        }
    }
}

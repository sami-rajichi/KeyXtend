//! Tests of `video`.

use std::cell::Cell;

use super::*;

/// What `close_until` gives over `tries` clicks when the strip goes after click `goes`, never with `None`, and the
/// clicks it made.
fn closed(tries: usize, goes: Option<usize>) -> (Result<usize, String>, usize) {
    let clicks = Cell::new(0);
    let click = || {
        clicks.set(clicks.get() + 1);
        Ok(())
    };
    let got = close_until(
        tries,
        click,
        || true,
        || goes.is_some_and(|n| clicks.get() >= n),
    );
    (got, clicks.get())
}

#[test]
fn a_close_the_strip_takes_ends_at_one_click() {
    assert_eq!(closed(3, Some(1)), (Ok(1), 1));
}

#[test]
fn a_close_that_lands_before_the_copy_told_its_end_is_clicked_again() {
    assert_eq!(closed(3, Some(2)), (Ok(2), 2));
    assert_eq!(closed(3, Some(3)), (Ok(3), 3), "the last try still counts");
}

#[test]
fn a_strip_that_never_goes_fails_after_the_last_try_naming_it() {
    let (got, clicks) = closed(3, None);
    assert_eq!(clicks, 3);
    let e = got.expect_err("the strip stayed");
    assert!(e.contains("3 clicks"), "{e}");
}

#[test]
fn a_click_that_fails_stops_at_once() {
    let asked = Cell::new(false);
    let got = close_until(
        3,
        || Err("covered".to_string()),
        || true,
        || {
            asked.set(true);
            true
        },
    );
    assert_eq!(got, Err("covered".to_string()));
    assert!(!asked.get(), "no wait after a failed click");
}

#[test]
fn a_strip_gone_just_after_the_wait_takes_no_more_clicks() {
    let clicks = Cell::new(0);
    let click = || {
        clicks.set(clicks.get() + 1);
        if clicks.get() > 1 {
            return Err("Close did not show on top".to_string());
        }
        Ok(())
    };
    let got = close_until(3, click, || false, || false);
    assert_eq!((got, clicks.get()), (Ok(1), 1), "a retry finds no Close");
}

//! The keyboard steps aside for a capture and comes back: a `Wake` tells QML, the mode says how, the keyboard and every
//! window of ours are kept out of every capture while `keyboard` is Bubble or Hidden, the module hears once the Qt
//! thread shielded for that aside, and every end brings it back (edges 16 and 17).

use super::{Heard, heard, idle, picking, screens, strip};
use crate::Rig;
use kx_module_api::ports::{Keep, SnipAside, SnipClick, SnipNote, SnipStage};
use kx_platform::ui::UiError;
use kx_ui::i18n::keys::{TRAY_WINDOWS, WINDOW_BUBBLE, WINDOW_KEYBOARD, WINDOWS};
use kx_ui::{Mode, Wake};
use serde_json::{Value, json};
use std::cell::RefCell;

/// The mode as QML reads it.
pub fn mode(rig: &Rig) -> Value {
    serde_json::from_str(&rig.hub.mode_json()).unwrap()
}

/// The keyboard's and the bubble's titles, which the aside makes first.
fn own(rig: &Rig) -> Vec<String> {
    let t = rig.hub.i18n();
    vec![t.text(WINDOW_KEYBOARD), t.text(WINDOW_BUBBLE)]
}

/// Every title of ours, each in its window's language, in order.
pub fn ours(rig: &Rig) -> Vec<String> {
    let (t, tray) = (rig.hub.i18n(), rig.hub.tray_i18n());
    let kept = WINDOWS.iter().map(|k| t.text(k));
    let mut all: Vec<String> = kept
        .chain(TRAY_WINDOWS.iter().map(|k| tray.text(k)))
        .collect();
    all.sort();
    all
}

/// The titles shielded now, in order.
pub fn shielded(rig: &Rig) -> Vec<String> {
    let mut on = rig.guard.shielded();
    on.sort();
    on
}

/// The Qt thread's work at an aside's wake: the keyboard and its bubble are made, then every window of ours shielded or
/// let back in; the titles made.
pub fn settle(rig: &Rig) -> Vec<String> {
    let made = RefCell::new(Vec::new());
    let _ = rig
        .hub
        .shield_aside(&|title| made.borrow_mut().push(title.to_owned()));
    made.into_inner()
}

/// A snip starts with `aside`, which the Qt thread settles.
fn start(rig: &Rig, heard: &Heard, aside: SnipAside) {
    heard.aside(aside);
    settle(rig);
    heard.shown(rig, picking(1, screens()));
}

/// The snip ends: the keyboard comes back, then nothing shows.
fn end(rig: &Rig, heard: &Heard) {
    heard.aside(SnipAside::Back);
    settle(rig);
    heard.shown(rig, idle(9));
}

#[test]
fn a_wake_steps_the_keyboard_into_its_bubble_and_back() {
    let rig = Rig::new();
    let heard = heard(&rig);
    rig.wakes();
    heard.aside(SnipAside::Bubble);
    assert_eq!(rig.wakes(), [Wake::Aside]);
    assert_eq!(
        mode(&rig),
        json!({"away": false, "minimised": false, "aside": "bubble"})
    );
    rig.hub.keep_mode(Mode::Minimised);
    rig.wakes();
    heard.aside(SnipAside::Back);
    assert_eq!(rig.wakes(), [Wake::Aside]);
    assert_eq!(
        mode(&rig),
        json!({"away": false, "minimised": false, "aside": null})
    );
    assert_eq!(rig.hub.mode(), Mode::Shown, "back as it was");
}

#[test]
fn shields_follow_keyboard() {
    for (aside, kept) in [
        (Some(SnipAside::Bubble), true),
        (Some(SnipAside::Hidden), true),
        (None, false),
    ] {
        let rig = Rig::new();
        let heard = heard(&rig);
        if let Some(aside) = aside {
            heard.aside(aside);
        }
        let made = settle(&rig);
        let want = if kept { ours(&rig) } else { Vec::new() };
        assert_eq!(shielded(&rig), want, "{aside:?}: out of every capture");
        if kept {
            assert_eq!(
                made,
                own(&rig),
                "made before shielded, so a bubble never shown is kept out too"
            );
            heard.aside(SnipAside::Back);
            settle(&rig);
        }
        assert!(shielded(&rig).is_empty(), "{aside:?}: in the picture again");
    }
}

#[test]
fn bubble_a_click_on_it_brings_the_keyboard_back_still_out_of_the_video() {
    let rig = Rig::new();
    let heard = heard(&rig);
    start(&rig, &heard, SnipAside::Bubble);
    rig.hub.keep_mode(Mode::Minimised);
    rig.hub.keep_mode(Mode::Shown);
    settle(&rig);
    assert_eq!(
        shielded(&rig),
        ours(&rig),
        "edge 16: still out of the video"
    );
    assert_eq!(mode(&rig)["aside"], json!("bubble"));
    end(&rig, &heard);
    assert_eq!(rig.hub.mode(), Mode::Shown);
    assert!(shielded(&rig).is_empty());
}

#[test]
fn hidden_shows_nothing_of_ours_and_shown_stays_in_the_picture() {
    let rig = Rig::new();
    let heard = heard(&rig);
    start(&rig, &heard, SnipAside::Hidden);
    assert_eq!(
        mode(&rig)["aside"],
        json!("hidden"),
        "edge 16: neither the keyboard nor its bubble"
    );
    assert_eq!(rig.hub.mode(), Mode::Shown, "the user's own mode stays");
    end(&rig, &heard);
    assert_eq!(mode(&rig)["aside"], json!(null));
    let shown = Rig::new();
    let heard = super::heard(&shown);
    heard.shown(&shown, picking(1, screens()));
    settle(&shown);
    assert_eq!(
        mode(&shown)["aside"],
        json!(null),
        "edge 16: Shown is in the picture"
    );
    assert!(shown.guard.shielded().is_empty());
}

#[test]
fn the_qt_thread_tells_the_module_once_it_shielded_for_an_aside() {
    for aside in [SnipAside::Bubble, SnipAside::Hidden] {
        let rig = Rig::new();
        let heard = heard(&rig);
        heard.aside(aside);
        let on = rig.hub.shield_aside(&|_| {});
        rig.hub.aside_shown(on);
        assert_eq!(heard.sent.clicks(), [SnipClick::Aside], "{aside:?}");
        heard.aside(SnipAside::Back);
        let on = rig.hub.shield_aside(&|_| {});
        rig.hub.aside_shown(on);
        assert_eq!(heard.sent.clicks(), [], "{aside:?}: back is no step aside");
    }
}

#[test]
fn a_stay_keeps_the_keyboard_and_answers_once_with_the_view_told_before_it() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, idle(7));
    rig.wakes();
    heard.aside(SnipAside::Stay);
    assert_eq!(rig.wakes(), [Wake::Aside]);
    assert_eq!(mode(&rig)["aside"], json!(null), "it stays");
    let on = rig.hub.shield_aside(&|_| {});
    rig.hub.aside_shown(on);
    assert!(shielded(&rig).is_empty());
    let rounds: Vec<u32> = heard.sent.acts().iter().map(|a| a.round).collect();
    assert_eq!(rounds, [7], "after the view that hid Snip's windows");
    let on = rig.hub.shield_aside(&|_| {});
    rig.hub.aside_shown(on);
    assert_eq!(heard.sent.clicks(), [], "once");
}

#[test]
fn an_aside_that_comes_after_its_wake_let_ours_back_in_waits_for_its_own_wake() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.aside(SnipAside::Bubble);
    settle(&rig);
    heard.aside(SnipAside::Back);
    let on = rig.hub.shield_aside(&|_| {});
    heard.aside(SnipAside::Bubble);
    rig.hub.aside_shown(on);
    assert_eq!(heard.sent.clicks(), [], "that wake let ours back in");
    assert!(shielded(&rig).is_empty());
    let on = rig.hub.shield_aside(&|_| {});
    rig.hub.aside_shown(on);
    assert_eq!(
        heard.sent.clicks(),
        [SnipClick::Aside],
        "the new aside's own wake"
    );
}

#[test]
fn interleaved_asides_keep_the_mode_from_before_the_first() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.aside(SnipAside::Bubble);
    rig.hub.keep_mode(Mode::Minimised);
    settle(&rig);
    heard.aside(SnipAside::Hidden);
    settle(&rig);
    assert_eq!(mode(&rig)["aside"], json!("hidden"));
    assert_eq!(shielded(&rig), ours(&rig), "still out of every capture");
    heard.aside(SnipAside::Bubble);
    settle(&rig);
    end(&rig, &heard);
    assert_eq!(rig.hub.mode(), Mode::Shown, "as before the first aside");
    assert!(shielded(&rig).is_empty());
}

#[test]
fn a_bubble_that_cannot_be_kept_out_hides_with_the_keyboard() {
    let rig = Rig::new();
    let heard = heard(&rig);
    rig.guard.set_shield_error(Some(UiError::Refused(5)));
    heard.aside(SnipAside::Bubble);
    settle(&rig);
    assert_eq!(mode(&rig)["aside"], json!("hidden"), "never in a video");
    rig.guard.set_shield_error(None);
    end(&rig, &heard);
    assert_eq!(mode(&rig)["aside"], json!(null));
}

#[test]
fn every_end_brings_the_keyboard_back_as_it_was() {
    let copied = |done| {
        let mut page = strip(5);
        if let SnipStage::Strip(taken) = &mut page.stage {
            taken.done = Some(done);
        }
        page
    };
    let mut fault = strip(5);
    fault.note = Some(SnipNote::RecordFailed);
    let ends = [
        ("copy", copied(Keep::Copy)),
        ("save", copied(Keep::Save)),
        ("close", strip(5)),
        ("cancel", idle(5)),
        ("fault", fault),
    ];
    for before in [Mode::Shown, Mode::Minimised, Mode::Away] {
        for (name, last) in ends.clone() {
            let rig = Rig::new();
            let heard = heard(&rig);
            rig.hub.keep_mode(before);
            start(&rig, &heard, SnipAside::Bubble);
            if before == Mode::Shown {
                rig.hub.keep_mode(Mode::Minimised);
            }
            heard.shown(&rig, last);
            end(&rig, &heard);
            assert_eq!(rig.hub.mode(), before, "edge 17: {name} from {before:?}");
            assert_eq!(mode(&rig)["aside"], json!(null), "{name}");
            assert!(shielded(&rig).is_empty(), "{name}: in captures again");
        }
    }
}

#[test]
fn windows_made_again_during_an_aside_step_aside_again() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.aside(SnipAside::Hidden);
    rig.hub.drop_waker();
    let woken = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let seen = std::sync::Arc::clone(&woken);
    rig.hub
        .set_waker(Box::new(move |w| seen.lock().unwrap().push(w)));
    assert!(
        woken.lock().unwrap().contains(&Wake::Aside),
        "the new windows hide and are shielded"
    );
}

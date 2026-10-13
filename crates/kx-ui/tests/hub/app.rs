//! Between the app and the UI: Quit and a second copy's commands reach the app, which ends the UI and its queue,
//! and the keyboard's mode outlives the windows that kept it.

use crate::logs::warnings_of;
use crate::{Rig, failed};
use kx_module_api::{ModuleId, Notice};
use kx_platform::ui::Command;
use kx_ui::{Hub, Mode, Request, Wake};
use std::sync::mpsc::{RecvError, TryRecvError};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;

/// The failed modules whose Try again races Quit, one thread each.
const RACERS: [&str; 6] = ["a", "b", "c", "d", "e", "f"];
/// How many races the order test runs; each lets Try again and Quit meet at a different point.
const ROUNDS: usize = 200;

#[test]
fn quit_reaches_the_app_once_and_then_the_queue_ends() {
    let rig = Rig::new();
    assert!(rig.hub.quit());
    assert_eq!(rig.requests.recv(), Ok(Request::Quit));
    assert_eq!(rig.requests.recv(), Err(RecvError), "the drain loop ends");
    assert!(!rig.hub.quit(), "only once");
}

#[test]
fn a_second_quit_leaves_ending_the_ui_to_the_app() {
    let rig = Rig::new();
    assert!(rig.hub.quit());
    assert!(!rig.hub.quit(), "asked already");
    rig.hub.command(Command::Quit);
    assert!(
        rig.wakes().is_empty(),
        "the UI ends once the modules stopped"
    );
    assert_eq!(rig.requests.try_recv(), Ok(Request::Quit));
    assert_eq!(rig.requests.try_recv(), Err(TryRecvError::Disconnected));
}

#[test]
fn quit_is_false_when_the_app_no_longer_listens() {
    let Rig { hub, requests, .. } = Rig::new();
    drop(requests);
    assert!(!hub.quit());
}

#[test]
fn a_second_copys_quit_goes_to_the_app_and_its_show_to_the_ui() {
    let rig = Rig::new();
    rig.hub.command(Command::Quit);
    assert_eq!(rig.requests.try_recv(), Ok(Request::Quit));
    assert!(
        rig.wakes().is_empty(),
        "the UI ends only when the app says so"
    );
    rig.hub.command(Command::Show);
    assert_eq!(rig.wakes(), [Wake::Show]);
}

#[test]
fn try_again_after_quit_asks_nothing() {
    let rig = Rig::new();
    rig.hub.notice(failed());
    let id = rig.hub.take_notices()[0].id;
    assert!(rig.hub.quit());
    assert!(!rig.hub.retry(id));
    assert_eq!(rig.requests.try_recv(), Ok(Request::Quit));
    assert_eq!(rig.requests.try_recv(), Err(TryRecvError::Disconnected));
}

#[test]
fn the_app_ends_the_ui_once_the_modules_stopped() {
    let rig = Rig::new();
    rig.hub.end_ui();
    assert_eq!(rig.wakes(), [Wake::End]);
}

#[test]
fn show_and_end_wait_for_the_waker_then_reach_the_ui_in_order() {
    let (hub, _requests) = Hub::new(Rig::new().ports()).unwrap();
    hub.command(Command::Show);
    hub.end_ui();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::clone(&seen);
    hub.set_waker(Box::new(move |w| log.lock().unwrap().push(w)));
    hub.command(Command::Show);
    assert_eq!(*seen.lock().unwrap(), [Wake::Show, Wake::End, Wake::Show]);
}

#[test]
fn a_second_waker_takes_the_wakes_and_warns() {
    let rig = Rig::new();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::clone(&seen);
    let waker: kx_ui::Waker = Box::new(move |w| log.lock().unwrap().push(w));
    let warned = warnings_of(|| rig.hub.set_waker(waker));
    assert_eq!(warned.len(), 1, "{warned:?}");
    rig.hub.command(Command::Show);
    assert_eq!(*seen.lock().unwrap(), [Wake::Show]);
    assert!(rig.wakes().is_empty(), "the first waker hears nothing");
}

/// A waker that keeps every wake in `seen`.
fn keeper(seen: &Arc<Mutex<Vec<Wake>>>) -> kx_ui::Waker {
    let log = Arc::clone(seen);
    Box::new(move |w| log.lock().unwrap().push(w))
}

#[test]
fn a_dropped_waker_leaves_show_and_end_to_the_next_ui_without_a_warning() {
    let rig = Rig::new();
    assert!(!rig.hub.ended());
    rig.hub.drop_waker();
    rig.hub.command(Command::Show);
    rig.hub.end_ui();
    assert!(rig.hub.ended());
    assert!(rig.wakes().is_empty(), "the last Ui object hears nothing");
    let seen = Arc::new(Mutex::new(Vec::new()));
    let warned = warnings_of(|| rig.hub.set_waker(keeper(&seen)));
    assert!(warned.is_empty(), "{warned:?}");
    assert_eq!(*seen.lock().unwrap(), [Wake::Show, Wake::End]);
}

#[test]
fn an_end_the_last_ui_may_have_missed_reaches_the_next_once() {
    let rig = Rig::new();
    rig.hub.end_ui();
    assert_eq!(rig.wakes(), [Wake::End]);
    rig.hub.drop_waker();
    rig.hub.drop_waker();
    let seen = Arc::new(Mutex::new(Vec::new()));
    rig.hub.set_waker(keeper(&seen));
    assert_eq!(*seen.lock().unwrap(), [Wake::End]);
}

#[test]
fn quit_stays_the_last_request_while_try_again_races_it() {
    for _ in 0..ROUNDS {
        let rig = Rig::new();
        for m in RACERS {
            let module = Some(ModuleId::new(m));
            rig.hub.notice(Notice { module, ..failed() });
        }
        let ids: Vec<u64> = rig.hub.take_notices().iter().map(|s| s.id).collect();
        let start = Arc::new(Barrier::new(ids.len() + 1));
        let racers: Vec<_> = ids
            .into_iter()
            .map(|id| {
                let (hub, start) = (Arc::clone(&rig.hub), Arc::clone(&start));
                thread::spawn(move || {
                    start.wait();
                    hub.retry(id)
                })
            })
            .collect();
        start.wait();
        assert!(rig.hub.quit());
        for racer in racers {
            assert!(racer.join().is_ok());
        }
        let got: Vec<Request> = rig.requests.iter().collect();
        assert_eq!(got.last(), Some(&Request::Quit), "{got:?}");
    }
}

#[test]
fn a_quit_the_app_can_no_longer_hear_ends_the_ui_itself() {
    let Rig {
        hub,
        requests,
        wakes,
        ..
    } = Rig::new();
    drop(requests);
    hub.command(Command::Quit);
    assert_eq!(*wakes.lock().unwrap(), [Wake::End]);
    assert!(!hub.quit(), "asked already");
}

#[test]
fn a_new_hub_keeps_the_keyboard_shown() {
    let rig = Rig::new();
    assert_eq!(rig.hub.mode(), Mode::Shown);
    assert_eq!(rig.hub.mode_json(), r#"{"away":false,"minimised":false}"#);
}

#[test]
fn the_mode_the_ui_keeps_outlives_it_for_the_next() {
    let rig = Rig::new();
    rig.hub.keep_mode(Mode::of(true, true));
    rig.hub.drop_waker();
    rig.hub.set_waker(keeper(&Arc::default()));
    assert_eq!(rig.hub.mode(), Mode::Minimised);
    assert_eq!(rig.hub.mode_json(), r#"{"away":true,"minimised":true}"#);
    rig.hub.keep_mode(Mode::of(true, false));
    assert_eq!(rig.hub.mode_json(), r#"{"away":true,"minimised":false}"#);
    assert!(rig.wakes().is_empty(), "keeping it wakes nothing");
}

#[test]
fn a_minimised_keyboard_is_away_too() {
    assert_eq!(Mode::of(false, false), Mode::Shown);
    assert_eq!(Mode::of(true, false), Mode::Away);
    assert_eq!(Mode::of(true, true), Mode::Minimised);
    assert_eq!(Mode::of(false, true), Mode::Minimised);
}

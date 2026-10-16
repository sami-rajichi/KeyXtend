//! The scroll pad over the whole app: an arrow scrolls what the fake target aims at with the user's `[scroll]` values,
//! and quitting while it scrolls closes the run, hides the outline and lets the mouse hook go (edge case 15).

use crate::rig::Rig;
use keyxtend::boot::Boot;
use keyxtend::drain;
use kx_mod_scroll::ScrollSettings;
use kx_module_api::ports::{
    Aim, Dir, FaceAction, Point, Rect, ScrollAimed, ScrollAsk, ScrollChanged, TargetId,
};
use kx_module_api::{Bus, Event, Subscription};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

/// The longest a test waits for a module's thread; it never sleeps.
const WAIT: Duration = Duration::from_secs(10);
/// The way the pad scrolls.
const DOWN: Dir = Dir::Down;

/// What the fake target aims at: an app's pane, where the user pointed.
const APP: Aim = Aim {
    at: Point { x: 123, y: 345 },
    on_pointer: false,
    area: Some(Rect {
        x: 101,
        y: 202,
        w: 640,
        h: 480,
    }),
    ours: false,
    id: TargetId::new(7),
};

/// The app over a rig, scrolling down, with ears on the scroll state and the outline.
struct Scrolling {
    boot: Boot,
    told: Receiver<ScrollChanged>,
    aimed: Receiver<ScrollAimed>,
    /// The ears stay on the bus while these live.
    _ears: [Subscription; 2],
}

/// Sends every `E` published on `bus` to the returned channel.
fn hear<E: Event + Clone>(bus: &Bus) -> (Subscription, Receiver<E>) {
    let (tx, rx) = mpsc::channel();
    let ear = bus.subscribe(move |e: &E| {
        let _ = tx.send(e.clone());
    });
    (ear.unwrap(), rx)
}

/// Waits for a scroll state that `want` takes; the ones before it go.
fn until(told: &Receiver<ScrollChanged>, want: impl Fn(&ScrollChanged) -> bool) {
    while !want(&told.recv_timeout(WAIT).unwrap()) {}
}

/// Starts the app over `rig` and taps the pad's down arrow once the keyboard heard the scroll module is live.
fn scroll_down(rig: &Rig) -> Scrolling {
    rig.scroll.push_aim(Ok(APP));
    let mut boot = rig.load();
    boot.start().unwrap();
    // Ears after the start, so the keyboard and the mouse module hear each state before the test does.
    let bus = boot.kernel.bus();
    let (e1, told) = hear::<ScrollChanged>(&bus);
    let (e2, aimed) = hear::<ScrollAimed>(&bus);
    bus.publish(&ScrollAsk).unwrap();
    until(&told, |s| s.live);
    let _ = boot.hub.act(FaceAction::Pad { dir: Some(DOWN) });
    until(&told, |s| s.dir == Some(DOWN));
    Scrolling {
        boot,
        told,
        aimed,
        _ears: [e1, e2],
    }
}

#[test]
fn quitting_while_scrolling_closes_the_run_hides_the_outline_and_lets_the_hook_go() {
    let rig = Rig::new();
    let Scrolling {
        boot,
        told,
        aimed,
        _ears,
    } = scroll_down(&rig);
    let hub = Arc::clone(&boot.hub);
    let thread = drain::spawn(boot.kernel, Arc::clone(&hub), boot.requests).unwrap();
    assert!(hub.quit(), "the tray's Quit reaches the kernel's thread");
    thread.join().unwrap();
    let target = &rig.scroll;
    let run = (target.running(), target.opens(), target.closes());
    assert_eq!(run, (false, 1, 1), "the one run closed");
    assert_eq!(told.try_iter().last(), Some(ScrollChanged::OFF));
    let boxes: Vec<ScrollAimed> = aimed.try_iter().collect();
    let outline = |area| ScrollAimed { area };
    assert!(boxes.contains(&outline(APP.area)), "it outlined");
    assert_eq!(boxes.last(), Some(&outline(None)), "no outline left");
    let hook = &rig.pointer;
    let held = (hook.running(), hook.starts(), hook.stops());
    assert_eq!(held, (false, 1, 1), "the hook ran; the quit let go");
}

#[test]
fn the_users_scroll_section_reaches_the_module() {
    let rig = Rig::new();
    let read_ms = ScrollSettings::from_defaults().unwrap().read_ms + 1;
    let section = format!("[{}]\nread_ms = {read_ms}\n", kx_mod_scroll::ID);
    std::fs::write(rig.files().settings, section).unwrap();
    let _scrolling = scroll_down(&rig);
    let read = rig.scroll.config().map(|c| c.read_ms);
    assert_eq!(read, Some(read_ms), "the run opened with the user's value");
}

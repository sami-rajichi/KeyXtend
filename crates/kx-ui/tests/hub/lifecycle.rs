//! Any run of the app's, a second copy's and the UI's calls: the mode is the last one kept, Quit reaches the app once
//! and last, and no Show or End is lost while wakers come and go; once ended, the next waker hears End.

use crate::{KEYBOARD, Rig, failed};
use kx_platform::ui::Command;
use kx_ui::{Hub, Mode, Request, Wake};
use proptest::collection::vec;
use proptest::prelude::*;
use serde_json::Value;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::{Arc, Mutex};

/// One call between the app, a second copy and the UI.
#[derive(Copy, Clone, Debug)]
enum Op {
    /// The UI keeps how the user left the keyboard, as QML's `away` and `minimised`.
    Keep(bool, bool),
    /// The windows are made again: the waker goes first.
    Drop,
    /// A new `Ui` object sets its waker.
    Set,
    /// A second copy asks to show the keyboard.
    Show,
    /// The app stopped the modules, so the UI ends.
    End,
    /// The user's Quit, or a second copy's when true.
    Quit(bool),
    /// Try again on a failed module's notice.
    Retry,
    /// The app's drain thread is gone.
    Gone,
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        any::<(bool, bool)>().prop_map(|(a, m)| Op::Keep(a, m)),
        Just(Op::Drop),
        Just(Op::Set),
        Just(Op::Show),
        Just(Op::End),
        any::<bool>().prop_map(Op::Quit),
        Just(Op::Retry),
        Just(Op::Gone),
    ]
}

/// The hub, the app's end of its queue while it listens, and each waker's wakes, oldest waker first.
struct World {
    hub: Arc<Hub>,
    requests: Option<Receiver<Request>>,
    logs: Vec<Arc<Mutex<Vec<Wake>>>>,
}

/// What the hub should hold and give, as its docs say.
#[derive(Default)]
struct Model {
    mode: Mode,
    /// `Show` and `End` wakes waiting for a waker.
    pending: Vec<Wake>,
    /// Each waker's wakes, oldest waker first; the last hears while `live`.
    logs: Vec<Vec<Wake>>,
    live: bool,
    ended: bool,
    /// Quit was asked, so the queue is gone.
    asked: bool,
    requests: Vec<Request>,
}

impl Model {
    /// The wakes of the waker set, while one is.
    fn heard(&mut self) -> Option<&mut Vec<Wake>> {
        self.logs.last_mut().filter(|_| self.live)
    }

    /// A `Show` or `End`: to the waker set, or kept for the next one.
    fn wake_or_wait(&mut self, wake: Wake) {
        match self.heard() {
            Some(log) => log.push(wake),
            None => self.pending.push(wake),
        }
    }

    /// Quit: the first one reaches a listening app, else ends the UI itself; true when it reached the app.
    fn quit(&mut self, listening: bool) -> bool {
        if std::mem::replace(&mut self.asked, true) {
            return false;
        }
        if listening {
            self.requests.push(Request::Quit);
        } else {
            self.ended = true;
            self.wake_or_wait(Wake::End);
        }
        listening
    }
}

/// Does `op` on the hub and on the model, and checks what Quit and Try again answer.
fn apply(w: &mut World, m: &mut Model, op: Op) -> Result<(), TestCaseError> {
    match op {
        Op::Keep(away, minimised) => {
            w.hub.keep_mode(Mode::of(away, minimised));
            m.mode = Mode::of(away, minimised);
        }
        Op::Drop => {
            w.hub.drop_waker();
            m.live = false;
            if m.ended && !m.pending.contains(&Wake::End) {
                m.pending.push(Wake::End);
            }
        }
        Op::Set => set(w, m),
        Op::Show => {
            w.hub.command(Command::Show);
            m.wake_or_wait(Wake::Show);
        }
        Op::End => {
            w.hub.end_ui();
            m.ended = true;
            m.wake_or_wait(Wake::End);
        }
        Op::Quit(copy) => quit(w, m, copy)?,
        Op::Retry => retry(w, m)?,
        Op::Gone => w.requests = None,
    }
    Ok(())
}

/// A new waker, which takes the wakes that waited.
fn set(w: &mut World, m: &mut Model) {
    let log = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&log);
    w.hub
        .set_waker(Box::new(move |wake| seen.lock().unwrap().push(wake)));
    w.logs.push(log);
    m.logs.push(std::mem::take(&mut m.pending));
    m.live = true;
}

/// Quit through the hub or a second copy; only the hub's answer can be read.
fn quit(w: &World, m: &mut Model, copy: bool) -> Result<(), TestCaseError> {
    let want = m.quit(w.requests.is_some());
    if copy {
        w.hub.command(Command::Quit);
    } else {
        prop_assert_eq!(w.hub.quit(), want, "only the first Quit reaches the app");
    }
    Ok(())
}

/// A failed module's notice, which wakes only a waker set, then its Try again, which reaches the app before Quit.
fn retry(w: &World, m: &mut Model) -> Result<(), TestCaseError> {
    w.hub.notice(failed());
    if let Some(log) = m.heard() {
        log.push(Wake::Notices);
    }
    let id = w.hub.take_notices()[0].id;
    let want = !m.asked && w.requests.is_some();
    prop_assert_eq!(w.hub.retry(id), want, "no Try again after Quit");
    if want {
        m.requests.push(Request::Retry(KEYBOARD));
    }
    Ok(())
}

/// Checks the mode, its JSON, the end and every waker's wakes against the model.
fn same(w: &World, m: &Model) -> Result<(), TestCaseError> {
    prop_assert_eq!(w.hub.mode(), m.mode);
    let flags: Value = serde_json::from_str(&w.hub.mode_json()).unwrap();
    prop_assert_eq!(flags["away"].as_bool(), Some(m.mode != Mode::Shown));
    prop_assert_eq!(
        flags["minimised"].as_bool(),
        Some(m.mode == Mode::Minimised)
    );
    prop_assert_eq!(w.hub.ended(), m.ended);
    let logs: Vec<Vec<Wake>> = w.logs.iter().map(|l| l.lock().unwrap().clone()).collect();
    prop_assert_eq!(&logs, &m.logs);
    Ok(())
}

/// The app's end of the queue: every request sent, Quit last, then the queue ends once Quit was asked.
fn drained(w: &World, m: &Model) -> Result<(), TestCaseError> {
    let Some(requests) = &w.requests else {
        return Ok(());
    };
    prop_assert_eq!(requests.try_iter().collect::<Vec<_>>(), m.requests.clone());
    let end = if m.asked {
        TryRecvError::Disconnected
    } else {
        TryRecvError::Empty
    };
    prop_assert_eq!(requests.try_recv(), Err(end));
    Ok(())
}

proptest! {
    #[test]
    fn the_mode_holds_quit_comes_once_and_last_and_no_show_or_end_is_lost(ops in vec(op(), 0..40)) {
        let (hub, requests) = Hub::new(Rig::new().ports()).unwrap();
        let mut w = World { hub, requests: Some(requests), logs: Vec::new() };
        let mut m = Model::default();
        let shows = ops.iter().filter(|o| matches!(o, Op::Show)).count();
        for op in ops.into_iter().chain([Op::Drop, Op::Set]) {
            apply(&mut w, &mut m, op)?;
            same(&w, &m)?;
        }
        drained(&w, &m)?;
        let heard: Vec<Wake> = w.logs.iter().flat_map(|l| l.lock().unwrap().clone()).collect();
        let heard = heard.iter().filter(|h| **h == Wake::Show).count();
        prop_assert_eq!(heard, shows, "each Show reaches one waker once");
        let last = w.logs.last().unwrap().lock().unwrap().clone();
        prop_assert_eq!(last.contains(&Wake::End), m.ended, "an ended UI's next waker hears End");
    }
}

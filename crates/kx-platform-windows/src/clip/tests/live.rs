//! The real clipboard, which these tests own briefly, one at a time. They write only test text, each write kept out of
//! Windows' history and cloud, and never show another note; a copy made in another app meanwhile can race them.

use std::sync::mpsc::{self, Receiver};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use kx_module_api::ports::{
    ClipConfig, ClipError, ClipNote, ClipRun, ClipWait, Clipboard, ClipboardWriter,
};

use super::*;
use crate::clip::open;

mod kinds;

/// The wait to open the clipboard, long enough for Windows' own history to read each change first.
const WAIT: ClipWait = ClipWait {
    open_ms: 2_000,
    poll_ms: 10,
};
/// How long a test waits to hear its note; a wait, never a sleep.
const HEAR: Duration = Duration::from_secs(5);
/// The cap of a watch whose test is not about the cap.
const ROOMY: u64 = 1 << 20;
/// Windows' history switch, which every test write sets to 0.
const HISTORY: &str = "CanIncludeInClipboardHistory";
/// The test text.
const TEXT: &str = "kx-test-clip-text";

/// Keeps the tests that use the real clipboard apart.
static REAL: Mutex<()> = Mutex::new(());

fn hold() -> MutexGuard<'static, ()> {
    REAL.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A watch whose notes arrive on a channel.
struct Watch {
    _run: Box<dyn ClipRun>,
    rx: Receiver<ClipNote>,
}

impl Watch {
    /// Watches with `marks` and a cap of `max_bytes`.
    fn start(marks: Marks, max_bytes: u64) -> Self {
        let (tx, rx) = mpsc::channel();
        let cfg = ClipConfig {
            max_bytes,
            wait: WAIT,
        };
        let tell = Box::new(move |n: ClipNote| {
            let _ = tx.send(n);
        });
        let run = WinClipboard::with_marks(marks).watch(cfg, tell).unwrap();
        Self { _run: run, rx }
    }

    /// Watches with the shipped markers, as the product does.
    fn shipped(max_bytes: u64) -> Self {
        Self::start(table().unwrap().marks(), max_bytes)
    }

    /// Watches with every marker but Windows' history switch, so it hears the tests' own writes.
    fn own(max_bytes: u64) -> Self {
        Self::start(table().unwrap().marks().without(HISTORY), max_bytes)
    }

    /// The first note that `wants`, within `HEAR`; other notes, maybe another app's copies, are dropped unshown.
    fn hear(&self, wants: impl Fn(&ClipNote) -> bool) -> Option<ClipNote> {
        let end = Instant::now() + HEAR;
        while let Some(left) = end.checked_duration_since(Instant::now()) {
            match self.rx.recv_timeout(left) {
                Ok(n) if wants(&n) => return Some(n),
                Ok(_) => {}
                Err(_) => return None,
            }
        }
        None
    }

    /// Whether a copy of exactly `want` comes, whichever app it names: the writer's window may be gone by then.
    fn hears(&self, want: &[ClipPart]) -> bool {
        let found = self.hear(|n| matches!(n, ClipNote::Copied { parts, .. } if parts == want));
        found.is_some()
    }
}

fn text(t: &str) -> ClipPart {
    ClipPart::Text(Redacted::new(t.to_owned()))
}

fn is_marked(n: &ClipNote) -> bool {
    matches!(n, ClipNote::Marked { .. })
}

/// Writes `parts` as the product does.
fn write(parts: &[ClipPart]) {
    WinClipboardWriter.write(parts, WAIT).unwrap();
}

#[test]
fn a_written_text_comes_back_with_windows_line_breaks() {
    let _real = hold();
    let watch = Watch::own(ROOMY);
    write(&[text("kx-test-clip\nline")]);
    assert!(watch.hears(&[text("kx-test-clip\r\nline")]));
}

#[test]
fn a_write_carries_our_marker_so_our_own_watch_skips_it() {
    let _real = hold();
    let watch = Watch::shipped(ROOMY);
    write(&[text(TEXT)]);
    assert!(watch.hear(is_marked).is_some());
}

#[test]
fn a_write_that_fails_part_way_still_carries_our_marker() {
    let _real = hold();
    let watch = Watch::shipped(ROOMY);
    let empty = ClipPart::Image {
        codec: ImageCodec::Png,
        bytes: Redacted::new(Vec::new()),
    };
    let got = WinClipboardWriter.write(&[text(TEXT), empty], WAIT);
    assert_eq!(got, Err(ClipError::Io), "Windows locks no empty memory");
    assert!(watch.hear(is_marked).is_some());
}

#[test]
fn each_marker_alone_skips_a_copy_before_its_size_is_looked_at() {
    let _real = hold();
    let shipped = table().unwrap().marks();
    for mark in shipped.names() {
        let others = shipped.names().filter(|m| *m != mark);
        let alone = others.fold(shipped.clone(), |m, other| m.without(other));
        let watch = Watch::start(alone, 1);
        write_text(Some(TEXT), &[mark, HISTORY], WAIT).unwrap();
        assert!(watch.hear(is_marked).is_some(), "{mark}");
    }
}

#[test]
fn an_emptied_clipboard_is_cleared_with_the_app_that_emptied_it() {
    let _real = hold();
    let watch = Watch::own(ROOMY);
    let owner = open::owner_window().unwrap();
    let emptied = open::open_for(owner, WAIT).and_then(|o| open::empty(&o));
    let note = watch.hear(|n| matches!(n, ClipNote::Cleared { .. }));
    open::close_window(owner);
    emptied.unwrap();
    let Some(ClipNote::Cleared { app: Some(app) }) = note else {
        panic!("no clear with its app");
    };
    let exe = std::env::current_exe().unwrap();
    let name = exe.file_name().unwrap().to_string_lossy().to_lowercase();
    assert_eq!(app.id, name);
}

#[test]
fn a_blank_text_is_cleared() {
    let _real = hold();
    let watch = Watch::own(ROOMY);
    write_text(Some(" \r\n\t"), &[HISTORY], WAIT).unwrap();
    assert!(
        watch
            .hear(|n| matches!(n, ClipNote::Cleared { .. }))
            .is_some()
    );
}

#[test]
fn a_copy_over_the_cap_is_too_big_and_says_its_size() {
    let _real = hold();
    let watch = Watch::own(10);
    write_text(Some("kx-test-clip-over-ten-bytes"), &[HISTORY], WAIT).unwrap();
    let note = watch.hear(|n| matches!(n, ClipNote::TooBig { .. }));
    assert!(matches!(note, Some(ClipNote::TooBig { bytes, .. }) if bytes > 10));
}

#[test]
fn the_tell_runs_on_the_watch_thread() {
    let _real = hold();
    let (tx, rx) = mpsc::channel();
    let cfg = ClipConfig {
        max_bytes: ROOMY,
        wait: WAIT,
    };
    let tell = Box::new(move |_: ClipNote| {
        let _ = tx.send(std::thread::current().name().map(str::to_owned));
    });
    let marks = table().unwrap().marks();
    let run = WinClipboard::with_marks(marks).watch(cfg, tell).unwrap();
    write(&[text(TEXT)]);
    let name = rx.recv_timeout(HEAR);
    drop(run);
    assert_eq!(name.unwrap().as_deref(), Some("kx-clip-watch"));
}

#[test]
fn a_dropped_watch_lets_its_tell_go() {
    let _real = hold();
    let held = std::sync::Arc::new(());
    let mine = std::sync::Arc::clone(&held);
    let cfg = ClipConfig {
        max_bytes: ROOMY,
        wait: WAIT,
    };
    let run = WinClipboard::new()
        .watch(
            cfg,
            Box::new(move |_: ClipNote| assert!(std::sync::Arc::strong_count(&mine) > 0)),
        )
        .unwrap();
    drop(run);
    assert_eq!(
        std::sync::Arc::strong_count(&held),
        1,
        "the tell went with its thread"
    );
}

#[test]
fn a_write_while_another_window_holds_the_clipboard_is_busy() {
    let _real = hold();
    let (held_tx, held_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel::<()>();
    let holder = std::thread::spawn(move || {
        let open = open::open(WAIT);
        let _ = held_tx.send(open.is_ok());
        let _ = done_rx.recv_timeout(HEAR);
    });
    assert_eq!(held_rx.recv_timeout(HEAR), Ok(true));
    let quick = ClipWait {
        open_ms: 50,
        poll_ms: 10,
    };
    let got = WinClipboardWriter.write(&[text(TEXT)], quick);
    drop(done_tx);
    holder.join().unwrap();
    assert_eq!(got, Err(ClipError::Busy));
}

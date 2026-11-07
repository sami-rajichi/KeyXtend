//! A scripted probe for the selection tests: the test sets its focus, it counts every call, and its gate can shut at
//! one place so the thread hangs there, as in a busy app, until the test opens it again.

use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use kx_module_api::Redacted;
use kx_module_api::ports::{Rect, Screen, SelError, SelKind, SelSpot};

use super::{Ours, SelProbe, lock};

/// How long a test waits for a thread or a gate; a bounded wait, never a sleep.
pub(super) const WAIT: Duration = Duration::from_secs(5);

/// A scripted element: whether it is ours, and the spot its box shows.
pub(super) type El = (bool, SelSpot);

/// How many times the probe was asked each thing.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Calls {
    pub(super) focused: usize,
    pub(super) spots: usize,
    pub(super) stills: usize,
    pub(super) reads: usize,
    pub(super) clips: usize,
}

/// Where a shut gate holds the thread.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum Stop {
    /// Before `serve`, as a rig's thread takes a job: it holds the job whatever its deadline.
    Take,
    /// Inside `spot`.
    Spot,
    /// Inside `read`, once it took the text.
    Read,
}

/// What the test set, and what the probe was asked.
struct Board {
    focus: Result<El, SelError>,
    text: String,
    clip: bool,
    /// Where the gate is shut, if it is.
    shut: Option<Stop>,
    /// How many calls waited at the gate.
    hung: usize,
    calls: Calls,
}

/// The scripted probe; clones share it.
#[derive(Clone)]
pub(super) struct Script(Arc<(Mutex<Board>, Condvar)>);

impl Script {
    /// A probe whose focus is `focus`, with an empty selection text and clipboard.
    pub(super) fn on(focus: Result<El, SelError>) -> Self {
        let board = Board {
            focus,
            text: String::new(),
            clip: false,
            shut: None,
            hung: 0,
            calls: Calls::default(),
        };
        Self(Arc::new((Mutex::new(board), Condvar::new())))
    }

    /// The board, locked.
    fn board(&self) -> MutexGuard<'_, Board> {
        lock(&self.0.0)
    }

    /// Moves the focus.
    pub(super) fn focus(&self, focus: Result<El, SelError>) {
        self.board().focus = focus;
    }

    /// Makes the selection hold `text`.
    pub(super) fn text(&self, text: &str) {
        text.clone_into(&mut self.board().text);
    }

    /// Makes the clipboard hold something or nothing.
    pub(super) fn clip(&self, holds: bool) {
        self.board().clip = holds;
    }

    /// Shuts the gate at `at`, so the next call there hangs until `open`.
    pub(super) fn shut(&self, at: Stop) {
        self.board().shut = Some(at);
    }

    /// Opens the gate, letting a hung call go on.
    pub(super) fn open(&self) {
        self.board().shut = None;
        self.0.1.notify_all();
    }

    /// Holds the caller at `at` while the gate is shut there, at most `WAIT`.
    pub(super) fn pass(&self, at: Stop) {
        drop(self.held(self.board(), at));
    }

    /// Like `pass`, keeping the board locked around the wait.
    fn held<'a>(&self, mut board: MutexGuard<'a, Board>, at: Stop) -> MutexGuard<'a, Board> {
        if board.shut != Some(at) {
            return board;
        }
        board.hung += 1;
        self.0.1.notify_all();
        let waited = self
            .0
            .1
            .wait_timeout_while(board, WAIT, |b| b.shut == Some(at));
        waited.map_or_else(|e| e.into_inner().0, |(board, _)| board)
    }

    /// Waits, at most `WAIT`, until a call hangs at the gate; true once one does.
    pub(super) fn hung(&self) -> bool {
        let board = self.board();
        let waited = self.0.1.wait_timeout_while(board, WAIT, |b| b.hung == 0);
        waited.is_ok_and(|(b, _)| b.hung > 0)
    }

    /// What the probe was asked so far.
    pub(super) fn calls(&self) -> Calls {
        self.board().calls
    }
}

impl Ours for Script {
    type El = El;

    fn ours(&self, el: &El) -> bool {
        el.0
    }
}

impl SelProbe for Script {
    fn focused(&self, _: Instant) -> Result<El, SelError> {
        let mut board = self.board();
        board.calls.focused += 1;
        board.focus
    }

    fn spot(&self, el: &El, _: Instant) -> Result<SelSpot, SelError> {
        let mut board = self.board();
        board.calls.spots += 1;
        drop(self.held(board, Stop::Spot));
        Ok(el.1)
    }

    fn still(&self, el: &El, box_id: u64, _: Instant) -> bool {
        self.board().calls.stills += 1;
        el.1.box_id == box_id && el.1.selected
    }

    fn read(&self, _: &El, max_chars: u16, _: Instant) -> Result<Redacted<String>, SelError> {
        let mut board = self.board();
        board.calls.reads += 1;
        let text = board.text.chars().take(usize::from(max_chars)).collect();
        drop(self.held(board, Stop::Read));
        Ok(Redacted::new(text))
    }

    fn clip_holds(&self) -> bool {
        let mut board = self.board();
        board.calls.clips += 1;
        board.clip
    }
}

/// A spot in a box of `kind` with id `box_id`, holding a selection when `selected`.
pub(super) fn spot(kind: SelKind, box_id: u64, selected: bool) -> SelSpot {
    let line = Rect {
        x: 10,
        y: 20,
        w: 30,
        h: 16,
    };
    SelSpot {
        box_id,
        kind,
        selected,
        first: selected.then_some(line),
        last: selected.then_some(line),
        caret: Some(line),
        screen: Screen::default(),
    }
}

/// An element of another app: a box of `kind` with id `box_id`, holding a selection.
pub(super) fn theirs(kind: SelKind, box_id: u64) -> El {
    (false, spot(kind, box_id, true))
}

/// One of our own elements, holding a selection.
pub(super) fn own(box_id: u64) -> El {
    (true, spot(SelKind::Editable, box_id, true))
}

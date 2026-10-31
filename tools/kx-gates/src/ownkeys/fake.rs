//! A fake desk for tests: a fixed window in front, titles read in turn, and every key and click recorded, never sent.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::path::PathBuf;

use windows::Win32::Foundation::HWND;

use super::Desk;
use crate::apps::{AppKind, Opened};

/// The gate's own Notepad file name in the tests.
pub const OURS: &str = "g20-keyxtend-1.txt";
/// The title of the gate's own tab.
pub const OUR_TAB: &str = "g20-keyxtend-1.txt - Notepad";
/// The title of one of the owner's tabs.
pub const OWNER_TAB: &str = "*Shopping list - Notepad";

/// A window handle for tests, never a real window's.
pub fn hwnd(n: usize) -> HWND {
    HWND(std::ptr::without_provenance_mut(n))
}

/// The gate's Notepad as opened: window `hwnd(1)`, our file name, and `file`.
pub fn notepad(file: Option<PathBuf>) -> Opened {
    Opened {
        kind: AppKind::Notepad,
        hwnd: hwnd(1),
        child: None,
        file,
        title_has: Some(OURS.to_string()),
    }
}

/// A desk with `front` in front whose app title reads `titles` in turn, the last one from then on.
pub struct Fake {
    front: HWND,
    titles: RefCell<VecDeque<String>>,
    /// Every key combo sent, in order.
    pub sent: RefCell<Vec<Vec<u16>>>,
    /// How many clicks went out.
    pub clicks: Cell<usize>,
}

impl Fake {
    /// A desk with `front` in front and the app's titles `titles`, read in turn.
    pub fn new(front: HWND, titles: &[&str]) -> Self {
        Self {
            front,
            titles: RefCell::new(titles.iter().map(ToString::to_string).collect()),
            sent: RefCell::default(),
            clicks: Cell::default(),
        }
    }

    /// True when no key and no click went out.
    pub fn quiet(&self) -> bool {
        self.sent.borrow().is_empty() && self.clicks.get() == 0
    }
}

impl Desk for Fake {
    fn front(&self) -> HWND {
        self.front
    }

    fn title(&self, _w: HWND) -> String {
        let mut titles = self.titles.borrow_mut();
        if titles.len() > 1 {
            return titles.pop_front().unwrap_or_default();
        }
        titles.front().cloned().unwrap_or_default()
    }

    fn describe(&self, _w: HWND) -> String {
        "a test window".to_string()
    }

    fn combo(&self, vks: &[u16]) -> Result<(), String> {
        self.sent.borrow_mut().push(vks.to_vec());
        Ok(())
    }

    fn click(&self, _act: &dyn Fn() -> Result<(), String>) -> Result<(), String> {
        self.clicks.set(self.clicks.get() + 1);
        Ok(())
    }

    fn settle(&self, _ms: u64) {}
}

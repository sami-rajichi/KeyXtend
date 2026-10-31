//! G20's ears: the product's clipboard watch with its notes on a channel, and the tally of what it heard.
#![cfg(windows)]

use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use kx_module_api::ports::{ClipConfig, ClipNote, ClipPart, ClipRun, Clipboard};
use kx_platform_windows::clip::WinClipboard;

/// The most bytes one character takes in UTF-16.
const BYTES_PER_CHAR: u64 = 4;

/// The product's cap for texts of up to `max_chars` characters.
pub fn max_bytes(max_chars: usize) -> u64 {
    u64::try_from(max_chars).map_or(u64::MAX, |c| c.saturating_mul(BYTES_PER_CHAR))
}

/// A running product watch; it stops when dropped.
pub struct Listener {
    _run: Box<dyn ClipRun>,
    rx: Receiver<ClipNote>,
}

impl Listener {
    /// Starts the product's watch as `cfg` says.
    pub fn start(cfg: ClipConfig) -> Result<Self, String> {
        let (tx, rx) = mpsc::channel();
        let tell = Box::new(move |n: ClipNote| {
            let _ = tx.send(n);
        });
        let run = WinClipboard::new()
            .watch(cfg, tell)
            .map_err(|e| e.to_string())?;
        Ok(Self { _run: run, rx })
    }

    /// The next note, if one comes within `wait_ms`.
    pub fn next(&self, wait_ms: u64) -> Option<ClipNote> {
        self.rx.recv_timeout(Duration::from_millis(wait_ms)).ok()
    }
}

/// What the listener heard: every change, the marked ones, and the kept texts in order.
#[derive(Default)]
pub struct Tally {
    /// Changes heard.
    pub seen: usize,
    /// Changes left out for a marker.
    pub skipped: usize,
    /// Kept texts, oldest first.
    pub entries: Vec<String>,
}

impl Tally {
    /// Counts `note`, and keeps its text if it has one.
    pub fn add(&mut self, note: ClipNote) {
        self.seen += 1;
        match note {
            ClipNote::Marked { .. } => self.skipped += 1,
            ClipNote::Copied { parts, .. } => self.entries.extend(parts.into_iter().find_map(text)),
            _ => {}
        }
    }
}

/// The text of a text part.
fn text(part: ClipPart) -> Option<String> {
    match part {
        ClipPart::Text(t) => Some(t.into_inner()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kx_module_api::Redacted;

    fn copy(t: &str) -> ClipNote {
        let parts = vec![
            ClipPart::Html(Redacted::new(b"<p>".to_vec())),
            ClipPart::Text(Redacted::new(t.to_owned())),
        ];
        ClipNote::Copied { parts, app: None }
    }

    #[test]
    fn two_quick_copies_are_both_counted_and_marks_skip() {
        let mut t = Tally::default();
        t.add(copy("a"));
        t.add(copy("a"));
        t.add(ClipNote::Marked { app: None });
        t.add(ClipNote::Cleared { app: None });
        assert_eq!((t.seen, t.skipped, t.entries.len()), (4, 1, 2));
        assert_eq!(t.entries[0], "a");
    }

    #[test]
    fn the_cap_holds_the_longest_characters() {
        assert_eq!(max_bytes(10), 40);
        assert_eq!(max_bytes(usize::MAX), u64::MAX);
    }
}

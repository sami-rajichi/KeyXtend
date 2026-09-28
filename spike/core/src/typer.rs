//! Types dictated text on its own thread, a character at a time, so the UI never waits and slow apps drop nothing.

use std::sync::Arc;
use std::sync::mpsc::{self, Sender};

use crate::note::{self, Note};

/// Shown when the typing thread is gone.
const TYPER_GONE: &str = "the typing thread stopped";

/// Types one text with a gap in ms between characters; `inject::text_paced` outside tests.
pub type TypeFn = Box<dyn Fn(&str, u64) -> Result<(), String> + Send>;

/// The typing thread and its latest error.
pub struct Typer {
    tx: Sender<String>,
    note: Note,
}

impl Typer {
    /// Starts the thread; texts are typed in the order sent, `gap_ms` between characters.
    pub fn start(gap_ms: u64, type_fn: TypeFn) -> Typer {
        let (tx, rx) = mpsc::channel::<String>();
        let note: Note = Arc::default();
        let slot = Arc::clone(&note);
        std::thread::spawn(move || {
            for text in rx {
                if let Err(e) = type_fn(&text, gap_ms) {
                    note::put(&slot, e);
                }
            }
        });
        Typer { tx, note }
    }

    /// Queues `text` for typing; an error only when the thread is gone.
    pub fn send(&self, text: String) -> Result<(), String> {
        self.tx.send(text).map_err(|_| TYPER_GONE.to_string())
    }

    /// The latest typing error, once.
    pub fn take_note(&self) -> Option<String> {
        note::take(&self.note)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Longest wait for the typing thread in a test; it answers at once.
    const WAIT: Duration = Duration::from_secs(5);

    #[test]
    fn texts_are_typed_in_order_and_a_failure_is_noted_once() {
        let (seen_tx, seen) = mpsc::channel();
        let t = Typer::start(
            7,
            Box::new(move |s: &str, gap| {
                let _ = seen_tx.send((s.to_string(), gap));
                if s == "bad" {
                    Err("blocked".into())
                } else {
                    Ok(())
                }
            }),
        );
        for s in ["one", "bad", "two"] {
            t.send(s.into()).expect("queued");
        }
        for want in ["one", "bad", "two"] {
            assert_eq!(seen.recv_timeout(WAIT).ok(), Some((want.to_string(), 7)));
        }
        assert_eq!(
            t.take_note().as_deref(),
            Some("blocked"),
            "noted before the next text"
        );
        assert_eq!(t.take_note(), None);
    }
}

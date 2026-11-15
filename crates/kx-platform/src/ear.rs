//! A run's callback, told notes on its stream's thread and closed by its owner at any time, even from inside the
//! callback itself; nothing is told once `close` returns.

use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

thread_local! {
    /// True while this thread runs a callback, so a close from inside one does not wait for itself.
    static TELLING: Cell<bool> = const { Cell::new(false) };
}

/// The callback an ear tells notes of type `T` to.
type Tell<T> = Box<dyn Fn(T) + Send>;

/// A callback that hears notes of type `T` until it is closed.
pub(crate) struct Ear<T> {
    open: AtomicBool,
    tell: Mutex<Option<Tell<T>>>,
}

impl<T> Ear<T> {
    /// An open ear that tells `tell`.
    pub(crate) fn new(tell: Tell<T>) -> Self {
        Self {
            open: AtomicBool::new(true),
            tell: Mutex::new(Some(tell)),
        }
    }

    /// Tells `notes` in order while the ear is open, holding the callback while it hears.
    pub(crate) fn tell(&self, notes: impl IntoIterator<Item = T>) {
        let held = self.held();
        let Some(tell) = held.as_ref() else {
            return;
        };
        let _telling = Telling::start();
        for note in notes {
            if !self.open.load(Ordering::Acquire) {
                break;
            }
            tell(note);
        }
    }

    /// Closes the ear: a note being told on another thread ends first, and the callback is dropped. From inside a
    /// callback it only stops later notes, as that callback still runs.
    pub(crate) fn close(&self) {
        self.open.store(false, Ordering::Release);
        if !TELLING.get() {
            self.held().take();
        }
    }

    /// The callback, held.
    fn held(&self) -> MutexGuard<'_, Option<Tell<T>>> {
        self.tell.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Marks this thread as running a callback until dropped, even when the callback panics.
struct Telling;

impl Telling {
    fn start() -> Self {
        TELLING.set(true);
        Self
    }
}

impl Drop for Telling {
    fn drop(&mut self) {
        TELLING.set(false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::sync::{Arc, OnceLock};

    #[test]
    fn an_ear_tells_in_order_until_closed() {
        let (said, heard) = mpsc::channel();
        let ear = Ear::new(Box::new(move |n: u8| {
            let _ = said.send(n);
        }));
        ear.tell([1, 2]);
        ear.close();
        ear.tell([3]);
        assert_eq!(heard.try_iter().collect::<Vec<_>>(), [1, 2]);
    }

    #[test]
    fn a_close_from_inside_its_own_callback_returns_and_stops_later_notes() {
        let (said, heard) = mpsc::channel();
        let me: Arc<OnceLock<Arc<Ear<u8>>>> = Arc::default();
        let inner = Arc::clone(&me);
        let ear = Arc::new(Ear::new(Box::new(move |n: u8| {
            let _ = said.send(n);
            if let Some(ear) = inner.get() {
                ear.close();
            }
        })));
        let _ = me.set(Arc::clone(&ear));
        ear.tell([1, 2]);
        ear.tell([3]);
        assert_eq!(heard.try_iter().collect::<Vec<_>>(), [1]);
    }
}

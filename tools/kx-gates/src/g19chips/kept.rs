//! A target-window the gate started, kept so it is moved back and closed also when a step panics, as a `Child` is
//! not ended when dropped.
#![cfg(windows)]

use std::cell::Cell;

use kx_module_api::ports::Point;
use windows::Win32::UI::WindowsAndMessaging::{
    SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SetWindowPos,
};

use crate::apps::{self, Ctx, Opened};
use crate::g19chips::GATE;
use crate::out::say;
use crate::win;

/// What a kept window does: move its top-left, and close once.
pub trait Hold {
    /// Moves the window's top-left to `to`.
    fn put(&self, to: Point) -> Result<(), String>;
    /// Closes it, once; what was left open.
    fn close(&mut self) -> Vec<String>;
}

/// A target-window the gate started, and the run's settings.
pub struct Target<'c> {
    ctx: &'c Ctx<'c>,
    /// The started window.
    pub app: Opened,
}

impl<'c> Target<'c> {
    /// Target-window `app`, started by the gate under `ctx`.
    pub fn new(ctx: &'c Ctx<'c>, app: Opened) -> Self {
        Self { ctx, app }
    }
}

impl Hold for Target<'_> {
    #[allow(unsafe_code, reason = "A plain move of the gate's own window.")]
    fn put(&self, to: Point) -> Result<(), String> {
        let flags = SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE;
        // SAFETY: a plain call on the gate's own window; a dead window gives an error.
        unsafe { SetWindowPos(self.app.hwnd, None, to.x, to.y, 0, 0, flags) }
            .map_err(|e| format!("moving {}: {e}", win::describe(self.app.hwnd)))
    }

    fn close(&mut self) -> Vec<String> {
        // Taking the process out makes a second close touch nothing.
        let app = Opened {
            child: self.app.child.take(),
            file: self.app.file.take(),
            title_has: self.app.title_has.take(),
            ..self.app
        };
        apps::clean_up(self.ctx, app)
    }
}

/// `held`, moved back to where a move found it and closed when dropped unclosed, as on a panic.
pub struct Kept<H: Hold> {
    held: H,
    from: Cell<Option<Point>>,
    closed: bool,
}

impl<H: Hold> Kept<H> {
    /// `held`, kept until `close`.
    pub fn new(held: H) -> Self {
        Self {
            held,
            from: Cell::new(None),
            closed: false,
        }
    }

    /// The kept window.
    pub fn get(&self) -> &H {
        &self.held
    }

    /// Moves the window to `to` from `from`, its place now; the first place not yet gone back to is kept.
    pub fn go(&self, to: Point, from: Point) -> Result<(), String> {
        self.from.set(Some(self.from.take().unwrap_or(from)));
        self.held.put(to)
    }

    /// Moves the window back where `go` found it, once; nothing when it was not moved.
    pub fn back(&self) -> Result<(), String> {
        self.from.take().map_or(Ok(()), |p| self.held.put(p))
    }

    /// Moves the window back if a step left it moved, then closes it; what was left.
    pub fn close(mut self) -> Vec<String> {
        self.closed = true;
        let mut left: Vec<String> = self.back().err().into_iter().collect();
        left.extend(self.held.close());
        left
    }
}

impl<H: Hold> Drop for Kept<H> {
    fn drop(&mut self) {
        if self.closed {
            return;
        }
        let back = self.back().err();
        for note in back.into_iter().chain(self.held.close()) {
            say!("{GATE}: {note}");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use super::*;

    /// A window that logs each move and close.
    struct Fake<'a>(&'a RefCell<Vec<String>>);

    impl Hold for Fake<'_> {
        fn put(&self, to: Point) -> Result<(), String> {
            self.0.borrow_mut().push(format!("put {}", to.x));
            Ok(())
        }

        fn close(&mut self) -> Vec<String> {
            self.0.borrow_mut().push("close".into());
            vec!["left".into()]
        }
    }

    fn at(x: i32) -> Point {
        Point { x, y: 0 }
    }

    #[test]
    fn a_moved_window_goes_back_once_and_closes_once() {
        let log = RefCell::new(Vec::new());
        let kept = Kept::new(Fake(&log));
        kept.go(at(5), at(1)).expect("moved");
        kept.go(at(9), at(5)).expect("moved again");
        kept.back().expect("back");
        kept.back().expect("nothing left to undo");
        assert_eq!(kept.close(), ["left"]);
        assert_eq!(*log.borrow(), ["put 5", "put 9", "put 1", "close"]);
    }

    #[test]
    fn a_panic_mid_move_puts_the_window_back_and_closes_it() {
        let log = RefCell::new(Vec::new());
        let panicked = catch_unwind(AssertUnwindSafe(|| {
            let kept = Kept::new(Fake(&log));
            kept.go(at(5), at(1)).expect("moved");
            panic!("a step broke");
        }));
        assert!(panicked.is_err());
        assert_eq!(*log.borrow(), ["put 5", "put 1", "close"]);
        let log = RefCell::new(Vec::new());
        let unmoved = catch_unwind(AssertUnwindSafe(|| {
            let _kept = Kept::new(Fake(&log));
            panic!("a step broke");
        }));
        assert!(unmoved.is_err());
        assert_eq!(*log.borrow(), ["close"], "never moved, so only closed");
    }

    #[test]
    fn a_window_a_step_left_moved_goes_back_before_it_closes() {
        let log = RefCell::new(Vec::new());
        let kept = Kept::new(Fake(&log));
        kept.go(at(5), at(1)).expect("moved");
        assert_eq!(kept.close(), ["left"]);
        assert_eq!(*log.borrow(), ["put 5", "put 1", "close"]);
    }
}

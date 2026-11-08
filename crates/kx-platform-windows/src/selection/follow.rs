//! Where the selection handler sits: on the box last looked at while it may show chips, added again where an add
//! failed, gone from a closed box, and left as it is after a look that failed or came late. Pure over a `Hook`, so a
//! fake proves each move.

use kx_module_api::ports::{SelError, SelKind, SelSpot};

/// Adds and removes the selection handler on an element.
pub(crate) trait Hook {
    /// One element, as the OS hands it over.
    type El: Clone;

    /// Adds the handler on `el` alone; true when the OS took it.
    fn add(&self, el: &Self::El) -> bool;

    /// Removes the handler from `el`, where it was added.
    fn remove(&self, el: &Self::El);
}

/// What a look found of its box.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Seen {
    /// A box that may show chips.
    Open,
    /// A closed box: a password box, one of unknown kind, or one of ours.
    Closed,
    /// Nothing sure: the look failed or came past its deadline, so no OS call follows.
    Unsure,
}

impl Seen {
    /// What `spot` found; unsure when it failed or came `late`.
    pub(crate) fn of(spot: &Result<SelSpot, SelError>, late: bool) -> Self {
        match spot {
            Err(_) => Self::Unsure,
            Ok(_) if late => Self::Unsure,
            Ok(spot) if spot.kind == SelKind::Closed => Self::Closed,
            Ok(_) => Self::Open,
        }
    }
}

/// Whether box ids `a` and `b` name the same box; a box with no id (0) is never the same.
pub(crate) fn same(a: u64, b: u64) -> bool {
    a != 0 && a == b
}

/// Where the selection handler is: the box last looked at, and the element it was added on while it is.
pub(crate) struct Heard<E> {
    id: u64,
    on: Option<E>,
}

impl<E> Default for Heard<E> {
    fn default() -> Self {
        Self { id: 0, on: None }
    }
}

impl<E: Clone> Heard<E> {
    /// Moves the handler after a look at box `id` in `el` that saw `seen`: it stays on the same open box, is added
    /// again where it failed, moves to another open box, leaves a closed one unheard, and an unsure look touches nothing.
    pub(crate) fn look<H: Hook<El = E>>(&mut self, hook: &H, el: &E, id: u64, seen: Seen) {
        if seen == Seen::Unsure {
            return;
        }
        let open = seen == Seen::Open;
        let here = open && same(self.id, id);
        if here && self.on.is_some() {
            return;
        }
        if !here {
            self.leave(hook);
        }
        self.id = id;
        if open && hook.add(el) {
            self.on = Some(el.clone());
        }
    }

    /// Removes the handler from where it is, if it is anywhere.
    pub(crate) fn leave<H: Hook<El = E>>(&mut self, hook: &H) {
        if let Some(old) = self.on.take() {
            hook.remove(&old);
        }
    }
}

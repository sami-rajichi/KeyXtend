//! Where the selection handler sits: on the box last looked at while it may show chips, added again where an add
//! failed, and gone from a closed box. Pure over a `Hook`, so a fake proves each move.

/// Adds and removes the selection handler on an element.
pub(crate) trait Hook {
    /// One element, as the OS hands it over.
    type El: Clone;

    /// Adds the handler on `el` alone; true when the OS took it.
    fn add(&self, el: &Self::El) -> bool;

    /// Removes the handler from `el`, where it was added.
    fn remove(&self, el: &Self::El);
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
    /// Moves the handler for a look at box `id` in `el`, which may show chips when `open`: it stays on the same box,
    /// is added again where it failed, moves to another box, and leaves a closed one unheard.
    pub(crate) fn look<H: Hook<El = E>>(&mut self, hook: &H, el: &E, id: u64, open: bool) {
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

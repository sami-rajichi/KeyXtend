//! Where the selection handler sits: on the box last looked at while it may show chips, added again where an add
//! failed, gone from a closed box, and left as it is after a look that failed or came late. A box whose window hangs
//! gets no add, and keeps a handler it has until a later look finds that window answering, or the watch stops. Pure
//! over a `Hook`, so a fake proves each move.

use kx_module_api::ports::{SelError, SelKind, SelSpot};

/// The one rule for whether two box ids name the same box, which the neutral read shares.
pub(crate) use kx_platform::selection::same;

/// Adds and removes the selection handler on an element.
pub(crate) trait Hook {
    /// One element, as the OS hands it over.
    type El;
    /// Where the handler goes on an element, with what its add, its removal and the hung check need.
    type On;

    /// Where an add on `el` would go, read now.
    fn at(&self, el: &Self::El) -> Self::On;

    /// Adds the handler on `on` alone; false when the OS refused it.
    fn add(&self, on: &Self::On) -> bool;

    /// Removes the handler from `on`, where it was added.
    fn remove(&self, on: &Self::On);

    /// True while the app holding `on` does not answer, so an add or a removal would wait on it.
    fn hung(&self, on: &Self::On) -> bool;
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

/// Where the selection handler is: the box last looked at, where it was added on that box, and the boxes it waits to
/// leave while their windows hang.
pub(crate) struct Heard<O> {
    id: u64,
    on: Option<O>,
    left: Vec<(u64, O)>,
}

impl<O> Default for Heard<O> {
    fn default() -> Self {
        Self {
            id: 0,
            on: None,
            left: Vec::new(),
        }
    }
}

impl<O> Heard<O> {
    /// Moves the handler after a look at box `id` in `el` that saw `seen`: it stays on the same open box, is added
    /// again where it failed, moves to another open box, leaves a closed one unheard, and an unsure look touches nothing.
    pub(crate) fn look<H: Hook<On = O>>(&mut self, hook: &H, el: &H::El, id: u64, seen: Seen) {
        if seen == Seen::Unsure {
            return;
        }
        self.sweep(hook);
        let open = seen == Seen::Open;
        let here = open && same(self.id, id);
        if here && self.on.is_some() {
            return;
        }
        if !here {
            self.away(hook);
        }
        self.id = id;
        if open {
            self.on = match self.left.iter().position(|(at, _)| same(*at, id)) {
                // The box whose removal waited takes its handler back, with no second add.
                Some(at) => Some(self.left.swap_remove(at).1),
                None => Self::add(hook, el),
            };
        }
    }

    /// Adds the handler on `el`, unless its window reads hung, when a later look adds it once that window answers.
    fn add<H: Hook<On = O>>(hook: &H, el: &H::El) -> Option<O> {
        let on = hook.at(el);
        (!hook.hung(&on) && hook.add(&on)).then_some(on)
    }

    /// Removes the handler from everywhere it is, at the watch's stop, hung or not, so none is left for good.
    pub(crate) fn leave<H: Hook<On = O>>(&mut self, hook: &H) {
        let left = self.left.drain(..).map(|(_, on)| on);
        for on in self.on.take().into_iter().chain(left) {
            hook.remove(&on);
        }
    }

    /// Takes the handler off the box last looked at, or keeps it there for later while that box's window hangs.
    fn away<H: Hook<On = O>>(&mut self, hook: &H) {
        if let Some(on) = self.on.take() {
            if hook.hung(&on) {
                self.left.push((self.id, on));
            } else {
                hook.remove(&on);
            }
        }
    }

    /// Removes the handlers kept on boxes whose windows answer again.
    fn sweep<H: Hook<On = O>>(&mut self, hook: &H) {
        self.left.retain(|(_, on)| {
            let wait = hook.hung(on);
            if !wait {
                hook.remove(on);
            }
            wait
        });
    }
}

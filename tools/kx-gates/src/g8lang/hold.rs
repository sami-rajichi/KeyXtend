//! `g8 arabic`'s holds: after each step that took the front, G8's own target-window comes back there in the run's
//! layout, a fresh one or the one a step keeps, such as the scroller's, whose close a fresh start would force.

use std::cell::RefCell;

use kx_module_api::ports::LayoutId;

use super::{Own, Tap, named};

/// Nothing when `w` types in `to`, as the session should; else why the hold failed.
fn typed<O: Own>(own: &O, w: &O::Window, to: LayoutId) -> Result<(), String> {
    match own.layout(w) {
        Some(now) if now == to => Ok(()),
        now => Err(format!(
            "the session types in layout {} instead of {to}",
            named(now)
        )),
    }
}

/// Brings a fresh target-window to the front again after a step closed its own, so the keyboard follows `to` and not
/// an app whose layout may lag; the layout the app in front showed before. Refused, with that window closed, when the
/// session no longer types in `to`.
pub(super) fn hold<O: Own>(own: &O, to: LayoutId) -> Result<Option<LayoutId>, String> {
    let behind = own.in_front();
    let w = own.open(Tap::Plain)?;
    if let Err(e) = typed(own, &w, to) {
        own.close(w);
        return Err(e);
    }
    Ok(behind)
}

/// Like `hold`, through `w`, a window a step keeps, as a fresh start would close it; `w` stays open either way.
pub(super) fn hold_on<O: Own>(
    own: &O,
    w: &O::Window,
    to: LayoutId,
) -> Result<Option<LayoutId>, String> {
    let behind = own.in_front();
    own.front(w)?;
    typed(own, w, to).map(|()| behind)
}

/// The holds a G8 run calls after each step that took the front: through a fresh own window, or through window `W`
/// that a step keeps.
pub trait Holds<W> {
    /// After a step that closed G8's own window.
    fn fresh(&self) -> Result<(), String>;
    /// While a step keeps `w`, whose close a fresh hold's start would force.
    fn kept(&self, w: &W) -> Result<(), String>;
}

/// The holds of a run in the owner's layout: nothing to bring back.
pub struct Free;

impl<W> Holds<W> for Free {
    fn fresh(&self) -> Result<(), String> {
        Ok(())
    }

    fn kept(&self, _: &W) -> Result<(), String> {
        Ok(())
    }
}

/// The holds of `g8 arabic` in `to`, each noting the layout it found in front.
pub(super) struct Held<'a, O: Own> {
    own: &'a O,
    to: LayoutId,
    behind: RefCell<Vec<Option<String>>>,
}

impl<'a, O: Own> Held<'a, O> {
    /// The holds through `own`'s windows in `to`, none taken yet.
    pub(super) fn new(own: &'a O, to: LayoutId) -> Self {
        Self {
            own,
            to,
            behind: RefCell::new(Vec::new()),
        }
    }

    /// The layout each hold found in front, in order.
    pub(super) fn behind(self) -> Vec<Option<String>> {
        self.behind.into_inner()
    }

    /// Notes the layout a hold that `found` it saw in front.
    fn note(&self, found: Result<Option<LayoutId>, String>) -> Result<(), String> {
        let behind = found?.map(|l| l.to_string());
        self.behind.borrow_mut().push(behind);
        Ok(())
    }
}

impl<O: Own> Holds<O::Window> for Held<'_, O> {
    fn fresh(&self) -> Result<(), String> {
        self.note(hold(self.own, self.to))
    }

    fn kept(&self, w: &O::Window) -> Result<(), String> {
        self.note(hold_on(self.own, w, self.to))
    }
}

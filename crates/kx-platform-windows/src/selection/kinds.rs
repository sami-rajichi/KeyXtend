//! The kind of the focused box for the chips: what UI Automation and Win32 tell about it, read through the field
//! watch's table (password boxes, hiding windows) and the selection table (control types, terminals), and the walk up
//! from a focused element with no text to the one that holds its selection.

use kx_module_api::ports::SelKind;

use super::Table;
use crate::field;

/// What UI Automation and Win32 tell about the focused element; never its text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Facts {
    /// Its UI Automation control type; `None` when it cannot be read.
    pub(crate) control: Option<i32>,
    /// UI Automation's password flag; `None` when it cannot be read.
    pub(crate) password: Option<bool>,
    /// The base class of its own window; empty when it has none.
    pub(crate) class: String,
    /// The style bits of its own window.
    pub(crate) style: u32,
    /// The class of the top-level window it sits in.
    pub(crate) top: String,
    /// Its value pattern's read-only answer; `None` without one.
    pub(crate) fixed: Option<bool>,
    /// It shows in our own process.
    pub(crate) ours: bool,
}

/// The kind `facts` tell under the field watch's table `boxes` and the selection table `table`: closed first, then a
/// terminal by its class, then closed again in a window that hides its boxes, then by control type. `range` asks the
/// selection's range whether it is read-only, only for an edit control whose value pattern did not say writable.
pub(crate) fn kind(
    facts: &Facts,
    boxes: &field::Table,
    table: &Table,
    range: impl FnOnce() -> Option<bool>,
) -> SelKind {
    let (class, top) = (facts.class.as_str(), facts.top.as_str());
    let Some(control) = facts.control else {
        return SelKind::Closed;
    };
    if closed(facts, boxes, table) {
        return SelKind::Closed;
    }
    if table.terminal(class) || table.terminal(top) {
        return SelKind::Terminal;
    }
    if boxes.opaque(class) || boxes.opaque(top) {
        return SelKind::Closed;
    }
    if table.edits(control) {
        if facts.fixed == Some(false) || range() == Some(false) {
            SelKind::Editable
        } else {
            SelKind::ReadOnly
        }
    } else if table.reads(control) {
        SelKind::ReadOnly
    } else {
        SelKind::Closed
    }
}

/// Whether `facts` close the box whatever its kind: our own, a password box by UI Automation's flag or the edit
/// style, an unread flag, or a table that did not parse. An unread control type closes it too, in `kind`.
fn closed(facts: &Facts, boxes: &field::Table, table: &Table) -> bool {
    let hides = boxes.hides(&facts.class, facts.style);
    let blind = boxes.is_closed() || table.is_closed();
    facts.ours || hides || blind || facts.password != Some(false)
}

/// Whether `facts` keep a look on the focused element itself, so no ancestor is read: an unread control type, a
/// closed box, or a window that hides its boxes.
pub(crate) fn shut(facts: &Facts, boxes: &field::Table, table: &Table) -> bool {
    let opaque = boxes.opaque(&facts.class) || boxes.opaque(&facts.top);
    facts.control.is_none() || closed(facts, boxes, table) || opaque
}

/// What the walk up from the focused element asks of the elements it meets.
pub(crate) trait Tree {
    /// One element, as the OS hands it over.
    type El;

    /// Whether `el` has a text pattern.
    fn texts(&self, el: &Self::El) -> bool;

    /// Whether `el`'s facts keep a look on it, by `shut`.
    fn shut(&self, el: &Self::El) -> bool;

    /// The parent of `el`; `None` at the top, past the look's deadline, or when the app does not answer.
    fn parent(&self, el: &Self::El) -> Option<Self::El>;
}

/// The element a look reads for the focused `el`: `el` when it has a text pattern or is shut, else its nearest
/// ancestor within `cap` levels that has one, else `el`, whose own kind then tells.
pub(crate) fn climb<T: Tree>(tree: &T, el: T::El, cap: u8) -> T::El {
    if tree.texts(&el) || tree.shut(&el) {
        return el;
    }
    let mut up: Option<T::El> = None;
    for _ in 0..cap {
        let Some(next) = tree.parent(up.as_ref().unwrap_or(&el)) else {
            break;
        };
        if tree.texts(&next) {
            return next;
        }
        up = Some(next);
    }
    el
}

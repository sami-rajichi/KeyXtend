//! The kind of the focused box for the chips: what UI Automation and Win32 tell about it, read through the field
//! watch's table (password boxes, hiding windows) and the selection table (control types, terminals).

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

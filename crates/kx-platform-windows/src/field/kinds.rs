//! The kind of the focused box: what UI Automation and Win32 tell about it, read through the kind table into the
//! shared facts, which `kx_platform::field::kind` orders.

use kx_module_api::ports::FieldKind;
use kx_platform::field::{self, BoxFacts};

use super::Table;

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
    /// Whether a writable value pattern or a text pattern answers.
    pub(crate) editable: bool,
    /// Whether its value pattern says it is read-only.
    pub(crate) read_only: bool,
}

/// The kind `facts` tell under `table`: UIA's flag or the edit style marks a password box, a closed table or an
/// unread control type leaves a fact missing, and `probe` asks a would-be text box whether it is read-only.
pub(crate) fn kind(facts: &Facts, table: &Table, probe: impl FnOnce() -> bool) -> FieldKind {
    let hides = table.hides(&facts.class, facts.style);
    let shared = BoxFacts {
        password: if hides { Some(true) } else { facts.password },
        known: !table.is_closed() && facts.control.is_some(),
        hidden: table.opaque(&facts.class) || table.opaque(&facts.top),
        text: facts.control.is_some_and(|c| table.holds_text(c)) || facts.editable,
        read_only: facts.read_only,
    };
    field::kind(shared, probe)
}

//! The kind of the focused box, decided from what UI Automation and Win32 tell about it and the kind table.

use kx_module_api::ports::FieldKind;

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
}

/// The kind `facts` tell under `table`: a password box by UIA or the edit style first, then unknown when a fact could
/// not be read or the window hides its boxes, then text for an editable element, else no box.
pub(crate) fn kind(facts: &Facts, table: &Table) -> FieldKind {
    if facts.password == Some(true) || table.hides(&facts.class, facts.style) {
        return FieldKind::Password;
    }
    let Some(control) = facts.control else {
        return FieldKind::Unknown;
    };
    if facts.password.is_none() || table.opaque(&facts.class) || table.opaque(&facts.top) {
        return FieldKind::Unknown;
    }
    if table.holds_text(control) || facts.editable {
        FieldKind::Text
    } else {
        FieldKind::None
    }
}

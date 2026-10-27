//! The kind of the focused box from neutral facts, in one fail-closed order every OS shares; an adapter only fills
//! the facts from its own signs and tables.

use kx_module_api::ports::FieldKind;

/// What an adapter learned about the focused element, in neutral terms; never its text.
#[allow(
    clippy::struct_excessive_bools,
    reason = "Each is one independent fact the order reads."
)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct BoxFacts {
    /// The platform marks it a password box, by any of its signs; `None` when that could not be read.
    pub password: Option<bool>,
    /// Every other fact was read, and the adapter's table is sound.
    pub known: bool,
    /// Its window hides what its boxes hold.
    pub hidden: bool,
    /// It can hold text: a text control, or an editable one.
    pub text: bool,
    /// It says it is read-only.
    pub read_only: bool,
}

/// The kind `facts` tell: a password box first, then unknown for a missing fact or a hiding window, then no box
/// without text, then text unless it is read-only by its facts or else by `probe`, asked for nothing else.
pub fn kind(facts: BoxFacts, probe: impl FnOnce() -> bool) -> FieldKind {
    if facts.password == Some(true) {
        return FieldKind::Password;
    }
    if !facts.known || facts.password.is_none() || facts.hidden {
        return FieldKind::Unknown;
    }
    if !facts.text {
        return FieldKind::None;
    }
    if facts.read_only || probe() {
        FieldKind::Unknown
    } else {
        FieldKind::Text
    }
}

#[cfg(test)]
mod tests;

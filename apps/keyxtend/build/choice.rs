//! The build script's choices, shared with its test: every release build opens no console, and only a release build
//! without a gate-only feature asks for uiAccess.

/// Cargo's release profile; debug builds keep a console and run unsigned, which Windows would refuse with uiAccess.
pub const RELEASE: &str = "release";
/// The variables Cargo sets for the build script while a gate-only feature (`gate-input`, `gate-voice`) is on.
pub const GATE_VARS: [&str; 2] = ["CARGO_FEATURE_GATE_INPUT", "CARGO_FEATURE_GATE_VOICE"];

/// True when a `profile` build opens no console window, which would take the foreground: every release build.
pub fn no_console(profile: &str) -> bool {
    profile == RELEASE
}

/// True when the manifest asks for uiAccess: a build with no console that is no `gate` build, which runs unsigned and
/// so must never hold uiAccess.
pub fn ui_access(profile: &str, gate: bool) -> bool {
    no_console(profile) && !gate
}

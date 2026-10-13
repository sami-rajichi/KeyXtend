//! The build script's uiAccess choice, shared with its test: only a release build without the gate-only feature asks.

/// Cargo profile that gets uiAccess; debug builds run unsigned, which Windows would refuse with it.
pub const UI_ACCESS_PROFILE: &str = "release";
/// The variable Cargo sets for the build script while the gate-only feature (`gate-input`) is on.
pub const GATE_INPUT_VAR: &str = "CARGO_FEATURE_GATE_INPUT";

/// True when the manifest asks for uiAccess: a `profile` build that is the release one and not the gate-only build,
/// which runs unsigned and so must never hold uiAccess.
pub fn ui_access(profile: &str, gate_input: bool) -> bool {
    profile == UI_ACCESS_PROFILE && !gate_input
}

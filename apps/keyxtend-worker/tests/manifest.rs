//! The built worker carries a manifest that runs it as the user who starts it, never with uiAccess (ADR-0004).
#![cfg(windows)]
#![allow(clippy::unwrap_used, reason = "a failed read should fail the test")]

/// How the manifest runs a program as its caller, with no elevation.
const AS_INVOKER: &[u8] = br#"level="asInvoker""#;
/// How the manifest asks for uiAccess, and how it does not.
const ASKS: &[u8] = br#"uiAccess="true""#;
const REFUSES: &[u8] = br#"uiAccess="false""#;

/// True when `text` is somewhere in `bytes`.
fn holds(bytes: &[u8], text: &[u8]) -> bool {
    bytes.windows(text.len()).any(|w| w == text)
}

#[test]
fn the_worker_runs_as_its_caller_without_ui_access() {
    let exe = std::fs::read(env!("CARGO_BIN_EXE_keyxtend-worker")).unwrap();
    assert!(holds(&exe, AS_INVOKER), "no asInvoker manifest");
    assert!(holds(&exe, REFUSES) && !holds(&exe, ASKS));
}

//! The built app asks for uiAccess in its manifest exactly in the build that pins where code loads from, which is also
//! the build that opens no console window.
#![cfg(windows)]
#![allow(clippy::unwrap_used, reason = "a failed read should fail the test")]

/// How the manifest asks for uiAccess, and how it does not.
const ASKS: &[u8] = br#"uiAccess="true""#;
const REFUSES: &[u8] = br#"uiAccess="false""#;

/// How the manifest asks for UTF-8 narrow text, so the C runtime narrows no variable's name by best fit.
const UTF8: &[u8] = b">UTF-8</activeCodePage>";
/// Where the PE header's place is written, and the subsystem's place after that header.
const PE_AT: usize = 0x3C;
const SUBSYSTEM_AT: usize = 4 + 20 + 68;
/// The subsystem of a window app, which opens no console.
const WINDOWS_APP: u16 = 2;

/// The built app.
fn exe() -> Vec<u8> {
    std::fs::read(env!("CARGO_BIN_EXE_keyxtend")).unwrap()
}

/// The subsystem in `exe`'s PE header.
fn subsystem(exe: &[u8]) -> u16 {
    let pe = u32::from_le_bytes(exe[PE_AT..PE_AT + 4].try_into().unwrap());
    let at = usize::try_from(pe).unwrap() + SUBSYSTEM_AT;
    u16::from_le_bytes([exe[at], exe[at + 1]])
}

/// True when `text` is somewhere in `bytes`.
fn holds(bytes: &[u8], text: &[u8]) -> bool {
    bytes.windows(text.len()).any(|w| w == text)
}

#[test]
fn the_exe_asks_for_ui_access_exactly_when_the_app_pins_its_folders() {
    let exe = exe();
    let (asks, refuses) = (holds(&exe, ASKS), holds(&exe, REFUSES));
    assert_ne!(asks, refuses, "the exe holds one manifest");
    assert_eq!(asks, keyxtend::UI_ACCESS);
}

#[test]
fn only_the_ui_access_build_opens_no_console() {
    assert_eq!(subsystem(&exe()) == WINDOWS_APP, keyxtend::UI_ACCESS);
}

#[test]
fn the_exe_reads_narrow_text_as_utf8() {
    assert!(holds(&exe(), UTF8));
}

//! The save question the gate's own edited tab asks when Ctrl+W closes it: answered with Don't save, found by its UI
//! Automation id, only while the gate's tab is the one in front.
#![cfg(windows)]

use crate::apps::Opened;
use crate::keymap::Reader;

/// The Don't save button `id` to press: only when the gate `sent` the close, its own tab is still `ours` in front, and
/// the app names one.
pub fn answer(sent: bool, ours: bool, id: Option<&str>) -> Option<&str> {
    id.filter(|_| sent && ours)
}

/// Presses button `id` in `app`'s window through its Invoke pattern; it still goes out after the mouse guard stopped
/// the gate, as it closes the gate's own tab.
pub fn dont_save(app: &Opened, id: &str) -> Result<(), String> {
    Reader::new()?.restore(app.hwnd, id)
}

/// The note on app `name`'s tab still open in window `w`, with why Don't save failed when it was `tried`.
pub fn note(name: &str, w: &str, tried: Option<&Result<(), String>>) -> String {
    match tried {
        Some(Err(e)) => format!("{name} left open: {w}; Don't save: {e}"),
        _ => format!("{name} left open: {w}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dont_save_is_pressed_only_in_the_gates_own_tab_it_closed() {
        assert_eq!(answer(true, true, Some("No")), Some("No"));
        assert_eq!(answer(false, true, Some("No")), None, "no close was sent");
        assert_eq!(
            answer(true, false, Some("No")),
            None,
            "another tab or app in front"
        );
        assert_eq!(answer(true, true, None), None, "the app names no button");
    }

    #[test]
    fn a_tab_left_open_says_why_dont_save_failed() {
        let w = "[Notepad] of Notepad.exe";
        assert_eq!(note("notepad", w, None), format!("notepad left open: {w}"));
        assert_eq!(
            note("notepad", w, Some(&Ok(()))),
            format!("notepad left open: {w}")
        );
        let failed = note("notepad", w, Some(&Err("not found".into())));
        assert!(failed.ends_with("; Don't save: not found"), "{failed}");
    }
}

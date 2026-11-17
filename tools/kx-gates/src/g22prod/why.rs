//! Why a dictation typed nothing: the bar's buttons by id with whether each can act, and the copy's log lines a mark of
//! `[g22_product] why_marks` finds since the language began. Neither holds text: the product never logs text.
#![cfg(windows)]

use windows::Win32::Foundation::HWND;
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Accessibility::{
    IUIAutomationElement, TreeScope_Descendants, UIA_ButtonControlTypeId, UIA_ControlTypePropertyId,
};

use crate::g23::live;
use crate::hooklog::read_since;
use crate::keymap::Reader;

/// The words a button's state prints as.
const ON: &str = "on";
const OFF: &str = "off";

/// The lines of `log` stamped at or after `since_us` that one of `marks` finds, each from its mark on.
pub(super) fn marked(log: &str, since_us: i64, marks: &[String]) -> Vec<String> {
    let mark = |l: &str| {
        Ok(marks
            .iter()
            .find_map(|m| l.find(m.as_str()))
            .map(|at| l[at..].trim().to_owned()))
    };
    read_since(log, since_us, mark).unwrap_or_default()
}

/// `buttons` as one line: each id with whether it can act.
pub(super) fn listed(buttons: &[(String, bool)]) -> String {
    let each = buttons
        .iter()
        .map(|(id, on)| format!("{id} {}", if *on { ON } else { OFF }));
    each.collect::<Vec<_>>().join(", ")
}

/// Error `e` with the bar `bar`'s buttons, or why they could not be read.
pub(super) fn with_bar(e: &str, reader: &Reader, bar: HWND) -> String {
    let buttons = buttons(reader, bar).map_or_else(|why| why, |b| listed(&b));
    format!("{e}; the bar's buttons: {buttons}")
}

/// Every button of window `bar`: its UI Automation id and whether it can act.
#[allow(
    unsafe_code,
    reason = "UI Automation calls on objects this client made."
)]
fn buttons(reader: &Reader, bar: HWND) -> Result<Vec<(String, bool)>, String> {
    let want = VARIANT::from(UIA_ButtonControlTypeId.0);
    // SAFETY: a plain call on this client, with a live value.
    let cond = unsafe {
        reader
            .auto()
            .CreatePropertyCondition(UIA_ControlTypePropertyId, &want)
    }
    .map_err(|e| e.to_string())?;
    // SAFETY: plain property reads on a live element.
    let read = |el: &IUIAutomationElement| unsafe {
        Ok((
            el.CurrentAutomationId()?.to_string(),
            el.CurrentIsEnabled()?.as_bool(),
        ))
    };
    live::each(reader, bar, TreeScope_Descendants, &cond, read)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::budget::stamp_us;

    /// Three stamps of the kernel's log format, a second apart.
    const EARLY: &str = "2026-11-16T08:00:01.000000Z";
    const START: &str = "2026-11-16T08:00:02.000000Z";
    const LATE: &str = "2026-11-16T08:00:03.000000Z";

    #[test]
    fn the_marked_lines_since_the_start_print_without_their_stamp() {
        let marks = crate::config::load().unwrap().g22_product.why_marks;
        let typed = "DEBUG kx_mod_voice::actor::typing: a text went to the keyboard";
        let looked =
            "INFO kx_mod_voice::actor::typing: the box in front was looked at again kind=Text";
        let log = [
            format!("{EARLY}  WARN kx_mod_voice: before"),
            format!("{START}  INFO keyxtend::run: started"),
            format!("{START}  DEBUG kx_platform::worker: spawned"),
            format!("{START} {looked}"),
            format!("{START}  {typed}"),
            format!("{LATE}  ERROR kx_mod_voice: broke"),
        ];
        let got = marked(&log.join("\n"), stamp_us(START).unwrap(), &marks);
        assert_eq!(got, [looked, typed, "ERROR kx_mod_voice: broke"]);
    }

    #[test]
    fn the_buttons_print_by_id_with_their_state() {
        let b = [
            ("voice.copy".to_owned(), true),
            ("voice.undo".to_owned(), false),
        ];
        assert_eq!(listed(&b), "voice.copy on, voice.undo off");
        assert_eq!(listed(&[]), "");
    }
}

//! Settings of the probes G7, G19 to G21 and G25: `[edit_keys]`, `[g7]`, `[g19]` to `[g21]` and `[g25]`. G22's
//! settings live in `g22cfg`, G23's in `p9cfg` and G24's in `p12cfg`.
#![cfg(windows)]

use kx_mod_keyboard::legend::is_mark;
use serde::Deserialize;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE;

use crate::board;
use crate::config::GatesConfig;

/// Refuses probe settings that could pick the wrong copy, clean up the owner's copies or pass with nothing checked.
pub fn check(c: &GatesConfig) -> Result<(), String> {
    check_g7(&c.g7)?;
    check_g25(&c.g25, c.mouse_guard.limit_ms)?;
    let g = &c.g20;
    if !(1..=g.copies).contains(&g.paste_back) {
        return Err(format!("g20.paste_back must be 1 to {}", g.copies));
    }
    if c.probes.forget_min_chars == 0 {
        return Err("probes.forget_min_chars must be above 0".to_string());
    }
    if [&g.prefix, &g.hidden_prefix, &c.g25.text]
        .iter()
        .any(|p| p.is_empty())
    {
        return Err("g20 prefixes and g25.text must not be empty".to_string());
    }
    if c.g19.lines.is_empty() || c.g19.lines.iter().any(String::is_empty) {
        return Err("g19.lines must hold at least one line, none empty".to_string());
    }
    Ok(())
}

/// Refuses a G7 run that would check nothing: no word, no keys, a word that starts with a mark, or no opener.
fn check_g7(g: &G7) -> Result<(), String> {
    if g.word.chars().next().is_none_or(is_mark) || g.keys.is_empty() || g.opener.is_empty() {
        return Err("g7 needs an opener, keys and a word that starts with a letter".to_string());
    }
    Ok(())
}

/// Refuses a G25 run that could miss a window or press another key: an empty class or program, an `esc` other than
/// the lone Esc key, which the power dialog gets, a V that is no character key of the product's board, the history key
/// on in its copy (G25 checks Windows' own history; G20 checks ours), or a ready wait outside 1 ms to `most_ms`.
fn check_g25(g: &G25, most_ms: u64) -> Result<(), String> {
    let empty = |list: &[String]| list.is_empty() || list.iter().any(String::is_empty);
    let names = [&g.power_class, &g.power_program];
    if empty(&g.panel_classes) || empty(&g.switcher_classes) || names.iter().any(|n| n.is_empty()) {
        return Err(
            "g25 needs panel and switcher classes, and the power dialog's class and program".into(),
        );
    }
    if g.esc != [VK_ESCAPE.0] {
        return Err("g25.esc must be the Esc key alone, as the power dialog gets it".into());
    }
    if g.history_key {
        return Err(
            "g25.history_key must be false: Win and V must reach Windows' own history".into(),
        );
    }
    crate::p8cfg::wait("g25.ready_ms", g.ready_ms, most_ms)?;
    board::char_key(g.v_scan)
        .map(drop)
        .map_err(|e| format!("g25.v_scan: {e}"))
}

/// Editing shortcuts the probes press: virtual-key combos, pressed in order.
#[derive(Debug, Clone, Deserialize)]
pub struct EditKeys {
    /// Go to the document start.
    pub doc_start: Vec<u16>,
    /// Go to the document end.
    pub doc_end: Vec<u16>,
    /// Go to the line start.
    pub line_start: Vec<u16>,
    /// Select to the line end.
    pub select_line: Vec<u16>,
    /// Go down one line.
    pub next_line: Vec<u16>,
    /// Paste.
    pub paste: Vec<u16>,
}

/// Selection read-back probe.
#[derive(Debug, Clone, Deserialize)]
pub struct G19 {
    /// Lines of the selection document; each is selected to its end and read through UIA.
    pub lines: Vec<String>,
    /// Longest selection text read, in characters, over all selected ranges.
    pub max_chars: usize,
    /// Name of Chrome's text box, from its `aria-label`.
    pub box_name: String,
    /// Style of Chrome's text box.
    pub box_css: String,
}

/// Clipboard history probe.
#[derive(Debug, Clone, Deserialize)]
pub struct G20 {
    /// Copies the listener must keep: half written by kx-gates, half copied in Notepad.
    pub copies: usize,
    /// Copies written with no wait between them, which must all be heard.
    pub burst: usize,
    /// Copies carrying an exclusion marker, which it must skip.
    pub excluded: usize,
    /// Start of every kept test text.
    pub prefix: String,
    /// Start of every excluded test text.
    pub hidden_prefix: String,
    /// The entry pasted back, counted from the newest (1 is the newest).
    pub paste_back: usize,
    /// Longest text the listener keeps, in characters.
    pub max_chars: usize,
    /// Longest wait for the listener to report a copy, in ms.
    pub event_wait_ms: u64,
}

/// Password-field probe.
#[derive(Debug, Clone, Deserialize)]
pub struct G21 {
    /// Name of the plain text field, from its `aria-label`.
    pub user_name: String,
    /// Name of the password field.
    pub pass_name: String,
    /// Style of both fields.
    pub field_css: String,
}

/// A caret key the G7 check presses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Move {
    /// To the start of the text.
    Home,
    /// To the end of the text.
    End,
    /// One letter left, which is forward in right-to-left text.
    Left,
    /// One letter right, which is back in right-to-left text.
    Right,
}

/// G7 Arabic panel check.
#[derive(Debug, Clone, Deserialize)]
pub struct G7 {
    /// The screen-reader name of the face's Settings key, which opens the panel.
    pub opener: String,
    /// The word typed into the panel's field, with harakat.
    pub word: String,
    /// Caret keys pressed after it, in order.
    pub keys: Vec<Move>,
    /// Longest wait for the panel to open, in ms.
    pub open_ms: u64,
}

/// Language key and shortcut probe.
#[derive(Debug, Clone, Deserialize)]
pub struct G25 {
    /// Notepad's text, copied and pasted; it is also the history clean-up prefix.
    pub text: String,
    /// Longest wait for a layout change, a window to show or close, or the shortcuts layer to flip or turn, in ms.
    pub switch_wait_ms: u64,
    /// Classes of the windows that show while the Windows clipboard panel is open (adapter table).
    pub panel_classes: Vec<String>,
    /// The scan code of the product's character key clicked after its Win key to open that panel: V's.
    pub v_scan: u16,
    /// Closes the switcher and the panel, and cancels the power dialog.
    pub esc: Vec<u16>,
    /// Classes of the task switcher's window, which Switch app keeps open (adapter table).
    pub switcher_classes: Vec<String>,
    /// Class of the power dialog's window (adapter table).
    pub power_class: String,
    /// Program that shows the power dialog (adapter table).
    pub power_program: String,
    /// The history key in G25's own copy of the test build; always off, so Win and V reach Windows.
    pub history_key: bool,
    /// Wait after that copy's keyboard shows before G25's first part, in ms.
    pub ready_ms: u64,
}

#[cfg(test)]
mod tests {
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_F4, VK_MENU, VK_RETURN};

    use super::*;

    #[test]
    fn check_refuses_a_bad_paste_back_min_length_or_prefix() {
        let good = crate::config::load().expect("kx-gates.toml loads");
        assert!(check(&good).is_ok());
        let bad: [fn(&mut GatesConfig); 11] = [
            |c| c.g7.word.clear(),
            |c| c.g7.keys.clear(),
            |c| c.g7.opener.clear(),
            |c| c.g7.word.insert(0, '\u{064E}'),
            |c| c.g19.lines.clear(),
            |c| c.g19.lines[0].clear(),
            |c| c.g20.paste_back = 0,
            |c| c.g20.paste_back = c.g20.copies + 1,
            |c| c.probes.forget_min_chars = 0,
            |c| c.g20.prefix.clear(),
            |c| c.g25.text.clear(),
        ];
        for (i, spoil) in bad.iter().enumerate() {
            let mut c = good.clone();
            spoil(&mut c);
            assert!(check(&c).is_err(), "case {i} passed");
        }
    }

    #[test]
    fn check_refuses_a_g25_that_could_miss_its_window_or_close_another() {
        let good = crate::config::load().expect("kx-gates.toml loads");
        let bad: [fn(&mut G25); 15] = [
            |g| g.history_key = true,
            |g| g.ready_ms = 0,
            |g| g.ready_ms = u64::MAX,
            |g| g.switcher_classes.clear(),
            |g| g.switcher_classes.push(String::new()),
            |g| g.panel_classes.clear(),
            |g| g.panel_classes[0].clear(),
            |g| g.power_class.clear(),
            |g| g.power_program.clear(),
            |g| g.esc.clear(),
            |g| g.esc = vec![VK_RETURN.0],
            |g| g.esc = vec![VK_MENU.0, VK_F4.0],
            |g| g.esc.push(VK_RETURN.0),
            |g| g.v_scan = 0x01,
            |g| g.v_scan = 0xFFFF,
        ];
        for (i, spoil) in bad.iter().enumerate() {
            let mut c = good.clone();
            spoil(&mut c.g25);
            let e = check(&c).expect_err(&format!("case {i} passed"));
            assert!(e.starts_with("g25"), "case {i}: {e}");
        }
    }
}

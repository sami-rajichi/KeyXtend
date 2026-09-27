//! Settings of the stage-1b probes G19, G20, G21 and G25, from `[edit_keys]` and `[g19]` to `[g25]`.

use serde::Deserialize;

use crate::config::HarnessConfig;

/// Refuses stage-1b settings that could pick the wrong copy or clean up the owner's copies.
pub fn check(c: &HarnessConfig) -> Result<(), String> {
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
    Ok(())
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
    /// Undo.
    pub undo: Vec<u16>,
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
    /// A stand-in pill whose anchor must land on screen.
    pub pill: PillCfg,
}

/// The stand-in pill's size and its gap from the text, in px.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct PillCfg {
    /// Width.
    pub w: i32,
    /// Height.
    pub h: i32,
    /// Gap between the text and the pill.
    pub gap: i32,
}

/// Clipboard history probe.
#[derive(Debug, Clone, Deserialize)]
pub struct G20 {
    /// Copies the listener must keep: half written by the harness, half copied in Notepad.
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

/// Language key and shortcut probe.
#[derive(Debug, Clone, Deserialize)]
pub struct G25 {
    /// Notepad's text, copied and pasted; it is also the history clean-up prefix.
    pub text: String,
    /// Longest wait for a layout change or a window switch, in ms.
    pub switch_wait_ms: u64,
    /// Classes of the windows that show while the Windows clipboard panel is open (adapter table).
    pub panel_classes: Vec<String>,
    /// Opens the Windows clipboard window.
    pub win_v: Vec<u16>,
    /// Closes it.
    pub esc: Vec<u16>,
    /// Switches to the previous window.
    pub alt_tab: Vec<u16>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_refuses_a_bad_paste_back_min_length_or_prefix() {
        let good = crate::config::load().expect("harness.toml loads");
        assert!(check(&good).is_ok());
        let bad: [fn(&mut HarnessConfig); 5] = [
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
}

//! Settings of the stage-1b probes G19 to G25, from `[edit_keys]` and `[g19]` to `[g25]`.

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
    if [&g.prefix, &g.hidden_prefix, &c.g25.text, &c.g23.length_mark]
        .iter()
        .any(|p| p.is_empty())
    {
        return Err("g20 prefixes, g23.length_mark and g25.text must not be empty".to_string());
    }
    if c.g19.lines.is_empty() || c.g19.lines.iter().any(String::is_empty) {
        return Err("g19.lines must hold at least one line, none empty".to_string());
    }
    if c.g23.prompt_programs.is_empty() {
        return Err("g23.prompt_programs must name the Hello prompt".to_string());
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
    /// Longest wait for the face's pill to show or hide, in ms.
    pub pill_wait_ms: u64,
    /// How far the pill may sit from where the harness expects it, in physical px.
    pub pill_slack_px: i32,
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

/// Windows Hello quick-fill probe.
#[derive(Debug, Clone, Deserialize)]
pub struct G23 {
    /// Longest wait for the owner to answer Windows Hello, in ms.
    pub hello_wait_ms: u64,
    /// Programs that show the Hello prompt (adapter table).
    pub prompt_programs: Vec<String>,
    /// Without Hello, how long Fill gets to (wrongly) type or prompt before the fields are checked, in ms.
    pub refuse_wait_ms: u64,
    /// Put before the password length that the test page adds to its title.
    pub length_mark: String,
}

/// Snip probe.
#[derive(Debug, Clone, Deserialize)]
pub struct G24 {
    /// Typed into target-window, so the snipped region holds a known pattern.
    pub text: String,
    /// The region from the text box's top-left: x, y, width and height in logical px.
    pub region_px: [f32; 4],
    /// Longest wait for the overlay to show or the snip file to appear, in ms.
    pub wait_ms: u64,
    /// Window class of the taskbar, which the overlay must cover (adapter table).
    pub taskbar_class: String,
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
        let bad: [fn(&mut HarnessConfig); 9] = [
            |c| c.g19.lines.clear(),
            |c| c.g19.lines[0].clear(),
            |c| c.g20.paste_back = 0,
            |c| c.g20.paste_back = c.g20.copies + 1,
            |c| c.probes.forget_min_chars = 0,
            |c| c.g20.prefix.clear(),
            |c| c.g25.text.clear(),
            |c| c.g23.prompt_programs.clear(),
            |c| c.g23.length_mark.clear(),
        ];
        for (i, spoil) in bad.iter().enumerate() {
            let mut c = good.clone();
            spoil(&mut c);
            assert!(check(&c).is_err(), "case {i} passed");
        }
    }
}

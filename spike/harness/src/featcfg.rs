//! Settings of the probes G7 and G19 to G25, from `[edit_keys]`, `[g7]` and `[g19]` to `[g25]`.

use std::collections::HashSet;

use serde::Deserialize;
use spike_core::legend::is_mark;
use spike_core::voicecfg::VoiceConfig;

use crate::config::HarnessConfig;

/// Refuses probe settings that could pick the wrong copy, clean up the owner's copies or pass with nothing checked.
pub fn check(c: &HarnessConfig) -> Result<(), String> {
    check_g7(&c.g7)?;
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
    let g = &c.g22;
    let bad_layout = g
        .sentences
        .iter()
        .any(|s| s.layout_id().is_none() || s.text.is_empty());
    if g.sentences.is_empty() || bad_layout || !plain_name(&g.clips) || g.type_gaps_ms.is_empty() {
        return Err(
            "g22 needs sentences with a 4-digit hex layout and text, a plain clips folder name and typing gaps"
                .to_string(),
        );
    }
    Ok(())
}

/// Refuses a G7 run that would check nothing: no word, no keys, or a word that starts with a mark.
fn check_g7(g: &G7) -> Result<(), String> {
    if g.word.chars().next().is_none_or(is_mark) || g.keys.is_empty() {
        return Err("g7 needs keys and a word that starts with a letter".to_string());
    }
    Ok(())
}

/// Refuses two G22 sentences in one voice language, since the bench scores each clip by its language.
pub fn one_per_language(g: &G22, v: &VoiceConfig) -> Result<(), String> {
    let mut seen = HashSet::new();
    let ids = g.sentences.iter().filter_map(Sentence::layout_id);
    if ids.map(|id| v.language(id)).all(|lang| seen.insert(lang)) {
        Ok(())
    } else {
        Err("g22.sentences must each be in a different voice language".to_string())
    }
}

/// A single folder name: not empty, not a dot name, no drive or separator; the bench deletes in it.
fn plain_name(n: &str) -> bool {
    let mut parts = std::path::Path::new(n).components();
    matches!(
        (parts.next(), parts.next()),
        (Some(std::path::Component::Normal(_)), None)
    ) && !n.contains(':')
}

/// One sentence the owner reads for G22.
#[derive(Debug, Clone, Deserialize)]
pub struct Sentence {
    /// Keyboard layout (LANGID, 4 hex digits) Notepad switches to first.
    pub layout: String,
    /// The words read aloud.
    pub text: String,
}

/// Digits in a LANGID written in hex.
const LAYOUT_DIGITS: usize = 4;
/// Base of the LANGID digits.
const HEX: u32 = 16;

impl Sentence {
    /// The layout as a LANGID; `None` unless it is 4 hex digits.
    pub fn layout_id(&self) -> Option<u16> {
        let l = &self.layout;
        (l.len() == LAYOUT_DIGITS)
            .then(|| u16::from_str_radix(l, HEX).ok())
            .flatten()
    }
}

/// Voice probe (G22) and its bench.
#[derive(Debug, Clone, Deserialize)]
pub struct G22 {
    /// The sentences, read in this order.
    pub sentences: Vec<Sentence>,
    /// How long the Mic stays on for each sentence, in ms.
    pub record_ms: u64,
    /// Longest wait for the words after Mic is clicked again, in ms.
    pub text_wait_ms: u64,
    /// How long a failed Mic stop click is retried, in ms.
    pub stop_wait_ms: u64,
    /// Pause between sentences, in ms.
    pub pause_ms: u64,
    /// Longest document text read back, in characters.
    pub max_chars: i32,
    /// Folder under TEMP where the face's worker keeps the clips; a plain name.
    pub clips: String,
    /// Folder of the staged worker the bench starts, relative to the harness settings folder.
    pub worker_dir: String,
    /// Longest wait for the bench worker to load or answer, in ms.
    pub answer_wait_ms: u64,
    /// Gaps between typed characters the typing check tries, in ms; 0 types one batch.
    pub type_gaps_ms: Vec<u64>,
}

impl G22 {
    /// The clips folder.
    pub fn clips_dir(&self) -> std::path::PathBuf {
        std::env::temp_dir().join(&self.clips)
    }
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
    /// The word typed into the panel's field, with harakat.
    pub word: String,
    /// Caret keys pressed after it, in order.
    pub keys: Vec<Move>,
    /// Longest wait for the panel to open, in ms.
    pub open_ms: u64,
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
        let bad: [fn(&mut HarnessConfig); 19] = [
            |c| c.g7.word.clear(),
            |c| c.g7.keys.clear(),
            |c| c.g7.word.insert(0, '\u{064E}'),
            |c| c.g22.type_gaps_ms.clear(),
            |c| c.g19.lines.clear(),
            |c| c.g19.lines[0].clear(),
            |c| c.g20.paste_back = 0,
            |c| c.g20.paste_back = c.g20.copies + 1,
            |c| c.probes.forget_min_chars = 0,
            |c| c.g20.prefix.clear(),
            |c| c.g25.text.clear(),
            |c| c.g23.prompt_programs.clear(),
            |c| c.g23.length_mark.clear(),
            |c| c.g22.sentences.clear(),
            |c| c.g22.sentences[0].layout = "EN".into(),
            |c| c.g22.clips.clear(),
            |c| c.g22.clips = "..".into(),
            |c| c.g22.clips = "a/b".into(),
            |c| c.g22.clips = "C:\\x".into(),
        ];
        for (i, spoil) in bad.iter().enumerate() {
            let mut c = good.clone();
            spoil(&mut c);
            assert!(check(&c).is_err(), "case {i} passed");
        }
    }

    #[test]
    fn each_sentence_needs_its_own_voice_language() {
        let g = crate::config::load().expect("harness.toml loads").g22;
        let v = spike_core::config::load().expect("spike.toml loads").voice;
        assert!(one_per_language(&g, &v).is_ok());
        let mut twice = g.clone();
        twice.sentences.push(g.sentences[0].clone());
        assert!(
            one_per_language(&twice, &v).is_err(),
            "a second sentence in one language would be scored against the first"
        );
    }
}

//! Settings of the voice probe G22 and its bench: the sentences read aloud, folders and waits, from `[g22]`.
#![cfg(windows)]

use std::collections::HashSet;

use serde::Deserialize;
use spike_core::voicecfg::VoiceConfig;

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
pub fn plain_name(n: &str) -> bool {
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
        let digits = l.len() == LAYOUT_DIGITS && l.bytes().all(|b| b.is_ascii_hexdigit());
        digits.then(|| u16::from_str_radix(l, HEX).ok()).flatten()
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
    /// Folder of the staged worker the bench starts, relative to the gate settings folder.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The LANGID of a sentence whose layout text is `layout`.
    fn id(layout: &str) -> Option<u16> {
        let text = String::new();
        let sentence = Sentence {
            layout: layout.into(),
            text,
        };
        sentence.layout_id()
    }

    #[test]
    fn a_layout_is_exactly_four_hex_digits() {
        assert_eq!(id("0409"), Some(0x0409));
        assert_eq!(id("040c"), Some(0x040c));
        assert_eq!(id("040C"), Some(0x040c));
        for bad in ["+409", "-409", "409", "04090", "04 9", "040g", ""] {
            assert_eq!(id(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn each_sentence_needs_its_own_voice_language() {
        let g = crate::config::load().expect("kx-gates.toml loads").g22;
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

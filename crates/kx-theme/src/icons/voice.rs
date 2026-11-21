//! The dictation bar's icons (design/voice-bar.js `bar()`): its buttons by id, each state's mark and each note's.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The dictation bar: each button by the id the hub gives it, the gear, the marks before a status or a text and each
/// note's icon; and the Voice panel's icons.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VoiceIcons {
    /// Cancel.
    pub cancel: String,
    /// Stop.
    pub stop: String,
    /// The gear, which opens the Voice panel.
    pub gear: String,
    /// Close.
    pub close: String,
    /// Copy.
    pub copy: String,
    /// Copy once it copied.
    pub copied: String,
    /// Undo insert.
    pub undo: String,
    /// Transcribe anyway.
    pub anyway: String,
    /// Try again, after a cloud service failed.
    pub again: String,
    /// Listen again.
    pub listen: String,
    /// Download.
    pub download: String,
    /// Open service, on a cloud note its card or key page settles.
    pub service: String,
    /// Open service on the key note, which opens the key page.
    pub service_key: String,
    /// "Use the local model", after a cloud note.
    pub local: String,
    /// The cloud mark beside the language while a cloud service takes the voice.
    pub cloud: String,
    /// Before Transcribing.
    pub transcribing: String,
    /// Before a text.
    pub text: String,
    /// Before a password box's dots.
    pub password: String,
    /// Before a note `notes` does not list.
    pub note: String,
    /// Before a note, by the hint id the hub gives it.
    pub notes: BTreeMap<String, String>,
    /// The Voice panel's pages.
    pub panel: super::VoicePanelIcons,
}

impl VoiceIcons {
    /// The bar's names with where each is used, such as `voice.stop` or `voice.notes.no-mic`; the panel's are its own.
    pub(super) fn named(&self) -> Vec<(String, &str)> {
        let fixed = [
            ("voice.cancel", &self.cancel),
            ("voice.stop", &self.stop),
            ("voice.gear", &self.gear),
            ("voice.close", &self.close),
            ("voice.copy", &self.copy),
            ("voice.copied", &self.copied),
            ("voice.undo", &self.undo),
            ("voice.anyway", &self.anyway),
            ("voice.again", &self.again),
            ("voice.listen", &self.listen),
            ("voice.download", &self.download),
            ("voice.service", &self.service),
            ("voice.service_key", &self.service_key),
            ("voice.local", &self.local),
            ("voice.cloud", &self.cloud),
            ("voice.transcribing", &self.transcribing),
            ("voice.text", &self.text),
            ("voice.password", &self.password),
            ("voice.note", &self.note),
        ];
        let fixed = fixed.map(|(at, v)| (at.to_owned(), v.as_str()));
        fixed
            .into_iter()
            .chain(super::by_id("voice.notes", &self.notes))
            .collect()
    }
}

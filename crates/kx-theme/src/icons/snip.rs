//! Snip's icons (design/snip-scene.js, snip-editor.js): each key by the hub's id, the editor's tools, the close
//! question and the panel, and the marks a keycap shows once it acted or while a sound is off.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Snip's windows: the shared marks, then each group's icon by id.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnipIcons {
    /// A keycap that acted, beside its done word.
    pub tick: String,
    /// Before a recorded video's length.
    pub length: String,
    /// The microphone's switch while it is off.
    pub mic_off: String,
    /// The computer sound's switch while it is off.
    pub sound_off: String,
    /// The capture bar's keys by id.
    pub bar: BTreeMap<String, String>,
    /// The strip's keys by id.
    pub strip: BTreeMap<String, String>,
    /// The record bar's keys by id.
    pub rec: BTreeMap<String, String>,
    /// The editor's tools by id.
    pub tools: BTreeMap<String, String>,
    /// The editor's keys by id.
    pub edit: BTreeMap<String, String>,
    /// Crop's ✓ and ✗ by id, while its frame waits.
    pub crop: BTreeMap<String, String>,
    /// The close question's mark and answers by id.
    pub ask: BTreeMap<String, String>,
    /// The Snip panel's folder keys by id.
    pub panel: BTreeMap<String, String>,
}

impl SnipIcons {
    /// Every name with where it is used, such as `snip.tick` or `snip.bar.region`.
    pub(super) fn named(&self) -> Vec<(String, &str)> {
        let fixed = [
            ("snip.tick", &self.tick),
            ("snip.length", &self.length),
            ("snip.mic_off", &self.mic_off),
            ("snip.sound_off", &self.sound_off),
        ];
        let maps = [
            ("snip.bar", &self.bar),
            ("snip.strip", &self.strip),
            ("snip.rec", &self.rec),
            ("snip.tools", &self.tools),
            ("snip.edit", &self.edit),
            ("snip.crop", &self.crop),
            ("snip.ask", &self.ask),
            ("snip.panel", &self.panel),
        ];
        let fixed = fixed.map(|(at, v)| (at.to_owned(), v.as_str()));
        let maps = maps.into_iter().flat_map(|(at, m)| super::by_id(at, m));
        fixed.into_iter().chain(maps).collect()
    }
}

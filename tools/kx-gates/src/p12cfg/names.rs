//! `[g24.ids]`, `[g24.clock]` and `[g24.titles]`: Snip's gate ids, its record bar's clock texts, and the window title
//! keys G24 tells our windows apart by.

use serde::Deserialize;

use crate::ftl;

/// Snip's gate ids, as kx-ui names its buttons.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnipIds {
    pub region: String,
    pub window: String,
    pub full: String,
    pub video_region: String,
    pub video_full: String,
    pub bar_close: String,
    pub copy: String,
    pub edit: String,
    pub save: String,
    pub close: String,
    pub record: String,
    pub stop: String,
    pub drop: String,
    pub keep_copy: String,
    pub edit_close: String,
    /// A recorded video's strip keys: Play, which shows only there, and Speed, which steps to the next speed.
    pub play: String,
    pub speed: String,
}

impl SnipIds {
    /// Every id by its key.
    pub(super) fn all(&self) -> [(&'static str, &str); 17] {
        [
            ("region", &self.region),
            ("window", &self.window),
            ("full", &self.full),
            ("video_region", &self.video_region),
            ("video_full", &self.video_full),
            ("bar_close", &self.bar_close),
            ("copy", &self.copy),
            ("edit", &self.edit),
            ("save", &self.save),
            ("close", &self.close),
            ("record", &self.record),
            ("stop", &self.stop),
            ("drop", &self.drop),
            ("keep_copy", &self.keep_copy),
            ("edit_close", &self.edit_close),
            ("play", &self.play),
            ("speed", &self.speed),
        ]
    }

    /// Refuses an empty id, and two keys sharing one.
    pub(super) fn check(&self) -> Result<(), String> {
        let ids = self.all();
        for (i, (key, id)) in ids.iter().enumerate() {
            if id.is_empty() || ids[..i].iter().any(|(_, e)| e == id) {
                return Err(format!("g24.ids.{key} must be an id of its own"));
            }
        }
        Ok(())
    }
}

/// kx-ui's text keys of the record bar's clock as a reader hears it: its word, the join, and the time's %1:%2 text.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnipClock {
    pub word: String,
    pub join: String,
    pub time: String,
}

impl SnipClock {
    /// Refuses a key kx-ui lacks in a shipped language.
    pub(super) fn check(&self) -> Result<(), String> {
        let keys = [
            ("word", &self.word),
            ("join", &self.join),
            ("time", &self.time),
        ];
        for (key, text) in keys {
            ftl::every(text).map_err(|e| format!("g24.clock.{key}: {e}"))?;
        }
        Ok(())
    }
}

/// kx-ui's window title keys G24 tells our windows apart by.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnipTitles {
    /// The frozen screen, the one window of a capture that is not kept out of it.
    pub frozen: String,
    /// The capture windows, each kept out of the capture.
    pub capture: Vec<String>,
    pub editor: String,
    pub ask: String,
    pub keyboard: String,
    pub bubble: String,
}

impl SnipTitles {
    /// Refuses a title key kx-ui lacks in a shipped language, and no capture window.
    pub(super) fn check(&self) -> Result<(), String> {
        if self.capture.is_empty() {
            return Err("g24.titles.capture must list the capture windows".into());
        }
        let one = [
            ("frozen", &self.frozen),
            ("editor", &self.editor),
            ("ask", &self.ask),
            ("keyboard", &self.keyboard),
            ("bubble", &self.bubble),
        ];
        for (key, title) in one {
            ftl::every(title).map_err(|e| format!("g24.titles.{key}: {e}"))?;
        }
        self.capture.iter().try_for_each(|k| {
            ftl::every(k)
                .map(drop)
                .map_err(|e| format!("g24.titles.capture: {e}"))
        })
    }
}

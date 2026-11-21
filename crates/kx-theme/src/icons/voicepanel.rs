//! The Voice panel's icons (design/voice-bar.js `PAGE`): each page, button, mark, line above a list and device kind by
//! the id the hub gives it, and the parts every page shares.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The Voice panel: the shared parts, then each id's icon; an id a map leaves out shows no icon.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VoicePanelIcons {
    /// The models folder line.
    pub room: String,
    /// A stepper's minus.
    pub minus: String,
    /// A stepper's plus.
    pub plus: String,
    /// A choice's chevron back.
    pub before: String,
    /// A choice's chevron on.
    pub after: String,
    /// A pinned text's mark.
    pub pinned: String,
    /// The tick in a list where one row is picked.
    pub ticked: String,
    /// Pin once every ticked text is pinned, which unpins them.
    pub unpin: String,
    /// The line that local models will be very slow on this PC.
    pub slow: String,
    /// The line that the model order is not tested for the language.
    pub untested: String,
    /// A row of the folder list.
    pub folder: String,
    /// Each page's header icon, and a home row's tile, by page id.
    pub pages: BTreeMap<String, String>,
    /// Each button by the hub's id, or its kind when it has none.
    pub buttons: BTreeMap<String, String>,
    /// Each mark beside a row by its id.
    pub marks: BTreeMap<String, String>,
    /// Each line above a list by its id.
    pub tops: BTreeMap<String, String>,
    /// Each Device row's tile by its kind.
    pub kinds: BTreeMap<String, String>,
    /// Each line of the cloud pages by its id: the switch, the off line, the day read, Untested, where keys are made
    /// and a key test's answer.
    pub cloud: BTreeMap<String, String>,
    /// Each Options hint row's bare icon by its option's key, as Insert the text's Paste at once hint.
    pub hints: BTreeMap<String, String>,
}

impl VoicePanelIcons {
    /// Every name with where it is used, such as `voice.panel.room` or `voice.panel.marks.used`.
    pub(super) fn named(&self) -> Vec<(String, &str)> {
        let fixed = [
            ("voice.panel.room", &self.room),
            ("voice.panel.minus", &self.minus),
            ("voice.panel.plus", &self.plus),
            ("voice.panel.before", &self.before),
            ("voice.panel.after", &self.after),
            ("voice.panel.pinned", &self.pinned),
            ("voice.panel.ticked", &self.ticked),
            ("voice.panel.unpin", &self.unpin),
            ("voice.panel.slow", &self.slow),
            ("voice.panel.untested", &self.untested),
            ("voice.panel.folder", &self.folder),
        ];
        let maps = [
            ("voice.panel.pages", &self.pages),
            ("voice.panel.buttons", &self.buttons),
            ("voice.panel.marks", &self.marks),
            ("voice.panel.tops", &self.tops),
            ("voice.panel.kinds", &self.kinds),
            ("voice.panel.cloud", &self.cloud),
            ("voice.panel.hints", &self.hints),
        ];
        let fixed = fixed.map(|(at, v)| (at.to_owned(), v.as_str()));
        let maps = maps.into_iter().flat_map(|(at, m)| super::by_id(at, m));
        fixed.into_iter().chain(maps).collect()
    }
}

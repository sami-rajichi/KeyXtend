//! The selection strip's icons, by chip id (design/selection-chips.html design 4).

use serde::{Deserialize, Serialize};

/// The selection strip: one icon per keycap, and the tick a clicked one shows.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ChipsIcons {
    /// Copy.
    pub copy: String,
    /// Cut.
    pub cut: String,
    /// Paste.
    pub paste: String,
    /// Translate.
    pub translate: String,
    /// A clicked keycap's tick, beside its done word.
    pub tick: String,
}

impl ChipsIcons {
    /// Every name with where it is used, such as `chips.copy`.
    pub(super) fn named(&self) -> [(&'static str, &str); 5] {
        [
            ("chips.copy", &self.copy),
            ("chips.cut", &self.cut),
            ("chips.paste", &self.paste),
            ("chips.translate", &self.translate),
            ("chips.tick", &self.tick),
        ]
    }
}

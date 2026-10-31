//! The clipboard panel's icons, by role (spec §7).

use serde::{Deserialize, Serialize};

/// The clipboard panel: its title, search line, row kinds, footer, storage line and preview.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClipIcons {
    /// The title, which the empty line shares.
    pub title: String,
    /// The search line.
    pub search: String,
    /// Close.
    pub close: String,
    /// A text row.
    pub kind_text: String,
    /// A link row.
    pub kind_link: String,
    /// An image row, until its thumbnail comes.
    pub kind_image: String,
    /// A file row.
    pub kind_file: String,
    /// A video or audio row.
    pub kind_media: String,
    /// A pinned row's mark, and Pin.
    pub pin: String,
    /// Unpin.
    pub unpin: String,
    /// Preview.
    pub preview: String,
    /// Delete, Delete all and the question's Yes.
    pub delete: String,
    /// Insert.
    pub insert: String,
    /// The question's No.
    pub cancel: String,
    /// The shown filter's mark.
    pub shown: String,
    /// The storage line.
    pub room: String,
    /// Open folder.
    pub folder: String,
    /// The preview's Back.
    pub back: String,
}

impl ClipIcons {
    /// Every name with where it is used, such as `clip.pin`.
    pub(super) fn named(&self) -> [(&'static str, &str); 18] {
        [
            ("clip.title", &self.title),
            ("clip.search", &self.search),
            ("clip.close", &self.close),
            ("clip.kind_text", &self.kind_text),
            ("clip.kind_link", &self.kind_link),
            ("clip.kind_image", &self.kind_image),
            ("clip.kind_file", &self.kind_file),
            ("clip.kind_media", &self.kind_media),
            ("clip.pin", &self.pin),
            ("clip.unpin", &self.unpin),
            ("clip.preview", &self.preview),
            ("clip.delete", &self.delete),
            ("clip.insert", &self.insert),
            ("clip.cancel", &self.cancel),
            ("clip.shown", &self.shown),
            ("clip.room", &self.room),
            ("clip.folder", &self.folder),
            ("clip.back", &self.back),
        ]
    }
}

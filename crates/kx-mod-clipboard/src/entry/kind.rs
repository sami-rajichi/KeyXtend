//! The kind rules (module shape): files are media when every extension is in `media.toml`, else files;
//! then text, a link when it is one web address; then an image; else text. An image beside text is not kept.

use std::path::Path;
use std::sync::LazyLock;

use kx_module_api::ports::{ClipKind, ClipPart};
use serde::Deserialize;

use super::{files_of, text_of};

/// The media table, compiled in so it is never read from disk.
const TABLE: &str = include_str!("../../media.toml");
/// The schemes that make a text of one word a link, matched ignoring case.
const LINK_SCHEMES: [&str; 2] = ["http://", "https://"];

/// The media table, parsed once; a broken table knows no media, and a test keeps it sound.
static MEDIA: LazyLock<Option<Media>> = LazyLock::new(|| toml::from_str(TABLE).ok());

/// The extensions of video and audio files, in lower case without the dot.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Media {
    /// Video files.
    video: Vec<String>,
    /// Audio files.
    audio: Vec<String>,
}

/// The parts of a copy worth keeping: an image beside text that is not blank goes, as the HTML or RTF holds the
/// text's look; `None` when no part is left, so no row is kept and no note shows.
#[must_use]
pub fn keep_parts(mut parts: Vec<ClipPart>) -> Option<Vec<ClipPart>> {
    if has_text(&parts) {
        parts.retain(|p| !matches!(p, ClipPart::Image { .. }));
    }
    (!parts.is_empty()).then_some(parts)
}

/// The kind of a copy of `parts`.
pub(super) fn kind_of(parts: &[ClipPart]) -> ClipKind {
    if parts.iter().any(|p| matches!(p, ClipPart::Files(_))) {
        let files = files_of(parts);
        let media = !files.is_empty() && files.iter().all(|p| is_media(p));
        return if media {
            ClipKind::Media
        } else {
            ClipKind::File
        };
    }
    if has_text(parts) {
        if text_of(parts).is_some_and(is_link) {
            ClipKind::Link
        } else {
            ClipKind::Text
        }
    } else if parts.iter().any(|p| matches!(p, ClipPart::Image { .. })) {
        ClipKind::Image
    } else {
        ClipKind::Text
    }
}

/// True when `parts` hold a text part that is not blank.
fn has_text(parts: &[ClipPart]) -> bool {
    text_of(parts).is_some_and(|t| !t.trim().is_empty())
}

/// True when `text`, less the blanks around it, is one web address: a link scheme, more, and no blank.
fn is_link(text: &str) -> bool {
    let text = text.trim();
    let scheme = |s: &str| {
        text.get(..s.len())
            .is_some_and(|h| h.eq_ignore_ascii_case(s))
    };
    let schemed = LINK_SCHEMES
        .iter()
        .any(|s| scheme(s) && text.len() > s.len());
    schemed && !text.chars().any(char::is_whitespace)
}

/// True when `path`'s extension, ignoring case, is in the media table.
fn is_media(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return false;
    };
    let ext = ext.to_lowercase();
    let listed = |m: &Media| m.video.iter().chain(&m.audio).any(|e| *e == ext);
    MEDIA.as_ref().is_some_and(listed)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn the_media_table_is_lower_case_without_dots_or_twins() {
        let media = MEDIA.as_ref().expect("media.toml parses");
        assert!(!media.video.is_empty() && !media.audio.is_empty());
        let mut seen = HashSet::new();
        for ext in media.video.iter().chain(&media.audio) {
            assert_eq!(*ext, ext.to_lowercase(), "{ext}");
            assert!(!ext.is_empty() && !ext.contains('.'), "{ext}");
            assert!(seen.insert(ext), "{ext} twice");
        }
    }

    #[test]
    fn a_shared_extension_is_not_media() {
        assert!(is_media(Path::new("song.OGG")));
        assert!(!is_media(Path::new("main.ts")), "TypeScript too");
        assert!(!is_media(Path::new("folder")));
    }
}

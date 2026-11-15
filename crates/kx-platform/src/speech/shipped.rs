//! The voice files the stage puts beside the worker (ADR-0044): the detector and the self-test clips, named once here
//! for the app and `cargo xtask stage`.

use super::Assets;
use std::path::Path;

/// The folder beside the worker that holds the shipped voice files.
pub const VOICE_FILES: &str = "voice";

/// The voice detector's model file.
pub const DETECTOR: &str = "silero-vad-v6.2-F32.gguf";

/// The self-test clips by ISO 639 code, in the order a model takes them: the first it knows (the voice design).
pub const CLIPS: [(&str, &str); 3] = [
    ("en", "check-en.wav"),
    ("fr", "check-fr.wav"),
    ("ar", "check-ar.wav"),
];

impl Assets {
    /// The shipped files in `VOICE_FILES` under `dir`, the worker's folder, with the clips in `CLIPS`' order.
    #[must_use]
    pub fn beside(dir: &Path) -> Self {
        Self::within(&dir.join(VOICE_FILES))
    }

    /// The shipped files directly in `folder`, with the clips in `CLIPS`' order.
    #[must_use]
    pub fn within(folder: &Path) -> Self {
        let clip = |(lang, file): (&str, &str)| (lang.to_owned(), folder.join(file));
        Self {
            detector: folder.join(DETECTOR),
            clips: CLIPS.into_iter().map(clip).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_files_sit_in_the_voice_folder_beside_the_worker() {
        let dir = Path::new("app");
        let assets = Assets::beside(dir);
        let folder = dir.join(VOICE_FILES);
        assert_eq!(assets.detector, folder.join(DETECTOR));
        let files: Vec<_> = assets.clips.iter().map(|(_, p)| p.clone()).collect();
        let want: Vec<_> = CLIPS.iter().map(|(_, f)| folder.join(f)).collect();
        assert_eq!(files, want);
    }

    #[test]
    fn the_files_sit_directly_in_a_folder_named_as_it_is() {
        let folder = Path::new("assets").join("voice");
        let assets = Assets::within(&folder);
        assert_eq!(assets.detector, folder.join(DETECTOR));
        let first = assets.clips.first().map(|(_, p)| p.clone());
        assert_eq!(first, Some(folder.join(CLIPS[0].1)));
    }

    #[test]
    fn the_clips_go_english_french_then_arabic_as_the_design_lists_them() {
        let langs: Vec<String> = Assets::beside(Path::new("app"))
            .clips
            .into_iter()
            .map(|(l, _)| l)
            .collect();
        assert_eq!(langs, ["en", "fr", "ar"], "ADR-0044 decision 7");
    }
}

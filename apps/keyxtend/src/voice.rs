//! Where voice typing's files live: its sealed history, recordings and counts in a folder of their own under the data
//! folder, with their own key (ADR-0042), and the shipped detector and check clips beside the worker (ADR-0044), or in
//! a debug build that has none there, the workspace's voice folder.

use kx_platform::AppDirs;
use kx_platform::speech::Assets;
use std::path::{Path, PathBuf};

pub use kx_platform::VOICE_DIR;

/// The root manifest, whose stage settings name the folder the voice files are staged from. It is read, not compiled
/// in, as it holds the gate-only flag that no app build but the gate's may hold.
#[cfg(debug_assertions)]
const ROOT_MANIFEST: &str = concat!(root!(), "/Cargo.toml");
/// The keys down to that folder in it.
#[cfg(debug_assertions)]
const FROM: [&str; 5] = ["workspace", "metadata", "stage", "voice", "from"];

/// The voice shelf's folder for the app's `dirs`.
#[must_use]
pub fn folder(dirs: &AppDirs) -> PathBuf {
    dirs.data_dir().path().join(VOICE_DIR)
}

/// The shipped voice files beside the worker, which sits in the program's folder `exe_dir`; a debug build without them
/// there reads the workspace's voice folder, as a plain `cargo run` stages nothing.
#[must_use]
pub fn assets(exe_dir: &Path) -> Assets {
    let staged = Assets::beside(exe_dir);
    if staged.detector.is_file() {
        return staged;
    }
    built().unwrap_or(staged)
}

/// The workspace's voice folder, as the root manifest's stage settings name it.
#[cfg(debug_assertions)]
fn built() -> Option<Assets> {
    let text = std::fs::read_to_string(ROOT_MANIFEST).ok()?;
    let manifest: toml::Value = text.parse::<toml::Table>().ok()?.into();
    let from = FROM.iter().try_fold(&manifest, |v, key| v.get(key))?;
    Some(Assets::within(&Path::new(root!()).join(from.as_str()?)))
}

/// None: a release build reads the voice files only beside the worker.
#[cfg(not(debug_assertions))]
fn built() -> Option<Assets> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use kx_platform::speech::{DETECTOR, VOICE_FILES};
    use kx_test_support::tempdir::TempDir;

    #[test]
    fn a_staged_detector_beside_the_worker_wins() {
        let exe = TempDir::new("voice-staged").unwrap();
        let folder = exe.path().join(VOICE_FILES);
        std::fs::create_dir(&folder).unwrap();
        std::fs::write(folder.join(DETECTOR), b"").unwrap();
        assert_eq!(assets(exe.path()).detector, folder.join(DETECTOR));
    }

    #[cfg(debug_assertions)]
    #[test]
    fn a_debug_build_without_a_stage_reads_the_workspaces_voice_folder() {
        let exe = TempDir::new("voice-plain").unwrap();
        let found = assets(exe.path());
        assert!(!found.detector.starts_with(exe.path()), "not beside it");
        assert!(found.detector.is_file(), "a plain run still has a detector");
        assert!(
            found.clips.iter().all(|(_, p)| p.is_file()),
            "and its clips"
        );
    }
}

//! The voice files go from the workspace's voice folder to the one beside the worker, the detector first and the clips
//! in the platform's order, and a stage missing any of them is refused.

use super::*;
use crate::test_support::{TempDir, stage_config};

/// The file names the platform gives the voice folder, in order, then the test notice.
fn names() -> Vec<String> {
    let clips = CLIPS.iter().map(|(_, file)| (*file).to_string());
    let shipped = std::iter::once(DETECTOR.to_string()).chain(clips);
    shipped.chain(stage_config().voice.notices).collect()
}

#[test]
fn the_test_voice_settings_pass() {
    assert_eq!(stage_config().voice.validate(), Ok(()));
}

#[test]
fn the_folder_is_relative_and_the_notices_plain_names() {
    let mut voice = stage_config().voice;
    voice.from = "../voice".to_string();
    assert_eq!(voice.validate(), Err(ConfigError::NotRelative("voice")));
    let mut voice = stage_config().voice;
    voice.notices = vec!["a/LICENSE".to_string()];
    assert_eq!(voice.validate(), Err(ConfigError::NotPlain("notices")));
    voice.notices.clear();
    assert_eq!(voice.validate(), Err(ConfigError::EmptyList("notices")));
}

#[test]
fn the_detector_the_clips_in_order_and_the_notices_go_beside_the_worker() {
    let (voice, root) = (stage_config().voice, Path::new("ws"));
    let from = paths::join_rel(root.to_path_buf(), &voice.from);
    let copy = |name: String| Copy {
        from: from.join(&name),
        to: Path::new(VOICE_FILES).join(name),
    };
    let want: Vec<Copy> = names().into_iter().map(copy).collect();
    assert_eq!(plan(&voice, root), want);
}

#[test]
fn a_stage_missing_a_voice_file_is_refused_and_one_holding_all_passes() {
    let (dir, voice) = (TempDir::new("voice-files"), stage_config().voice);
    let folder = dir.path().join(VOICE_FILES);
    std::fs::create_dir_all(&folder).unwrap();
    for name in names() {
        let gone = paths::text(&Path::new(VOICE_FILES).join(&name));
        assert_eq!(
            check(dir.path(), &voice),
            Err(StageError::NoVoice(gone).into())
        );
        std::fs::write(folder.join(name), b"").unwrap();
    }
    assert_eq!(check(dir.path(), &voice), Ok(()));
}

#[test]
fn only_files_in_the_voice_folder_count_as_voice_files() {
    assert!(inside(&Path::new(VOICE_FILES).join(DETECTOR)));
    assert!(!inside(Path::new(VOICE_FILES)), "the folder alone");
    assert!(!inside(Path::new(DETECTOR)), "beside the program");
    assert!(!inside(&Path::new("qml").join(VOICE_FILES).join(DETECTOR)));
}

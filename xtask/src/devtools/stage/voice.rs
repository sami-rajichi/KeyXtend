//! The voice files in the stage (ADR-0044): the detector and the check clips the platform names, then their notices, go
//! from the workspace's voice folder into the platform's folder beside the worker; the check refuses a stage missing any.

use std::ffi::OsStr;
use std::path::Path;

use kx_platform::speech::{CLIPS, DETECTOR, VOICE_FILES};
use serde::Deserialize;

use super::config::each;
use super::{Copy, StageError};
use crate::config::ConfigError;
use crate::devtools::{DevError, paths};

/// Where the voice files come from and the notices that ship beside them, from `[workspace.metadata.stage] voice`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Voice {
    /// The folder, under the workspace root, holding the files.
    pub from: String,
    /// The licence and credit files that ship beside them.
    pub notices: Vec<String>,
}

impl Voice {
    /// Rejects a folder that is not a relative path, and notices that are missing or not plain names.
    pub(crate) fn validate(&self) -> Result<(), ConfigError> {
        if !paths::is_relative_plain(&self.from) {
            return Err(ConfigError::NotRelative("voice"));
        }
        each(
            "notices",
            &self.notices,
            paths::is_plain_name,
            ConfigError::NotPlain,
        )
    }

    /// Every file the voice folder ships: the detector, the clips in the platform's order, then the notices.
    fn names(&self) -> Vec<String> {
        let clips = CLIPS.iter().map(|(_, file)| (*file).to_string());
        let shipped = std::iter::once(DETECTOR.to_string()).chain(clips);
        shipped.chain(self.notices.iter().cloned()).collect()
    }
}

/// The copies of every voice file from its folder under the workspace `root` into `VOICE_FILES`.
pub(crate) fn plan(voice: &Voice, root: &Path) -> Vec<Copy> {
    let from = paths::join_rel(root.to_path_buf(), &voice.from);
    let copy = |name: String| Copy {
        from: from.join(&name),
        to: Path::new(VOICE_FILES).join(name),
    };
    voice.names().into_iter().map(copy).collect()
}

/// Refuses `stage` when its voice folder lacks any file `plan` stages, naming the first.
pub(crate) fn check(stage: &Path, voice: &Voice) -> Result<(), DevError> {
    let mut files = voice
        .names()
        .into_iter()
        .map(|n| Path::new(VOICE_FILES).join(n));
    match files.find(|f| !stage.join(f).is_file()) {
        Some(gone) => Err(StageError::NoVoice(paths::text(&gone)).into()),
        None => Ok(()),
    }
}

/// True when the stage-relative `file` lies in `VOICE_FILES`.
pub(crate) fn inside(file: &Path) -> bool {
    let mut parts = file.iter();
    parts.next() == Some(OsStr::new(VOICE_FILES)) && parts.next().is_some()
}

#[cfg(test)]
mod tests;

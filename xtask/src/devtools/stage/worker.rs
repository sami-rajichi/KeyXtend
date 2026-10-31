//! The media worker in the stage (ADR-0004, ADR-0034): its program sits beside the app, and its manifest never asks
//! for uiAccess, which would let a decoder of untrusted images drive other apps.

use std::path::Path;

use super::{StageConfig, StageError};
use crate::devtools::DevError;
use crate::devtools::check::manifest_value;
use crate::devtools::config::DevToolsConfig;

/// What opens an XML element; the program's text is cut at each one, so every element is read.
const TAG_OPEN: char = '<';

/// Refuses `stage` when the worker program is missing, or when any element of its manifest asks for uiAccess as
/// `devtools` names it, in any letter case.
pub(crate) fn check(
    stage: &Path,
    config: &StageConfig,
    devtools: &DevToolsConfig,
) -> Result<(), DevError> {
    let name = config.worker_program();
    let path = stage.join(&name);
    if !path.is_file() {
        return Err(StageError::NoWorker(name).into());
    }
    let text = String::from_utf8_lossy(&std::fs::read(&path)?).into_owned();
    if text.split(TAG_OPEN).any(|piece| asks(piece, devtools)) {
        return Err(StageError::WorkerUiAccess(name).into());
    }
    Ok(())
}

/// True when `piece`, the text after one `<`, is a level element that turns uiAccess on; a comment around it does not
/// hide it.
fn asks(piece: &str, devtools: &DevToolsConfig) -> bool {
    let element = format!("{TAG_OPEN}{piece}");
    let value = manifest_value(&element, &devtools.ui_access_tag, &devtools.ui_access_attr);
    value.is_some_and(|v| v.trim().eq_ignore_ascii_case(&devtools.ui_access_on))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{TempDir, devtools_config, stage_config};

    /// A worker manifest's level element, asking for uiAccess as `value` says.
    fn level(value: &str) -> String {
        format!(r#"<requestedExecutionLevel level="asInvoker" uiAccess="{value}"/>"#)
    }

    /// A stage holding the worker program with `bytes`.
    fn stage(label: &str, bytes: &[u8]) -> TempDir {
        let dir = TempDir::new(label);
        std::fs::write(dir.path().join(stage_config().worker_program()), bytes).unwrap();
        dir
    }

    /// The refusal `check` gives `dir`, if any.
    fn refusal(dir: &TempDir) -> Option<StageError> {
        match check(dir.path(), &stage_config(), &devtools_config()) {
            Err(DevError::Stage(why)) => Some(why),
            _ => None,
        }
    }

    #[test]
    fn a_worker_without_ui_access_passes() {
        let dir = stage("worker-pass", level("false").as_bytes());
        assert_eq!(
            check(dir.path(), &stage_config(), &devtools_config()),
            Ok(())
        );
    }

    #[test]
    fn a_missing_worker_is_refused() {
        let dir = TempDir::new("worker-gone");
        let name = stage_config().worker_program();
        assert_eq!(refusal(&dir), Some(StageError::NoWorker(name)));
    }

    #[test]
    fn a_worker_whose_manifest_asks_for_ui_access_is_refused() {
        let name = stage_config().worker_program();
        let asks = [
            level("true"),
            level("TRUE"),
            "<ms_asmv2:requestedExecutionLevel uiAccess='true'/>".to_string(),
            format!("{}\0junk<{}", level("false"), &level("true")[1..]),
        ];
        for (n, manifest) in asks.iter().enumerate() {
            let dir = stage(&format!("worker-asks-{n}"), manifest.as_bytes());
            let want = Some(StageError::WorkerUiAccess(name.clone()));
            assert_eq!(refusal(&dir), want, "{manifest}");
        }
    }

    #[test]
    fn a_commented_out_ask_still_counts() {
        let manifest = format!("<!-- {} -->", level("true"));
        let dir = stage("worker-comment", manifest.as_bytes());
        assert!(matches!(refusal(&dir), Some(StageError::WorkerUiAccess(_))));
    }
}

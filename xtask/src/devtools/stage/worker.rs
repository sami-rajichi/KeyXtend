//! The media worker in the stage (ADR-0004, ADR-0034): its program sits beside the app, and its manifest never asks
//! for uiAccess, which would let a decoder of untrusted images drive other apps.

use std::path::Path;

use super::{StageConfig, StageError};
use crate::devtools::DevError;
use crate::devtools::check::manifest_value;
use crate::devtools::config::DevToolsConfig;

/// What opens an XML element; the program's text is cut at each one, so every element is read.
const TAG_OPEN: char = '<';
/// The bytes in one UTF-16 code unit, the other text a manifest resource may be stored in.
const WIDE: usize = size_of::<u16>();

/// Refuses `stage` when the worker program is missing, or when any element of its manifest, in UTF-8 or UTF-16LE,
/// asks for uiAccess as `devtools` names it, in any letter case.
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
    let texts = renderings(&std::fs::read(&path)?);
    if texts
        .iter()
        .flat_map(|t| t.split(TAG_OPEN))
        .any(|piece| asks(piece, devtools))
    {
        return Err(StageError::WorkerUiAccess(name).into());
    }
    Ok(())
}

/// `bytes` read as UTF-8, then as UTF-16LE from each byte alignment, so a manifest in either text is seen.
fn renderings(bytes: &[u8]) -> Vec<String> {
    let wide = |skip: usize| {
        let rest = bytes.get(skip..).unwrap_or_default();
        let (pairs, _) = rest.as_chunks::<WIDE>();
        let units: Vec<u16> = pairs.iter().copied().map(u16::from_le_bytes).collect();
        String::from_utf16_lossy(&units)
    };
    let narrow = String::from_utf8_lossy(bytes).into_owned();
    std::iter::once(narrow).chain((0..WIDE).map(wide)).collect()
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

    /// `text` in UTF-16LE, after `lead` bytes that shift its alignment.
    fn wide(text: &str, lead: usize) -> Vec<u8> {
        let units = text.encode_utf16().flat_map(u16::to_le_bytes);
        std::iter::repeat_n(0, lead).chain(units).collect()
    }

    #[test]
    fn a_utf16_manifest_is_read_at_either_alignment() {
        for lead in [0, 1, 2, 3] {
            let dir = stage(&format!("worker-wide-{lead}"), &wide(&level("true"), lead));
            let got = refusal(&dir);
            assert!(
                matches!(got, Some(StageError::WorkerUiAccess(_))),
                "lead {lead}"
            );
            let dir = stage(
                &format!("worker-wide-off-{lead}"),
                &wide(&level("false"), lead),
            );
            assert_eq!(refusal(&dir), None, "lead {lead}");
        }
    }

    #[test]
    fn a_commented_out_ask_still_counts() {
        let manifest = format!("<!-- {} -->", level("true"));
        let dir = stage("worker-comment", manifest.as_bytes());
        assert!(matches!(refusal(&dir), Some(StageError::WorkerUiAccess(_))));
    }
}

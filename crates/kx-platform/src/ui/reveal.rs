//! The reveal port: shows a folder in the system's file manager, as the clipboard panel's Open folder does (ADR-0034).

use std::path::Path;
use thiserror::Error;

/// Why a folder was not shown; no variant names the folder, so no user name reaches a log.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum RevealError {
    /// The folder does not exist.
    #[error("the folder does not exist")]
    Missing,
    /// The system or its file manager refused, with its error code.
    #[error("the file manager refused to show the folder (code {0})")]
    Refused(i32),
}

/// Shows folders in the system's file manager, which runs as the user's own program, never with our rights.
pub trait Reveal: Send + Sync {
    /// Shows `folder`; it may wait on the file manager, so call it off the UI thread.
    ///
    /// # Errors
    /// `Missing` when the folder does not exist, `Refused` when the system or the file manager says no.
    fn show(&self, folder: &Path) -> Result<(), RevealError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_never_name_a_folder() {
        let texts = [RevealError::Missing, RevealError::Refused(5)].map(|e| e.to_string());
        assert!(texts[1].contains("code 5"), "{}", texts[1]);
        assert!(texts.iter().all(|t| !t.contains('\\') && !t.contains('/')));
    }
}

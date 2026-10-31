//! Why a stage, or the build folder an install copies, was refused: its own enum, so `DevError` stays the tools' list.

use std::fmt;

use crate::devtools::DevError;

/// Why the stage check, or the install's look at its build folder, refused it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StageError {
    /// The folder has no program directly inside it.
    NoExe(String),
    /// A staged file lies in a folder the stage must never hold.
    Banned(String),
    /// A staged file imports a library found neither in the stage nor in the system folder.
    Unresolved {
        /// The staged file, relative to the stage.
        by: String,
        /// The library it names.
        lib: String,
    },
    /// A library beside the program that nothing staged imports.
    Unused(String),
    /// A staged file that is not in the stage's list.
    Unplanned(String),
    /// A file that is not a readable Windows program or library.
    BadPe(String),
    /// A staged file other than the listed Qt libraries imports Qt's network library.
    NetworkImport {
        /// The staged file, relative to the stage.
        by: String,
        /// The network library it names.
        lib: String,
    },
    /// The program holds the gate-only build's flag, so it would act on another program's input.
    GateFlag {
        /// The program, relative to the stage.
        exe: String,
        /// The flag it holds.
        flag: String,
    },
    /// The media worker's program is not beside the app's.
    NoWorker(String),
    /// The media worker's manifest asks for uiAccess.
    WorkerUiAccess(String),
    /// The packs folder lacks its index, a pack the index lists, or a credit file.
    NoWords(String),
    /// The pack index is empty or does not read.
    BadIndex(String),
    /// The stage's download, its files deflated, is over the budget.
    OverBudget {
        /// Its deflated size in MB.
        mb: u64,
        /// The budget in MB.
        max: u64,
    },
    /// `QMAKE` names a Qt other than the reviewed one.
    QtVersion {
        /// The version qmake reports.
        found: String,
        /// The reviewed version.
        want: String,
    },
}

impl From<StageError> for DevError {
    fn from(err: StageError) -> Self {
        Self::Stage(err)
    }
}

impl fmt::Display for StageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoExe(path) => write!(f, "no program directly in {path}; pick the build output"),
            Self::Banned(path) => write!(f, "{path} is a network, TLS or debug plugin; none ship"),
            Self::Unresolved { by, lib } => {
                write!(f, "{by} needs {lib}: not staged, not in the system folder")
            }
            Self::Unused(lib) => write!(f, "{lib} is staged but nothing staged imports it"),
            Self::Unplanned(path) => write!(
                f,
                "{path} is not in the stage's list; only listed files ship"
            ),
            Self::BadPe(path) => write!(f, "{path} is not a readable Windows program or library"),
            Self::NetworkImport { by, lib } => {
                write!(f, "{by} imports {lib}; only the listed Qt libraries may")
            }
            Self::GateFlag { exe, flag } => {
                write!(
                    f,
                    "{exe} holds {flag}: a gate-only build, which never ships"
                )
            }
            Self::NoWorker(exe) => write!(f, "{exe} is missing; the worker ships beside the app"),
            Self::WorkerUiAccess(exe) => write!(f, "{exe} asks for uiAccess; the worker never may"),
            Self::NoWords(path) => {
                write!(
                    f,
                    "{path} is missing; the packs ship with their index and credits"
                )
            }
            Self::BadIndex(path) => write!(f, "{path} is empty or not a pack index"),
            Self::OverBudget { mb, max } => {
                write!(f, "{mb} MB deflated is over the {max} MB download budget")
            }
            Self::QtVersion { found, want } => {
                write!(f, "qmake reports Qt {found}; only Qt {want} is reviewed")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stage_error_is_a_dev_error_that_reads_the_same() {
        let unplanned = StageError::Unplanned("x.dll".to_string());
        let text = "x.dll is not in the stage's list; only listed files ship";
        assert_eq!(unplanned.to_string(), text);
        let dev = DevError::from(unplanned.clone());
        assert_eq!(
            (dev.to_string(), dev),
            (text.to_string(), DevError::Stage(unplanned))
        );
    }

    #[test]
    fn a_removed_stage_names_the_stage_error_that_removed_it() {
        let banned = StageError::Banned("tls/x.dll".to_string());
        let removed = DevError::StageRemoved(Box::new(banned.into()));
        let text = "tls/x.dll is a network, TLS or debug plugin; none ship; the stage was removed";
        assert_eq!(removed.to_string(), text);
    }
}

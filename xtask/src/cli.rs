//! Subcommand parsing for the xtask CLI.

use std::fmt;

/// Exit code for a successful run.
pub(crate) const EXIT_SUCCESS: u8 = 0;
/// Exit code when a subcommand finds a problem.
pub(crate) const EXIT_CHECK_FAILED: u8 = 1;
/// Exit code for a usage error: unknown, missing or malformed subcommand.
pub(crate) const EXIT_USAGE_ERROR: u8 = 2;

const SUB_TIDY: &str = "tidy";
const SUB_DCO: &str = "dco";
const SUB_LICENCES: &str = "licences";
const SUB_DIST: &str = "dist";
/// Creates or removes the uiAccess test certificate.
pub(crate) const SUB_DEV_CERT: &str = "dev-cert";
/// Installs or removes a signed test build under Program Files.
const SUB_DEV_INSTALL: &str = "dev-install";
/// Checks the Windows conditions for uiAccess.
const SUB_CHECK_UIACCESS: &str = "check-uiaccess";
/// Flag that turns `dev-cert` and `dev-install` into their undo.
const FLAG_REMOVE: &str = "--remove";

/// Builds the usage line from the subcommand-name constants, so each name is
/// spelled exactly once.
fn usage_line() -> String {
    let forms = [
        SUB_TIDY.to_string(),
        format!("{SUB_DCO} <base> <head>"),
        SUB_LICENCES.to_string(),
        SUB_DIST.to_string(),
        format!("{SUB_DEV_CERT} [{FLAG_REMOVE}]"),
        format!("{SUB_DEV_INSTALL} <folder>"),
        format!("{SUB_DEV_INSTALL} {FLAG_REMOVE} <name>"),
        format!("{SUB_CHECK_UIACCESS} <exe>"),
    ];
    format!("usage: cargo xtask <{}>", forms.join("|"))
}

/// A parsed xtask subcommand.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Command {
    /// Run the workspace tidy checks.
    Tidy,
    /// Check DCO sign-offs between two git revisions.
    Dco {
        /// The base revision, exclusive.
        base: String,
        /// The head revision, inclusive.
        head: String,
    },
    /// Check third-party licences.
    Licences,
    /// Build release distribution artifacts.
    Dist,
    /// Create the test certificate, or remove it.
    DevCert {
        /// Remove instead of create.
        remove: bool,
    },
    /// Sign a build folder and install it under Program Files.
    DevInstall {
        /// The build folder.
        folder: String,
    },
    /// Delete an installed test build.
    DevUninstall {
        /// The install name.
        name: String,
    },
    /// Check the three Windows conditions for uiAccess.
    CheckUiAccess {
        /// The installed program.
        exe: String,
    },
}

/// A command-line usage error. Its `Display` is the usage line.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct UsageError;

impl fmt::Display for UsageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", usage_line())
    }
}

/// Parses xtask's arguments (excluding the program name) into a [`Command`].
pub(crate) fn parse(args: &[String]) -> Result<Command, UsageError> {
    match args {
        [name] if name == SUB_TIDY => Ok(Command::Tidy),
        [name] if name == SUB_LICENCES => Ok(Command::Licences),
        [name] if name == SUB_DIST => Ok(Command::Dist),
        [name, base, head] if name == SUB_DCO => Ok(Command::Dco {
            base: base.clone(),
            head: head.clone(),
        }),
        [name] if name == SUB_DEV_CERT => Ok(Command::DevCert { remove: false }),
        [name, flag] if name == SUB_DEV_CERT && flag == FLAG_REMOVE => {
            Ok(Command::DevCert { remove: true })
        }
        [name, flag, target] if name == SUB_DEV_INSTALL && flag == FLAG_REMOVE => {
            Ok(Command::DevUninstall {
                name: target.clone(),
            })
        }
        [name, folder] if name == SUB_DEV_INSTALL && folder != FLAG_REMOVE => {
            Ok(Command::DevInstall {
                folder: folder.clone(),
            })
        }
        [name, exe] if name == SUB_CHECK_UIACCESS => {
            Ok(Command::CheckUiAccess { exe: exe.clone() })
        }
        _ => Err(UsageError),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| (*v).to_string()).collect()
    }

    #[test]
    fn unknown_subcommand_is_a_usage_error() {
        assert_eq!(parse(&args(&["foo"])), Err(UsageError));
    }

    #[test]
    fn no_subcommand_is_a_usage_error() {
        assert_eq!(parse(&args(&[])), Err(UsageError));
    }

    #[test]
    fn tidy_parses() {
        assert_eq!(parse(&args(&["tidy"])), Ok(Command::Tidy));
    }

    #[test]
    fn licences_parses() {
        assert_eq!(parse(&args(&["licences"])), Ok(Command::Licences));
    }

    #[test]
    fn dist_parses() {
        assert_eq!(parse(&args(&["dist"])), Ok(Command::Dist));
    }

    #[test]
    fn dco_parses_with_base_and_head() {
        assert_eq!(
            parse(&args(&["dco", "main", "HEAD"])),
            Ok(Command::Dco {
                base: "main".into(),
                head: "HEAD".into(),
            })
        );
    }

    #[test]
    fn dco_with_zero_arguments_is_a_usage_error() {
        assert_eq!(parse(&args(&["dco"])), Err(UsageError));
    }

    #[test]
    fn dco_with_one_argument_is_a_usage_error() {
        assert_eq!(parse(&args(&["dco", "main"])), Err(UsageError));
    }

    #[test]
    fn dco_with_three_arguments_is_a_usage_error() {
        assert_eq!(
            parse(&args(&["dco", "main", "HEAD", "extra"])),
            Err(UsageError)
        );
    }

    #[test]
    fn dev_cert_parses_with_and_without_remove() {
        assert_eq!(
            parse(&args(&["dev-cert"])),
            Ok(Command::DevCert { remove: false })
        );
        assert_eq!(
            parse(&args(&["dev-cert", "--remove"])),
            Ok(Command::DevCert { remove: true })
        );
    }

    #[test]
    fn dev_cert_with_another_argument_is_a_usage_error() {
        assert_eq!(parse(&args(&["dev-cert", "x"])), Err(UsageError));
    }

    #[test]
    fn dev_cert_remove_with_an_extra_argument_is_a_usage_error() {
        assert_eq!(
            parse(&args(&["dev-cert", "--remove", "x"])),
            Err(UsageError)
        );
    }

    #[test]
    fn dev_install_with_two_folders_is_a_usage_error() {
        assert_eq!(parse(&args(&["dev-install", "a", "b"])), Err(UsageError));
    }

    #[test]
    fn dev_install_parses_a_folder() {
        assert_eq!(
            parse(&args(&["dev-install", "out/kb"])),
            Ok(Command::DevInstall {
                folder: "out/kb".into()
            })
        );
    }

    #[test]
    fn dev_install_remove_parses_a_name() {
        assert_eq!(
            parse(&args(&["dev-install", "--remove", "kb"])),
            Ok(Command::DevUninstall { name: "kb".into() })
        );
    }

    #[test]
    fn dev_install_without_a_folder_is_a_usage_error() {
        assert_eq!(parse(&args(&["dev-install"])), Err(UsageError));
        assert_eq!(parse(&args(&["dev-install", "--remove"])), Err(UsageError));
    }

    #[test]
    fn check_uiaccess_parses_an_exe() {
        assert_eq!(
            parse(&args(&["check-uiaccess", "kb.exe"])),
            Ok(Command::CheckUiAccess {
                exe: "kb.exe".into()
            })
        );
        assert_eq!(parse(&args(&["check-uiaccess"])), Err(UsageError));
    }

    #[test]
    fn usage_error_message_mentions_every_subcommand() {
        let message = UsageError.to_string();
        let names = [
            SUB_TIDY,
            SUB_DCO,
            SUB_LICENCES,
            SUB_DIST,
            SUB_DEV_CERT,
            SUB_DEV_INSTALL,
            SUB_CHECK_UIACCESS,
            FLAG_REMOVE,
        ];
        for name in names {
            assert!(message.contains(name), "usage message missing {name}");
        }
    }
}

//! Tests of the xtask subcommand parser.

use super::*;

/// The command-line arguments `values`, as owned strings.
pub(super) fn args(values: &[&str]) -> Vec<String> {
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
fn stage_parses_alone() {
    assert_eq!(parse(&args(&["stage"])), Ok(Command::Stage));
    assert_eq!(parse(&args(&["stage", "x"])), Err(UsageError));
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
        SUB_STAGE,
        SUB_DEV_CERT,
        SUB_DEV_INSTALL,
        SUB_CHECK_UIACCESS,
        SUB_SOUNDS,
        SUB_WORDS,
        FLAG_REMOVE,
    ];
    for name in names {
        assert!(message.contains(name), "usage message missing {name}");
    }
}

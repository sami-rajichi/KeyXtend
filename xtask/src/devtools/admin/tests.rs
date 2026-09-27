use super::*;
use crate::test_support::devtools_config;

fn text_of(params: &[(&'static str, String)], name: &str) -> Option<String> {
    params
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, v)| v.clone())
}

#[test]
fn codes_carry_cancel_and_fail() {
    assert_eq!(
        codes(&devtools_config()),
        vec![
            ("CancelCode", "1223".to_string()),
            ("FailCode", "1".to_string())
        ]
    );
}

#[test]
fn install_params_name_everything_the_script_checks() {
    let params = build_params(&devtools_config(), Some(Path::new("src")), Path::new("dst"));
    assert_eq!(text_of(&params, "Action"), Some("Install".to_string()));
    assert_eq!(text_of(&params, "Target"), Some("dst".to_string()));
    assert_eq!(text_of(&params, "Source"), Some("src".to_string()));
    assert_eq!(
        text_of(&params, "InstallDir"),
        Some("KeyXtend-dev".to_string())
    );
    assert_eq!(
        text_of(&params, "SecureEnv"),
        Some("ProgramFiles".to_string())
    );
    assert_eq!(text_of(&params, "CancelCode"), Some("1223".to_string()));
    assert_eq!(text_of(&params, "FailCode"), Some("1".to_string()));
}

#[test]
fn uninstall_params_have_no_source() {
    let params = build_params(&devtools_config(), None, Path::new("dst"));
    assert_eq!(text_of(&params, "Action"), Some("Uninstall".to_string()));
    assert_eq!(text_of(&params, "Source"), None);
}

#[test]
fn elevation_keeps_the_cancel_error_code() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .join("dev-admin.ps1");
    let text = std::fs::read_to_string(path).unwrap();
    let code = text.lines().filter(|l| !l.trim_start().starts_with('#'));
    assert!(text.contains("[Diagnostics.Process]::Start("));
    assert!(
        !code.into_iter().any(|l| l.contains("Start-Process")),
        "Start-Process hides a cancel"
    );
}

/// Runs the real admin script with `-ValidateOnly`, which exits before any Windows prompt.
#[cfg(windows)]
mod script_checks {
    use std::path::PathBuf;
    use std::process::Command;

    use super::*;
    use crate::test_support::{TempDir, make_dir_link};

    const TP: &str = "0123456789ABCDEF0123456789ABCDEF01234567";

    fn setup() -> DevSetup {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        DevSetup {
            root: root.to_path_buf(),
            config: devtools_config(),
        }
    }

    /// The exit code of the admin script checking `params`.
    fn check(params: &[(&'static str, String)]) -> Option<i32> {
        let s = setup();
        let shell = s.powershell(|key| std::env::var(key).ok()).unwrap();
        let mut args = ps::script_args(&s.script(&s.config.admin_script), params);
        args.push(script::VALIDATE_ONLY.to_string());
        Command::new(shell)
            .args(args)
            .output()
            .unwrap()
            .status
            .code()
    }

    fn install_root() -> PathBuf {
        PathBuf::from(std::env::var("ProgramFiles").unwrap()).join("KeyXtend-dev")
    }

    fn uninstall(name: &str) -> Option<i32> {
        let target = paths::text(&install_root()) + "\\" + name;
        check(&build_params(&devtools_config(), None, Path::new(&target)))
    }

    #[test]
    fn a_plain_target_passes() {
        assert_eq!(uninstall("kb"), Some(0));
    }

    #[test]
    fn targets_that_leave_the_install_folder_fail() {
        for bad in [
            "..",
            "kb\\..\\..",
            ". ",
            "..\\x",
            "[kq]b",
            "NUL",
            "COM¹",
            "",
        ] {
            assert_eq!(uninstall(bad), Some(1), "{bad:?} passed");
        }
    }

    #[test]
    fn a_non_plain_install_dir_fails() {
        let mut config = devtools_config();
        config.install_dir = "..".to_string();
        let target = install_root().join("kb");
        let params = build_params(&config, None, &target);
        assert_eq!(check(&params), Some(1));
    }

    #[test]
    fn a_missing_source_fails_before_anything_is_deleted() {
        let dir = TempDir::new("no-source");
        let source = dir.path().join("absent");
        let params = build_params(
            &devtools_config(),
            Some(&source),
            &install_root().join("kb"),
        );
        assert_eq!(check(&params), Some(1));
    }

    fn trust_params(cert_file: &Path, thumbprint: &str) -> Vec<(&'static str, String)> {
        let mut params = codes(&devtools_config());
        params.push((script::ACTION, script::TRUST.to_string()));
        params.push((script::CERT_FILE, paths::text(cert_file)));
        params.push((script::THUMBPRINT, thumbprint.to_string()));
        params
    }

    #[test]
    fn trust_needs_a_real_thumbprint_and_file() {
        let dir = TempDir::new("trust");
        let file = dir.path().join("x.cer");
        assert_eq!(check(&trust_params(&file, TP)), Some(1));
        std::fs::write(&file, b"").unwrap();
        assert_eq!(check(&trust_params(&file, "zz")), Some(1));
        assert_eq!(check(&trust_params(&file, TP)), Some(0));
    }

    #[test]
    fn a_real_source_folder_passes() {
        let dir = TempDir::new("source");
        let target = install_root().join("kb");
        let params = build_params(&devtools_config(), Some(dir.path()), &target);
        assert_eq!(check(&params), Some(0));
    }

    #[test]
    fn a_source_with_a_folder_link_fails() {
        let dir = TempDir::new("source-link");
        let outside = TempDir::new("source-outside");
        make_dir_link(&dir.path().join("big"), outside.path());
        let target = install_root().join("kb");
        let params = build_params(&devtools_config(), Some(dir.path()), &target);
        assert_eq!(check(&params), Some(1));
    }

    #[test]
    fn an_unset_secure_variable_fails() {
        let mut config = devtools_config();
        config.secure_env = vec!["KX_NO_SUCH_VARIABLE".to_string()];
        let params = build_params(&config, None, &install_root().join("kb"));
        assert_eq!(check(&params), Some(1));
    }

    #[test]
    fn untrust_needs_thumbprints_and_a_subject() {
        let mut params = codes(&devtools_config());
        params.push((script::ACTION, script::UNTRUST.to_string()));
        params.push((script::SUBJECT, "CN=KeyXtend Dev Test".to_string()));
        params.push((script::THUMBPRINT, [TP, TP].join(script::LIST_SEP)));
        assert_eq!(check(&params), Some(0));
        params.pop();
        params.push((script::THUMBPRINT, [TP, "zz"].join(script::LIST_SEP)));
        assert_eq!(check(&params), Some(1));
    }
}

use super::*;
use crate::test_support::{devtools_config as config, devtools_json};

/// The error `validate` gives after `change` is applied to the test settings.
fn invalid(change: impl FnOnce(&mut DevToolsConfig)) -> Result<(), ConfigError> {
    let mut c = config();
    change(&mut c);
    c.validate()
}

#[test]
fn test_settings_pass_validation() {
    assert_eq!(config().validate(), Ok(()));
}

#[test]
fn zero_counts_are_rejected() {
    assert_eq!(
        invalid(|c| c.cert_days = 0),
        Err(ConfigError::ZeroValue("cert_days"))
    );
    assert_eq!(
        invalid(|c| c.install_max_mb = 0),
        Err(ConfigError::ZeroValue("install_max_mb"))
    );
}

#[test]
fn min_days_must_be_below_cert_days() {
    assert_eq!(
        invalid(|c| c.cert_min_days = c.cert_days),
        Err(ConfigError::NotBelow("cert_min_days", "cert_days"))
    );
}

#[test]
fn exit_codes_must_be_positive_and_distinct() {
    assert_eq!(
        invalid(|c| c.cancel_code = -1),
        Err(ConfigError::NotPositive("cancel_code"))
    );
    assert_eq!(
        invalid(|c| c.fail_code = 0),
        Err(ConfigError::NotPositive("fail_code"))
    );
    assert_eq!(
        invalid(|c| c.fail_code = c.cancel_code),
        Err(ConfigError::Same("cancel_code", "fail_code"))
    );
}

#[test]
fn install_dir_must_be_a_plain_name() {
    for bad in ["..", ".", "a\\b", "a/b", "NUL", "x.", "[x]"] {
        assert_eq!(
            invalid(|c| c.install_dir = bad.to_string()),
            Err(ConfigError::NotPlain("install_dir")),
            "{bad:?} was accepted"
        );
    }
}

#[test]
fn file_names_must_be_plain() {
    assert_eq!(
        invalid(|c| c.signtool = "  ".to_string()),
        Err(ConfigError::NotPlain("signtool"))
    );
    assert_eq!(
        invalid(|c| c.admin_script = "..\\x.ps1".to_string()),
        Err(ConfigError::NotPlain("admin_script"))
    );
}

#[test]
fn folders_must_be_relative_and_plain() {
    assert_eq!(
        invalid(|c| c.out_dir = "../x".to_string()),
        Err(ConfigError::NotRelative("out_dir"))
    );
    assert_eq!(
        invalid(|c| c.powershell = "C:/x/powershell.exe".to_string()),
        Err(ConfigError::NotRelative("powershell"))
    );
}

#[test]
fn empty_texts_are_rejected() {
    assert_eq!(
        invalid(|c| c.cert_subject = String::new()),
        Err(ConfigError::EmptyValue("cert_subject"))
    );
    assert_eq!(
        invalid(|c| c.sdk_bin = String::new()),
        Err(ConfigError::EmptyValue("sdk_bin"))
    );
}

#[test]
fn the_subject_must_survive_command_line_quoting() {
    for bad in ["CN=\"KeyXtend\"", "CN=KeyXtend\\"] {
        assert_eq!(
            invalid(|c| c.cert_subject = bad.to_string()),
            Err(ConfigError::NotQuotable("cert_subject")),
            "{bad:?} was accepted"
        );
    }
}

#[test]
fn installs_use_the_first_secure_variable() {
    assert_eq!(config().install_env(), "ProgramFiles");
}

#[test]
fn secure_env_must_list_names() {
    assert_eq!(
        invalid(|c| c.secure_env.clear()),
        Err(ConfigError::EmptyList("secure_env"))
    );
    assert_eq!(
        invalid(|c| c.secure_env = vec![" ".to_string()]),
        Err(ConfigError::EmptyValue("secure_env"))
    );
}

#[test]
fn load_reads_the_section_and_the_root() {
    let json = serde_json::json!({
        "workspace_root": "/ws",
        "metadata": { "devtools": devtools_json() }
    });
    let setup = load(&json.to_string()).unwrap();
    assert_eq!(setup.root, PathBuf::from("/ws"));
    assert_eq!(setup.config, config());
}

#[test]
fn load_names_a_missing_section() {
    let json = r#"{"workspace_root": "/ws", "metadata": {}}"#;
    assert!(matches!(
        load(json),
        Err(LoadError::MissingSection("devtools"))
    ));
}

#[test]
fn load_rejects_an_invalid_value() {
    let mut value = devtools_json();
    value["install_dir"] = serde_json::json!("..");
    let json = serde_json::json!({ "workspace_root": "/ws", "metadata": { "devtools": value } });
    assert!(matches!(
        load(&json.to_string()),
        Err(LoadError::BadConfig(_))
    ));
}

#[test]
fn unknown_field_is_rejected() {
    let mut value = devtools_json();
    value["bogus"] = serde_json::json!(1);
    assert!(serde_json::from_value::<DevToolsConfig>(value).is_err());
}

#[test]
fn a_missing_windows_folder_variable_is_named() {
    let setup = DevSetup {
        root: PathBuf::from("/ws"),
        config: config(),
    };
    assert_eq!(
        setup.powershell(|_| None),
        Err(DevError::MissingEnv("SystemRoot".to_string()))
    );
}

#[cfg(windows)]
mod windows {
    use super::*;

    fn setup() -> DevSetup {
        DevSetup {
            root: PathBuf::from("D:\\ws"),
            config: config(),
        }
    }

    #[test]
    fn settings_paths_use_windows_separators() {
        assert_eq!(
            setup().out("f.txt"),
            PathBuf::from("D:\\ws\\target\\dev-tools\\f.txt")
        );
    }

    #[test]
    fn secure_roots_drop_empty_and_relative_values() {
        let roots = config().secure_roots(|key| match key {
            "ProgramFiles" => Some("C:\\PF".to_string()),
            _ => Some("relative".to_string()),
        });
        assert_eq!(roots, vec![PathBuf::from("C:\\PF")]);
        assert!(config().secure_roots(|_| Some(String::new())).is_empty());
    }

    #[test]
    fn powershell_comes_from_the_windows_folder() {
        let path = setup().powershell(|_| Some("C:\\Windows".to_string()));
        let expected = "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe";
        assert_eq!(path, Ok(PathBuf::from(expected)));
    }

    #[test]
    fn the_install_root_comes_only_from_the_first_variable() {
        let only_x86 = |key: &str| (key == "ProgramFiles(x86)").then(|| "C:\\PF86".to_string());
        assert_eq!(
            config().install_root(only_x86),
            Err(DevError::MissingEnv("ProgramFiles".to_string()))
        );
        let root = config().install_root(|_| Some("C:\\PF".to_string()));
        assert_eq!(root, Ok(PathBuf::from("C:\\PF")));
    }

    #[test]
    fn a_relative_windows_folder_is_refused() {
        let path = setup().powershell(|_| Some("Windows".to_string()));
        assert!(matches!(path, Err(DevError::MissingEnv(_))));
    }
}

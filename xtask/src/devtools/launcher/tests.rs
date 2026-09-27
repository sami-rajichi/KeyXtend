use super::*;
use crate::test_support::devtools_config;

const TP: &str = "0123456789ABCDEF0123456789ABCDEF01234567";
const TP2: &str = "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF";

fn setup(root: &str) -> DevSetup {
    DevSetup {
        root: PathBuf::from(root),
        config: devtools_config(),
    }
}

fn trust() -> String {
    text(
        &setup("D:\\ws"),
        Path::new("ps.exe"),
        &Launch::Trust(TP.to_string()),
    )
}

#[test]
fn launchers_switch_cmd_to_utf8_first() {
    assert!(trust().starts_with("@echo off\r\nchcp 65001>nul\r\nps.exe "));
}

#[test]
fn trust_launcher_passes_the_thumbprint_and_codes() {
    let text = trust();
    assert!(text.contains("-Action Trust"));
    assert!(text.contains(&format!("-Thumbprint {TP}")));
    assert!(text.contains("-CancelCode 1223 -FailCode 1"));
    assert!(text.trim_end().ends_with("-Notify"));
}

#[test]
fn untrust_launcher_lists_thumbprints_and_quotes_the_subject() {
    let launch = Launch::Untrust(vec![TP.to_string(), TP2.to_string()]);
    let text = text(&setup("D:\\ws"), Path::new("ps.exe"), &launch);
    assert!(text.contains("-Action Untrust"));
    assert!(text.contains(&format!("-Thumbprint {TP},{TP2}")));
    assert!(text.contains("-Subject \"CN=KeyXtend Dev Test\""));
}

#[test]
fn percent_signs_are_doubled() {
    let text = text(
        &setup("D:\\50%off"),
        Path::new("ps.exe"),
        &Launch::Untrust(vec![]),
    );
    assert!(text.contains("50%%off"));
}

#[test]
fn cmd_special_characters_are_quoted() {
    let text = text(
        &setup("D:\\a&b"),
        Path::new("ps.exe"),
        &Launch::Untrust(vec![]),
    );
    assert!(text.contains("\"D:\\a&b"));
}

#[cfg(windows)]
#[test]
fn trust_launcher_names_the_exported_file() {
    assert!(trust().contains("-CertFile D:\\ws\\target\\dev-tools\\keyxtend-dev.cer"));
}

/// Runs real launchers through cmd.exe, with `-ValidateOnly` so no Windows prompt appears.
#[cfg(windows)]
mod through_cmd {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    use super::*;
    use crate::test_support::TempDir;

    /// Gives cmd.exe its own hidden console, so `chcp` never touches the test console.
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    /// A workspace in `dir` holding the admin script and an exported certificate file.
    fn workspace(dir: &Path) -> DevSetup {
        let setup = setup(&paths::text(dir));
        let c = &setup.config;
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts");
        std::fs::create_dir_all(setup.path(&c.scripts_dir)).unwrap();
        std::fs::copy(repo.join(&c.admin_script), setup.script(&c.admin_script)).unwrap();
        std::fs::create_dir_all(setup.out("")).unwrap();
        std::fs::write(setup.out(&c.cert_file), b"").unwrap();
        setup
    }

    /// The exit code and error text of `launch` written as a launcher and run by cmd.exe.
    fn run(setup: &DevSetup, launch: &Launch) -> (Option<i32>, String) {
        let shell = setup.powershell(|key| std::env::var(key).ok()).unwrap();
        let text = text(setup, &shell, launch).replace(script::NOTIFY, script::VALIDATE_ONLY);
        let file = setup.out(&setup.config.trust_launcher);
        std::fs::write(&file, text).unwrap();
        // `/s` strips only the outer quotes, so the inner ones keep the path whole.
        let output = Command::new("cmd")
            .args(["/d", "/s", "/c"])
            .raw_arg(format!("\"\"{}\"\"", paths::text(&file)))
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .unwrap();
        let errors = String::from_utf8_lossy(&output.stderr).into_owned();
        (output.status.code(), errors)
    }

    #[test]
    fn percent_ampersand_and_spaces_survive_cmd() {
        let dir = TempDir::new("50%off & (x)");
        let setup = workspace(dir.path());
        let (code, errors) = run(&setup, &Launch::Trust(TP.to_string()));
        assert_eq!(code, Some(0), "{errors}");
        let untrust = Launch::Untrust(vec![TP.to_string(), TP2.to_string()]);
        let (code, errors) = run(&setup, &untrust);
        assert_eq!(code, Some(0), "{errors}");
    }

    #[test]
    fn a_bad_thumbprint_still_fails_through_cmd() {
        let dir = TempDir::new("launch-bad");
        let mut setup = workspace(dir.path());
        setup.config.fail_code = 3;
        let (code, errors) = run(&setup, &Launch::Trust("zz".to_string()));
        assert_eq!(code, Some(3), "{errors}");
    }
}

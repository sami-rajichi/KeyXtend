use super::*;
use crate::test_support::{TempDir, devtools_config};

const TAG: &str = "requestedExecutionLevel";
const ATTR: &str = "uiAccess";

fn value(xml: &str) -> Option<String> {
    manifest_value(xml, TAG, ATTR)
}

#[test]
fn reads_double_quotes() {
    let xml = r#"<requestedExecutionLevel level="asInvoker" uiAccess="true"/>"#;
    assert_eq!(value(xml), Some("true".to_string()));
}

#[test]
fn reads_an_element_with_a_namespace_prefix() {
    let xml = r#"<ms_asmv2:requestedExecutionLevel uiAccess="true"/>"#;
    assert_eq!(value(xml), Some("true".to_string()));
}

#[test]
fn a_prefix_without_an_opening_bracket_does_not_match() {
    assert_eq!(
        value(r#"x:requestedExecutionLevel uiAccess="true"/>"#),
        None
    );
    assert_eq!(
        value(r#"<a b:requestedExecutionLevel uiAccess="true"/>"#),
        None
    );
}

#[test]
fn an_unclosed_comment_hides_the_rest() {
    assert_eq!(
        value(r#"<!-- <requestedExecutionLevel uiAccess="true"/>"#),
        None
    );
}

#[test]
fn a_manifest_file_is_read_then_deleted() {
    let dir = TempDir::new("manifest");
    let file = dir.path().join("m.xml");
    std::fs::write(&file, b"ok \xFF").unwrap();
    assert_eq!(read_and_remove(&file), Ok("ok \u{FFFD}".to_string()));
    assert!(!file.exists());
    assert!(matches!(read_and_remove(&file), Err(DevError::Io(_))));
}

#[test]
fn reads_single_quotes_and_spaces() {
    let xml = "<requestedExecutionLevel\n  uiAccess = 'false' />";
    assert_eq!(value(xml), Some("false".to_string()));
}

#[test]
fn a_missing_attribute_is_none() {
    assert_eq!(
        value(r#"<requestedExecutionLevel level="asInvoker"/>"#),
        None
    );
}

#[test]
fn a_longer_attribute_name_does_not_match() {
    assert_eq!(
        value(r#"<requestedExecutionLevel xuiAccess="true"/>"#),
        None
    );
}

#[test]
fn a_commented_out_decoy_is_ignored() {
    let xml = r#"<!-- <requestedExecutionLevel uiAccess="true"/> -->
        <requestedExecutionLevel uiAccess="false"/>"#;
    assert_eq!(value(xml), Some("false".to_string()));
}

#[test]
fn the_attribute_on_another_element_is_ignored() {
    let xml = r#"<other uiAccess="true"/><requestedExecutionLevel level="asInvoker"/>"#;
    assert_eq!(value(xml), None);
}

#[test]
fn a_longer_element_name_does_not_match() {
    assert_eq!(
        value(r#"<requestedExecutionLevelX uiAccess="true"/>"#),
        None
    );
}

#[test]
fn an_empty_tag_finds_nothing() {
    assert_eq!(
        manifest_value(r#"<requestedExecutionLevel uiAccess="true"/>"#, "", ATTR),
        None
    );
}

#[test]
fn an_empty_attribute_name_finds_nothing() {
    assert_eq!(
        manifest_value(r#"<requestedExecutionLevel a="1"/>"#, TAG, ""),
        None
    );
}

#[test]
fn only_true_passes_the_manifest_check() {
    let config = devtools_config();
    let xml = |v: &str| format!(r#"<requestedExecutionLevel uiAccess="{v}"/>"#);
    assert!(manifest_check(&xml("true"), &config).passed);
    assert!(!manifest_check(&xml("false"), &config).passed);
    let missing = manifest_check("<requestedExecutionLevel/>", &config);
    assert!(!missing.passed);
    assert_eq!(missing.detail, "uiAccess=");
}

#[test]
fn verify_args_also_search_catalogs() {
    assert_eq!(
        verify_args(Path::new("kb.exe")),
        ["verify", "/pa", "/a", "kb.exe"]
    );
}

#[test]
fn mt_args_read_resource_one() {
    let args = mt_args(Path::new("kb.exe"), Path::new("m.xml"));
    assert_eq!(args, ["-nologo", "-inputresource:kb.exe;#1", "-out:m.xml"]);
}

#[test]
fn report_lines_start_with_the_verdict() {
    let check = |passed, detail: &str| Check {
        name: "x",
        passed,
        detail: detail.to_string(),
    };
    assert_eq!(report_line(&check(true, "C:\\PF")), "PASS  x: C:\\PF");
    assert_eq!(report_line(&check(false, "boom")), "FAIL  x: boom");
    assert_eq!(report_line(&check(true, "")), "PASS  x");
}

#[test]
fn a_failed_step_fails_the_check_but_not_the_run() {
    let failed = Err(DevError::Failed {
        step: "s".to_string(),
        detail: "no".to_string(),
    });
    assert!(!step_check("x", failed).unwrap().passed);
    assert_eq!(
        step_check("x", Err(DevError::Cancelled)),
        Err(DevError::Cancelled)
    );
}

#[cfg(windows)]
mod windows {
    use super::*;

    fn env(key: &str) -> Option<String> {
        (key == "ProgramFiles").then(|| "C:\\Program Files".to_string())
    }

    #[test]
    fn a_program_in_program_files_passes_the_folder_check() {
        let exe = Path::new("C:\\Program Files\\KeyXtend-dev\\kb\\kb.exe");
        assert!(check_folder(&devtools_config(), exe, env).passed);
    }

    #[test]
    fn a_program_elsewhere_fails_the_folder_check() {
        let exe = Path::new("C:\\Program Files Evil\\kb.exe");
        assert!(!check_folder(&devtools_config(), exe, env).passed);
        assert!(!check_folder(&devtools_config(), exe, |_| Some(String::new())).passed);
    }
}

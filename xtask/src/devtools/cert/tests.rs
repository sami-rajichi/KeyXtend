use std::path::Path;

use super::*;
use crate::test_support::devtools_config;

const TP: &str = "0123456789ABCDEF0123456789ABCDEF01234567";
const TP2: &str = "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF";

/// A fake store that records calls and returns preset answers.
#[derive(Default)]
struct FakeStore {
    existing: Option<String>,
    listed: Vec<String>,
    export_fails: bool,
    created: u32,
    exported: Vec<String>,
    removed: bool,
}

impl CertStore for FakeStore {
    fn find(&mut self) -> Result<Option<String>, DevError> {
        Ok(self.existing.clone())
    }

    fn create(&mut self) -> Result<String, DevError> {
        self.created += 1;
        Ok(TP.to_string())
    }

    fn export(&mut self, thumbprint: &str) -> Result<(), DevError> {
        self.exported.push(thumbprint.to_string());
        if self.export_fails {
            return Err(DevError::Io("disk full".to_string()));
        }
        Ok(())
    }

    fn list(&mut self) -> Result<Vec<String>, DevError> {
        Ok(self.listed.clone())
    }

    fn remove(&mut self) -> Result<(), DevError> {
        self.removed = true;
        Ok(())
    }
}

fn text_of(params: &[(&'static str, String)], name: &str) -> Option<String> {
    params
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, v)| v.clone())
}

#[test]
fn an_existing_certificate_is_reused_not_duplicated() {
    let mut store = FakeStore {
        existing: Some(TP2.to_string()),
        ..FakeStore::default()
    };
    assert_eq!(ensure(&mut store), Ok(TP2.to_string()));
    assert_eq!(store.created, 0);
    assert_eq!(store.exported, vec![TP2.to_string()]);
}

#[test]
fn a_missing_certificate_is_created_then_exported() {
    let mut store = FakeStore::default();
    assert_eq!(ensure(&mut store), Ok(TP.to_string()));
    assert_eq!(store.created, 1);
    assert_eq!(store.exported, vec![TP.to_string()]);
}

#[test]
fn an_export_failure_is_returned() {
    let mut store = FakeStore {
        export_fails: true,
        ..FakeStore::default()
    };
    assert!(matches!(ensure(&mut store), Err(DevError::Io(_))));
}

#[test]
fn retire_writes_the_launcher_before_removing() {
    let mut store = FakeStore {
        listed: vec![TP.to_string()],
        ..FakeStore::default()
    };
    let mut written = Vec::new();
    let result = retire(&mut store, |list| {
        written = list.to_vec();
        Ok(())
    });
    assert_eq!(result, Ok(vec![TP.to_string()]));
    assert_eq!(written, vec![TP.to_string()]);
    assert!(store.removed);
}

#[test]
fn retire_keeps_the_certificate_if_the_launcher_fails() {
    let mut store = FakeStore {
        listed: vec![TP.to_string()],
        ..FakeStore::default()
    };
    let result = retire(&mut store, |_| Err(DevError::Io("locked".to_string())));
    assert!(result.is_err());
    assert!(!store.removed);
}

#[test]
fn retire_with_no_certificate_does_nothing() {
    let mut store = FakeStore::default();
    let mut called = false;
    let result = retire(&mut store, |_| {
        called = true;
        Ok(())
    });
    assert_eq!(result, Ok(vec![]));
    assert!(!called);
    assert!(!store.removed);
}

#[test]
fn thumbprint_is_the_last_line_in_upper_case() {
    let out = format!("WARNING: something\r\n{}\r\n\r\n", TP.to_lowercase());
    assert_eq!(parse_thumbprint(&out), Ok(TP.to_string()));
}

#[test]
fn malformed_thumbprints_are_rejected() {
    for bad in [&TP[1..], &format!("G{}", &TP[1..]), " \n"] {
        assert!(matches!(
            parse_thumbprint(bad),
            Err(DevError::BadThumbprint(_))
        ));
    }
}

#[test]
fn blank_find_output_means_none() {
    assert_eq!(parse_found("\r\n"), Ok(None));
    assert_eq!(parse_found(TP), Ok(Some(TP.to_string())));
}

#[test]
fn list_output_gives_every_thumbprint() {
    let out = format!("{}\r\n{TP2}\r\n", TP.to_lowercase());
    assert_eq!(parse_list(&out), Ok(vec![TP.to_string(), TP2.to_string()]));
    assert_eq!(parse_list(""), Ok(vec![]));
    assert!(parse_list(&format!("{TP}\nzz")).is_err());
}

#[test]
fn find_skips_certificates_close_to_expiry() {
    let params = find_params(&devtools_config());
    assert_eq!(text_of(&params, "MinDays"), Some("14".to_string()));
}

#[test]
fn create_sets_lifetime_and_digest() {
    let params = create_params(&devtools_config());
    assert_eq!(text_of(&params, "Days"), Some("90".to_string()));
    assert_eq!(text_of(&params, "Digest"), Some("SHA256".to_string()));
}

#[test]
fn every_action_names_the_subject_and_store_once() {
    let extra = vec![("Days", "90".to_string())];
    let params = action_params(&devtools_config(), "Create", extra);
    let names: Vec<&str> = params.iter().map(|(n, _)| *n).collect();
    assert_eq!(names, ["Action", "Subject", "Store", "Days"]);
    assert_eq!(text_of(&params, "Store"), Some("My".to_string()));
}

#[test]
fn the_create_step_limits_what_the_key_can_do() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .join("dev-cert.ps1");
    let text = std::fs::read_to_string(path).unwrap();
    for needle in [
        "-Type CodeSigningCert",
        "-KeyExportPolicy NonExportable",
        "-KeyUsage DigitalSignature",
        "ca=false",
    ] {
        assert!(text.contains(needle), "the create step lacks {needle}");
    }
}

#[cfg(windows)]
#[test]
fn the_real_find_step_reports_an_absent_subject_as_none() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut config = devtools_config();
    config.cert_subject = "CN=KeyXtend Absent Probe".to_string();
    let setup = DevSetup {
        root: root.to_path_buf(),
        config,
    };
    assert_eq!(ScriptStore { setup: &setup }.find(), Ok(None));
}

//! The box proofs' pure parts: their names, the HTTPS ask's one address and wrong size, and each verdict.

use super::*;
use kx_mod_voice::models::table;

/// A folder on a second drive, as the proofs make it.
const FOLDER: &str = r"E:\kx-proof";

fn words(w: &[&str]) -> Vec<String> {
    w.iter().map(ToString::to_string).collect()
}

fn row() -> &'static Row {
    table()
        .expect("the table")
        .default_for("en")
        .expect("a default")
}

#[test]
fn each_proof_takes_its_one_word() {
    assert_eq!(Proof::parse(&words(&[HTTPS])), Some(Proof::Https));
    assert_eq!(Proof::parse(&words(&[FOLDER_WORD])), Some(Proof::Folder));
    for bad in [&[][..], &["web"], &[HTTPS, FOLDER_WORD], &["--mb", "64"]] {
        assert_eq!(Proof::parse(&words(bad)), None, "{bad:?}");
    }
}

#[test]
fn the_https_ask_names_one_listed_address_and_a_size_no_model_has() {
    let settings = VoiceSettings::from_defaults().expect("the defaults");
    let ask = https_ask(row(), Path::new(FOLDER), &settings).expect("an ask");
    assert_eq!(ask.links.len(), 1, "one request");
    assert_eq!(ask.links[0].expose(), row().links[0]);
    let host = host(ask.links[0].expose()).expect("a host");
    assert!(row().hosts.iter().any(|h| h == host), "{host}");
    assert_ne!(ask.bytes, row().bytes);
    assert_eq!(ask.file.expose().parent(), Some(Path::new(FOLDER)));
    assert_eq!(ask.check(), Ok(()));
}

#[test]
fn a_host_is_the_https_addresss_own() {
    assert_eq!(
        host("https://huggingface.co/a/b.gguf"),
        Some("huggingface.co")
    );
    assert_eq!(host("https://cdn.example.org"), Some("cdn.example.org"));
    assert_eq!(host("http://huggingface.co/a"), None);
    assert_eq!(host("https:///a"), None);
}

#[test]
fn the_https_proof_passes_only_on_a_size_refusal_that_left_nothing() {
    let size = FetchNote::Fault(ModelError::Size);
    assert!(https_pass(Some(&size), 0));
    assert!(!https_pass(Some(&size), 1), "a part stayed");
    for other in [
        ModelError::Net,
        ModelError::Host,
        ModelError::Io,
        ModelError::Stalled,
    ] {
        assert!(!https_pass(Some(&FetchNote::Fault(other)), 0), "{other:?}");
    }
    assert!(!https_pass(Some(&FetchNote::Done), 0), "a body was saved");
    assert!(!https_pass(None, 0), "no end");
}

#[test]
fn the_folder_proof_passes_only_once_the_job_named_the_file_it_read() {
    assert!(folder_pass(Some(&FetchNote::Done), true, true));
    assert!(
        !folder_pass(Some(&FetchNote::Done), false, true),
        "other bytes"
    );
    assert!(
        !folder_pass(Some(&FetchNote::Done), true, false),
        "the part stayed"
    );
    let fault = FetchNote::Fault(ModelError::ReadOnly);
    assert!(!folder_pass(Some(&fault), true, true));
    assert!(!folder_pass(None, true, true));
}

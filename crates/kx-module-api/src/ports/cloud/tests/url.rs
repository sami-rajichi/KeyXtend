//! An address built from any text never panics, starts with `https://`, holds no user name or fragment, and names one
//! plain host; a typed one gets its scheme and host in lower case, as both ignore case, and one refused for its path
//! or query still has a host.

use super::*;
use proptest::prelude::*;

/// Fails the case unless an address built from `text`, if any, is a plain HTTPS one.
fn plain(text: &str) -> Result<(), TestCaseError> {
    if let Some(url) = HttpsUrl::new(text) {
        let shown = url.expose();
        prop_assert!(shown.starts_with(kx_ipc::HTTPS), "{shown}");
        prop_assert!(!shown.contains(['@', '#']), "{shown}");
        prop_assert!(kx_ipc::rules::listed(url.host()).is_ok(), "{}", url.host());
    }
    Ok(())
}

proptest! {
    #[test]
    fn any_text_builds_no_address_or_a_plain_https_one(text in any::<String>()) {
        plain(&text)?;
    }

    #[test]
    fn any_https_text_builds_no_address_or_a_plain_one(
        rest in "[A-Za-z0-9._~:/?#\\[\\]@!$&'()*+,;=%-]{0,80}",
    ) {
        plain(&format!("{}{rest}", kx_ipc::HTTPS))?;
    }
}

#[test]
fn a_typed_address_gets_its_scheme_and_host_in_lower_case_and_keeps_its_path() {
    let typed = "HTTPS://Speech.Example.COM:8443/V1/Audio?Lang=EN";
    let lowered = HttpsUrl::lowered(typed);
    assert_eq!(lowered, "https://speech.example.com:8443/V1/Audio?Lang=EN");
    assert_eq!(
        HttpsUrl::new(&lowered)
            .map(|u| u.host().to_owned())
            .as_deref(),
        Some("speech.example.com")
    );
    for other in ["", "http://X.com/A", "Speech.Example.com/A"] {
        assert_eq!(
            HttpsUrl::lowered(other),
            other,
            "only an https address changes"
        );
    }
    assert_eq!(HttpsUrl::MAX_BYTES, kx_ipc::MAX_URL_BYTES);
}

proptest! {
    #[test]
    fn lowering_keeps_the_length_and_an_address_already_taken(text in any::<String>()) {
        let lowered = HttpsUrl::lowered(&text);
        prop_assert_eq!(lowered.len(), text.len());
        if HttpsUrl::new(&text).is_some() {
            prop_assert_eq!(lowered, text);
        }
    }
}

#[test]
fn a_typed_address_with_a_good_host_and_a_bad_path_still_has_a_host() {
    for text in [
        "https://speech.example.com/v1 audio",
        "https://speech.example.com/a@b",
        "https://speech.example.com/صوت",
    ] {
        assert!(
            HttpsUrl::new(text).is_none() && HttpsUrl::has_host(text),
            "{text}"
        );
    }
    for text in [
        "https://10.0.0.1/v1",
        "https://user@speech.example.com/v1",
        "http://speech.example.com/v1",
    ] {
        assert!(!HttpsUrl::has_host(text), "{text}");
    }
}

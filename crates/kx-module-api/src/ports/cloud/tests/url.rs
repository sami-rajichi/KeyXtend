//! An address built from any text never panics, starts with `https://`, holds no user name or fragment, and names one
//! plain host.

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

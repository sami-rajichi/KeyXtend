//! The notebook's emails: only emails, once each in any case and in their order, and `Debug` shows only the count.

use super::*;

/// The notebook's `texts`, as `VaultEmails` carries them.
pub(in crate::email) fn notebook(texts: &[&str]) -> Saved {
    let secret = |t: &&str| Redacted::new(Zeroizing::new((*t).to_owned()));
    Saved::new(texts.iter().map(secret).collect())
}

/// The saved emails, in order.
fn texts(saved: &Saved) -> Vec<&str> {
    saved.keyed().map(|(_, text)| text).collect()
}

#[test]
fn only_emails_are_kept_once_each_in_any_case_in_their_order() {
    let saved = notebook(&["alex@x.org", "not an email", "lea@z.fr", "ALEX@X.ORG", ""]);
    assert_eq!(texts(&saved), ["alex@x.org", "lea@z.fr"]);
    let keys: Vec<&str> = saved.keyed().map(|(key, _)| key).collect();
    assert_eq!(keys, ["alex@x.org", "lea@z.fr"], "each key in lower case");
    assert!(texts(&notebook(&[])).is_empty());
}

#[test]
fn an_arabic_address_is_kept_and_keyed_as_it_is() {
    let email = "أليكس@مثال.شبكة";
    let saved = notebook(&[email]);
    assert_eq!(saved.keyed().collect::<Vec<_>>(), [(email, email)]);
}

#[test]
fn debug_shows_only_how_many_are_held() {
    let saved = notebook(&["secret@x.co", "أليكس@مثال.شبكة"]);
    let shown = format!("{saved:?}");
    assert!(
        !shown.contains("secret") && !shown.contains("أليكس"),
        "{shown}"
    );
    assert_eq!(shown, "Saved { count: 2 }");
}

//! A link: only https, no space, quote, control or hidden mark, within the cap, and a `Debug` that hides it.

use super::*;
use crate::{HIDDEN_MARKS, INVISIBLE, REDACTED_MARKER};
use proptest::prelude::*;
use std::collections::HashSet;

/// Google's translate page for "hello" in Arabic, as the selection module builds it.
const GOOGLE: &str =
    "https://translate.google.com/?sl=auto&tl=en&text=%D9%85%D8%B1%D8%AD%D8%A8%D8%A7&op=translate";

/// `text` as a link.
fn link(text: &str) -> Result<Link, LinkError> {
    Link::new(text.to_owned())
}

/// Control characters, hidden marks and characters a command line or a browser reads apart.
fn refused_chars() -> Vec<char> {
    vec![
        '\u{0}', '\u{7}', '\t', '\n', '\r', '\u{7f}', '\u{85}', '\u{202e}', '\u{2068}', '\u{200b}',
        '\u{feff}', '\u{ad}', ' ', '\u{a0}', '"', '<', '>', '\\', '^', '`', '{', '|', '}', 'م',
    ]
}

/// Every character `is_hidden` names: the controls, the hidden marks and the invisible runs.
fn hidden() -> Vec<char> {
    let controls = ('\0'..='\u{1f}').chain('\u{7f}'..='\u{9f}');
    let runs = INVISIBLE.iter().flat_map(|&(first, last)| first..=last);
    controls.chain(HIDDEN_MARKS).chain(runs).collect()
}

#[test]
fn a_translate_link_is_taken_as_it_is() {
    let taken = link(GOOGLE).map(|l| l.expose().to_owned());
    assert_eq!(taken.as_deref(), Ok(GOOGLE));
}

#[test]
fn only_an_https_link_with_a_host_is_taken() {
    let other = [
        "http://translate.google.com/",
        "HTTPS://translate.google.com/",
        "javascript:alert(1)",
        "file:///C:/Windows/notepad.exe",
        "ftp://example.org/",
        "https:/translate.google.com/",
        "https://",
        "https:///etc",
        "",
    ];
    for text in other {
        assert_eq!(link(text), Err(LinkError::NotHttps), "{text}");
    }
}

#[test]
fn a_control_a_hidden_mark_a_space_or_a_quote_is_refused() {
    for c in refused_chars() {
        let text = format!("https://example.org/?q=a{c}b");
        assert_eq!(link(&text), Err(LinkError::BadChar), "{c:?}");
    }
}

#[test]
fn a_link_up_to_the_cap_is_taken_and_one_byte_more_refused() {
    let head = "https://example.org/?q=";
    let full = format!("{head}{}", "a".repeat(MAX_LINK_BYTES - head.len()));
    assert_eq!(full.len(), MAX_LINK_BYTES);
    assert!(link(&full).is_ok());
    let over = format!("{full}a");
    assert_eq!(link(&over), Err(LinkError::TooLong));
    let long_http = format!("http://{}", "a".repeat(MAX_LINK_BYTES));
    assert_eq!(
        link(&long_http),
        Err(LinkError::TooLong),
        "the length is checked first"
    );
}

#[test]
fn debug_prints_only_the_marker() {
    let text = format!("{:?}", link(GOOGLE).unwrap());
    assert_eq!(text, format!("Link({REDACTED_MARKER})"));
    assert!(
        !text.contains("translate") && !text.contains("%D9"),
        "{text}"
    );
}

#[test]
fn errors_never_show_a_link_and_each_says_something_else() {
    let errors = [
        LinkError::NotHttps,
        LinkError::BadChar,
        LinkError::TooLong,
        LinkError::Unavailable,
        LinkError::Refused(5),
    ];
    let texts: HashSet<_> = errors.iter().map(ToString::to_string).collect();
    assert_eq!(texts.len(), errors.len());
    assert!(texts.iter().all(|t| !t.contains(LINK_SCHEME)), "{texts:?}");
    assert!(LinkError::Refused(5).to_string().contains("code 5"));
}

#[test]
fn the_web_link_is_named_and_gated_by_open_link() {
    assert_eq!(WebLinkKey::ID.as_str(), "web-link");
    let gate = Some(Capability::OpenLink);
    assert_eq!(WebLinkKey::CAPABILITY, gate, "ADR-0041 decision 3");
}

proptest! {
    #[test]
    fn a_link_of_url_characters_within_the_cap_is_taken(
        host in "[a-z0-9]{1,20}",
        rest in "[A-Za-z0-9._~:/?#\\[\\]@!$&'()*+,;=%-]{0,200}",
    ) {
        let text = format!("https://{host}{rest}");
        prop_assert_eq!(link(&text).map(|l| l.expose().to_owned()), Ok(text));
    }

    #[test]
    fn any_hidden_character_anywhere_after_the_scheme_is_refused(
        rest in "[a-z0-9/?=&%.-]{1,40}",
        at in any::<prop::sample::Index>(),
        c in prop::sample::select(hidden()),
    ) {
        let mut chars: Vec<char> = rest.chars().collect();
        chars.insert(at.index(chars.len() + 1), c);
        let text = format!("https://a{}", chars.into_iter().collect::<String>());
        prop_assert_eq!(link(&text), Err(LinkError::BadChar));
    }
}

//! Properties over any text of Latin, French, Arabic with harakat, Chinese, emoji, marks, spaces, breaks and hidden
//! marks: each link passes the strict check and both caps, and carries a start of the clean text, cut at a whole word.
//! Any tag the system gives turns into a language tag.

use super::tests::sent;
use super::*;
use crate::settings::TRANSLATE_CHARS;
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;

/// Random texts per property.
const CASES: u32 = 256;
/// The longest text tried, in characters, past every cap.
const MAX_CHARS: usize = 3_000;
/// The characters texts are made of.
const ALPHABET: [char; 26] = [
    'a', 'Z', '7', 'é', 'ç', 'œ', 'م', 'ر', 'ح', 'ب', 'ا', '\u{64e}', '\u{650}', '你', '😀', '%',
    '&', '+', '~', ' ', ' ', '\n', '\t', '\u{a0}', '\u{200b}', '\u{202e}',
];

fn text() -> impl Strategy<Value = String> {
    vec(select(ALPHABET.to_vec()), 0..=MAX_CHARS).prop_map(|cs| cs.into_iter().collect())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    /// No link for a blank text; any other link fits both caps and carries a clean start of the text, ended at a
    /// whole word or inside a single long one.
    #[test]
    fn each_link_fits_its_caps_and_ends_at_a_whole_word(
        text in text(),
        chars in TRANSLATE_CHARS,
        code in select(vec!["fr", "ar", "zh-TW", "en"]),
    ) {
        let site = sites().unwrap().first().unwrap();
        let clean = clean(&text);
        let Some(link) = site.link(&text, code, chars) else {
            // Only a blank text, or a first letter with more marks than the cap, leaves nothing.
            let first = clean.get(..kx_module_api::caret::on(&clean, 0)).unwrap_or_default();
            prop_assert!(clean.is_empty() || first.chars().count() > usize::from(chars));
            return Ok(());
        };
        prop_assert!(Link::new(link.expose().to_owned()).is_ok());
        prop_assert!(link.expose().len() <= site.link_max);
        let kept = sent(link.expose());
        prop_assert!(!kept.is_empty() && kept.trim() == kept);
        prop_assert!(kept.chars().count() <= usize::from(chars));
        prop_assert!(!kept.chars().any(kx_module_api::is_hidden));
        prop_assert!(clean.starts_with(&kept));
        let next = clean[kept.len()..].chars().next();
        let whole = next.is_none_or(char::is_whitespace) || !kept.contains(char::is_whitespace);
        prop_assert!(whole, "cut inside a word: {:?}", next);
    }

    /// A read that hit its cap gives a start of the clean text ended at a whole word, and never keeps the read's last
    /// word unless a space ended it.
    #[test]
    fn a_capped_reads_link_never_keeps_a_word_the_read_cut(
        text in text(),
        chars in TRANSLATE_CHARS,
    ) {
        let site = sites().unwrap().first().unwrap();
        let clean = clean(&text);
        let Some(link) = site.cut_link(&text, "ar", chars) else {
            return Ok(());
        };
        prop_assert!(link.expose().len() <= site.link_max);
        let kept = sent(link.expose());
        prop_assert!(kept.chars().count() <= usize::from(chars) && clean.starts_with(&kept));
        let next = clean[kept.len()..].chars().next();
        let whole = next.is_none_or(char::is_whitespace) || !kept.contains(char::is_whitespace);
        prop_assert!(whole, "cut inside a word: {:?}", next);
        prop_assert!(spaced(&text) || next.is_some(), "the read's last word kept");
    }
}

/// The characters system tags are made of: letters, both joins, and what a broken tag may hold.
const TAG_ALPHABET: [char; 14] = [
    'a', 'Z', 'f', 'r', 'h', 'n', 't', '-', '_', '1', ' ', 'é', 'م', '😀',
];
/// The longest tag tried, past `TAG_CHARS`.
const MAX_TAG: usize = 24;

fn tag() -> impl Strategy<Value = String> {
    let formed = vec(select(TAG_ALPHABET.to_vec()), 0..=MAX_TAG);
    prop_oneof![
        formed.prop_map(|cs| cs.into_iter().collect()),
        any::<String>(),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    /// The system's tag is outside input: any `translate_to` and tag give a language tag, which is the site's own code,
    /// its fallback, or one language part in lower case.
    #[test]
    fn any_tag_gives_a_language_tag_and_never_panics(set in tag(), os in tag()) {
        let site = sites().unwrap().first().unwrap();
        let code = site.target(&set, &os);
        prop_assert!(tag_ok(&code), "{:?}", code);
        let own = site.whole.contains(&code) || site.alias.values().any(|v| *v == code);
        let lower = code.bytes().all(|b| b.is_ascii_lowercase());
        prop_assert!(own || code == site.fallback || lower, "{:?}", code);
    }
}

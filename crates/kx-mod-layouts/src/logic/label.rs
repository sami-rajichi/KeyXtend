//! How the keyboard names a layout: the language key's short name, the space bar's parts,
//! the direction, and whether it needs the Arabic font.

use super::script;
use kx_module_api::ports::{LangFacts, LangLabel, LayoutId};
use std::collections::BTreeMap;

/// The label of layout `id` from its `facts`; `short` gives the language key's text by ISO 639 code.
/// A language counts as Arabic when its own name holds an Arabic letter.
#[must_use]
pub fn label(id: LayoutId, facts: &LangFacts, short: &BTreeMap<String, String>) -> LangLabel {
    LangLabel {
        id,
        short: short_name(&facts.iso, short),
        name: name_parts(facts),
        iso: facts.iso.to_ascii_lowercase(),
        rtl: facts.rtl,
        arabic: script::is_arabic(&facts.native),
    }
}

/// The configured short name of language `iso`, else its code in capitals (as the spike did).
fn short_name(iso: &str, short: &BTreeMap<String, String>) -> String {
    short
        .get(&iso.to_ascii_lowercase())
        .cloned()
        .unwrap_or_else(|| iso.to_uppercase())
}

/// The space bar's parts: the language, then the layout, leaving out an unknown one.
fn name_parts(facts: &LangFacts) -> Vec<String> {
    [&facts.native, &facts.layout]
        .into_iter()
        .filter(|part| !part.is_empty())
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: LayoutId = LayoutId::new(0x0401_0401);

    fn facts(iso: &str, native: &str, layout: &str, rtl: bool) -> LangFacts {
        LangFacts {
            iso: iso.into(),
            native: native.into(),
            layout: layout.into(),
            rtl,
        }
    }

    fn short() -> BTreeMap<String, String> {
        [("ar".to_owned(), "ع".to_owned())].into()
    }

    #[test]
    fn arabic_takes_its_configured_short_name_and_the_arabic_font() {
        let l = label(ID, &facts("ar", "العربية", "Arabic (101)", true), &short());
        assert_eq!(l.id, ID);
        assert_eq!(l.short, "ع");
        assert_eq!(l.name, ["العربية", "Arabic (101)"]);
        assert!(l.rtl && l.arabic);
    }

    #[test]
    fn an_unlisted_language_shows_its_code_in_capitals() {
        let l = label(ID, &facts("fr", "français", "French", false), &short());
        assert_eq!(l.short, "FR");
        assert_eq!(l.name, ["français", "French"]);
        assert!(!l.rtl && !l.arabic);
    }

    #[test]
    fn a_code_in_capitals_still_finds_its_short_name() {
        assert_eq!(label(ID, &facts("AR", "", "", true), &short()).short, "ع");
    }

    #[test]
    fn the_label_carries_the_language_code_in_lower_case() {
        assert_eq!(label(ID, &facts("FR", "", "", false), &short()).iso, "fr");
        assert_eq!(label(ID, &facts("ar", "", "", true), &short()).iso, "ar");
        assert!(label(ID, &LangFacts::default(), &short()).iso.is_empty());
    }

    #[test]
    fn right_to_left_is_not_arabic_by_itself() {
        let l = label(ID, &facts("he", "עברית", "Hebrew", true), &short());
        assert!(l.rtl && !l.arabic);
        let fa = label(ID, &facts("fa", "فارسی", "Persian", true), &short());
        assert_eq!(fa.short, "FA");
        assert!(
            fa.arabic,
            "found from its name, not from a list of languages"
        );
    }

    #[test]
    fn an_unknown_part_is_left_out_of_the_space_bar() {
        assert_eq!(
            label(ID, &facts("en", "English", "", false), &short()).name,
            ["English"]
        );
        assert_eq!(
            label(ID, &facts("en", "", "US", false), &short()).name,
            ["US"]
        );
        assert!(label(ID, &LangFacts::default(), &short()).name.is_empty());
    }
}

//! What the language key and space bar say about a layout: a short name, the language and layout names, and its direction.
//! The facts come from the product (kx-platform-windows); the naming goes to the `[layouts]` settings later in P3.

use std::collections::BTreeMap;

use kx_module_api::ports::LayoutId;
use serde::Deserialize;

/// `spike.toml [lang]`: short names that differ from the ISO code, and what joins the space bar's names.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LangConfig {
    /// Short names by ISO 639 code, such as `ar = "ع"`; others show the code in capitals.
    pub short: BTreeMap<String, String>,
    /// Between the language and the layout on the space bar.
    pub joiner: String,
}

/// One layout as the keyboard names it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LangInfo {
    /// For the language key, such as `EN` or `ع`.
    pub short: String,
    /// For the space bar, such as `English · US`.
    pub name: String,
    /// The language is written right to left.
    pub rtl: bool,
}

/// The short name for ISO code `iso`: the configured one, else the code in capitals.
pub fn short(iso: &str, c: &LangConfig) -> String {
    c.short
        .get(iso)
        .cloned()
        .unwrap_or_else(|| iso.to_uppercase())
}

/// The space bar text: the language, then the layout when known.
pub fn space_label(language: &str, layout: Option<&str>, c: &LangConfig) -> String {
    match layout {
        Some(l) if !l.is_empty() => format!("{language}{}{l}", c.joiner),
        _ => language.to_string(),
    }
}

/// What the keyboard says about layout `hkl`.
pub fn info(hkl: isize, c: &LangConfig) -> LangInfo {
    let facts = kx_platform_windows::langinfo::facts(LayoutId::new(hkl as u64));
    LangInfo {
        short: short(&facts.iso, c),
        name: space_label(&facts.native, Some(&facts.layout), c),
        rtl: facts.rtl,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> LangConfig {
        crate::config::load().expect("spike.toml loads").lang
    }

    #[test]
    fn a_short_name_is_the_configured_one_or_the_code_in_capitals() {
        let c = cfg();
        assert_eq!(short("ar", &c), "ع");
        assert_eq!(short("en", &c), "EN");
    }

    #[test]
    fn the_space_bar_joins_language_and_layout() {
        let c = cfg();
        let both = space_label("English", Some("US"), &c);
        assert_eq!(both, format!("English{}US", c.joiner));
        assert_eq!(space_label("English", None, &c), "English");
        assert_eq!(space_label("English", Some(""), &c), "English");
    }

    #[test]
    fn windows_names_english_and_marks_arabic_right_to_left() {
        let c = cfg();
        let en = info(0x0409_0409, &c);
        assert_eq!((en.short.as_str(), en.rtl), ("EN", false));
        assert!(en.name.starts_with("English"), "{}", en.name);
        let ar = info(0x0401_0401, &c);
        assert_eq!((ar.short.as_str(), ar.rtl), ("ع", true));
        assert!(ar.name.starts_with("العربية"), "{}", ar.name);
    }
}

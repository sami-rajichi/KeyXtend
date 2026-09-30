//! What the language key and space bar say about a layout: the product's `LangLabel`, named by kx-mod-layouts.
//! The joiner waits here for the translations (P3 Task 15); the product's UI joins the space bar's parts itself.

use std::collections::BTreeMap;

use kx_module_api::ports::{LangLabel, LayoutId};
use serde::Deserialize;

/// `spike.toml [lang]`: what joins the space bar's names, and the product's short names.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LangConfig {
    /// Short names by ISO 639 code from kx-mod-layouts' defaults, filled at load; spike.toml may not set them.
    #[serde(skip)]
    pub short: BTreeMap<String, String>,
    /// Between the language and the layout on the space bar.
    pub joiner: String,
}

/// Layout `hkl` as the product labels it, with the short names of `c`.
pub fn label(hkl: isize, c: &LangConfig) -> LangLabel {
    let id = LayoutId::new(hkl as u64);
    let facts = kx_platform_windows::langinfo::facts(id);
    kx_mod_layouts::label(id, &facts, &c.short)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> LangConfig {
        crate::config::load().expect("spike.toml loads").lang
    }

    #[test]
    fn the_short_names_come_from_the_product() {
        assert_eq!(cfg().short.get("ar").map(String::as_str), Some("ع"));
    }

    #[test]
    fn spike_toml_may_not_set_short_names() {
        let text = "short = { ar = \"x\" }\njoiner = \" · \"";
        assert!(toml::from_str::<LangConfig>(text).is_err());
    }

    #[test]
    fn windows_names_english_and_marks_arabic_right_to_left() {
        let c = cfg();
        let en = label(0x0409_0409, &c);
        assert_eq!((en.short.as_str(), en.rtl, en.arabic), ("EN", false, false));
        assert!(en.name[0].starts_with("English"), "{:?}", en.name);
        let ar = label(0x0401_0401, &c);
        assert_eq!((ar.short.as_str(), ar.rtl, ar.arabic), ("ع", true, true));
        assert!(ar.name[0].starts_with("العربية"), "{:?}", ar.name);
    }
}

//! What the language key and space bar say about a layout, named by the product's layouts module (ADR-0014).
//! The joiner waits here for the translations (P3 Task 15), and `LangInfo` goes to kx-mod-keyboard (Task 13).

use std::collections::BTreeMap;

use kx_module_api::ports::LayoutId;
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

/// What the keyboard says about layout `hkl`: the product's label, its space bar parts joined.
pub fn info(hkl: isize, c: &LangConfig) -> LangInfo {
    let id = LayoutId::new(hkl as u64);
    let facts = kx_platform_windows::langinfo::facts(id);
    let label = kx_mod_layouts::label(id, &facts, &c.short);
    LangInfo {
        short: label.short,
        name: label.name.join(&c.joiner),
        rtl: label.rtl,
    }
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
        let en = info(0x0409_0409, &c);
        assert_eq!((en.short.as_str(), en.rtl), ("EN", false));
        assert!(en.name.starts_with("English"), "{}", en.name);
        assert!(en.name.contains(c.joiner.as_str()), "{}", en.name);
        let ar = info(0x0401_0401, &c);
        assert_eq!((ar.short.as_str(), ar.rtl), ("ع", true));
        assert!(ar.name.starts_with("العربية"), "{}", ar.name);
    }
}

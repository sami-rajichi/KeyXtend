//! What the language key and space bar say about a layout: a short name, the language and layout names, and its direction.

use std::collections::BTreeMap;

use serde::Deserialize;
use windows::Win32::Globalization::{
    GetLocaleInfoEx, LCIDToLocaleName, LOCALE_IREADINGLAYOUT, LOCALE_RETURN_NUMBER,
    LOCALE_SISO639LANGNAME, LOCALE_SNATIVELANGUAGENAME,
};
use windows::Win32::System::Registry::{HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ, RegGetValueW};
use windows::core::{HSTRING, PCWSTR, w};

use crate::layout::lang_id;

/// Registry folder of the installed layouts, one subkey per layout id.
const LAYOUTS_KEY: &str = r"SYSTEM\CurrentControlSet\Control\Keyboard Layouts";
/// Registry value with a layout's English name.
const LAYOUT_TEXT: PCWSTR = w!("Layout Text");
/// A layout handle's high word marks a layout variant when these bits are set.
const VARIANT_BITS: u16 = 0xF000;
/// Bits of a layout handle's high word.
const WORD_SHIFT: u32 = 16;
/// Longest name read from Windows, in UTF-16 units.
const NAME_CAP: usize = 128;
/// `LOCALE_IREADINGLAYOUT` value of right-to-left scripts.
const RTL_LAYOUT: u32 = 1;

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

/// The registry id of layout `hkl`, such as `0000040C`; `None` for variants, which need a search.
pub fn klid(hkl: isize) -> Option<String> {
    let high = ((hkl as usize) >> WORD_SHIFT) as u16;
    (high & VARIANT_BITS != VARIANT_BITS).then(|| format!("{high:08X}"))
}

/// The space bar text: the language, then the layout when known.
pub fn space_label(language: &str, layout: Option<&str>, c: &LangConfig) -> String {
    match layout {
        Some(l) if !l.is_empty() => format!("{language}{}{l}", c.joiner),
        _ => language.to_string(),
    }
}

/// The locale name of language `id`, null-terminated, such as `fr-FR`.
fn locale(id: u16) -> Option<Vec<u16>> {
    let mut buf = vec![0u16; NAME_CAP];
    // SAFETY: the buffer is valid for its whole length.
    let n = unsafe { LCIDToLocaleName(u32::from(id), Some(&mut buf), 0) };
    let n = usize::try_from(n).ok().filter(|&n| n > 0)?;
    buf.truncate(n);
    Some(buf)
}

/// Locale text `what` for `loc`; empty when Windows has none.
fn text(loc: &[u16], what: u32) -> String {
    let mut buf = [0u16; NAME_CAP];
    // SAFETY: `loc` is null-terminated and the buffer is valid for its whole length.
    let n = unsafe { GetLocaleInfoEx(PCWSTR(loc.as_ptr()), what, Some(&mut buf)) };
    let n = usize::try_from(n).unwrap_or(0).saturating_sub(1);
    String::from_utf16_lossy(&buf[..n.min(NAME_CAP)])
}

/// Locale number `what` for `loc`.
fn number(loc: &[u16], what: u32) -> Option<u32> {
    let mut buf = [0u16; 2];
    // SAFETY: `loc` is null-terminated; with LOCALE_RETURN_NUMBER Windows writes one u32 into the 2-unit buffer.
    let n = unsafe {
        GetLocaleInfoEx(
            PCWSTR(loc.as_ptr()),
            what | LOCALE_RETURN_NUMBER,
            Some(&mut buf),
        )
    };
    (n > 0).then(|| u32::from(buf[0]) | (u32::from(buf[1]) << WORD_SHIFT))
}

/// The English layout name from the registry, such as `French (Legacy, AZERTY)`.
fn layout_text(hkl: isize) -> Option<String> {
    let key = HSTRING::from(format!(r"{LAYOUTS_KEY}\{}", klid(hkl)?));
    let mut buf = [0u16; NAME_CAP];
    let mut size = (NAME_CAP * size_of::<u16>()) as u32;
    let data = Some(buf.as_mut_ptr().cast());
    // SAFETY: reads one string into `buf`, whose size in bytes is passed in `size`.
    let r = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            &key,
            LAYOUT_TEXT,
            RRF_RT_REG_SZ,
            None,
            data,
            Some(&mut size),
        )
    };
    let units = (size as usize / size_of::<u16>())
        .saturating_sub(1)
        .min(NAME_CAP);
    r.is_ok().then(|| String::from_utf16_lossy(&buf[..units]))
}

/// What the keyboard says about layout `hkl`.
pub fn info(hkl: isize, c: &LangConfig) -> LangInfo {
    let Some(loc) = locale(lang_id(hkl)) else {
        return LangInfo::default();
    };
    let native = text(&loc, LOCALE_SNATIVELANGUAGENAME);
    LangInfo {
        short: short(&text(&loc, LOCALE_SISO639LANGNAME), c),
        name: space_label(&native, layout_text(hkl).as_deref(), c),
        rtl: number(&loc, LOCALE_IREADINGLAYOUT) == Some(RTL_LAYOUT),
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
    fn plain_layouts_have_a_registry_id_and_variants_do_not() {
        assert_eq!(klid(0x040C_040C).as_deref(), Some("0000040C"));
        assert_eq!(klid(0xF0A8_1C01_u32 as i32 as isize), None);
    }

    #[test]
    fn the_space_bar_joins_language_and_layout() {
        let c = cfg();
        let both = space_label("English", Some("US"), &c);
        assert_eq!(both, format!("English{}US", c.joiner));
        assert_eq!(space_label("English", None, &c), "English");
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

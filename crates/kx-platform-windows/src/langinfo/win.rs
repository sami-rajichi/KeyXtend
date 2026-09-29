//! The Windows side of the language facts: the locale queries and the layout's name in the registry.

use kx_module_api::ports::{LangFacts, LayoutId};
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::Globalization::{
    GetLocaleInfoEx, LCIDToLocaleName, LOCALE_IREADINGLAYOUT, LOCALE_RETURN_NUMBER,
    LOCALE_SISO639LANGNAME, LOCALE_SNATIVELANGUAGENAME,
};
use windows::Win32::System::Registry::{
    HKEY, HKEY_LOCAL_MACHINE, KEY_READ, RRF_RT_REG_SZ, RegCloseKey, RegEnumKeyW, RegGetValueW,
    RegOpenKeyExW,
};
use windows::core::{PCWSTR, w};

use super::{
    RegName, is_rtl, lang_id, number_of, parse_layout_id, reg_name, reg_text, text_of, until_null,
};

/// The registry folder of the installed layouts, one key per layout.
const LAYOUTS_KEY: PCWSTR = w!(r"SYSTEM\CurrentControlSet\Control\Keyboard Layouts");
/// The value with a layout's English name.
const LAYOUT_TEXT: PCWSTR = w!("Layout Text");
/// The value with a variant's number.
const LAYOUT_ID: PCWSTR = w!("Layout Id");
/// Longest name read, in UTF-16 units.
const NAME_CAP: usize = 128;

/// The locale name of layout `id`'s language, such as `fr-FR`; `None` when Windows has none.
#[must_use]
pub fn locale(id: LayoutId) -> Option<String> {
    let units = locale_units(lang_id(id))?;
    Some(text_of(i32::try_from(units.len()).ok()?, &units))
}

/// The language facts of layout `id`; empty when Windows has none.
#[must_use]
pub fn facts(id: LayoutId) -> LangFacts {
    let Some(loc) = locale_units(lang_id(id)) else {
        return LangFacts::default();
    };
    LangFacts {
        iso: text(&loc, LOCALE_SISO639LANGNAME),
        native: text(&loc, LOCALE_SNATIVELANGUAGENAME),
        layout: layout_name(&reg_name(id)).unwrap_or_default(),
        rtl: number(&loc, LOCALE_IREADINGLAYOUT).is_some_and(is_rtl),
    }
}

/// The locale name of language `lang`, null included, such as `fr-FR`.
#[allow(unsafe_code, reason = "A locale query into a live buffer.")]
fn locale_units(lang: u16) -> Option<Vec<u16>> {
    let mut buf = [0u16; NAME_CAP];
    // SAFETY: Windows writes at most the buffer's length.
    let n = unsafe { LCIDToLocaleName(u32::from(lang), Some(&mut buf), 0) };
    let n = usize::try_from(n).ok().filter(|&n| n > 0)?;
    buf.get(..n).map(<[u16]>::to_vec)
}

/// Locale text `what` of locale `loc`; empty when Windows has none.
#[allow(unsafe_code, reason = "A locale query into a live buffer.")]
fn text(loc: &[u16], what: u32) -> String {
    let mut buf = [0u16; NAME_CAP];
    // SAFETY: `loc` ends with a null; Windows writes at most the buffer's length.
    let n = unsafe { GetLocaleInfoEx(PCWSTR(loc.as_ptr()), what, Some(&mut buf)) };
    text_of(n, &buf)
}

/// Locale number `what` of locale `loc`.
#[allow(unsafe_code, reason = "A locale query into a live buffer.")]
fn number(loc: &[u16], what: u32) -> Option<u32> {
    let mut buf = [0u16; 2];
    let asked = what | LOCALE_RETURN_NUMBER;
    // SAFETY: `loc` ends with a null; with LOCALE_RETURN_NUMBER Windows writes one u32 into the two units.
    let n = unsafe { GetLocaleInfoEx(PCWSTR(loc.as_ptr()), asked, Some(&mut buf)) };
    (n > 0).then(|| number_of(buf))
}

/// The layouts folder of the registry, open for reading and closed once when dropped.
struct Layouts(HKEY);

impl Layouts {
    /// Opens the folder; `None` when Windows refuses.
    #[allow(unsafe_code, reason = "Opens a registry key that Drop closes.")]
    fn open() -> Option<Self> {
        let mut key = HKEY::default();
        // SAFETY: `key` is a local the call fills; `Drop` closes it once.
        let r = unsafe {
            RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                LAYOUTS_KEY,
                None,
                KEY_READ,
                &raw mut key,
            )
        };
        (r == ERROR_SUCCESS).then_some(Self(key))
    }

    /// Text value `value` of layout key `sub` (null included); `None` when missing or too long.
    #[allow(unsafe_code, reason = "Reads one registry text into a live buffer.")]
    fn value(&self, sub: &[u16], value: PCWSTR) -> Option<String> {
        let mut buf = [0u16; NAME_CAP];
        let mut bytes = u32::try_from(size_of_val(&buf)).ok()?;
        let data = Some(buf.as_mut_ptr().cast());
        // SAFETY: `sub` ends with a null; Windows writes at most `bytes` bytes into `buf` and the size back.
        let r = unsafe {
            RegGetValueW(
                self.0,
                PCWSTR(sub.as_ptr()),
                value,
                RRF_RT_REG_SZ,
                None,
                data,
                Some(&raw mut bytes),
            )
        };
        (r == ERROR_SUCCESS).then(|| reg_text(bytes, &buf))
    }

    /// The name of layout key number `index`, null included; `None` past the last.
    #[allow(unsafe_code, reason = "Reads one key name into a live buffer.")]
    fn key(&self, index: u32) -> Option<Vec<u16>> {
        let mut buf = [0u16; NAME_CAP];
        // SAFETY: Windows writes at most the buffer's length, null included.
        let r = unsafe { RegEnumKeyW(self.0, index, Some(&mut buf)) };
        (r == ERROR_SUCCESS).then(|| wide(&until_null(&buf, buf.len())))
    }

    /// The layout key whose `Layout Id` is `number`.
    fn variant(&self, number: u16) -> Option<Vec<u16>> {
        let id_of = |sub: &Vec<u16>| self.value(sub, LAYOUT_ID).and_then(|t| parse_layout_id(&t));
        (0..)
            .map_while(|i| self.key(i))
            .find(|sub| id_of(sub) == Some(number))
    }
}

impl Drop for Layouts {
    #[allow(unsafe_code, reason = "Closes the key opened in `open`.")]
    fn drop(&mut self) {
        // SAFETY: closes the key `open` opened, once.
        let _ = unsafe { RegCloseKey(self.0) };
    }
}

/// `text` as UTF-16 with an ending null.
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain([0]).collect()
}

/// The English name the registry gives the layout at `name`, such as `French (Legacy, AZERTY)`.
fn layout_name(name: &RegName) -> Option<String> {
    let layouts = Layouts::open()?;
    let sub = match name {
        RegName::Key(key) => wide(key),
        RegName::Variant(number) => layouts.variant(*number)?,
    };
    layouts.value(&sub, LAYOUT_TEXT)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// US English, French (Legacy, AZERTY), Arabic (101) on Arabic (Saudi Arabia).
    const EN: LayoutId = LayoutId::new(0x0409_0409);
    const FR: LayoutId = LayoutId::new(0x040C_040C);
    const AR_101: LayoutId = LayoutId::new(0x0401_0401);
    /// Arabic (102) AZERTY on Arabic (Tunisia), a variant found by its `Layout Id`.
    const AR_102: LayoutId = LayoutId::new(0xFFFF_FFFF_F029_1C01);
    /// Arabic in Arabic.
    const ARABIC: &str = "العربية";

    #[test]
    fn windows_names_english_and_french_left_to_right() {
        let en = facts(EN);
        assert_eq!(
            (en.iso.as_str(), en.layout.as_str(), en.rtl),
            ("en", "US", false)
        );
        assert!(en.native.starts_with("English"), "{}", en.native);
        let fr = facts(FR);
        assert_eq!((fr.iso.as_str(), fr.native.as_str()), ("fr", "français"));
        assert!(fr.layout.starts_with("French"), "{}", fr.layout);
    }

    #[test]
    fn arabic_is_right_to_left_with_its_own_name() {
        let ar = facts(AR_101);
        assert_eq!((ar.iso.as_str(), ar.native.as_str()), ("ar", ARABIC));
        assert_eq!((ar.layout.as_str(), ar.rtl), ("Arabic (101)", true));
    }

    #[test]
    fn a_variant_finds_its_name_by_its_layout_id() {
        let ar = facts(AR_102);
        assert_eq!(
            (ar.iso.as_str(), ar.native.as_str(), ar.rtl),
            ("ar", ARABIC, true)
        );
        assert_eq!(ar.layout, "Arabic (102) AZERTY");
    }

    #[test]
    fn the_locale_names_language_and_region() {
        assert_eq!(locale(EN).as_deref(), Some("en-US"));
        assert_eq!(locale(FR).as_deref(), Some("fr-FR"));
        assert_eq!(locale(AR_102).as_deref(), Some("ar-TN"));
    }
}

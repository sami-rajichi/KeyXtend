//! What Windows says about a layout's language: its ISO code, its name in itself, the layout's name and direction.
//! Reading a layout id and Windows' answers is pure and tested everywhere; the queries are in `win`.

use kx_module_api::ports::LayoutId;

#[cfg(windows)]
mod win;

#[cfg(windows)]
pub use win::{facts, locale};

/// Hex digits of a registry layout key, such as `0000040C`.
const KEY_DIGITS: usize = 8;
/// The device-word bits that say what kind of layout it is.
const KIND_BITS: u16 = 0xF000;
/// The kind of a layout variant, whose other device bits are its registry `Layout Id`.
const VARIANT: u16 = 0xF000;
/// The kind of an input method editor, whose registry key is its whole handle.
const IME: u16 = 0xE000;
/// The base of a registry `Layout Id` text.
const HEX: u32 = 16;
/// `LOCALE_IREADINGLAYOUT` value of right-to-left scripts.
const RTL_READING: u32 = 1;

/// Where the registry keeps a layout's name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegName {
    /// The layout's own key, such as `0000040C`.
    Key(String),
    /// A variant: the key whose `Layout Id` value is this number.
    Variant(u16),
}

/// The 32-bit handle inside layout `id`; the rest is sign extension.
fn handle(id: LayoutId) -> u32 {
    let [a, b, c, d, ..] = id.get().to_le_bytes();
    u32::from_le_bytes([a, b, c, d])
}

/// The language word and the device word of layout `id`.
fn words(id: LayoutId) -> (u16, u16) {
    let [a, b, c, d] = handle(id).to_le_bytes();
    (u16::from_le_bytes([a, b]), u16::from_le_bytes([c, d]))
}

/// The language id (LANGID) of layout `id`: its low word, such as `0x040C` for French (France).
#[must_use]
pub fn lang_id(id: LayoutId) -> u16 {
    words(id).0
}

/// Where the registry keeps the name of layout `id`.
#[must_use]
pub fn reg_name(id: LayoutId) -> RegName {
    let device = words(id).1;
    match device & KIND_BITS {
        VARIANT => RegName::Variant(device & !KIND_BITS),
        IME => RegName::Key(format!("{:0KEY_DIGITS$X}", handle(id))),
        _ => RegName::Key(format!("{device:0KEY_DIGITS$X}")),
    }
}

/// Reads a registry `Layout Id` value, such as `0029`; `None` when it is not hex.
#[must_use]
pub fn parse_layout_id(text: &str) -> Option<u16> {
    let hex = !text.is_empty() && text.bytes().all(|b| b.is_ascii_hexdigit());
    hex.then(|| u16::from_str_radix(text, HEX).ok()).flatten()
}

/// The first `units` of `buf`, up to the first null.
fn until_null(buf: &[u16], units: usize) -> String {
    let text = &buf[..units.min(buf.len())];
    let end = text.iter().position(|&u| u == 0).unwrap_or(text.len());
    String::from_utf16_lossy(&text[..end])
}

/// The text of a Windows answer of `n` units, the last one being the ending null.
#[must_use]
pub fn text_of(n: i32, buf: &[u16]) -> String {
    until_null(buf, usize::try_from(n).unwrap_or(0).saturating_sub(1))
}

/// A registry text of `bytes` bytes in `buf`, up to its first null.
#[must_use]
pub fn reg_text(bytes: u32, buf: &[u16]) -> String {
    let units = usize::try_from(bytes).unwrap_or(usize::MAX) / size_of::<u16>();
    until_null(buf, units)
}

/// A locale number as `GetLocaleInfoEx` writes it: a `u32` in two UTF-16 units, low unit first.
#[must_use]
pub fn number_of(units: [u16; 2]) -> u32 {
    let [a, b] = units[0].to_le_bytes();
    let [c, d] = units[1].to_le_bytes();
    u32::from_le_bytes([a, b, c, d])
}

/// True for a `LOCALE_IREADINGLAYOUT` value of right-to-left scripts.
#[must_use]
pub fn is_rtl(reading: u32) -> bool {
    reading == RTL_READING
}

#[cfg(test)]
mod tests {
    use super::*;

    /// US English, French (Legacy, AZERTY), Arabic (101) on Arabic (Tunisia).
    const EN: LayoutId = LayoutId::new(0x0409_0409);
    const FR: LayoutId = LayoutId::new(0x040C_040C);
    const AR_101: LayoutId = LayoutId::new(0x0401_1C01);
    /// Arabic (102) AZERTY on Arabic (Tunisia): a variant, sign-extended as a 64-bit handle.
    const AR_102: LayoutId = LayoutId::new(0xFFFF_FFFF_F029_1C01);
    /// A legacy Japanese input method editor.
    const IME_JA: LayoutId = LayoutId::new(0xFFFF_FFFF_E001_0411);

    fn units(text: &str) -> Vec<u16> {
        text.encode_utf16().chain([0]).collect()
    }

    #[test]
    fn the_language_id_is_the_low_word() {
        assert_eq!(lang_id(EN), 0x0409);
        assert_eq!(lang_id(AR_101), 0x1C01);
        assert_eq!(lang_id(AR_102), 0x1C01, "sign extension is ignored");
    }

    #[test]
    fn plain_layouts_name_their_key_and_variants_their_layout_id() {
        assert_eq!(reg_name(EN), RegName::Key("00000409".into()));
        assert_eq!(reg_name(FR), RegName::Key("0000040C".into()));
        assert_eq!(reg_name(AR_101), RegName::Key("00000401".into()));
        assert_eq!(reg_name(AR_102), RegName::Variant(0x029));
    }

    #[test]
    fn an_input_method_names_its_whole_handle() {
        assert_eq!(reg_name(IME_JA), RegName::Key("E0010411".into()));
    }

    #[test]
    fn a_layout_id_value_is_hex_in_any_case() {
        assert_eq!(parse_layout_id("0029"), Some(0x29));
        assert_eq!(parse_layout_id("00a8"), Some(0xA8));
        assert_eq!(parse_layout_id("00A8"), Some(0xA8));
        for bad in ["", "x9", "+29", "0x29", "10000", " 29"] {
            assert_eq!(parse_layout_id(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn a_windows_text_drops_its_ending_null_and_stays_in_its_buffer() {
        let buf = units("fr-FR");
        assert_eq!(text_of(6, &buf), "fr-FR");
        assert_eq!(text_of(3, &buf), "fr");
        assert_eq!(text_of(0, &buf), "", "Windows had nothing");
        assert_eq!(text_of(-1, &buf), "", "a failure count");
        assert_eq!(text_of(99, &buf), "fr-FR", "a count past the buffer");
    }

    #[test]
    fn a_registry_text_ends_at_its_null_or_its_size() {
        let buf = units("US");
        assert_eq!(reg_text(6, &buf), "US", "the size counts the null");
        assert_eq!(reg_text(4, &buf), "US", "no null in the size");
        assert_eq!(reg_text(5, &buf), "US", "an odd size");
        assert_eq!(reg_text(0, &buf), "");
        assert_eq!(reg_text(u32::MAX, &buf), "US", "a size past the buffer");
        let arabic = units("العربية");
        assert_eq!(reg_text(u32::MAX, &arabic), "العربية");
    }

    #[test]
    fn a_locale_number_is_low_unit_first() {
        assert_eq!(number_of([1, 0]), 1);
        assert_eq!(number_of([0, 1]), 0x1_0000);
        assert_eq!(number_of([0xFFFF, 0xFFFF]), u32::MAX);
    }

    #[test]
    fn only_the_right_to_left_reading_layout_is_rtl() {
        assert!(is_rtl(1));
        for reading in [0, 2, 3] {
            assert!(!is_rtl(reading), "{reading}");
        }
    }
}

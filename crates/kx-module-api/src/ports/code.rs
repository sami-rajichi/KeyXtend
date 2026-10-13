//! Scan codes and layout ids as text: `0x` and hex digits.
//! A layout id travels as this text, since JSON numbers lose precision above 2^53.

use super::{LayoutId, Scan};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

/// What starts a code written as text.
const HEX_PREFIX: &str = "0x";
/// The base of a code written as text.
const HEX: u32 = 16;
/// Hex digits of a written scan code.
const SCAN_DIGITS: usize = 4;
/// Hex digits of a written layout id.
const ID_DIGITS: usize = 16;

/// Text that is not `0x` and hex digits, or is too big for its type.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
#[error("{0:?} is not a 0x hex code in range")]
pub struct CodeError(String);

/// Reads `text` as `0x` and hex digits, in any width.
fn parse_hex(text: &str) -> Result<u64, CodeError> {
    let bad = || CodeError(text.to_owned());
    let digits = text.strip_prefix(HEX_PREFIX).ok_or_else(bad)?;
    if !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(bad());
    }
    u64::from_str_radix(digits, HEX).map_err(|_| bad())
}

impl fmt::Display for Scan {
    /// Writes `0x` and four hex digits, such as `0xE035`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{HEX_PREFIX}{:0w$X}", self.code(), w = SCAN_DIGITS)
    }
}

impl FromStr for Scan {
    type Err = CodeError;

    /// Reads `0x` and hex digits in any width, such as `0x1C` or `0x001C`.
    fn from_str(text: &str) -> Result<Self, CodeError> {
        let value = parse_hex(text)?;
        u16::try_from(value)
            .map(Self::new)
            .map_err(|_| CodeError(text.to_owned()))
    }
}

impl fmt::Display for LayoutId {
    /// Writes `0x` and sixteen hex digits, so any id survives JSON.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{HEX_PREFIX}{:0w$X}", self.get(), w = ID_DIGITS)
    }
}

impl FromStr for LayoutId {
    type Err = CodeError;

    /// Reads `0x` and hex digits in any width.
    fn from_str(text: &str) -> Result<Self, CodeError> {
        parse_hex(text).map(Self::new)
    }
}

impl Serialize for LayoutId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for LayoutId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        text.parse().map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_read_in_any_width_and_write_in_a_fixed_one() {
        assert_eq!("0x1C".parse::<Scan>(), Ok(Scan::new(0x1C)));
        assert_eq!("0x001c".parse::<Scan>(), Ok(Scan::new(0x1C)));
        assert_eq!(Scan::new(0x1C).to_string(), "0x001C");
        assert_eq!(Scan::new(0xE035).to_string(), "0xE035");
        assert_eq!(LayoutId::new(0x0409).to_string(), "0x0000000000000409");
    }

    #[test]
    fn text_that_is_not_a_hex_code_is_refused() {
        for bad in ["1C", "0x", "0x+1C", "0xZZ", "0X1C", " 0x1C", "0x10000"] {
            assert!(bad.parse::<Scan>().is_err(), "{bad}");
        }
        assert!("0x1FFFFFFFFFFFFFFFF".parse::<LayoutId>().is_err());
    }

    #[test]
    fn every_scan_survives_its_text() {
        for code in 0..=u16::MAX {
            let scan = Scan::new(code);
            assert_eq!(scan.to_string().parse::<Scan>(), Ok(scan));
        }
    }

    #[test]
    fn a_sign_extended_layout_id_travels_as_hex_text() {
        let id = LayoutId::new(0xFFFF_FFFF_F002_0409);
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, r#""0xFFFFFFFFF0020409""#);
        assert_eq!(serde_json::from_str::<LayoutId>(&json).unwrap(), id);
        assert!(serde_json::from_str::<LayoutId>("123").is_err());
        assert!(serde_json::from_str::<LayoutId>(r#""0xZZ""#).is_err());
    }
}

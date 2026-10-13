//! A colour with alpha: read from `#RRGGBB` or `#RRGGBBAA`, written as Qt reads it, and blended over another.

use serde::{Deserialize, Serialize, Serializer};

/// Hex digits in `#RRGGBB`.
const RGB_DIGITS: usize = 6;
/// Hex digits in `#RRGGBBAA`.
const RGBA_DIGITS: usize = 8;

/// A colour with alpha, written `#RRGGBB` or `#RRGGBBAA`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub struct Rgba {
    /// Red.
    pub r: u8,
    /// Green.
    pub g: u8,
    /// Blue.
    pub b: u8,
    /// Alpha; 255 is opaque.
    pub a: u8,
}

impl Rgba {
    /// Nothing at all: see-through black.
    pub const CLEAR: Rgba = Rgba {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };
    /// Opaque black, the darkest ink there is.
    pub const BLACK: Rgba = Rgba {
        r: 0,
        g: 0,
        b: 0,
        a: u8::MAX,
    };
    /// Opaque white, the lightest ink; black or white reads at 4.58:1 or more on any opaque colour.
    pub const WHITE: Rgba = Rgba {
        r: u8::MAX,
        g: u8::MAX,
        b: u8::MAX,
        a: u8::MAX,
    };

    /// This colour at `share` opacity over `under`, blending each channel; exact over an opaque colour.
    #[must_use]
    pub fn over(self, under: Rgba, share: f32) -> Rgba {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a blend of two bytes, clamped to a byte"
        )]
        let mix = |a: u8, b: u8| {
            let v = share * f32::from(a) + (1.0 - share) * f32::from(b);
            v.round().clamp(0.0, f32::from(u8::MAX)) as u8
        };
        Rgba {
            r: mix(self.r, under.r),
            g: mix(self.g, under.g),
            b: mix(self.b, under.b),
            a: mix(self.a, under.a),
        }
    }

    /// This colour laid on `under` at its own opacity, as Qt draws a tint: the result is as opaque as `under`.
    #[must_use]
    pub fn on(self, under: Rgba) -> Rgba {
        Rgba {
            a: under.a,
            ..self.over(under, self.share())
        }
    }

    /// This tint laid over the tint `under`: one tint that draws as the two laid in turn (source over).
    #[must_use]
    pub fn stack(self, under: Rgba) -> Rgba {
        let (top, low) = (self.share(), under.share() * (1.0 - self.share()));
        let a = top + low;
        if a <= 0.0 {
            return Rgba::CLEAR;
        }
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a blend of two bytes, clamped to a byte"
        )]
        let byte = |v: f32| v.round().clamp(0.0, f32::from(u8::MAX)) as u8;
        let mix = |c: u8, d: u8| byte((top * f32::from(c) + low * f32::from(d)) / a);
        Rgba {
            r: mix(self.r, under.r),
            g: mix(self.g, under.g),
            b: mix(self.b, under.b),
            a: byte(a * f32::from(u8::MAX)),
        }
    }

    /// Its opacity, from 0 (clear) to 1 (opaque).
    #[must_use]
    pub fn share(self) -> f32 {
        f32::from(self.a) / f32::from(u8::MAX)
    }

    /// Reads `#RRGGBB` (opaque) or `#RRGGBBAA`.
    ///
    /// # Errors
    /// A text that is not `#` and six or eight hex digits.
    pub fn parse(s: &str) -> Result<Rgba, String> {
        let bad = || format!("{s:?} is not #RRGGBB or #RRGGBBAA");
        let hex = s.strip_prefix('#').ok_or_else(bad)?;
        let digits = hex.bytes().all(|b| b.is_ascii_hexdigit());
        if !digits || !matches!(hex.len(), RGB_DIGITS | RGBA_DIGITS) {
            return Err(bad());
        }
        let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| bad());
        let a = if hex.len() == RGBA_DIGITS {
            byte(RGB_DIGITS)?
        } else {
            u8::MAX
        };
        Ok(Rgba {
            r: byte(0)?,
            g: byte(2)?,
            b: byte(4)?,
            a,
        })
    }
}

impl Serialize for Rgba {
    /// Written as Qt reads colours: `#AARRGGBB`.
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            self.a, self.r, self.g, self.b
        ))
    }
}

impl From<kx_platform::ui::Rgba> for Rgba {
    /// A system colour, given as red, green, blue and alpha.
    fn from([r, g, b, a]: kx_platform::ui::Rgba) -> Self {
        Rgba { r, g, b, a }
    }
}

impl TryFrom<String> for Rgba {
    type Error = String;

    fn try_from(s: String) -> Result<Self, String> {
        Rgba::parse(&s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colours_take_six_or_eight_hex_digits() {
        let c = Rgba::parse("#F3F3F3").expect("six digits");
        assert_eq!((c.r, c.g, c.b, c.a), (0xF3, 0xF3, 0xF3, 0xFF));
        assert_eq!(Rgba::parse("#0000001F").map(|c| c.a), Ok(0x1F));
        for bad in ["F3F3F3", "#FFF", "#GGGGGG", "#F3F3F3F", ""] {
            assert!(Rgba::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn colours_reach_qt_alpha_first() {
        let c = Rgba::parse("#0000001F").expect("colour");
        assert_eq!(
            serde_json::to_string(&c).ok().as_deref(),
            Some("\"#1F000000\"")
        );
    }

    #[test]
    fn a_colour_over_another_blends_each_channel() {
        let (white, black) = (Rgba::parse("#FFFFFF"), Rgba::parse("#000000"));
        let (white, black) = (white.expect("white"), black.expect("black"));
        assert_eq!(
            white.over(black, 0.4),
            Rgba::parse("#666666").expect("grey")
        );
        assert_eq!(white.over(black, 1.0), white);
        assert_eq!(white.over(black, 0.0), black);
    }

    #[test]
    fn a_tint_laid_on_an_opaque_colour_blends_and_stays_opaque() {
        let tint = Rgba::parse("#00000033").expect("a fifth of black");
        let laid = tint.on(Rgba::WHITE);
        assert_eq!(laid, Rgba::parse("#CCCCCC").expect("grey"));
        assert_eq!(Rgba::CLEAR.on(Rgba::WHITE), Rgba::WHITE, "no tint");
    }

    #[test]
    fn a_tint_stacked_on_a_tint_draws_as_both_laid_in_turn() {
        let hover = Rgba::parse("#0000000F").expect("hover");
        assert_eq!(hover.stack(hover), Rgba::parse("#0000001D").expect("both"));
        let top = Rgba::parse("#FFFFFF80").expect("half white");
        for under in ["#FFFFFF", "#000000", "#3B3E43", "#F8F6F1"] {
            let fill = Rgba::parse(under).expect("fill");
            let (one, two) = (top.stack(hover).on(fill), top.on(hover.on(fill)));
            let step = |a: u8, b: u8| a.abs_diff(b);
            let most = [step(one.r, two.r), step(one.g, two.g), step(one.b, two.b)];
            assert!(most.iter().all(|&s| s <= 1), "{under}: {one:?} {two:?}");
        }
        assert_eq!(Rgba::CLEAR.stack(hover), hover, "a clear top");
        assert_eq!(Rgba::WHITE.stack(hover), Rgba::WHITE, "an opaque top");
        assert_eq!(Rgba::CLEAR.stack(Rgba::CLEAR), Rgba::CLEAR);
    }

    #[test]
    fn a_colours_share_is_its_opacity() {
        assert!(Rgba::WHITE.share() > 0.999 && Rgba::CLEAR.share() < 0.001);
        let half = Rgba::parse("#00000080").expect("half");
        assert!((half.share() - 0.502).abs() < 0.001);
    }
}

//! The theme tokens, compiled in: three themes in light and soft dark, the sizes, motion, icons and sounds they share,
//! and the look a face draws now, with contrast and high contrast. It has no Qt, so its tests run everywhere.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod accent;
pub mod assets;
mod common;
pub mod contrast;
pub mod drawn;
mod extras;
mod hc;
mod icons;
mod look;
mod mode;
mod motion;
mod palette;
mod rgba;
mod ringmove;
mod shape;
pub mod sounds;
mod theme;
mod view;

pub use accent::{accent_fill, with_accent};
pub use common::Common;
pub use hc::{contrast_common, contrast_palette, hover_line};
pub use icons::{BarIcons, DpadIcons, Icons, LangIcons, NoticeIcons, PageIcons, TrayIcons};
pub use look::{Cap, Dpad, DpadRefs, Gradient, Look};
pub use mode::{Mode, ModeChoice, mode_for};
pub use motion::{Amounts, MOVES, Motion, Motions, Timed};
pub use palette::{Palette, Shadow, Skirt};
pub use rgba::Rgba;
pub use ringmove::{RingAmounts, RingMotion};
pub use shape::Shape;
pub use theme::Theme;
pub use view::{LookChoice, LookView, look_view, reach};

use thiserror::Error;

/// Name of the themes file, for errors.
const THEMES_FILE: &str = "themes.toml";
/// Name of the shared sizes file, for errors.
const SHAPE_FILE: &str = "shape.toml";
/// Name of the motion file, for errors.
const MOTION_FILE: &str = "motion.toml";
/// Name of the icons file, for errors.
const ICONS_FILE: &str = "icons.toml";
/// Name of the sounds file, for errors.
const SOUNDS_FILE: &str = "sounds.toml";
/// The themes, compiled in so the app reads nothing it draws from disk (ADR-0016).
const THEMES: &str = include_str!("../tokens/themes.toml");
/// The shared sizes, compiled in.
const SHAPE: &str = include_str!("../tokens/shape.toml");
/// The motion, compiled in.
const MOTION: &str = include_str!("../tokens/motion.toml");
/// The icon names, compiled in.
const ICONS: &str = include_str!("../tokens/icons.toml");
/// The sound tokens' file, written once for `include_str!` and `SOUNDS_PATH`.
macro_rules! sounds_path {
    () => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/tokens/sounds.toml")
    };
}
/// Where the sound tokens sit, so kx-ui's build and the sound tool watch and read the file compiled in.
pub const SOUNDS_PATH: &str = sounds_path!();
/// The sound format, voices and events, compiled in.
const SOUNDS: &str = include_str!(sounds_path!());

/// A token file that does not load, and why; the texts name only tokens, never user data.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
#[error("{file}: {why}")]
pub struct ThemeError {
    /// The token file at fault.
    pub file: &'static str,
    /// What is wrong in it.
    pub why: String,
}

/// Every theme, in file order, and the colours, sizes, motion and icons they share.
#[derive(Debug, Clone, PartialEq)]
pub struct Themes {
    /// The themes, in the order the theme button cycles them.
    pub list: Vec<Theme>,
    /// Shared colours.
    pub common: Common,
    /// Shared sizes.
    pub shape: Shape,
    /// Animations, full and reduced.
    pub motion: Motions,
    /// The icon each part shows.
    pub icons: Icons,
}

impl Themes {
    /// The shipped tokens, checked.
    ///
    /// # Errors
    /// A token file that does not parse or breaks a rule, named with the reason.
    pub fn shipped() -> Result<Self, ThemeError> {
        let at = |file| move |why| ThemeError { file, why };
        let (mut list, common) = theme::parse(THEMES).map_err(at(THEMES_FILE))?;
        let shape = shape::parse(SHAPE).map_err(at(SHAPE_FILE))?;
        let motion = motion::parse(MOTION).map_err(at(MOTION_FILE))?;
        let icons = icons::parse(ICONS).map_err(at(ICONS_FILE))?;
        ringmove::fits(&shape.ring, &motion.full.ring.amount).map_err(at(SHAPE_FILE))?;
        for p in list.iter_mut().flat_map(|t| [&mut t.light, &mut t.dark]) {
            p.fade(shape.dim.share);
        }
        Ok(Self {
            list,
            common,
            shape,
            motion,
            icons,
        })
    }

    /// The theme with id `id`, if there is one.
    #[must_use]
    pub fn find(&self, id: &str) -> Option<&Theme> {
        self.list.iter().find(|t| t.look.id == id)
    }
}

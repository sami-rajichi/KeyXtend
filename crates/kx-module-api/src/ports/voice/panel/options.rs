//! The Options page: switches, steppers and choices, each with the facts its value and dimmed ends need.

use crate::ports::Hundredths;

/// Where a stepper's or a choice's value stands, so minus or the left chevron dims at the lowest, plus or the right
/// chevron at the highest.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Ends {
    /// It is at its lowest.
    pub low: bool,
    /// It is at its highest.
    pub high: bool,
}

/// An option's value and how it changes.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OptionValue {
    /// A switch, on or off.
    Switch(bool),
    /// A whole number with minus and plus.
    Number {
        /// The value, in the unit its text names.
        value: u32,
        /// What 0 stands for now, as `threads`' real cores in Auto (8); `None` when 0 is just 0.
        auto: Option<u32>,
        /// Where it stands.
        ends: Ends,
    },
    /// A factor with minus and plus, such as `card_gain`'s 1.5×.
    Factor {
        /// The value.
        value: Hundredths,
        /// Where it stands.
        ends: Ends,
    },
    /// One of a few names, such as `place`'s, with chevrons to the one before or after.
    Choice {
        /// The value's name in the setting, which also names its text.
        name: &'static str,
        /// Where it stands.
        ends: Ends,
    },
}

/// One row of the Options page.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct OptionRow {
    /// The setting's key, which also names its text.
    pub key: &'static str,
    /// Its value.
    pub value: OptionValue,
    /// It acts now; a stepper whose switch is off shows dimmed.
    pub live: bool,
}

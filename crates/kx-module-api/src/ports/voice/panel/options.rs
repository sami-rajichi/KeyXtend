//! The Options page: switches, steppers and choices, each with the facts its value, its unit and dimmed ends need.

use crate::ports::Hundredths;
use serde::Deserialize;

listed! {
    /// How a whole number reads, which names its text; the module's options table gives each number one.
    #[derive(Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum OptionUnit ("Every unit.") {
        /// A bare count, as `threads`.
        Count,
        /// Whole seconds.
        Seconds,
        /// Milliseconds, read as seconds.
        MsAsSeconds,
        /// Seconds, read in minutes from one whole minute.
        SecondsOrMinutes,
        /// Minutes.
        Minutes,
        /// Milliseconds.
        Ms,
        /// Binary megabytes, read in gigabytes from one.
        Mb,
        /// History's texts.
        Texts,
        /// Words.
        Words,
    }
}

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
        /// The value, in its unit.
        value: u32,
        /// What 0 stands for now, as `threads`' real cores in Auto (8), and 0 until they are read; `None` when 0 is
        /// just 0.
        auto: Option<u32>,
        /// How it reads.
        unit: OptionUnit,
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

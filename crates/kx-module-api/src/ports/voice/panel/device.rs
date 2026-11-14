//! The Device page: Automatic, the processor with its cores, and each card with its memory, built-in or not, and its
//! check, whose verdict carries the numbers its mark and the note band show.

use crate::ports::{Memory, Verdict};

/// A card's check, on its Device row.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CardCheck {
    /// Not checked for the models in use.
    Unchecked,
    /// Warming up or checking, in the background.
    Running {
        /// The warm-up's progress, 0 to `MAX_PERCENT`.
        percent: u8,
    },
    /// Its check ended, passed or with the reason it is not used.
    Done(Verdict),
}

/// What a Device row shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeviceItem {
    /// Automatic: the processor until a card passes its check.
    Auto,
    /// The processor, as in 8 cores · 16 GB memory, 9.4 GB free.
    Cpu {
        /// Its real cores, which `threads` 0 means.
        cores: u16,
        /// Its memory now.
        memory: Memory,
    },
    /// A card, as in Graphics card · 8 GB, 7.1 GB free.
    Card {
        /// Its name as the system gives it.
        name: String,
        /// It is built-in graphics sharing the processor's memory, as in Built-in graphics · 4 GB shared.
        built_in: bool,
        /// Its memory now.
        memory: Memory,
        /// Its check.
        check: CardCheck,
    },
}

/// One row of the Device page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceRow {
    /// What it shows.
    pub item: DeviceItem,
    /// It is the device picked.
    pub used: bool,
    /// It is ticked.
    pub tick: bool,
}

/// A card's failed check for the note band, as in its name: Check failed. Its text differed by 3 words; 1 allowed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FailedCheck {
    /// The card's name.
    pub name: String,
    /// Why it failed.
    pub verdict: Verdict,
}

/// The Device page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceSheet {
    /// The shown page's rows.
    pub rows: Vec<DeviceRow>,
    /// The latest failed check, for the note band; `None` for no band.
    pub failed: Option<FailedCheck>,
}

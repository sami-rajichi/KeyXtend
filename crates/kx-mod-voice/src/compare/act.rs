//! What a Compare run takes and gives: the ready models, the plan, each model's result line and the actor's effects.

use kx_module_api::ports::{CardId, Date, Fingerprint, SpeechAsk};
use kx_module_api::{Redacted, Secret};

use crate::counts::Board;
use crate::flow::{Ready, RunCall};

/// One ready model in a run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entrant {
    /// Its shown name, or the user's own file name.
    pub name: Redacted<String>,
    /// Its model and device, as the language gets them.
    pub ready: Ready,
}

/// What a run compares: the language, the set text the ui showed, the ready models, the day and the device pick.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    /// The language, an ISO 639 code.
    pub lang: String,
    /// The set text; empty for a language with none, which gives no count.
    pub set: Redacted<String>,
    /// The ready models that know the language, in its order.
    pub entrants: Vec<Entrant>,
    /// The local day, kept with the numbers.
    pub day: Date,
    /// The device is Automatic.
    pub auto: bool,
}

/// One model's result while the panel shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    /// Its shown name.
    pub name: Redacted<String>,
    /// Its file's fingerprint, which the numbers are kept by.
    pub print: Fingerprint,
    /// Words wrong out of `words`; `None` with no set text.
    pub wrong: Option<u16>,
    /// The set text's words.
    pub words: u16,
    /// How long its text took, in ms.
    pub wait_ms: u32,
    /// The most memory its job took, in MB.
    pub mb: u32,
    /// The card that wrote it to the end; `None` for the processor, a card's fallback included.
    pub card: Option<CardId>,
    /// Its text, only with no set text; shown, never kept.
    pub text: Option<Secret>,
}

/// One effect for the actor, on Compare's own run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    /// Open Compare's run `take` as `ask` says.
    Open {
        /// The run, which its notes name.
        take: u32,
        /// The ask, which loads no model.
        ask: SpeechAsk,
    },
    /// Call run `take`.
    Run {
        /// The run.
        take: u32,
        /// The call.
        call: RunCall,
    },
    /// The sound level while the set text is read, for the bar.
    Level(u8),
    /// Keep these numbers with `Scores::put`, then save them.
    Save(Board),
    /// The card failed mid-run: drop its check of this file, as the flow's `Out::Fell`.
    Fell {
        /// The card.
        card: CardId,
        /// The model file's fingerprint.
        model: Fingerprint,
    },
}

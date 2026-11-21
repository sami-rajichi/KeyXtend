//! What a Compare run takes and gives: the models, the plan, each model's result line and the actor's effects.

use kx_module_api::ports::{CardId, Date, Fingerprint, ServiceName, SpeechAsk};
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

/// A ticked cloud model in a run, which gets the recording through its service.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Remote {
    /// Its shown name, or the id the user typed.
    pub name: Redacted<String>,
    /// Its service as the results name it.
    pub shown: ServiceName,
    /// The service's row id.
    pub service: String,
    /// The model's id, as a pick names it; redacted, as a typed Other id is the user's own.
    pub model: Redacted<String>,
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
    /// The ticked cloud models, which get the recording once it ends; empty for none.
    pub clouds: Vec<Remote>,
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
    /// Its file's fingerprint, which the numbers are kept by; `None` for a cloud model, whose numbers are not kept.
    pub print: Option<Fingerprint>,
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
    /// The cloud model that wrote it; `None` for a model on this PC.
    pub cloud: Option<Remote>,
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
    /// Send run `take`'s recording, once handed over, to each of `to` for language `lang`; model `at` of `to` tells
    /// `Panel::compare_net`.
    Send {
        /// The run.
        take: u32,
        /// The language, an ISO 639 code.
        lang: String,
        /// The ticked cloud models, in order.
        to: Vec<Remote>,
    },
    /// Drop run `take`'s uploads, which cancels their requests.
    Halt(u32),
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

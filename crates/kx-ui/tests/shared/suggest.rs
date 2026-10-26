//! Chips and lanes as the prediction module sends them, shared by the test binaries that draw or check the lane.

use kx_module_api::Redacted;
use kx_module_api::ports::{Chip, ChipKind, Lane, Suggested};

/// A chip of `kind` showing `text`.
pub fn chip(text: &str, kind: ChipKind, rtl: bool) -> Chip {
    Chip {
        text: Redacted::new(text.to_owned()),
        kind,
        rtl,
    }
}

/// Lane `round` showing `lane`, learning and prediction on, no list open.
pub fn lane(round: u32, lane: Lane) -> Suggested {
    Suggested {
        round,
        lane,
        forgetting: false,
        learn: true,
        on: true,
        list: None,
    }
}

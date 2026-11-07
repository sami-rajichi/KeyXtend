//! A Quick-fill fill (ADR-0038 decision 4): the keyboard stays the only typist, so each text part goes out by the route
//! a letter takes, each Tab as the board's Tab key, and the part after a Tab waits the gap the vault module set, but
//! never goes more than `fill_late_ms` after it or to another app (design "Filling"). Word prediction hears one break,
//! as after a paste, and never a value.

use super::checks::reached;
use super::{Io, Typing};
use crate::plan::text_route;
use kx_module_api::ports::{FillPart, FillText, FillType, InjectError, Piece};
use kx_module_api::{Mono, Redacted, Zeroizing};
use std::collections::VecDeque;
use std::fmt;
use std::time::Duration;

/// A fill on its way out: its round, the parts left, and when the next may go.
pub struct Fill {
    round: u32,
    /// The parts not sent yet, wiped when dropped.
    left: VecDeque<FillPart>,
    /// The wait after each Tab.
    gap: Duration,
    /// When the next part may go, after a Tab.
    due: Option<Mono>,
    /// The app in front as its first part went out; none when the system could not name it.
    app: Option<String>,
    /// Some of it reached the app.
    any: bool,
}

/// Shows the round and how many parts are left, never a part or the app.
impl fmt::Debug for Fill {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Fill")
            .field("round", &self.round)
            .field("left", &self.left.len())
            .finish_non_exhaustive()
    }
}

impl Fill {
    /// The round its answer names.
    #[must_use]
    pub fn round(&self) -> u32 {
        self.round
    }

    /// True when its next part may go at `now`.
    #[must_use]
    pub fn due(&self, now: Mono) -> bool {
        self.due.is_none_or(|at| now >= at)
    }

    /// Ends it unfinished: the parts left are wiped unsent.
    fn end(&mut self) -> Filled {
        self.left.clear();
        Filled::Done(false)
    }

    /// True when the app in front is the one, named, that its first part went to.
    fn same_app(&self, io: &Io<'_>) -> bool {
        self.app
            .as_ref()
            .is_some_and(|app| io.injector.target_app().as_ref() == Some(app))
    }
}

/// What a fill asks for next.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Filled {
    /// A Tab went out with parts left: the next goes at this point.
    Wait(Mono),
    /// The fill is done; true when every part went out.
    Done(bool),
}

impl Typing {
    /// The fill `ask` asks for, refused while a dead key waits, as the app may hold an accent that would join its first
    /// character, or when it has a Tab and the board names no Tab key. Then every latch lets go whatever the mode and a
    /// burst ends, as before a paste.
    pub fn fill(&mut self, ask: &FillType) -> Option<Fill> {
        let tabs = ask.parts.iter().any(|p| matches!(p, FillPart::Tab));
        if self.dead || (tabs && self.tab.is_none()) {
            return None;
        }
        self.burst = None;
        self.latches.clear();
        Some(Fill {
            round: ask.round,
            left: ask.parts.iter().cloned().collect(),
            gap: ask.tab_gap,
            due: None,
            app: None,
            any: false,
        })
    }

    /// True when `fill`'s next part is more than `fill_late_ms` past due at `now`.
    #[must_use]
    pub fn overdue(&self, fill: &Fill, now: Mono) -> bool {
        fill.due
            .is_some_and(|at| now.saturating_since(at) > self.late)
    }

    /// Sends `fill`'s parts in order once due at `now`, until a Tab goes out with parts left; each that reached the app
    /// counts as a key sent, so a picked word going out stops, and the first tells a break. A part the app did not take
    /// whole ends it, and so does one more than `fill_late_ms` past due, or one after a Tab while an app other than its
    /// first part's is in front.
    pub fn fill_on(&mut self, io: &Io<'_>, fill: &mut Fill, now: Mono) -> Filled {
        if let Some(at) = fill.due.filter(|&at| now < at) {
            return Filled::Wait(at);
        }
        if self.overdue(fill, now) {
            return fill.end();
        }
        if fill.due.is_none() {
            fill.app = io.injector.target_app();
        }
        while let Some(part) = fill.left.pop_front() {
            if fill.due.is_some() && !fill.same_app(io) {
                return fill.end();
            }
            let went = self.fill_part(io, &part);
            if reached(went) {
                self.sent = self.sent.wrapping_add(1);
                if !std::mem::replace(&mut fill.any, true) {
                    self.tell(Piece::Break);
                }
            }
            if went.is_err() {
                return fill.end();
            }
            if matches!(part, FillPart::Tab) && !fill.left.is_empty() {
                let at = now.saturating_add(fill.gap);
                fill.due = Some(at);
                return Filled::Wait(at);
            }
        }
        Filled::Done(true)
    }

    /// Sends one part: text by the route a letter takes now, a Tab as the board's Tab key alone.
    fn fill_part(&self, io: &Io<'_>, part: &FillPart) -> Result<(), InjectError> {
        match part {
            FillPart::Text(text) => self.fill_text(io, text),
            FillPart::Tab => self
                .tab
                .map_or(Err(InjectError::Unmappable), |key| io.injector.tap(key)),
        }
    }

    /// Types `text` by the route a letter takes now, through a copy wiped once sent.
    fn fill_text(&self, io: &Io<'_>, text: &FillText) -> Result<(), InjectError> {
        let by = text_route(&self.apps, || io.injector.target_app());
        let copy = Redacted::new(text.text().expose().as_str().to_owned());
        let went = io.injector.text(&copy, by);
        drop(Zeroizing::new(copy.into_inner()));
        went
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kx_module_api::Secret;

    /// A value that appears nowhere else.
    const SECRET: &str = "zebra-quartz";

    /// An app name that appears nowhere else.
    const APP: &str = "quill-ledger.exe";

    #[test]
    fn a_fill_shows_its_round_and_count_never_a_part_or_its_app() {
        let text = FillText::new(Secret::new(Zeroizing::new(SECRET.into()))).unwrap();
        let fill = Fill {
            round: 3,
            left: VecDeque::from([FillPart::Text(text), FillPart::Tab]),
            gap: Duration::ZERO,
            due: None,
            app: Some(APP.into()),
            any: false,
        };
        let shown = format!("{fill:?}");
        assert!(!shown.contains(SECRET), "{shown}");
        assert!(!shown.contains(APP), "{shown}");
        assert!(
            shown.contains("round: 3") && shown.contains("left: 2"),
            "{shown}"
        );
    }

    #[test]
    fn a_fill_is_due_at_once_then_from_its_point_on() {
        let mut fill = Fill {
            round: 1,
            left: VecDeque::new(),
            gap: Duration::ZERO,
            due: None,
            app: None,
            any: false,
        };
        assert!(fill.due(Mono::default()));
        fill.due = Some(Mono::from_us(10));
        assert!(!fill.due(Mono::from_us(9)));
        assert!(fill.due(Mono::from_us(10)) && fill.due(Mono::from_us(11)));
    }
}

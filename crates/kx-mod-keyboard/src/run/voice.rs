//! A dictated text (ADR-0042 decision 7): the keyboard stays the only typist, so the voice module's text erases with
//! the board's erase key, paced by `erase_gap_ms` as a picked word's Backspaces are, then types by the route a letter
//! takes, paced by the ask, in pieces under `text_cap`, keeping every latch. Word prediction hears one break, never the
//! text, and nothing of it is ever logged.

use super::checks::reached;
use super::{Io, Part, Typing};
use crate::plan::text_route;
use kx_module_api::ports::{Piece, TextAsk, TextDone};
use kx_module_api::{Redacted, Secret, Zeroizing, is_hidden};
use std::fmt;
use std::time::Duration;

/// Characters or Backspaces in one paced piece: each goes alone, as Notepad drops most of a sentence typed in one batch
/// (the spike's G22 typing check).
const PACED: usize = 1;

/// How a run of Backspaces or characters goes out: so many to a piece, with a wait after each piece but the last.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Pace {
    /// The most in one piece.
    step: usize,
    /// The wait after each piece.
    gap: Duration,
}

impl Pace {
    /// Paced by `gap`, each alone; with no gap, pieces of up to `cap` at once.
    fn new(gap: Duration, cap: u16) -> Self {
        let step = if gap.is_zero() {
            usize::from(cap).max(PACED)
        } else {
            PACED
        };
        Self { step, gap }
    }
}

/// A text on its way out: the Backspaces left, the text and how far it went, and what reached the app.
pub struct Text {
    round: u32,
    /// Backspaces still to send.
    erase: u32,
    /// The Backspaces' pace, from `erase_gap_ms`.
    wipe: Pace,
    /// What to type, wiped when dropped.
    body: Secret,
    /// The byte where the next piece starts.
    at: usize,
    /// The characters' pace, from the ask's `gap_ms`.
    pace: Pace,
    /// Characters that reached the app.
    count: u32,
    /// Some of it reached the app, so its break was told.
    any: bool,
    /// Nothing was refused or stopped.
    ok: bool,
    /// The keys sent so far: one more sent by anyone else stops it, as that key may have moved the caret.
    keys: u64,
}

/// Shows the round and the counts, never the text.
impl fmt::Debug for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Text")
            .field("round", &self.round)
            .field("erase", &self.erase)
            .field("count", &self.count)
            .finish_non_exhaustive()
    }
}

impl Text {
    /// Its answer: the round, the characters typed and whether all of it went.
    #[must_use]
    pub fn done(&self) -> TextDone {
        TextDone {
            round: self.round,
            count: self.count,
            ok: self.ok,
        }
    }

    /// True while a Backspace or a character is left to send.
    #[must_use]
    pub fn left(&self) -> bool {
        self.erase > 0 || self.at < self.body.expose().len()
    }

    /// Ends it unfinished, with nothing more sent.
    pub fn stop(&mut self) -> Part {
        self.erase = 0;
        self.at = self.body.expose().len();
        self.ok = false;
        Part::Done(self.any)
    }

    /// Takes the next piece of the text, as a copy the caller wipes, with its length in characters.
    fn next_piece(&mut self) -> (Redacted<String>, u32) {
        let rest = &self.body.expose()[self.at..];
        let end = rest
            .char_indices()
            .nth(self.pace.step)
            .map_or(rest.len(), |(i, _)| i);
        let piece = &rest[..end];
        let chars = u32::try_from(piece.chars().count()).unwrap_or(u32::MAX);
        let copy = Redacted::new(piece.to_owned());
        self.at += end;
        (copy, chars)
    }
}

impl Typing {
    /// The text `ask` asks for, when it may go: refused while a dead key waits, as the app may hold an accent that
    /// would join its first character, for a hidden mark (`is_hidden`), or to erase on a board with no erase key.
    #[must_use]
    pub fn text(&self, ask: &TextAsk) -> Option<Text> {
        let plain = !ask.text.expose().chars().any(is_hidden);
        let can = ask.erase == 0 || self.erase.is_some();
        let gap = Duration::from_millis(u64::from(ask.gap_ms));
        (!self.dead && plain && can).then(|| Text {
            round: ask.round,
            erase: ask.erase,
            wipe: Pace::new(self.gap, self.cap),
            body: ask.text.clone(),
            at: 0,
            pace: Pace::new(gap, self.cap),
            count: 0,
            any: false,
            ok: true,
            keys: self.sent,
        })
    }

    /// Sends `text`'s next piece, Backspaces first, then its pace's gap, or its end once nothing is left. A piece not
    /// taken whole, or a key sent by anyone else since the last piece, ends it unfinished; the latches never change.
    pub fn text_part(&mut self, io: &Io<'_>, text: &mut Text) -> Part {
        if self.sent != text.keys {
            return text.stop();
        }
        if !text.left() {
            return Part::Done(text.any);
        }
        let (pace, (any, whole)) = if text.erase > 0 {
            (text.wipe, self.erase_piece(io, text))
        } else {
            (text.pace, self.text_piece(io, text))
        };
        if any {
            text.keys = self.count_key();
            if !std::mem::replace(&mut text.any, true) {
                self.tell(Piece::Break);
            }
        }
        match (whole, text.left()) {
            (false, _) => text.stop(),
            (true, true) => Part::Gap(pace.gap),
            (true, false) => Part::Done(text.any),
        }
    }

    /// Taps one piece of Backspaces; whether any reached the app, and whether all did.
    fn erase_piece(&self, io: &Io<'_>, text: &mut Text) -> (bool, bool) {
        let Some(key) = self.erase else {
            return (false, false);
        };
        let step = u32::try_from(text.wipe.step).unwrap_or(u32::MAX);
        let mut any = false;
        for _ in 0..text.erase.min(step) {
            text.erase -= 1;
            let went = io.injector.tap(key);
            any |= reached(went);
            if went.is_err() {
                return (any, false);
            }
        }
        (any, true)
    }

    /// Types one piece of the text by the route a letter takes now, through a copy wiped once sent; a piece taken in
    /// part counts whole, as a key does. Whether any reached the app, and whether all did.
    fn text_piece(&self, io: &Io<'_>, text: &mut Text) -> (bool, bool) {
        let (copy, chars) = text.next_piece();
        let by = text_route(&self.apps, || io.injector.target_app());
        let went = io.injector.text(&copy, by);
        drop(Zeroizing::new(copy.into_inner()));
        let any = reached(went);
        if any {
            text.count = text.count.saturating_add(chars);
        }
        (any, went.is_ok())
    }
}

#[cfg(test)]
mod tests;

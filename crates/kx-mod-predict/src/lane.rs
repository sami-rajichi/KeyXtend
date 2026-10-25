//! The lane's chips and what a tap on each does, from what the tracker shows: word guesses first, then the email
//! chips, at most the lane's size.

use kx_module_api::Redacted;
use kx_module_api::ports::{Chip, ChipKind, TypingMode};

use crate::email::{self, EmailConfig, Emails};
use crate::menu::Menu;
use crate::pack::Pack;
use crate::score::{Guess, Learned, ScoreConfig, guesses};
use crate::track::{EmailAsk, Plan, Show};

/// What a tap on one chip does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Pick {
    /// Types a word by `plan`; a forget passes its `stored` spelling.
    Word {
        /// What to erase and type.
        plan: Plan,
        /// The spelling the pack or the learner stores.
        stored: Redacted<String>,
    },
    /// Types an email or a domain by its plan.
    Email(Plan),
    /// The list's chip: the UI answers it with `WordsAsk::More`.
    More,
    /// Brings back the word just forgotten.
    Undo,
}

/// A lane's chips and their picks, in the same order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Chips {
    /// What the lane shows.
    pub(crate) chips: Vec<Chip>,
    /// What a tap on each does.
    pub(crate) picks: Vec<Pick>,
}

/// What the chips are drawn from.
pub(crate) struct Sources<'a> {
    /// The language in front.
    pub(crate) iso: &'a str,
    /// Its pack, when it has one.
    pub(crate) seed: Option<&'a Pack<'a>>,
    /// The learned counts.
    pub(crate) learned: &'a dyn Learned,
    /// The latches.
    pub(crate) mode: TypingMode,
    /// The scorer's settings.
    pub(crate) score: &'a ScoreConfig,
    /// The known emails.
    pub(crate) emails: &'a Emails,
    /// The email chips' settings.
    pub(crate) email: &'a EmailConfig,
    /// The most chips.
    pub(crate) limit: usize,
}

impl Chips {
    /// The chips for `show`: the email chips take their room first, and word guesses fill the rest before them.
    pub(crate) fn of(show: &Show, src: &Sources<'_>) -> Self {
        let mut out = Self::default();
        let emails = show.emails.as_ref().map(|ask| (ask, email_chips(ask, src)));
        let taken = emails.as_ref().map_or(0, |(_, chips)| chips.len());
        if let Some(query) = &show.words {
            let room = src.limit.saturating_sub(taken);
            let found = guesses(
                src.iso,
                src.seed,
                src.learned,
                query,
                src.mode,
                room,
                src.score,
            );
            for guess in found {
                out.word(query.plan(guess.text.expose()), guess);
            }
        }
        if let Some((ask, chips)) = emails {
            out.emails(ask, chips);
        }
        out
    }

    /// Adds the email chips `ask` offered; More opens the list, the others type their plan.
    fn emails(&mut self, ask: &EmailAsk, chips: Vec<Chip>) {
        for chip in chips {
            let pick = match chip.kind {
                ChipKind::More => Some(Pick::More),
                _ => ask.plan(chip.text.expose()).map(Pick::Email),
            };
            self.push_opt(chip, pick);
        }
    }

    /// Puts an Undo chip, reading as `rtl`, at `at`, keeping at most `limit` chips.
    pub(crate) fn with_undo(&mut self, at: usize, rtl: bool, limit: usize) {
        let at = at.min(self.chips.len());
        let chip = Chip {
            text: Redacted::default(),
            kind: ChipKind::Undo,
            rtl,
        };
        self.chips.insert(at, chip);
        self.picks.insert(at, Pick::Undo);
        self.chips.truncate(limit);
        self.picks.truncate(limit);
    }

    /// Shows the menu's state: the Undo chip at its place, reading as `rtl`, and in forget mode only the word chips.
    pub(crate) fn with_menu(&mut self, menu: Menu, rtl: bool, limit: usize) {
        if let Some(at) = menu.undo {
            self.with_undo(at, rtl, limit);
        }
        if menu.forgetting {
            self.words_only();
        }
    }

    /// Keeps only the word chips, as forget mode offers only what can be forgotten.
    fn words_only(&mut self) {
        let pairs = std::mem::take(&mut self.chips)
            .into_iter()
            .zip(std::mem::take(&mut self.picks));
        (self.chips, self.picks) = pairs
            .filter(|(_, pick)| matches!(pick, Pick::Word { .. }))
            .unzip();
    }

    /// Adds the word chip of `guess`, when its plan counts.
    fn word(&mut self, plan: Option<Plan>, guess: Guess) {
        let pick = plan.map(|plan| Pick::Word {
            plan,
            stored: guess.stored,
        });
        let chip = Chip {
            text: guess.text,
            kind: ChipKind::Word,
            rtl: guess.rtl,
        };
        self.push_opt(chip, pick);
    }

    /// Adds `chip` with `pick`; a chip with no pick is left out.
    fn push_opt(&mut self, chip: Chip, pick: Option<Pick>) {
        if let Some(pick) = pick {
            self.chips.push(chip);
            self.picks.push(pick);
        }
    }
}

/// The email chips `ask` offers, at most the lane's size.
fn email_chips(ask: &EmailAsk, src: &Sources<'_>) -> Vec<Chip> {
    match ask {
        EmailAsk::Domains(partial) => email::domains(partial.expose(), src.email, src.limit),
        EmailAsk::All(_) => src.emails.all(src.limit),
        EmailAsk::Start(tail) => src.emails.starting(tail.expose(), src.email, src.limit),
    }
}

#[cfg(test)]
mod tests;

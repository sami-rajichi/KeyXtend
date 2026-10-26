//! The lane's chips and what a tap on each does, from what the tracker shows: word guesses first, then the email
//! chips, at most the lane's size.

use kx_module_api::Redacted;
use kx_module_api::ports::{Chip, ChipKind, TypeWord, TypingMode};

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
    /// Types a known email by its plan; a forget removes the email it types.
    Email(Plan),
    /// Types a domain by its plan.
    Domain(Plan),
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
    /// The code of the words in front, which its pack carries.
    pub(crate) iso: &'a str,
    /// Its pack, when it has one.
    pub(crate) seed: Option<&'a Pack<'a>>,
    /// The learned counts.
    pub(crate) learned: &'a dyn Learned,
    /// The latches and the keyboard's text cap.
    pub(crate) mode: TypingMode,
    /// The scorer's settings.
    pub(crate) score: &'a ScoreConfig,
    /// The known emails.
    pub(crate) emails: &'a Emails,
    /// The email chips' settings.
    pub(crate) email: &'a EmailConfig,
    /// The most chips.
    pub(crate) limit: usize,
    /// The box confirms what a pick erases, so a chip may change the typed word.
    pub(crate) confirms: bool,
    /// Forget mode is on, so the email chips leave out More.
    pub(crate) forgetting: bool,
}

impl Sources<'_> {
    /// True when a tap can send `plan`, as `sendable` says for this lane's mode and box.
    fn can_send(&self, plan: &Plan) -> bool {
        sendable(plan, self.mode, self.confirms)
    }
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
                let plan = query.plan(guess.text.expose());
                out.word(plan.filter(|p| src.can_send(p)), guess);
            }
        }
        if let Some((ask, chips)) = emails {
            out.emails(ask, chips);
        }
        out
    }

    /// Adds the email chips `ask` offered, each one a tap can send; More opens the list, the others type their plan.
    fn emails(&mut self, ask: &EmailAsk, chips: Vec<Chip>) {
        for chip in chips {
            let plan = || ask.plan(chip.text.expose());
            let pick = match chip.kind {
                ChipKind::More => Some(Pick::More),
                ChipKind::Domain => plan().map(Pick::Domain),
                _ => plan().map(Pick::Email),
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

    /// Shows the menu's state: in forget mode only what can be forgotten, else the Undo chip at its place, reading as
    /// `rtl`.
    pub(crate) fn with_menu(&mut self, menu: Menu, rtl: bool, limit: usize) {
        if menu.forgetting {
            self.forgettable();
        } else if let Some(at) = menu.undo {
            self.with_undo(at, rtl, limit);
        }
    }

    /// Keeps only the word and known-email chips, as forget mode offers only what can be forgotten.
    fn forgettable(&mut self) {
        let pairs = std::mem::take(&mut self.chips)
            .into_iter()
            .zip(std::mem::take(&mut self.picks));
        (self.chips, self.picks) = pairs
            .filter(|(_, pick)| matches!(pick, Pick::Word { .. } | Pick::Email(_)))
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

/// True when `plan` fits the keyboard's text cap in `mode`, or the keyboard told none yet.
pub(crate) fn fits(plan: &Plan, mode: TypingMode) -> bool {
    mode.cap
        .is_none_or(|cap| TypeWord::fits(plan.erase, plan.text.expose(), cap))
}

/// True when a tap can send `plan`: it fits the text cap in `mode`, and a box that cannot confirm an erase
/// (`confirms` false) still gets the rest of the chip after exactly what was typed. No chip is a dead tap.
pub(crate) fn sendable(plan: &Plan, mode: TypingMode, confirms: bool) -> bool {
    fits(plan, mode) && (confirms || plan.rest().is_some())
}

/// The known emails the `@@` of `ask` can send in `mode`, in a box that `confirms` erases or not.
pub(crate) fn listed(emails: &Emails, ask: &EmailAsk, mode: TypingMode, confirms: bool) -> Emails {
    emails.kept(|e| ask.plan(e).is_some_and(|p| sendable(&p, mode, confirms)))
}

/// The email chips `ask` offers that a tap can send, at most the lane's size; forget mode lists no More.
fn email_chips(ask: &EmailAsk, src: &Sources<'_>) -> Vec<Chip> {
    let found = match ask {
        EmailAsk::All(_) => {
            let sent = listed(src.emails, ask, src.mode, src.confirms);
            return if src.forgetting {
                sent.first(src.limit)
            } else {
                sent.all(src.limit)
            };
        }
        EmailAsk::Domains(partial) => email::domains(partial.expose(), src.email, usize::MAX),
        EmailAsk::Start(tail) => src.emails.starting(tail.expose(), src.email, usize::MAX),
    };
    let can = |c: &Chip| ask.plan(c.text.expose()).is_some_and(|p| src.can_send(&p));
    found.into_iter().filter(can).take(src.limit).collect()
}

#[cfg(test)]
mod tests;

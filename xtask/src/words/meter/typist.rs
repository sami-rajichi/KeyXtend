//! The simulated user (spec §6 "Target"): types each sentence through the module's own tracker as the keyboard
//! would, and before each letter picks a chip when one holds the word. A capital costs Shift on both sides.

use kx_mod_predict::learn::Learner;
use kx_mod_predict::score::{Guess, Learned, NoLearned, ScoreConfig, guesses, knows};
use kx_mod_predict::split::{is_word_char, normalise, spans};
use kx_mod_predict::track::{Plan, Query, Told, Tracker};
use kx_mod_predict::{Pack, PredictSettings};
use kx_module_api::Redacted;
use kx_module_api::ports::{Piece, TypingMode};
use kx_module_api::text::apostrophe;

use super::tally::Tally;
use super::{DAY, new_learner, timed_us};
use crate::words::WordsError;

/// Types sentences in one language and counts the presses.
pub(crate) struct Typist<'p> {
    /// The language code.
    iso: String,
    /// The language's pack.
    seed: &'p Pack<'p>,
    /// The learner, in the run that learns.
    learner: Option<Learner>,
    /// The module's tracker.
    tracker: Tracker,
    /// The latches, as the keyboard tells them.
    mode: TypingMode,
    /// The scorer's settings.
    score: ScoreConfig,
    /// The chips in the lane.
    chips: usize,
    /// The packs' apostrophe, which every form in a sentence becomes.
    apostrophe: char,
    /// The counts so far.
    tally: Tally,
}

impl<'p> Typist<'p> {
    /// A typist for `iso` over `seed` with the module's `settings`, and a learner when `learn`.
    pub(crate) fn new(
        iso: &str,
        seed: &'p Pack<'p>,
        settings: &PredictSettings,
        learn: bool,
    ) -> Result<Self, WordsError> {
        let learner = learn.then(|| learner(iso, seed, settings)).transpose()?;
        let apostrophe = apostrophe()
            .ok_or_else(|| WordsError::Meter("the fold table names no apostrophe".to_owned()))?;
        Ok(Self {
            iso: iso.to_owned(),
            seed,
            learner,
            tracker: Tracker::new(settings.track_config()),
            mode: TypingMode::default(),
            score: settings.score_config(),
            chips: usize::from(settings.chips),
            apostrophe,
            tally: Tally::default(),
        })
    }

    /// Types `line` after an Enter, which ends the last word and empties the context; it counts on neither side.
    pub(crate) fn sentence(&mut self, line: &str) {
        let text = normalise(line, self.apostrophe);
        let chars: Vec<char> = text.chars().collect();
        let capitals = chars.iter().filter(|c| c.is_uppercase()).count();
        self.feed(&Piece::Advance);
        self.tally.sentences += 1;
        self.tally.words += count(spans(&text, self.apostrophe).len());
        self.tally.without += count(chars.len() + capitals);
        let mut at = 0;
        while at < chars.len() {
            at = self.step(&chars, at);
        }
    }

    /// The counts.
    pub(crate) fn into_tally(self) -> Tally {
        self.tally
    }

    /// Types what comes at `at` of `text`: a chip when one holds the word, else the character, after Shift for a
    /// capital. Gives where the next step starts.
    fn step(&mut self, text: &[char], at: usize) -> usize {
        let Some(&c) = text.get(at) else {
            return text.len();
        };
        if !c.is_whitespace() {
            if let Some(next) = self.pick(text, at) {
                return next;
            }
            if c.is_uppercase() {
                self.press();
                self.mode.shift = true;
                if let Some(next) = self.pick(text, at) {
                    return next;
                }
            }
        }
        self.press();
        self.feed(&Piece::Text(Redacted::new(c.to_string())));
        self.mode = TypingMode::default();
        at + 1
    }

    /// Asks for the chips the tracker allows and picks the first that types the word at `at`: one press, then the
    /// keyboard erases and types only what differs from the open word. Gives where the next step starts.
    fn pick(&mut self, text: &[char], at: usize) -> Option<usize> {
        let seed = self.seed;
        let query = self.tracker.show(&|w: &str| knows(Some(seed), w)).words?;
        let found = self.guesses(&query);
        let (plan, next) = found.iter().find_map(|guess| {
            let plan = query.plan(guess.text.expose())?;
            let next = landing(text, at, &plan, guess.text.expose(), self.apostrophe)?;
            Some((plan, next))
        })?;
        self.press();
        for _ in 0..plan.erase {
            self.feed(&Piece::Erase);
        }
        self.feed(&Piece::Text(plan.text));
        self.mode = TypingMode::default();
        Some(next)
    }

    /// The scorer's chips for `query`, timed.
    fn guesses(&mut self, query: &Query) -> Vec<Guess> {
        let view = self.learner.as_ref().map(|l| l.view(Some(self.seed)));
        let learned: &dyn Learned = view.as_ref().map_or(&NoLearned, |v| v);
        let (seed, mode) = (Some(self.seed), self.mode);
        let (found, us) = timed_us(|| {
            let (iso, score) = (&self.iso, &self.score);
            guesses(iso, seed, learned, query, mode, self.chips, score)
        });
        self.tally.times.push(us);
        found
    }

    /// Hands `piece` to the tracker, and learns each word it finishes in the run that learns.
    fn feed(&mut self, piece: &Piece) {
        let seed = self.seed;
        let told = self.tracker.typed(piece, &|w: &str| knows(Some(seed), w));
        let Some(learner) = self.learner.as_mut() else {
            return;
        };
        for item in told {
            if let Told::Learn { word, context } = item {
                let context: Vec<&str> = context.expose().iter().map(String::as_str).collect();
                learner.learn(Some(seed), word.expose(), &context, DAY);
            }
        }
    }

    /// Counts one press with chips.
    fn press(&mut self) {
        self.tally.with += 1;
    }
}

/// Where the user stands after picking `guess`, typed by `plan`, at `at` of `text`, when the chip types the word
/// there: it replaces the open word, and its gap types the space that follows, or stays before marks that end the
/// token (spec §6). `None` when the chip is another word, only starts it, would part it from a joined word
/// (`vas-tu`), or moves nothing forward.
fn landing(text: &[char], at: usize, plan: &Plan, guess: &str, apostrophe: char) -> Option<usize> {
    let start = at.checked_sub(plan.gone.expose().chars().count())?;
    let end = start + guess.chars().count();
    if !text.get(start..end)?.iter().copied().eq(guess.chars()) {
        return None;
    }
    let chip = plan.chip();
    let gap = chip.expose().strip_prefix(guess).unwrap_or_default();
    let (gapped, rest) = (!gap.is_empty(), text.get(end..).unwrap_or_default());
    let mut token = rest.iter().take_while(|c| !c.is_whitespace());
    let next = match rest.first() {
        Some(c) if gapped && c.is_whitespace() => end + 1,
        Some(&c) if gapped && (c == apostrophe || token.any(|&t| is_word_char(t))) => return None,
        _ => end,
    };
    (next > at).then_some(next)
}

/// A learner for `iso` reconciled with `seed`, as the module loads a new one.
fn learner(iso: &str, seed: &Pack<'_>, settings: &PredictSettings) -> Result<Learner, WordsError> {
    let mut learner = new_learner(iso, settings.learn_config())?;
    learner.reconcile(Some(seed));
    Ok(learner)
}

/// `n` as a count.
fn count(n: usize) -> u64 {
    u64::try_from(n).unwrap_or(u64::MAX)
}

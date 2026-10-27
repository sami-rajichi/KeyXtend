//! What the lane may offer for the tracker's token, and the plan a pick sends: erase what the chip replaces past their
//! exact common start, then type the rest of the chip, with its script's gap after a word only.

use kx_module_api::Redacted;
use kx_module_api::ports::TypeWord;

use super::token::{Token, apostrophe, learnable, parts};
use super::{TrackConfig, push_capped};
use crate::email;
use crate::fold::is_apostrophe;
use crate::script::{self, Script};
use crate::split::normalise;

/// What the lane may offer now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Show {
    /// The word guesses to ask the scorer for; `None` for a token that gets none.
    pub words: Option<Query>,
    /// The email chips to offer, if any.
    pub emails: Option<EmailAsk>,
}

/// The word guesses to ask for: every way to read the open word.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    /// The open word as typed, which a word chip replaces; empty for next-word guesses.
    pub word: Redacted<String>,
    /// The readings, the whole word first; a guess for a reading is its lead, then the guess.
    pub readings: Vec<Reading>,
}

/// One way to read the open word: its elisions kept, and the part to complete after them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reading {
    /// The words before the part to complete, oldest first.
    pub context: Redacted<Vec<String>>,
    /// The elisions kept as typed (`l'`), which every guess of this reading starts with.
    pub lead: Redacted<String>,
    /// The part to complete, as typed; empty for next-word guesses.
    pub prefix: Redacted<String>,
}

/// The email chips to offer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmailAsk {
    /// The domains completing the text after the one `@`, which a chip replaces.
    Domains(Redacted<String>),
    /// Every known email, replacing the typed `@@`.
    All(Redacted<String>),
    /// The known emails that start with the token's email-like end, which a chip replaces.
    Start(Redacted<String>),
}

/// What a pick sends: erase the last `erase` characters of `gone`, then type `text`, the rest of the chip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// How many characters to erase first: those of `gone` past its exact common start with the chip.
    pub erase: u16,
    /// The whole text the chip replaces, so the module can check the box still ends with it.
    pub gone: Redacted<String>,
    /// What to type then: the chip past that common start.
    pub text: Redacted<String>,
}

impl Plan {
    /// The plan a box that cannot confirm an erase may get: this one when it erases nothing, else none.
    #[must_use]
    pub fn rest(&self) -> Option<Self> {
        (self.erase == 0).then(|| self.clone())
    }

    /// The chip's whole text: the start of `gone` it keeps, then `text`.
    #[must_use]
    pub fn chip(&self) -> Redacted<String> {
        let gone = self.gone.expose();
        let keep = gone.chars().count().saturating_sub(usize::from(self.erase));
        let mut whole: String = gone.chars().take(keep).collect();
        whole.push_str(self.text.expose());
        Redacted::new(whole)
    }
}

impl Query {
    /// The plan for the word chip `guess`: it replaces the open word and adds its script's gap, except after an
    /// elision (`l'`), which joins the next word.
    #[must_use]
    pub fn plan(&self, guess: &str) -> Option<Plan> {
        plan(self.word.expose(), &format!("{guess}{}", after(guess)))
    }

    /// True when picking `guess` changes the text: it differs from the open word, apostrophe forms aside, or adds a
    /// gap.
    #[must_use]
    pub fn changes(&self, guess: &str) -> bool {
        let ap = apostrophe();
        normalise(guess, ap) != normalise(self.word.expose(), ap) || !after(guess).is_empty()
    }

    /// `guess` with every apostrophe in the form the open word was typed with, when it holds one.
    #[must_use]
    pub fn styled(&self, guess: &str) -> String {
        let typed = self.word.expose().chars().find(|&c| is_apostrophe(c));
        let swap = |c: char| typed.filter(|_| is_apostrophe(c)).unwrap_or(c);
        guess.chars().map(swap).collect()
    }
}

/// What a word chip types after `guess`: its script's gap, but nothing after an elision, which ends with an apostrophe.
fn after(guess: &str) -> &'static str {
    if guess.chars().next_back().is_some_and(is_apostrophe) {
        return "";
    }
    script::of(guess).map_or("", Script::gap)
}

impl EmailAsk {
    /// The plan for the email or domain chip `text`: it replaces what this asks about and adds no space.
    #[must_use]
    pub fn plan(&self, text: &str) -> Option<Plan> {
        let (Self::Domains(gone) | Self::All(gone) | Self::Start(gone)) = self;
        plan(gone.expose(), text)
    }
}

/// Replace `gone` by `text`, keeping their exact common start, compared by `char` with case and marks; `None` when
/// the rest of `gone` is too long to count, or the plan would change nothing.
fn plan(gone: &str, text: &str) -> Option<Plan> {
    let kept: usize = gone
        .chars()
        .zip(text.chars())
        .take_while(|(a, b)| a == b)
        .map(|(c, _)| c.len_utf8())
        .sum();
    let (erased, typed) = (gone.get(kept..)?, text.get(kept..)?);
    let erase = TypeWord::erase_of(erased)?;
    (erase > 0 || !typed.is_empty()).then(|| Plan {
        erase,
        gone: Redacted::new(gone.to_owned()),
        text: Redacted::new(typed.to_owned()),
    })
}

/// What to offer for `token` after the words `before`.
pub(super) fn show(
    token: &Token,
    before: &[String],
    cfg: TrackConfig,
    knows: &dyn Fn(&str) -> bool,
) -> Show {
    let (words, emails) = offers(token, before, cfg, knows);
    Show { words, emails }
}

/// The word guesses and email chips for `token`: every known email after a typed `@@`, domains after one `@`, else
/// word guesses and the known emails its email-like end starts.
fn offers(
    token: &Token,
    before: &[String],
    cfg: TrackConfig,
    knows: &dyn Fn(&str) -> bool,
) -> (Option<Query>, Option<EmailAsk>) {
    let at = email::at();
    let run = at.zip(email::at_run());
    if let Some(typed) = run.and_then(|(a, n)| token.typed_run(a, n)) {
        return (None, Some(EmailAsk::All(Redacted::new(typed.to_owned()))));
    }
    let tail = email::tail(token.text());
    if token.has_at() {
        return (None, domains(tail));
    }
    let start = (!tail.is_empty()).then(|| EmailAsk::Start(Redacted::new(tail.to_owned())));
    (words(token, before, cfg, knows), start)
}

/// The domain ask for an email-like `tail` with one at sign after a local part.
fn domains(tail: &str) -> Option<EmailAsk> {
    let at = email::at()?;
    let (local, partial) = tail.split_once(at)?;
    let one = !local.is_empty() && !partial.contains(at);
    one.then(|| EmailAsk::Domains(Redacted::new(partial.to_owned())))
}

/// The word guesses for `token`: next-word guesses when it is empty, none after a mark or for an unlearnable word.
fn words(
    token: &Token,
    before: &[String],
    cfg: TrackConfig,
    knows: &dyn Fn(&str) -> bool,
) -> Option<Query> {
    let normal = token.normal();
    let found = parts(&normal);
    if !normal.is_empty() && !learnable(found.word, cfg.max_word) {
        return None;
    }
    let base = if found.marked { &[][..] } else { before };
    let typed = token.last(found.word.chars().count());
    let readings = readings(found.word, typed, base, cfg.context_words, knows);
    let word = Redacted::new(typed.to_owned());
    Some(Query { word, readings })
}

/// The readings of the open `word` (normalised) typed as `typed`: whole, then after each elision `knows` holds,
/// until an apostrophe that is not one.
fn readings(
    word: &str,
    typed: &str,
    base: &[String],
    cap: usize,
    knows: &dyn Fn(&str) -> bool,
) -> Vec<Reading> {
    let apostrophe = apostrophe();
    let mut context = base.to_vec();
    let (mut out, mut lead, mut rest) = (Vec::new(), 0, word);
    loop {
        let at = typed
            .char_indices()
            .nth(lead)
            .map_or(typed.len(), |(at, _)| at);
        let (kept, prefix) = typed.split_at(at);
        out.push(Reading {
            context: Redacted::new(context.clone()),
            lead: Redacted::new(kept.to_owned()),
            prefix: Redacted::new(prefix.to_owned()),
        });
        let Some(found) = rest.find(apostrophe) else {
            return out;
        };
        let (elided, tail) = rest.split_at(found + apostrophe.len_utf8());
        if !knows(elided) {
            return out;
        }
        push_capped(&mut context, elided.to_owned(), cap);
        lead += elided.chars().count();
        rest = tail;
    }
}

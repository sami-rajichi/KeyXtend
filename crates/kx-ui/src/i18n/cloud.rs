//! Voice typing's cloud words (ADR-0047): a card's fact filled with its row's values, money in its currency, and each
//! reason a cloud note or a Compare row gives. A name from a service's own pages stays a part of its own, so the UI lays
//! it out apart in the UI's order, as ADR-0022 asks in place of bidi marks.

use super::I18n;
use super::compose::{clean, form};
use super::keys::vars::{HTTPS, LANG};
use super::keys::{
    VOICE_HOURS, VOICE_MINUTES, VOICE_MONEY, VOICE_MONEY_ANY, VOICE_NOTE_CLOUD, VOICE_NOTE_RENEWS,
    cloud_why_id, fact_key,
};
use fluent_bundle::{FluentArgs, FluentValue};
use kx_module_api::ports::{CloudWhy, Fact, FactValue, Hundredths, LINK_SCHEME, VoiceLang};
use serde::Serialize;
use std::time::Duration;

/// What a counted value's plural form takes after its name, as in `$days-form`.
const FORM_END: &str = "-form";
/// The money texts' variables.
const AMOUNT_VAR: &str = "amount";
const CODE_VAR: &str = "code";
/// The money texts' case, `$case`: alone, or after a preposition such as Arabic's "من", where a dual is genitive.
const CASE_VAR: &str = "case";
const ALONE: &str = "alone";
const AFTER: &str = "after";
/// What a money value's form after a preposition takes after its name, as in `$low-after`.
const AFTER_END: &str = "-after";
/// A renewal's variable.
const IN_VAR: &str = "in";
/// What marks a name from a service's pages while its text is filled: these around the value's name.
const OPEN: char = '\u{1}';
const SHUT: char = '\u{2}';
/// Money's decimals, shown only when it has cents.
const CENTS: u32 = 2;
/// CLDR's plural form for an Arabic fraction, by name, set for every language though only Arabic's money selects on it:
/// `intl_pluralrules` 7.0.2 reads a fraction's few and many by its whole part, so 7.20 would take the plural.
const FRACTION_FORM: &str = "other";
/// Seconds in a minute, and minutes in an hour: a renewal reads in minutes below an hour.
const SECS_PER_MIN: u64 = 60;
const MIN_PER_HOUR: u64 = 60;

/// A sentence in parts: `own` parts are names from a service's pages, laid out apart; `text` is the whole.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Said {
    /// The whole sentence, as readers hear it.
    pub text: String,
    /// Its parts in reading order.
    pub parts: Vec<SaidPart>,
}

/// A part of a `Said`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SaidPart {
    /// Its words.
    pub text: String,
    /// It is a name from a service's own pages, kept apart and left to right.
    pub own: bool,
}

impl I18n {
    /// Fact `fact` of `group` as its text says it, filled with its values: counts with their plural form, numbers and
    /// money with the language's marks, days as dates, and names from the service's pages as parts of their own.
    #[must_use]
    pub fn voice_fact(&self, group: &str, fact: &Fact) -> Said {
        let mut values = FluentArgs::new();
        let mut names = Vec::new();
        for arg in &fact.args {
            let name = arg.name.clone();
            match &arg.value {
                FactValue::Count(n) => {
                    let wide = usize::try_from(*n).unwrap_or(usize::MAX);
                    values.set(format!("{name}{FORM_END}"), form(wide));
                    values.set(name, self.grouped(u128::from(*n)));
                }
                FactValue::Number(h) => values.set(name, self.hundredths(*h, false)),
                FactValue::Money { amount, currency } => {
                    let after = self.money(*amount, currency, AFTER);
                    values.set(format!("{name}{AFTER_END}"), after);
                    values.set(name, self.money(*amount, currency, ALONE));
                }
                FactValue::Day(d) => values.set(name, self.voice_date(*d)),
                FactValue::Text(t) => {
                    values.set(name.clone(), format!("{OPEN}{name}{SHUT}"));
                    names.push((name, clean(t)));
                }
            }
        }
        let filled = self.format(&fact_key(group, &fact.id), Some(&values));
        said(&filled, &names)
    }

    /// `amount` hundredths of `currency` in this language's way, cents only when it has some; `$amount-form` gives the
    /// plural form its noun takes, as Arabic's dollars agree with their number.
    #[must_use]
    pub fn voice_money(&self, amount: Hundredths, currency: &str) -> String {
        self.money(amount, currency, ALONE)
    }

    /// `voice_money` in `case`, which `$case` names: alone, or after a preposition, where Arabic's dual is genitive.
    fn money(&self, amount: Hundredths, currency: &str, case: &'static str) -> String {
        let cents = !amount.0.is_multiple_of(Hundredths::PER_ONE);
        let shown = self.hundredths(amount, cents);
        let code = clean(currency);
        let key = format!("{VOICE_MONEY}{}", code.to_lowercase());
        let mut values = FluentArgs::new();
        values.set(AMOUNT_VAR, shown);
        values.set(format!("{AMOUNT_VAR}{FORM_END}"), money_form(amount, cents));
        values.set(CASE_VAR, case);
        if self.has(&key) {
            return self.format(&key, Some(&values));
        }
        values.set(CODE_VAR, code);
        self.format(VOICE_MONEY_ANY, Some(&values))
    }

    /// Text `key` with `$https` set to the scheme every typed address starts with.
    #[must_use]
    pub fn voice_https(&self, key: &str) -> String {
        self.with(key, [(HTTPS, LINK_SCHEME.to_owned())])
    }

    /// The cloud note's line for `why`, naming the language or the renewal where it has one.
    #[must_use]
    pub fn voice_cloud(&self, why: &CloudWhy) -> String {
        match why {
            CloudWhy::Allowance { renews_in: Some(d) } => {
                self.with(VOICE_NOTE_RENEWS, [(IN_VAR, self.renewal(*d))])
            }
            CloudWhy::Language(lang) => self.cloud_line(cloud_why_id(why), lang),
            _ => self.text(&format!("{VOICE_NOTE_CLOUD}{}", cloud_why_id(why))),
        }
    }

    /// The cloud line `id`, with `$lang` set to `lang`'s name for the lines that take it.
    fn cloud_line(&self, id: &str, lang: &VoiceLang) -> String {
        let key = format!("{VOICE_NOTE_CLOUD}{id}");
        self.with(&key, [(LANG, self.voice_lang(lang))])
    }

    /// How long until an allowance renews: whole minutes below an hour, else whole hours, each rounded up.
    fn renewal(&self, d: Duration) -> String {
        let minutes = d.as_secs().div_ceil(SECS_PER_MIN).max(1);
        let (key, n) = if minutes < MIN_PER_HOUR {
            (VOICE_MINUTES, minutes)
        } else {
            (VOICE_HOURS, minutes.div_ceil(MIN_PER_HOUR))
        };
        self.counted(key, usize::try_from(n).unwrap_or(usize::MAX))
    }

    /// Hundredths `h` with the language's decimal mark; `cents` keeps both decimals, else trailing zeros drop.
    fn hundredths(&self, h: Hundredths, cents: bool) -> String {
        let per = u128::from(Hundredths::PER_ONE);
        self.places(u128::from(h.0), per, CENTS, !cents)
    }
}

/// The plural form `amount` takes: its whole number's, or CLDR's form for a fraction when it has `cents`.
fn money_form(amount: Hundredths, cents: bool) -> FluentValue<'static> {
    if cents {
        return FluentValue::from(FRACTION_FORM);
    }
    let whole = amount.0 / Hundredths::PER_ONE;
    form(usize::try_from(whole).unwrap_or(usize::MAX))
}

/// `filled` cut at each marked name of `names` into parts, the names in their own; the whole joined again.
fn said(filled: &str, names: &[(String, String)]) -> Said {
    let mut parts = Vec::new();
    let mut rest = filled;
    while let Some(open) = rest.find(OPEN) {
        let after = &rest[open + OPEN.len_utf8()..];
        let Some(shut) = after.find(SHUT) else { break };
        let name = &after[..shut];
        push(&mut parts, &rest[..open], false);
        let own = names
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, t)| t.as_str());
        push(&mut parts, own.unwrap_or_default(), true);
        rest = &after[shut + SHUT.len_utf8()..];
    }
    push(&mut parts, rest, false);
    let text = parts.iter().map(|p| p.text.as_str()).collect();
    Said { text, parts }
}

/// Adds `text` as a part, unless it is empty.
fn push(parts: &mut Vec<SaidPart>, text: &str, own: bool) {
    if !text.is_empty() {
        parts.push(SaidPart {
            text: text.to_owned(),
            own,
        });
    }
}

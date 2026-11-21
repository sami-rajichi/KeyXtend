//! The Options page as QML reads it: each row's label from its `[voice]` key, its value with its unit, and its switch,
//! its minus and plus with the dimmed ends (ADR-0008: no slider), or its two chips, whose hint takes the next row.

use super::panel::{Cx, Page, Sheet};
use super::sheets::tab_id;
use crate::I18n;
use crate::i18n::keys::vars::NAME;
use crate::i18n::keys::{
    VOICE_APPLIES, VOICE_AUTO, VOICE_AUTO_N, VOICE_FOR_ALL, VOICE_MINUS, VOICE_MINUTES, VOICE_MS,
    VOICE_OFF, VOICE_ON, VOICE_OPTION, VOICE_PLUS, VOICE_SECONDS, VOICE_TEXTS, VOICE_TIMES,
    VOICE_WORD, VOICE_WORDS,
};
use kx_module_api::MS_PER_S;
use kx_module_api::ports::{Ends, OptionRow, OptionUnit, OptionValue, S_PER_MIN, VoiceTab};
use serde::Serialize;

/// A row's kind: a switch, a number with minus and plus, a choice with chevrons, two chips, or a hint line; a hint's
/// text key ends in its kind.
const SWITCH: &str = "switch";
const NUMBER: &str = "number";
const CHOICE: &str = "choice";
const PAIR: &str = "pair";
const HINT: &str = "hint";
/// What a setting key writes for an underscore, and what a choice's text puts before its name.
const KEY_SEPARATOR: char = '_';
const TEXT_SEPARATOR: &str = "-";

/// An option's row; `steps` is set for a number or a choice, `chips` for a pair.
#[derive(Serialize)]
pub(super) struct OptionView {
    key: &'static str,
    label: String,
    kind: &'static str,
    value: String,
    on: bool,
    live: bool,
    steps: Option<Steps>,
    chips: Option<Vec<Chip>>,
}

/// One of a pair's chips: its words, and whether it is the one chosen.
#[derive(Serialize)]
struct Chip {
    label: String,
    on: bool,
}

/// A stepper's ends and the names readers hear for its minus and plus.
#[derive(Serialize)]
struct Steps {
    low: bool,
    high: bool,
    minus: String,
    plus: String,
}

/// The Options page: the shown rows, and the footer's line that changes apply at once.
pub(super) fn page(cx: &Cx<'_>, rows: &[OptionRow]) -> Page {
    let t = cx.t;
    let sheet = Sheet::Options {
        rows: rows.iter().map(|r| row(t, r)).collect(),
    };
    let tab = VoiceTab::Options;
    let mut page = Page::new(tab_id(tab), cx.tab(tab), t.text(VOICE_FOR_ALL), sheet);
    page.line = Some(t.text(VOICE_APPLIES));
    page
}

/// Option `r` in `t`'s words; a hint row reads its text, a pair its two chips.
fn row(t: &I18n, r: &OptionRow) -> OptionView {
    let key = r.key.replace(KEY_SEPARATOR, TEXT_SEPARATOR);
    let word = |name: &str| t.text(&format!("{VOICE_WORD}{key}{TEXT_SEPARATOR}{name}"));
    let label = match r.value {
        OptionValue::Hint => word(HINT),
        _ => t.text(&format!("{VOICE_OPTION}{key}")),
    };
    let (kind, value, ends) = shown(t, r.value, &word);
    let chips = match r.value {
        OptionValue::Pair { names, second } => {
            let chip = |(at, name): (usize, &&str)| Chip {
                label: word(name),
                on: (at > 0) == second,
            };
            Some(names.iter().enumerate().map(chip).collect())
        }
        _ => None,
    };
    OptionView {
        key: r.key,
        kind,
        value,
        on: r.value == OptionValue::Switch(true),
        live: r.live,
        steps: ends.map(|e| steps(t, &label, e)),
        chips,
        label,
    }
}

/// Value `v`'s kind, its words, and a stepper's ends; `word` reads a word of the row's own.
fn shown(
    t: &I18n,
    v: OptionValue,
    word: &dyn Fn(&str) -> String,
) -> (&'static str, String, Option<Ends>) {
    match v {
        OptionValue::Switch(on) => (SWITCH, t.text(if on { VOICE_ON } else { VOICE_OFF }), None),
        OptionValue::Number {
            value,
            auto,
            unit,
            ends,
        } => (NUMBER, number(t, unit, value, auto), Some(ends)),
        OptionValue::Factor { value, ends } => {
            (NUMBER, t.voice_times(VOICE_TIMES, value), Some(ends))
        }
        OptionValue::Choice { name, ends } => (CHOICE, word(name), Some(ends)),
        OptionValue::Pair { names, second } => (PAIR, word(names[usize::from(second)]), None),
        OptionValue::Hint => (HINT, String::new(), None),
    }
}

/// A stepper's ends, and its minus and plus named for option `label`.
fn steps(t: &I18n, label: &str, ends: Ends) -> Steps {
    let name = || [(NAME, label.to_owned())];
    Steps {
        low: ends.low,
        high: ends.high,
        minus: t.voice_with(VOICE_MINUS, &name()),
        plus: t.voice_with(VOICE_PLUS, &name()),
    }
}

/// Number `value` in `unit`; 0 with an `auto` value reads as Auto with it, or Auto alone while that is 0.
fn number(t: &I18n, unit: OptionUnit, value: u32, auto: Option<u32>) -> String {
    let n = |v: u32| usize::try_from(v).unwrap_or(usize::MAX);
    // A duration reads in minutes from one.
    let per_min = u32::try_from(S_PER_MIN).unwrap_or(u32::MAX);
    match auto.filter(|_| value == 0) {
        Some(0) => return t.text(VOICE_AUTO),
        Some(real) => return t.counted(VOICE_AUTO_N, n(real)),
        None => {}
    }
    match unit {
        OptionUnit::Count => value.to_string(),
        OptionUnit::Seconds => t.counted(VOICE_SECONDS, n(value)),
        // Whole seconds take their plural form, as `done_ms` steps by a second.
        OptionUnit::MsAsSeconds if value.is_multiple_of(MS_PER_S) => {
            t.counted(VOICE_SECONDS, n(value / MS_PER_S))
        }
        OptionUnit::MsAsSeconds => t.voice_secs(value),
        OptionUnit::SecondsOrMinutes if value < per_min || !value.is_multiple_of(per_min) => {
            t.counted(VOICE_SECONDS, n(value))
        }
        OptionUnit::SecondsOrMinutes => t.counted(VOICE_MINUTES, n(value / per_min)),
        OptionUnit::Minutes => t.counted(VOICE_MINUTES, n(value)),
        OptionUnit::Ms => t.counted(VOICE_MS, n(value)),
        OptionUnit::Mb => t.voice_room(u64::from(value)),
        OptionUnit::Texts => t.counted(VOICE_TEXTS, n(value)),
        OptionUnit::Words => t.counted(VOICE_WORDS, n(value)),
    }
}

//! The Options page as QML reads it: each row's label from its `[voice]` key, its value with its unit, and its switch
//! or its minus and plus with the dimmed ends (ADR-0008: no slider).

use super::panel::{Cx, Page, Sheet};
use super::sheets::tab_id;
use crate::I18n;
use crate::i18n::keys::vars::NAME;
use crate::i18n::keys::{
    VOICE_APPLIES, VOICE_AUTO_N, VOICE_FOR_ALL, VOICE_MINUS, VOICE_MINUTES, VOICE_MS, VOICE_OFF,
    VOICE_ON, VOICE_OPTION, VOICE_PLUS, VOICE_SECONDS, VOICE_TEXTS, VOICE_TIMES, VOICE_WORD,
    VOICE_WORDS,
};
use kx_module_api::ports::{Ends, OptionRow, OptionValue, VoiceTab};
use serde::Serialize;

/// A row's kind: a switch, a number with minus and plus, or a choice with chevrons.
const SWITCH: &str = "switch";
const NUMBER: &str = "number";
const CHOICE: &str = "choice";
/// Seconds in a minute: a duration reads in minutes from one.
const SECS_PER_MIN: u32 = 60;
/// Milliseconds in a second.
const MS_PER_S: u32 = 1000;
/// What a setting key writes for an underscore, and what a choice's text puts before its name.
const KEY_SEPARATOR: char = '_';
const TEXT_SEPARATOR: &str = "-";

/// How a number reads.
#[derive(Copy, Clone)]
enum Unit {
    /// Whole seconds.
    Seconds,
    /// Milliseconds, read as seconds.
    MsAsSeconds,
    /// Seconds, read in minutes from one minute.
    Duration,
    /// Minutes.
    Minutes,
    /// Milliseconds.
    Ms,
    /// Binary megabytes, read in gigabytes from one.
    Mb,
    /// History's texts.
    Texts,
    /// Words.
    Words,
}

/// Each `[voice]` number's unit by its key (kx-mod-voice `options.toml`); any other number reads bare, as `threads`.
/// It goes once `OptionValue::Number` names its unit.
const UNITS: [(&str, Unit); 13] = [
    ("silence_stop_s", Unit::Seconds),
    ("done_ms", Unit::MsAsSeconds),
    ("history_max", Unit::Texts),
    ("voice_max_mb", Unit::Mb),
    ("idle_min", Unit::Minutes),
    ("record_max_s", Unit::Duration),
    ("nothing_s", Unit::Seconds),
    ("debounce_ms", Unit::Ms),
    ("type_gap_ms", Unit::Ms),
    ("wave_ms", Unit::Ms),
    ("min_free_mb", Unit::Mb),
    ("card_margin_mb", Unit::Mb),
    ("card_diff", Unit::Words),
];

/// An option's row; `steps` is set for a number or a choice.
#[derive(Serialize)]
pub(super) struct OptionView {
    key: &'static str,
    label: String,
    kind: &'static str,
    value: String,
    on: bool,
    live: bool,
    steps: Option<Steps>,
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

/// Option `r` in `t`'s words.
fn row(t: &I18n, r: &OptionRow) -> OptionView {
    let key = r.key.replace(KEY_SEPARATOR, TEXT_SEPARATOR);
    let label = t.text(&format!("{VOICE_OPTION}{key}"));
    let (kind, value, ends) = match r.value {
        OptionValue::Switch(on) => (SWITCH, t.text(if on { VOICE_ON } else { VOICE_OFF }), None),
        OptionValue::Number { value, auto, ends } => {
            (NUMBER, number(t, r.key, value, auto), Some(ends))
        }
        OptionValue::Factor { value, ends } => {
            (NUMBER, t.voice_times(VOICE_TIMES, value), Some(ends))
        }
        OptionValue::Choice { name, ends } => {
            let text = format!("{VOICE_WORD}{key}{TEXT_SEPARATOR}{name}");
            (CHOICE, t.text(&text), Some(ends))
        }
    };
    OptionView {
        key: r.key,
        kind,
        value,
        on: r.value == OptionValue::Switch(true),
        live: r.live,
        steps: ends.map(|e| steps(t, &label, e)),
        label,
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

/// Number `value` of setting `key` with its unit; 0 with an `auto` value reads as Auto with it.
fn number(t: &I18n, key: &str, value: u32, auto: Option<u32>) -> String {
    let n = |v: u32| usize::try_from(v).unwrap_or(usize::MAX);
    if let Some(real) = auto.filter(|_| value == 0) {
        return t.counted(VOICE_AUTO_N, n(real));
    }
    let unit = UNITS.iter().find(|(k, _)| *k == key).map(|(_, u)| *u);
    let Some(unit) = unit else {
        return value.to_string();
    };
    match unit {
        Unit::Seconds => t.counted(VOICE_SECONDS, n(value)),
        // Whole seconds take their plural form, as `done_ms` steps by a second.
        Unit::MsAsSeconds if value.is_multiple_of(MS_PER_S) => {
            t.counted(VOICE_SECONDS, n(value / MS_PER_S))
        }
        Unit::MsAsSeconds => t.voice_secs(value),
        Unit::Duration if value < SECS_PER_MIN || !value.is_multiple_of(SECS_PER_MIN) => {
            t.counted(VOICE_SECONDS, n(value))
        }
        Unit::Duration => t.counted(VOICE_MINUTES, n(value / SECS_PER_MIN)),
        Unit::Minutes => t.counted(VOICE_MINUTES, n(value)),
        Unit::Ms => t.counted(VOICE_MS, n(value)),
        Unit::Mb => t.voice_room(u64::from(value)),
        Unit::Texts => t.counted(VOICE_TEXTS, n(value)),
        Unit::Words => t.counted(VOICE_WORDS, n(value)),
    }
}

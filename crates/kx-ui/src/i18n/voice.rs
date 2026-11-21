//! Voice typing's worded parts (P11): a note's line with its language, file sizes, memory, seconds and factors. Digits
//! stay Western (ADR-0017), with the language's decimal mark; trailing zeros drop, but seconds keep their one decimal.

use super::I18n;
use super::compose::{FORM_VAR, FRACTION_FORM, clean, count};
use super::keys::vars::LANG;
use super::keys::{
    LANG_IN, LANG_NAME, NUM_DECIMAL, VOICE_GB, VOICE_MB, VOICE_NOTE, VOICE_SECS, bar_hint_id,
};
use fluent_bundle::{FluentArgs, FluentValue};
use kx_module_api::MS_PER_S;
use kx_module_api::ports::{BarHint, Date, Hundredths, VoiceLang};

/// Bytes in a megabyte as the models table writes sizes, in decimal units.
const SI_MB: u128 = 1_000_000;
/// Megabytes in a gigabyte, decimal for file sizes and binary for memory.
const SI_GB_MB: u128 = 1000;
const BIN_GB_MB: u128 = 1024;
/// A file size shows three significant digits: two decimals below ten gigabytes, one below a hundred.
const TWO_PLACES_BELOW_GB: u128 = 10;
const ONE_PLACE_BELOW_GB: u128 = 100;
/// The decimals memory, seconds and factors show at most.
const MEMORY_PLACES: u32 = 1;
const SECONDS_PLACES: u32 = 1;
const FACTOR_PLACES: u32 = 2;
/// The base decimals count in.
const TEN: u128 = 10;
/// The variables these texts take.
const MB_VAR: &str = "mb";
const GB_VAR: &str = "gb";
const S_VAR: &str = "s";
const TIMES_VAR: &str = "times";

impl I18n {
    /// The line of note `hint`, naming its language where it has one.
    #[must_use]
    pub fn voice_hint(&self, hint: &BarHint) -> String {
        if let BarHint::Cloud { why, .. } = hint {
            return self.voice_cloud(why);
        }
        let key = format!("{VOICE_NOTE}{}", bar_hint_id(hint));
        match lang_of(hint) {
            Some(lang) => self.with(&key, [(LANG, self.voice_lang(lang))]),
            None => self.text(&key),
        }
    }

    /// Language `lang` as a sentence of these texts names it: their own name for its code, French in lower case, else
    /// the system's name for it.
    #[must_use]
    pub fn voice_lang(&self, lang: &VoiceLang) -> String {
        self.lang_in(LANG_IN, lang)
    }

    /// Language `lang` as a title, as the chip shows it: these texts' own name for its code, else the system's name.
    #[must_use]
    pub fn voice_title(&self, lang: &VoiceLang) -> String {
        self.lang_in(LANG_NAME, lang)
    }

    /// Language `lang` by its code in name group `group`, else the system's name for it.
    fn lang_in(&self, group: &str, lang: &VoiceLang) -> String {
        let key = format!("{group}.{}", lang.iso);
        if lang.iso.is_empty() || !self.has(&key) {
            return clean(&lang.name);
        }
        self.text(&key)
    }

    /// Text `key` with text arguments `args`, which the caller cleaned of hidden marks.
    #[must_use]
    pub fn voice_with(&self, key: &str, args: &[(&'static str, String)]) -> String {
        let mut values = FluentArgs::new();
        put(&mut values, args);
        self.format(key, Some(&values))
    }

    /// Text `key` with `$count` at `n` in the plural form it takes, and text arguments `args`.
    #[must_use]
    pub fn voice_counted(&self, key: &str, n: usize, args: &[(&'static str, String)]) -> String {
        let mut values = self.count_args(n);
        put(&mut values, args);
        self.format(key, Some(&values))
    }

    /// `date` as the clipboard panel writes a day.
    #[must_use]
    pub fn voice_date(&self, date: Date) -> String {
        self.date(date)
    }

    /// A file's or a drive's size in decimal units, as the models table writes them: whole megabytes below a gigabyte,
    /// else gigabytes to three significant digits.
    #[must_use]
    pub fn voice_size(&self, bytes: u64) -> String {
        let (bytes, gb) = (u128::from(bytes), SI_MB * SI_GB_MB);
        if round(bytes, SI_MB) < SI_GB_MB {
            return self.with(VOICE_MB, [(MB_VAR, self.decimals(bytes, SI_MB, 0))]);
        }
        let places = match bytes / gb {
            n if n < TWO_PLACES_BELOW_GB => 2,
            n if n < ONE_PLACE_BELOW_GB => 1,
            _ => 0,
        };
        self.with(VOICE_GB, [(GB_VAR, self.decimals(bytes, gb, places))])
    }

    /// Memory of `mb` binary megabytes, in gigabytes with one decimal at most, as in 9.4 GB or 0.5 GB.
    #[must_use]
    pub fn voice_memory(&self, mb: u32) -> String {
        let gb = self.decimals(u128::from(mb), BIN_GB_MB, MEMORY_PLACES);
        self.with(VOICE_GB, [(GB_VAR, gb)])
    }

    /// A room of `mb` binary megabytes: megabytes below a gigabyte, else gigabytes with one decimal at most.
    #[must_use]
    pub fn voice_room(&self, mb: u64) -> String {
        let mb = u128::from(mb);
        if mb < BIN_GB_MB {
            return self.with(VOICE_MB, [(MB_VAR, self.grouped(mb))]);
        }
        let gb = self.decimals(mb, BIN_GB_MB, MEMORY_PLACES);
        self.with(VOICE_GB, [(GB_VAR, gb)])
    }

    /// `ms` as seconds with one decimal, as the mock-up writes a wait (2.1 s, 4.0 s): the column lines up, and an
    /// Arabic decimal always takes the singular.
    #[must_use]
    pub fn voice_secs(&self, ms: u32) -> String {
        let s = self.places(u128::from(ms), u128::from(MS_PER_S), SECONDS_PLACES, false);
        self.with(VOICE_SECS, [(S_VAR, s)])
    }

    /// Text `key` with factor `value` as `$times`, and `$form` its plural form when it is whole.
    #[must_use]
    pub fn voice_times(&self, key: &str, value: Hundredths) -> String {
        let (per, value) = (Hundredths::PER_ONE, value.0);
        let mut args = FluentArgs::new();
        let times = self.decimals(u128::from(value), u128::from(per), FACTOR_PLACES);
        args.set(TIMES_VAR, times);
        let whole = value.is_multiple_of(per);
        let form = if whole {
            count(usize::try_from(value / per).unwrap_or(usize::MAX))
        } else {
            FluentValue::from(FRACTION_FORM)
        };
        args.set(FORM_VAR, form);
        self.format(key, Some(&args))
    }

    /// `num` / `den` rounded to `places` decimals, grouped, with the language's decimal mark and no trailing zeros.
    fn decimals(&self, num: u128, den: u128, places: u32) -> String {
        self.places(num, den, places, true)
    }

    /// `num` / `den` rounded to `places` decimals, grouped, with the language's decimal mark; `trim` drops trailing
    /// zeros, and the mark with them.
    pub(super) fn places(&self, num: u128, den: u128, places: u32, trim: bool) -> String {
        let scale = TEN.pow(places);
        let scaled = round(num.saturating_mul(scale), den);
        let whole = self.grouped(scaled / scale);
        let width = usize::try_from(places).unwrap_or_default();
        let digits = format!("{:0width$}", scaled % scale);
        let part = if trim || width == 0 {
            digits.trim_end_matches('0')
        } else {
            &digits
        };
        if part.is_empty() {
            whole
        } else {
            format!("{whole}{}{part}", self.text(NUM_DECIMAL))
        }
    }
}

/// The language `hint` names, if any.
const fn lang_of(hint: &BarHint) -> Option<&VoiceLang> {
    match hint {
        BarHint::NoModel { lang, .. } | BarHint::Language(lang) | BarHint::NoLanguage(lang) => {
            Some(lang)
        }
        _ => None,
    }
}

/// Sets each of `args` in `values`.
fn put(values: &mut FluentArgs<'static>, args: &[(&'static str, String)]) {
    for (name, value) in args {
        values.set(*name, FluentValue::from(value.clone()));
    }
}

/// `num` / `den`, rounded half up; 0 for no `den`.
fn round(num: u128, den: u128) -> u128 {
    num.saturating_add(den / 2)
        .checked_div(den)
        .unwrap_or_default()
}

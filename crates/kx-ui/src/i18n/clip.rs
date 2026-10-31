//! The clipboard panel's worded numbers (spec §7): sizes with two decimals and the language's marks, the storage
//! line, a row's time and meta line, and counted texts. Digits stay Western (ADR-0017), grouped by threes.

use super::I18n;
use super::compose::{clean, count, form};
use super::keys::{
    CLIP_DATE, CLIP_DAY, CLIP_KEPT, CLIP_META, CLIP_META_BARE, CLIP_SIZE, CLIP_SIZE_GB,
    CLIP_SIZE_SMALL, CLIP_STORAGE, CLIP_TIME, CLIP_TODAY, CLIP_WHEN, CLIP_YESTERDAY, NUM_DECIMAL,
    NUM_GROUP,
};
use fluent_bundle::{FluentArgs, FluentValue};
use kx_module_api::ports::{Date, MB, Room, Weekday, When};

/// Megabytes in a gigabyte.
const MB_PER_GB: u64 = 1 << 10;
/// Hundredths in one: sizes show two decimals.
const HUNDRED: u128 = 100;
/// Digits a size's decimals, a time's hour and minute and a date's day and month take.
const PAIR: usize = 2;
/// Digits between two group marks.
const GROUP: usize = 3;
/// Days in a week: within one a weekday names the day, past it the date does.
const WEEK: usize = Weekday::ALL.len();
/// The variables the clipboard's texts take.
const COUNT_VAR: &str = "count";
const FORM_VAR: &str = "form";
const DAYS_VAR: &str = "days";
const MB_VAR: &str = "mb";
const GB_VAR: &str = "gb";
const USED_VAR: &str = "used";
const CAP_VAR: &str = "cap";
const FREE_VAR: &str = "free";
const DRIVE_VAR: &str = "drive";
const WORD_VAR: &str = "word";
const TIME_VAR: &str = "time";
const HOUR_VAR: &str = "hour";
const MINUTE_VAR: &str = "minute";
const DAY_VAR: &str = "day";
const MONTH_VAR: &str = "month";
const YEAR_VAR: &str = "year";
const APP_VAR: &str = "app";
const WHEN_VAR: &str = "when";
const SIZE_VAR: &str = "size";

impl I18n {
    /// A row's size: megabytes with two decimals, or `clip.size-small` below 0.01 MB.
    #[must_use]
    pub fn clip_size(&self, bytes: u64) -> String {
        if u128::from(bytes) * HUNDRED < u128::from(MB) {
            return self.with(CLIP_SIZE_SMALL, [(MB_VAR, self.decimal(1))]);
        }
        self.with(CLIP_SIZE, [(MB_VAR, self.decimal(hundredths(bytes, MB)))])
    }

    /// The storage line: what the history takes of `cap`, and what its drive has free, each in megabytes below a
    /// gigabyte and in gigabytes from one, with two decimals.
    #[must_use]
    pub fn clip_room(&self, room: &Room, cap: u64) -> String {
        let args = [
            (USED_VAR, self.big(room.used)),
            (CAP_VAR, self.big(cap)),
            (FREE_VAR, self.big(room.free)),
            (DRIVE_VAR, clean(&room.drive)),
        ];
        self.with(CLIP_STORAGE, args)
    }

    /// When a row was copied: today, yesterday or a weekday with the 24-hour time within a week, else the date.
    #[must_use]
    pub fn clip_when(&self, when: &When) -> String {
        let word = match usize::try_from(when.days_ago).unwrap_or(usize::MAX) {
            0 => self.text(CLIP_TODAY),
            1 => self.text(CLIP_YESTERDAY),
            n if n < WEEK => self.text(&format!("{CLIP_DAY}{}", when.date.weekday().number())),
            _ => return self.date(when.date),
        };
        let time = self.with(
            CLIP_TIME,
            [(HOUR_VAR, pair(when.hour)), (MINUTE_VAR, pair(when.minute))],
        );
        self.with(CLIP_WHEN, [(WORD_VAR, word), (TIME_VAR, time)])
    }

    /// A row's line under its text: the app when known, the time and the size; an app of hidden marks alone is
    /// unknown.
    #[must_use]
    pub fn clip_meta(&self, app: Option<&str>, when: &str, size: &str) -> String {
        let (when, size) = ((WHEN_VAR, when.to_owned()), (SIZE_VAR, size.to_owned()));
        match app.map(clean).filter(|a| !a.is_empty()) {
            Some(app) => self.with(CLIP_META, [(APP_VAR, app), when, size]),
            None => self.with(CLIP_META_BARE, [when, size]),
        }
    }

    /// The text of `key` with `$count` at `n`, grouped by threes, in the plural form `n` takes.
    #[must_use]
    pub fn counted(&self, key: &str, n: usize) -> String {
        let mut values = FluentArgs::new();
        let wide = u64::try_from(n).map_or(u128::MAX, u128::from);
        values.set(COUNT_VAR, self.grouped(wide));
        values.set(FORM_VAR, form(n));
        self.format(key, Some(&values))
    }

    /// How long entries are kept: `days` calendar days.
    #[must_use]
    pub fn clip_kept(&self, days: u8) -> String {
        let mut values = FluentArgs::new();
        values.set(DAYS_VAR, count(usize::from(days)));
        values.set(FORM_VAR, form(usize::from(days)));
        self.format(CLIP_KEPT, Some(&values))
    }

    /// `date` as `clip.date` orders it, its day and month in two digits.
    fn date(&self, date: Date) -> String {
        let args = [
            (DAY_VAR, pair(date.day())),
            (MONTH_VAR, pair(date.month())),
            (YEAR_VAR, date.year().to_string()),
        ];
        self.with(CLIP_DATE, args)
    }

    /// A storage-line size: megabytes below a gigabyte, gigabytes from one, chosen after rounding.
    fn big(&self, bytes: u64) -> String {
        let (gb, mb) = (MB * MB_PER_GB, hundredths(bytes, MB));
        if mb < u128::from(MB_PER_GB) * HUNDRED {
            self.with(CLIP_SIZE, [(MB_VAR, self.decimal(mb))])
        } else {
            self.with(
                CLIP_SIZE_GB,
                [(GB_VAR, self.decimal(hundredths(bytes, gb)))],
            )
        }
    }

    /// `hundredths` / 100, its whole part grouped, with two decimals after the language's decimal mark.
    fn decimal(&self, hundredths: u128) -> String {
        let (whole, part) = (hundredths / HUNDRED, hundredths % HUNDRED);
        let whole = self.grouped(whole);
        format!("{whole}{}{part:0PAIR$}", self.text(NUM_DECIMAL))
    }

    /// `n` with the language's group mark before each three digits from the right.
    fn grouped(&self, n: u128) -> String {
        let digits = n.to_string();
        let mark = self.text(NUM_GROUP);
        let mut out = String::new();
        for (at, digit) in digits.chars().enumerate() {
            if at > 0 && (digits.len() - at).is_multiple_of(GROUP) {
                out.push_str(&mark);
            }
            out.push(digit);
        }
        out
    }

    /// The text of `key` with text arguments `args`.
    fn with<const N: usize>(&self, key: &str, args: [(&'static str, String); N]) -> String {
        let mut values = FluentArgs::new();
        for (name, value) in args {
            values.set(name, FluentValue::from(value));
        }
        self.format(key, Some(&values))
    }
}

/// `bytes` in hundredths of `unit`, rounded half up.
fn hundredths(bytes: u64, unit: u64) -> u128 {
    let unit = u128::from(unit);
    (u128::from(bytes) * HUNDRED + unit / 2) / unit
}

/// `n` in at least two digits.
fn pair(n: u8) -> String {
    format!("{n:0PAIR$}")
}

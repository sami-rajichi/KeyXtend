//! Quick-fill's worded parts (design "The panel"): table labels by id, a row's field count, a place's last write, a
//! found box, the waiting footer and the import note. Dates and times are the clipboard's, with Western digits.

use super::compose::clean;
use super::keys::{
    FILL_FIELDS, FILL_FOREIGN_UNDATED, FILL_LABEL, FILL_NOTE, FILL_REMOVE_PLACE, FILL_UPDATED_DATE,
    FILL_UPDATED_TODAY, FILL_UPDATED_YESTERDAY, FILL_WAITING, fill_note_id,
};
use super::{I18n, find_in, split};
use kx_module_api::ports::{FillNote, NoteCounts, When};
use std::collections::BTreeMap;

/// The variables Quick-fill's texts take.
const KIND_VAR: &str = "kind";
const SKIPPED_VAR: &str = "skipped";
const PLACE_VAR: &str = "place";
const DATE_VAR: &str = "date";
const TIME_VAR: &str = "time";

impl I18n {
    /// Table label `id`'s text, or `id` itself when no language has one.
    #[must_use]
    pub fn fill_label(&self, id: &str) -> String {
        find_in(&self.bundles, &format!("{FILL_LABEL}{id}"), None).unwrap_or_else(|| id.to_owned())
    }

    /// Every table label's text by its id, as the vault's search and By label match what the user reads; the ids are
    /// the fallback's, so a label this language lacks still shows in English.
    #[must_use]
    pub fn fill_labels(&self) -> BTreeMap<String, String> {
        let (group, prefix) = split(FILL_LABEL);
        let prefix = prefix.unwrap_or_default();
        let message = self.bundles.last().and_then(|b| b.get_message(group));
        let ids = message.into_iter().flat_map(|m| m.attributes());
        let ids = ids.filter_map(|a| a.id().strip_prefix(prefix).map(str::to_owned));
        ids.map(|id| {
            let text = self.fill_label(&id);
            (id, text)
        })
        .collect()
    }

    /// A row's hint with no glimpse: kind name `kind`, without hidden marks, and `n` fields.
    #[must_use]
    pub fn fill_fields(&self, kind: &str, n: usize) -> String {
        let mut args = self.count_args(n);
        args.set(KIND_VAR, clean(kind));
        self.format(FILL_FIELDS, Some(&args))
    }

    /// When a place's box was written: today or yesterday with the time, else the date.
    #[must_use]
    pub fn fill_updated(&self, when: &When) -> String {
        match when.days_ago {
            0 => self.with(FILL_UPDATED_TODAY, [(TIME_VAR, self.time(when))]),
            1 => self.with(FILL_UPDATED_YESTERDAY, [(TIME_VAR, self.time(when))]),
            _ => self.with(FILL_UPDATED_DATE, [(DATE_VAR, self.date(when.date))]),
        }
    }

    /// Text `key`, a found box's or a foreign one's, with its `place`, without hidden marks, and its date.
    #[must_use]
    pub fn fill_found(&self, key: &str, place: &str, when: &When) -> String {
        let args = [(PLACE_VAR, clean(place)), (DATE_VAR, self.date(when.date))];
        self.with(key, args)
    }

    /// The question about a box from another computer in `place`, whose date the system does not tell.
    #[must_use]
    pub fn fill_foreign_undated(&self, place: &str) -> String {
        self.with(FILL_FOREIGN_UNDATED, [(PLACE_VAR, clean(place))])
    }

    /// Remove place's question about `place`, without hidden marks.
    #[must_use]
    pub fn fill_remove_place(&self, place: &str) -> String {
        self.with(FILL_REMOVE_PLACE, [(PLACE_VAR, clean(place))])
    }

    /// The footer while the backup waits for `place`, without hidden marks.
    #[must_use]
    pub fn fill_waiting(&self, place: &str) -> String {
        self.with(FILL_WAITING, [(PLACE_VAR, clean(place))])
    }

    /// The import note: how many entries came in, plural by that count, and how many entries or values were left out.
    #[must_use]
    pub fn fill_imported(&self, counts: NoteCounts) -> String {
        let mut args = self.count_args(usize::try_from(counts.added).unwrap_or(usize::MAX));
        args.set(SKIPPED_VAR, self.grouped(u128::from(counts.skipped)));
        let key = format!("{FILL_NOTE}{}", fill_note_id(FillNote::Imported));
        self.format(&key, Some(&args))
    }
}

//! Many new entries at once, as an import brings them: each under `add`'s rules, with the byte total and the ids kept as
//! it goes, so a whole file takes one pass over the notebook.

use std::collections::HashSet;

use kx_module_api::ports::Random;

use super::{Limits, Notebook, draw_id};
use crate::entry::{Draft, Entry, Stamp};
use crate::store::entry_len;

impl Notebook {
    /// Adds each of `drafts` changed at `now`, with ids from `random`; gives how many came in. One refused or past
    /// `max_bytes` stays out while a later, smaller one may fit, and `max_entries` or no random bytes ends it.
    pub fn add_all(
        &mut self,
        drafts: Vec<Draft>,
        now: Stamp,
        random: &dyn Random,
        limits: &Limits,
    ) -> u32 {
        let now = self.stamp(now);
        let mut bytes = self.bytes();
        let mut ids: HashSet<u64> = self.entries.iter().map(Entry::id).collect();
        let mut added = 0_u32;
        for draft in drafts {
            if self.entries.len() >= limits.max_entries {
                break;
            }
            let Ok(id) = draw_id(random, |id| ids.contains(&id)) else {
                break;
            };
            let Ok(entry) = Entry::new(id, draft, now, limits) else {
                continue;
            };
            let after = bytes.saturating_add(entry_len(&entry));
            if after > limits.max_bytes {
                continue;
            }
            (bytes, added) = (after, added.saturating_add(1));
            ids.insert(id);
            self.entries.push(entry);
        }
        added
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{LIMITS, draft, field};
    use kx_module_api::ports::{EntryKind, RandomError};
    use kx_platform_fake::FakeRandom;

    /// When the entries come in.
    const NOW: Stamp = Stamp::from_unix_ms(1_700_000_000_000);

    /// A note named `name`.
    fn note(name: &str) -> Draft {
        let fields = vec![field("text", "42", false)];
        draft(name, EntryKind::Note, fields)
    }

    /// A source that always fails.
    struct NoSource;

    impl Random for NoSource {
        fn fill(&self, _: &mut [u8]) -> Result<(), RandomError> {
            Err(RandomError)
        }
    }

    #[test]
    fn the_total_and_the_ids_stay_true_over_many() {
        let (mut book, random) = (Notebook::default(), FakeRandom::new(9));
        let drafts = (0..2000).map(|n| note(&format!("Note {n}"))).collect();
        assert_eq!(book.add_all(drafts, NOW, &random, &LIMITS), 2000);
        assert_eq!(book.len(), 2000);
        let ids: HashSet<u64> = book.entries().iter().map(Entry::id).collect();
        assert_eq!(ids.len(), 2000);
        assert!(book.bytes() <= LIMITS.max_bytes);
    }

    #[test]
    fn entries_that_fit_alone_but_not_together_stop_at_the_total() {
        let random = FakeRandom::new(9);
        let mut one = Notebook::default();
        one.add_all(vec![note("A")], NOW, &random, &LIMITS);
        let limits = Limits {
            max_bytes: one.bytes() + 1,
            ..LIMITS
        };
        let mut book = Notebook::default();
        let drafts = vec![note("A"), note("B")];
        assert_eq!(book.add_all(drafts, NOW, &random, &limits), 1);
        assert!(book.bytes() <= limits.max_bytes);
    }

    #[test]
    fn a_clock_set_back_still_reads_as_newer() {
        let (mut book, random) = (Notebook::default(), FakeRandom::new(9));
        book.add(note("First"), NOW, &random, &LIMITS).unwrap();
        let earlier = Stamp::from_unix_ms(1);
        book.add_all(vec![note("Next")], earlier, &random, &LIMITS);
        assert_eq!(book.entries()[1].changed(), NOW.after());
    }

    #[test]
    fn a_refused_draft_stays_out_and_the_rest_come_in() {
        let (mut book, random) = (Notebook::default(), FakeRandom::new(9));
        let drafts = vec![note("bad\u{202e}"), note("good")];
        assert_eq!(book.add_all(drafts, NOW, &random, &LIMITS), 1);
    }

    #[test]
    fn no_random_bytes_end_it_with_nothing_lost() {
        let mut book = Notebook::default();
        let drafts = vec![note("one"), note("two")];
        assert_eq!(book.add_all(drafts, NOW, &NoSource, &LIMITS), 0);
        assert!(book.is_empty());
    }
}

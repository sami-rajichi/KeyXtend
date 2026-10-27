//! The candidates' spellings indexed for the text counter: a token finds its candidate by its own spelling, then its
//! lower case, then its fold key, so `L'`, `LONDON` and `انا` find `l'`, `London` and `أنا`.

use std::collections::HashMap;

use kx_mod_predict::{Casing, fold};

use super::cands::{Cand, Fate};
use super::text::{KeyCounts, TextCounts};

/// One spelling a candidate may take.
#[derive(Debug)]
struct Form {
    /// Its spelling.
    spelling: String,
    /// The candidate it belongs to.
    cand: u32,
    /// Its fold key's place in `keys`.
    key: u32,
}

/// Where a token was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Hit {
    /// The candidate's place in the candidate list.
    pub(crate) cand: u32,
    /// The fold key's place among the keys.
    pub(crate) key: u32,
    /// The form's place, when the token is spelt exactly as one.
    pub(crate) form: Option<u32>,
    /// The place of the form the token was taken as, by spelling, case or fold.
    pub(crate) found: u32,
}

/// The spellings of every candidate the text may count: the kept ones and the stems.
#[derive(Debug)]
pub(crate) struct Vocab {
    /// Every form.
    forms: Vec<Form>,
    /// Each form's spelling, to its place.
    exact: HashMap<String, u32>,
    /// Each fold key.
    keys: Vec<String>,
    /// Each fold key, to the form of its most frequent candidate.
    folded: HashMap<String, u32>,
    /// How the language cases tokens, which the keys were folded with.
    casing: &'static Casing,
}

impl Vocab {
    /// The index of `cands` in a language cased by `casing`; a spelling or key shared by two candidates goes to the
    /// more frequent one.
    pub(crate) fn new(cands: &[Cand], casing: &'static Casing) -> Self {
        let mut order: Vec<(usize, &Cand)> = (cands.iter().enumerate())
            .filter(|(_, c)| matches!(c.fate, Fate::Kept | Fate::Stem))
            .collect();
        order.sort_by_key(|(_, c)| (c.bucket, c.rank));
        let mut vocab = Self {
            forms: Vec::new(),
            exact: HashMap::new(),
            keys: Vec::new(),
            folded: HashMap::new(),
            casing,
        };
        let mut key_ids: HashMap<String, u32> = HashMap::new();
        for (cand, c) in order {
            let Ok(cand) = u32::try_from(cand) else {
                continue;
            };
            for spelling in &c.forms {
                vocab.add(spelling, cand, &mut key_ids);
            }
        }
        vocab
    }

    /// Adds `spelling` of candidate `cand`, unless a more frequent candidate already holds it.
    fn add(&mut self, spelling: &str, cand: u32, key_ids: &mut HashMap<String, u32>) {
        let (Ok(id), false) = (
            u32::try_from(self.forms.len()),
            self.exact.contains_key(spelling),
        ) else {
            return;
        };
        let key = fold(spelling, self.casing);
        let next = u32::try_from(self.keys.len()).unwrap_or(u32::MAX);
        let key_id = *key_ids.entry(key.clone()).or_insert(next);
        if key_id == next {
            self.keys.push(key.clone());
        }
        self.folded.entry(key).or_insert(id);
        self.exact.insert(spelling.to_string(), id);
        self.forms.push(Form {
            spelling: spelling.to_string(),
            cand,
            key: key_id,
        });
    }

    /// Where `token` is found: as a form, as a form in lower case, or by its fold key.
    pub(crate) fn find(&self, token: &str) -> Option<Hit> {
        if let Some(&form) = self.exact.get(token) {
            return self.hit(form, true);
        }
        let lower = self.casing.lower(token);
        if let Some(&form) = self.exact.get(&lower) {
            return self.hit(form, false);
        }
        let form = *self.folded.get(&fold(&lower, self.casing))?;
        self.hit(form, false)
    }

    /// True when `token` is found.
    pub(crate) fn knows(&self, token: &str) -> bool {
        self.find(token).is_some()
    }

    /// The hit of form `form`, naming the form only when the token was spelt as it.
    fn hit(&self, form: u32, exact: bool) -> Option<Hit> {
        let f = self.forms.get(usize::try_from(form).ok()?)?;
        Some(Hit {
            cand: f.cand,
            key: f.key,
            form: exact.then_some(form),
            found: form,
        })
    }

    /// How many forms and fold keys there are, to size the counters.
    pub(crate) fn sizes(&self) -> (usize, usize) {
        (self.forms.len(), self.keys.len())
    }

    /// The text counts from the counters: `total` tokens, each key's standalone count, and each form's standalone
    /// sightings away from sentence starts (`seen`) and its sightings anywhere (`found`).
    pub(crate) fn counts(
        &self,
        total: u64,
        alone: &[u64],
        seen: &[u64],
        found: &[u64],
    ) -> TextCounts {
        let mut keys: HashMap<String, KeyCounts> = HashMap::new();
        for (i, key) in self.keys.iter().enumerate() {
            if let n @ 1.. = at(alone, i) {
                keys.entry(key.clone()).or_default().alone = n;
            }
        }
        let mut by_form = HashMap::new();
        for (i, form) in self.forms.iter().enumerate() {
            if let n @ 1.. = at(found, i) {
                by_form.insert(form.spelling.clone(), n);
            }
            let key = self
                .keys
                .get(usize::try_from(form.key).unwrap_or(usize::MAX));
            if let (Some(key), n @ 1..) = (key, at(seen, i)) {
                let counts = keys.entry(key.clone()).or_default();
                counts.spellings.insert(form.spelling.clone(), n);
            }
        }
        TextCounts {
            total,
            keys,
            found: by_form,
            casing: self.casing,
        }
    }
}

/// The count at `i`, or 0.
fn at(counts: &[u64], i: usize) -> u64 {
    counts.get(i).copied().unwrap_or_default()
}

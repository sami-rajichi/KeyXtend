//! The main script of a word list: a list mostly in one script drops the words of the others. Each letter's script
//! comes from kx-mod-predict's table, which the tracker shares.

use std::collections::HashMap;

pub(crate) use kx_mod_predict::script::{Script, of, single};

use super::float;

/// The commonest script in `scripts`, when it and the scripts it holds (Hanja for Hangul) have at least `share` of
/// the words with a script.
pub(crate) fn main(scripts: impl Iterator<Item = Option<Script>>, share: f64) -> Option<Script> {
    let mut counts: HashMap<Script, u64> = HashMap::new();
    for script in scripts.flatten() {
        *counts.entry(script).or_default() += 1;
    }
    let total: u64 = counts.values().sum();
    let top = *counts.iter().max_by_key(|(_, n)| **n)?.0;
    let held: u64 = (counts.iter())
        .filter(|(s, _)| top.holds(**s))
        .map(|(_, n)| n)
        .sum();
    (float(held) >= share * float(total)).then_some(top)
}

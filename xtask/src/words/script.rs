//! The main script of a word list: a list mostly in one script drops the words of the others. Each letter's script
//! comes from kx-mod-predict's table, which the tracker shares.

use std::collections::HashMap;

pub(crate) use kx_mod_predict::script::{Script, of, single};

use super::float;

/// The script holding at least `share` of the words with a script in `scripts`, if one does.
pub(crate) fn main(scripts: impl Iterator<Item = Option<Script>>, share: f64) -> Option<Script> {
    let mut counts: HashMap<Script, u64> = HashMap::new();
    let mut total = 0;
    for script in scripts.flatten() {
        *counts.entry(script).or_default() += 1;
        total += 1;
    }
    let (top, count) = counts.into_iter().max_by_key(|(_, n)| *n)?;
    (float(count) >= share * float(total)).then_some(top)
}

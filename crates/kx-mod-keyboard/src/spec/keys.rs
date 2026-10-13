//! The board's keys as lists: each main row's keys, the character keys' scan codes and ids, every key's role, and
//! the keys that repeat while held.

use super::{BoardSpec, KeySpec, Role, RowItem};
use kx_module_api::ports::Scan;
use std::collections::HashSet;

/// What starts a character key's id, `c-` and its scan code.
const CHAR_PREFIX: &str = "c";

/// One key of a main row, as `rows` lists it.
#[derive(Clone, Debug, PartialEq)]
pub enum RowKey<'a> {
    /// A named key.
    Named(&'a KeySpec),
    /// A character key.
    Char {
        /// Its id, `c-` and its scan code, such as `c-0x002C`.
        id: String,
        /// Its scan code.
        sc: Scan,
    },
}

impl BoardSpec {
    /// The keys of main row `row`, left to right.
    #[must_use]
    pub fn row_keys(&self, row: usize) -> Vec<RowKey<'_>> {
        let Some(items) = self.rows.get(row) else {
            return Vec::new();
        };
        let mut keys = Vec::new();
        for item in items {
            match item {
                RowItem::Key(k) => keys.push(RowKey::Named(k)),
                RowItem::Chars(c) => keys.extend(c.chars.iter().map(|&sc| RowKey::Char {
                    id: char_id(sc),
                    sc,
                })),
            }
        }
        keys
    }

    /// The scan codes of every character key, row by row, left to right.
    #[must_use]
    pub fn char_scans(&self) -> Vec<Scan> {
        let runs = self.rows.iter().flatten().filter_map(|i| match i {
            RowItem::Chars(c) => Some(c.chars.iter().copied()),
            RowItem::Key(_) => None,
        });
        runs.flatten().collect()
    }

    /// Every key's id, role and scan code: the main rows left to right, then the side keys.
    #[must_use]
    pub fn roles(&self) -> Vec<(String, Role, Option<Scan>)> {
        let main = (0..self.rows.len()).flat_map(|r| self.row_keys(r));
        let main = main.map(|k| match k {
            RowKey::Named(n) => (n.id.clone(), n.role, n.sc),
            RowKey::Char { id, sc } => (id, Role::Char, Some(sc)),
        });
        let side = self.side.iter().map(|s| (s.id.clone(), s.role, None));
        main.chain(side).collect()
    }

    /// The ids of the keys that repeat while held: every character key, and each named key marked `repeat`.
    #[must_use]
    pub fn repeating(&self) -> HashSet<String> {
        let named = self.rows.iter().flatten().filter_map(|i| match i {
            RowItem::Key(k) => k.repeat.then(|| k.id.clone()),
            RowItem::Chars(_) => None,
        });
        named
            .chain(self.char_scans().into_iter().map(char_id))
            .collect()
    }
}

/// The id of the character key that sends `sc`; it stays when keys are added or moved.
#[must_use]
pub fn char_id(sc: Scan) -> String {
    format!("{CHAR_PREFIX}-{sc}")
}

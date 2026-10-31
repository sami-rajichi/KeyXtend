//! The "leave me out of history" markers: a copy that carries one is reported without a byte of it being read.

#[cfg(windows)]
use super::open::{self, Open};

/// The markers a watch checks before it reads anything; the shipped ones come from `clip.toml`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Marks {
    /// Formats whose presence alone marks a copy.
    present: Vec<String>,
    /// Formats holding a 32-bit number, where 0 marks a copy.
    zero: Vec<String>,
}

impl Marks {
    /// Markers of both kinds.
    pub(crate) fn new(present: Vec<String>, zero: Vec<String>) -> Self {
        Self { present, zero }
    }

    /// Whether `name` is one of the markers, ignoring case as Windows does.
    pub(crate) fn has(&self, name: &str) -> bool {
        self.names().any(|n| n.eq_ignore_ascii_case(name))
    }

    /// These markers without `name`, so a test can watch its own writes.
    #[cfg(test)]
    pub(crate) fn without(&self, name: &str) -> Self {
        let keep = |list: &[String]| {
            let others = list.iter().filter(|n| !n.eq_ignore_ascii_case(name));
            others.cloned().collect()
        };
        Self::new(keep(&self.present), keep(&self.zero))
    }

    /// Every marker's name, presence ones first.
    pub(crate) fn names(&self) -> impl Iterator<Item = &str> {
        self.present.iter().chain(&self.zero).map(String::as_str)
    }

    /// Whether a copy is marked, given whether it `has` a format and the number a format holds (`dword`).
    pub(crate) fn marked(
        &self,
        has: impl Fn(&str) -> bool,
        dword: impl Fn(&str) -> Option<u32>,
    ) -> bool {
        self.present.iter().any(|n| has(n)) || self.zero.iter().any(|n| dword(n) == Some(0))
    }

    /// Whether the open clipboard carries one of these markers. A marker Windows cannot name counts as there, so a
    /// failure never reads a marked copy.
    #[cfg(windows)]
    pub(crate) fn on(&self, open: &Open) -> bool {
        let has = |n: &str| open::format_id(n).map_or(true, |id| open::available(open, id));
        let dword = |n: &str| open::format_id(n).map_or(Some(0), |id| open::dword_of(open, id));
        self.marked(has, dword)
    }
}

/// Whether the open clipboard carries a shipped marker, as for a password; a table that does not parse says yes.
#[cfg(windows)]
#[must_use]
pub fn marked(open: &Open) -> bool {
    super::table().map_or(true, |t| t.marks().on(open))
}

/// Every shipped marker's name, so a test tool can write marked copies; none when the table does not parse.
#[cfg(windows)]
#[must_use]
pub fn markers() -> Vec<String> {
    super::table().map_or_else(
        |_| Vec::new(),
        |t| t.marks().names().map(str::to_owned).collect(),
    )
}

//! Make one (design "Make one"): a random password of `make_chars` characters from `make.toml`'s groups, each group
//! present at least once. A draw past the largest multiple of the pool's size is thrown back, so no character is more
//! likely than another, and a password missing a group is drawn again whole, so every good password is equally likely.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use kx_module_api::ports::{Random, RandomError};
use kx_module_api::{Redacted, Secret, Zeroizing, is_hidden};
use serde::Deserialize;
use thiserror::Error;

/// The groups table, compiled in so it is never read from disk.
const TABLE: &str = include_str!("../make.toml");
/// Draws of one character, or of a whole password, before the source counts as stuck: a fair one fails 64 in a row
/// far less often than once in 2⁶⁴.
const TRIES: usize = 64;

/// The groups, parsed once; a broken table would hold none, and a test keeps it sound.
static GROUPS: LazyLock<Vec<Vec<char>>> = LazyLock::new(|| parse(TABLE).unwrap_or_default());

/// Why no password was made.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum MakeError {
    /// The system gave no random bytes, or only bytes that never made a password.
    #[error("the system gave no random bytes")]
    NoRandom,
    /// The length cannot hold one character of each group.
    #[error("the password is shorter than its groups")]
    Short,
}

impl From<RandomError> for MakeError {
    fn from(_: RandomError) -> Self {
        Self::NoRandom
    }
}

/// Why the groups table is unusable.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
enum TableError {
    /// It is not TOML of the expected shape, or lists no group.
    #[error("the table does not parse: {0}")]
    Parse(String),
    /// A group is empty, or holds a blank, a hidden mark or a character listed before.
    #[error("group {0:?} is empty or holds a blank, hidden or repeated character")]
    Group(String),
}

/// The groups file.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GroupFile {
    group: Vec<GroupLine>,
}

/// One group as the file writes it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GroupLine {
    name: String,
    chars: String,
}

/// The shipped groups: letters, digits and symbols.
#[must_use]
pub fn groups() -> &'static [Vec<char>] {
    &GROUPS
}

/// A random password of `len` characters holding at least one of each group, from `random`.
///
/// # Errors
/// `Short` when `len` cannot hold each group, `NoRandom` when the source fails or sticks.
pub fn make(len: usize, random: &dyn Random) -> Result<Secret, MakeError> {
    let groups = groups();
    if groups.is_empty() || len < groups.len() {
        return Err(MakeError::Short);
    }
    let pool: Vec<char> = groups.iter().flatten().copied().collect();
    for _ in 0..TRIES {
        let password = draw(&pool, len, random)?;
        if groups
            .iter()
            .all(|g| password.chars().any(|c| g.contains(&c)))
        {
            return Ok(Redacted::new(password));
        }
    }
    Err(MakeError::NoRandom)
}

/// `len` characters of `pool`, each drawn alone, in a buffer that never grows.
fn draw(pool: &[char], len: usize, random: &dyn Random) -> Result<Zeroizing<String>, MakeError> {
    let mut out = Zeroizing::new(String::with_capacity(
        len.saturating_mul(char::MAX_LEN_UTF8),
    ));
    for _ in 0..len {
        let at = Zeroizing::new(below(pool.len(), random)?);
        out.push(pool.get(*at).copied().ok_or(MakeError::NoRandom)?);
    }
    Ok(out)
}

/// A number below `n`, each as likely: a 32-bit draw at or past the largest multiple of `n` is thrown back. The draw
/// and its number are wiped after use.
fn below(n: usize, random: &dyn Random) -> Result<usize, MakeError> {
    let n = u64::try_from(n).map_err(|_| MakeError::Short)?;
    let span = u64::from(u32::MAX) + 1;
    if n == 0 || n > span {
        return Err(MakeError::Short);
    }
    let fair = span - span % n;
    for _ in 0..TRIES {
        let mut bytes = Zeroizing::new([0; size_of::<u32>()]);
        random.fill(&mut *bytes)?;
        let x = Zeroizing::new(u64::from(u32::from_le_bytes(*bytes)));
        if *x < fair {
            return usize::try_from(*x % n).map_err(|_| MakeError::NoRandom);
        }
    }
    Err(MakeError::NoRandom)
}

/// Reads and checks the groups `text` lists: at least one, none empty, and no blank, hidden or repeated character.
fn parse(text: &str) -> Result<Vec<Vec<char>>, TableError> {
    let table: GroupFile = toml::from_str(text).map_err(|e| TableError::Parse(e.to_string()))?;
    if table.group.is_empty() {
        return Err(TableError::Parse(String::new()));
    }
    let mut seen = BTreeSet::new();
    let mut groups = Vec::with_capacity(table.group.len());
    for group in table.group {
        let fine = |c: &char| !c.is_whitespace() && !is_hidden(*c) && seen.insert(*c);
        let chars: Vec<char> = group.chars.chars().collect();
        if chars.is_empty() || !chars.iter().all(fine) {
            return Err(TableError::Group(group.name));
        }
        groups.push(chars);
    }
    Ok(groups)
}

#[cfg(test)]
mod tests;

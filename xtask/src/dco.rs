//! Commit parsing and the DCO sign-off check: pure logic, no process spawning.
//!
//! [`git`] runs `git log` and hands its raw output to [`parse_log`].

pub(crate) mod git;

use std::fmt;

use crate::config::DcoConfig;
use crate::text::plural;

/// ASCII Unit Separator: splits fields within one `git log` record.
const FIELD_SEP: char = '\u{1f}';
/// ASCII Record Separator: splits records emitted by `git log`.
const RECORD_SEP: char = '\u{1e}';
/// Fields in one record: hash, author name, author email, parent hashes, raw body.
const FIELD_COUNT: usize = 5;
/// Leading hash characters shown in a failing-commit line.
const SHORT_HASH_LEN: usize = 7;
/// The reason printed for every failing commit.
const REASON_MISSING: &str = "missing sign-off";
/// The label [`summary`] prints its line under.
const SUMMARY_LABEL: &str = "dco";
/// The noun [`summary`] counts, pluralized by [`plural`].
const FAILING_COMMIT: &str = "failing commit";

/// One commit read from `git log`, ready for the DCO check.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Commit {
    /// The full commit hash.
    pub hash: String,
    /// The commit author's name.
    pub author_name: String,
    /// The commit author's email.
    pub author_email: String,
    /// True if the commit has more than one parent.
    pub is_merge: bool,
    /// The raw, unwrapped commit message (subject and body).
    pub body: String,
}

/// A `git log` record that didn't split into exactly [`FIELD_COUNT`] fields.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ParseError {
    /// The malformed record, for the error message.
    record: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "malformed git log record: {}", self.record)
    }
}

/// Parses `git log` output (built from [`git::LOG_FORMAT`]) into [`Commit`]s.
///
/// Pure: takes text, returns commits, so it's tested without running git.
pub(crate) fn parse_log(raw: &str) -> Result<Vec<Commit>, ParseError> {
    raw.split(RECORD_SEP)
        .map(str::trim)
        .filter(|record| !record.is_empty())
        .map(parse_record)
        .collect()
}

/// Parses one trimmed record into a [`Commit`].
fn parse_record(record: &str) -> Result<Commit, ParseError> {
    let parts: Vec<&str> = record.splitn(FIELD_COUNT, FIELD_SEP).collect();
    let [hash, author_name, author_email, parents, body] = parts[..] else {
        return Err(ParseError {
            record: record.to_string(),
        });
    };
    Ok(Commit {
        hash: hash.to_string(),
        author_name: author_name.to_string(),
        author_email: author_email.to_string(),
        is_merge: parents.split_whitespace().count() > 1,
        body: body.to_string(),
    })
}

/// One commit that fails the DCO check.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Violation {
    /// The commit's full hash.
    pub hash: String,
    /// The commit author's name.
    pub author_name: String,
    /// The commit author's email.
    pub author_email: String,
}

/// Checks every non-skipped commit for a sign-off trailer matching its author.
pub(crate) fn check(commits: &[Commit], dco: &DcoConfig) -> Vec<Violation> {
    commits
        .iter()
        .filter(|c| !is_skipped(c, dco))
        .filter(|c| !has_sign_off(c, &dco.trailer))
        .map(to_violation)
        .collect()
}

/// True for a merge commit or a commit by an author in `dco.exempt_authors`.
fn is_skipped(commit: &Commit, dco: &DcoConfig) -> bool {
    commit.is_merge
        || dco
            .exempt_authors
            .iter()
            .any(|name| name == &commit.author_name)
}

/// True if some line of the commit body, trimmed, is `<trailer>: <name> <<email>>` for
/// this commit: the trailer and email compared case-insensitively, the name exactly.
fn has_sign_off(commit: &Commit, trailer: &str) -> bool {
    commit.body.lines().any(|line| {
        parse_trailer(line.trim()).is_some_and(|t| {
            t.key.eq_ignore_ascii_case(trailer)
                && t.name == commit.author_name
                && t.email.eq_ignore_ascii_case(&commit.author_email)
        })
    })
}

/// One parsed `key: name <email>` trailer line.
struct Trailer<'a> {
    /// The part before the first `:`.
    key: &'a str,
    /// The part between the key and the final `<...>`.
    name: &'a str,
    /// The part inside the final `<...>`.
    email: &'a str,
}

/// Splits a trailer line into its key, name and email, or `None` if it's not that shape.
fn parse_trailer(line: &str) -> Option<Trailer<'_>> {
    let (key, rest) = line.split_once(':')?;
    let before_email = rest.trim().strip_suffix('>')?;
    let (name, email) = before_email.rsplit_once('<')?;
    Some(Trailer {
        key: key.trim(),
        name: name.trim(),
        email,
    })
}

/// Builds a [`Violation`] from a failing [`Commit`].
fn to_violation(commit: &Commit) -> Violation {
    Violation {
        hash: commit.hash.clone(),
        author_name: commit.author_name.clone(),
        author_email: commit.author_email.clone(),
    }
}

/// Renders one violation as `<short hash> <name> <<email>>: missing sign-off`.
pub(crate) fn render(v: &Violation) -> String {
    format!(
        "{} {} <{}>: {REASON_MISSING}",
        short_hash(&v.hash),
        v.author_name,
        v.author_email
    )
}

/// The first [`SHORT_HASH_LEN`] characters of `hash`, or all of it if shorter.
fn short_hash(hash: &str) -> &str {
    hash.get(..SHORT_HASH_LEN).unwrap_or(hash)
}

/// True when `violations` is empty.
pub(crate) fn passed(violations: &[Violation]) -> bool {
    violations.is_empty()
}

/// One summary line, e.g. `dco: 1 failing commit`.
pub(crate) fn summary(violations: &[Violation]) -> String {
    let n = violations.len();
    format!("{SUMMARY_LABEL}: {n} {}", plural(n, FAILING_COMMIT))
}

#[cfg(test)]
mod tests;

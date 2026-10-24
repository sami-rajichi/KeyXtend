//! `words fetch`: checks each source's size and SHA-256, and downloads a missing one with the system's curl into a
//! part file that is checked before it is renamed.

use std::fmt::Write as _;
use std::fs::File;
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};

use super::manifest::{FetchConfig, HASH_HEX, Manifest, Origin, Source};
use super::{WordsError, io_error, with_added_ext};
use crate::devtools::paths::join_rel;

/// Bytes read per hashing step: one MiB keeps the 3 GB check fast with little memory.
const CHUNK: usize = 1 << 20;
/// The status of a file whose size and hash match.
const OK: &str = "ok";
/// The status of a file just downloaded and checked.
const FETCHED: &str = "fetched";
/// The status of a file that failed.
const BAD: &str = "BAD";
/// The status column's width: the longest status and a space.
const STATUS_WIDTH: usize = FETCHED.len() + 1;

/// What a file on disk is, against its manifest entry.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Check {
    /// Its size and hash match.
    Good,
    /// It is not there.
    Missing,
    /// It differs, and why.
    Wrong(String),
}

/// Downloads a URL into a file.
pub(crate) type Get<'a> = dyn FnMut(&str, &Path) -> Result<(), WordsError> + 'a;

/// Checks every source under `src`, downloading the missing ones with `get`, and says one line per file.
pub(crate) fn run(
    manifest: &Manifest,
    src: &Path,
    get: &mut Get<'_>,
    say: &mut dyn FnMut(&str),
) -> Result<(), WordsError> {
    let mut bad = Vec::new();
    for (rel, source) in &manifest.sources {
        let path = join_rel(src.to_path_buf(), rel);
        let outcome = match check(&path, source)? {
            Check::Good => Ok(OK),
            Check::Wrong(why) => Err(format!("{rel}: {why}")),
            Check::Missing => download(manifest, rel, &path, get),
        };
        match outcome {
            Ok(status) => say(&format!("{status:<STATUS_WIDTH$}{rel}")),
            Err(why) => {
                say(&format!("{BAD:<STATUS_WIDTH$}{why}"));
                bad.push(why);
            }
        }
    }
    if bad.is_empty() {
        Ok(())
    } else {
        Err(WordsError::Bad(bad))
    }
}

/// Fails unless each of `files` is under `src` with its manifest size and hash.
pub(crate) fn verify(manifest: &Manifest, src: &Path, files: &[&str]) -> Result<(), WordsError> {
    let mut bad = Vec::new();
    for rel in files {
        let source = manifest.sources.get(*rel);
        let path = join_rel(src.to_path_buf(), rel);
        match source.map(|s| check(&path, s)).transpose()? {
            Some(Check::Good) => {}
            Some(Check::Wrong(why)) => bad.push(format!("{rel}: {why}")),
            Some(Check::Missing) => bad.push(format!("{rel}: missing; run `words fetch`")),
            None => bad.push(format!("{rel}: not in words.toml")),
        }
    }
    if bad.is_empty() {
        Ok(())
    } else {
        Err(WordsError::Bad(bad))
    }
}

/// Downloads the missing source `rel` to `path` through a part file; a zip's files are only named.
fn download(
    manifest: &Manifest,
    rel: &str,
    path: &Path,
    get: &mut Get<'_>,
) -> Result<&'static str, String> {
    let url = match manifest.origin(rel) {
        Some(Origin::Url(url)) => url,
        Some(Origin::Zip(zip)) => return Err(format!("{rel}: missing; unzip it from {zip}")),
        None => return Err(format!("{rel}: not in words.toml")),
    };
    let source = manifest
        .sources
        .get(rel)
        .ok_or_else(|| format!("{rel}: not in words.toml"))?;
    let part = with_added_ext(path, &manifest.fetch.part);
    let fetched = path
        .parent()
        .map_or(Ok(()), std::fs::create_dir_all)
        .map_err(|e| io_error(path, &e))
        .and_then(|()| get(&url, &part))
        .and_then(|()| check(&part, source));
    let why = match fetched {
        Ok(Check::Good) => {
            std::fs::rename(&part, path).map_err(|e| format!("{rel}: {}", io_error(path, &e)))?;
            return Ok(FETCHED);
        }
        Ok(Check::Missing) => "the download wrote nothing".to_string(),
        Ok(Check::Wrong(why)) => format!("the download is wrong: {why}"),
        Err(err) => err.to_string(),
    };
    // A failed download leaves no part file that could pass for a source; a failed delete changes nothing.
    let _ = std::fs::remove_file(&part);
    Err(format!("{rel}: {why}"))
}

/// The file at `path` against its manifest entry: size first, then its SHA-256.
pub(crate) fn check(path: &Path, source: &Source) -> Result<Check, WordsError> {
    let size = match std::fs::metadata(path) {
        Ok(meta) => meta.len(),
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(Check::Missing),
        Err(err) => return Err(io_error(path, &err)),
    };
    if size != source.size {
        return Ok(Check::Wrong(format!(
            "{size} bytes, expected {}",
            source.size
        )));
    }
    let hash = sha256(path)?;
    Ok(if hash == source.sha256 {
        Check::Good
    } else {
        Check::Wrong(format!("SHA-256 {hash}, expected {}", source.sha256))
    })
}

/// The SHA-256 of the file at `path`, in lowercase hex.
fn sha256(path: &Path) -> Result<String, WordsError> {
    let mut file = File::open(path).map_err(|e| io_error(path, &e))?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0; CHUNK];
    loop {
        let n = file.read(&mut buf).map_err(|e| io_error(path, &e))?;
        let Some(chunk) = buf.get(..n).filter(|c| !c.is_empty()) else {
            break;
        };
        hasher.update(chunk);
    }
    let mut hex = String::with_capacity(HASH_HEX);
    for byte in hasher.finalize() {
        // Writing to a String cannot fail.
        let _ = write!(hex, "{byte:02x}");
    }
    Ok(hex)
}

/// Downloads `url` into `out` with the system's curl.
pub(crate) fn curl(config: &FetchConfig, url: &str, out: &Path) -> Result<(), WordsError> {
    let program = std::env::var(&config.root_env).map_or_else(
        |_| PathBuf::from(&config.fallback),
        |root| join_rel(PathBuf::from(root), &config.curl),
    );
    let status = Command::new(&program)
        .args(&config.args)
        .arg(out)
        .arg(url)
        .status()
        .map_err(|e| WordsError::Download(format!("{}: {e}", program.display())))?;
    if status.success() {
        Ok(())
    } else {
        Err(WordsError::Download(format!(
            "{url}: curl ended with {status}"
        )))
    }
}

//! `words fetch`: checks each source's size and SHA-256, and downloads a missing one with curl into a part file that
//! is checked before it is renamed.

use std::ffi::OsString;
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
/// Why a folder fails where a source file belongs.
const FOLDER: &str = "a folder, not a file";

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

/// Downloads a URL of a known size into a file.
pub(crate) type Get<'a> = dyn FnMut(&str, u64, &Path) -> Result<(), WordsError> + 'a;

/// Checks every source under `src`, downloading the missing ones with `get`, and says one line per file; a file
/// that cannot be read is one bad line, and the others are still checked.
pub(crate) fn run(
    manifest: &Manifest,
    src: &Path,
    get: &mut Get<'_>,
    say: &mut dyn FnMut(&str),
) -> Result<(), WordsError> {
    let mut bad = Vec::new();
    for (rel, source) in &manifest.sources {
        let path = join_rel(src.to_path_buf(), rel);
        let outcome = match check(&path, source) {
            Ok(Check::Good) => Ok(OK),
            Ok(Check::Wrong(why)) => Err(format!("{rel}: {why}")),
            Ok(Check::Missing) => download(manifest, rel, &path, get),
            Err(err) => Err(format!("{rel}: {err}")),
        };
        match outcome {
            Ok(status) => say(&format!("{status:<STATUS_WIDTH$}{rel}")),
            Err(why) => {
                say(&format!("{BAD:<STATUS_WIDTH$}{why}"));
                bad.push(why);
            }
        }
    }
    all_good(bad)
}

/// Fails unless each of `files` is under `src` with its manifest size and hash.
pub(crate) fn verify(manifest: &Manifest, src: &Path, files: &[&str]) -> Result<(), WordsError> {
    let mut bad = Vec::new();
    for rel in files {
        let source = manifest.sources.get(*rel);
        let path = join_rel(src.to_path_buf(), rel);
        match source.map(|s| check(&path, s)) {
            Some(Ok(Check::Good)) => {}
            Some(Ok(Check::Wrong(why))) => bad.push(format!("{rel}: {why}")),
            Some(Ok(Check::Missing)) => bad.push(format!("{rel}: missing; run `words fetch`")),
            Some(Err(err)) => bad.push(format!("{rel}: {err}")),
            None => bad.push(format!("{rel}: not in words.toml")),
        }
    }
    all_good(bad)
}

/// `Ok` when no file is in `bad`.
fn all_good(bad: Vec<String>) -> Result<(), WordsError> {
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
    let (url, source) = match (manifest.origin(rel), manifest.sources.get(rel)) {
        (Some(Origin::Url(url)), Some(source)) => (url, source),
        (Some(Origin::Zip(zip)), _) => return Err(format!("{rel}: missing; unzip it from {zip}")),
        _ => return Err(format!("{rel}: not in words.toml")),
    };
    let part = with_added_ext(path, &manifest.fetch.part);
    let fetched = path
        .parent()
        .map_or(Ok(()), std::fs::create_dir_all)
        .map_err(|e| io_error(path, &e))
        .and_then(|()| get(&url, source.size, &part))
        .and_then(|()| check(&part, source));
    let why = match fetched {
        Ok(Check::Good) => match std::fs::rename(&part, path) {
            Ok(()) => return Ok(FETCHED),
            Err(err) => io_error(path, &err).to_string(),
        },
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
    let meta = match std::fs::metadata(path) {
        Ok(meta) => meta,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(Check::Missing),
        Err(err) => return Err(io_error(path, &err)),
    };
    if meta.is_dir() {
        return Ok(Check::Wrong(FOLDER.to_string()));
    }
    if meta.len() != source.size {
        let size = meta.len();
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

/// The curl to run: the system's own under the folder `root_env` names when it is there, else the fallback on PATH.
pub(crate) fn curl_program(config: &FetchConfig, env: impl Fn(&str) -> Option<String>) -> PathBuf {
    env(&config.root_env)
        .map(|root| join_rel(PathBuf::from(root), &config.curl))
        .filter(|path| path.is_file())
        .unwrap_or_else(|| PathBuf::from(&config.fallback))
}

/// curl's arguments: the manifest's first, then the size limit, the output file and the URL, each after its flag.
pub(crate) fn curl_args(config: &FetchConfig, url: &str, size: u64, out: &Path) -> Vec<OsString> {
    let mut args: Vec<OsString> = config.args.iter().map(OsString::from).collect();
    args.extend([
        OsString::from(&config.size_flag),
        OsString::from(size.to_string()),
        OsString::from(&config.out_flag),
        out.as_os_str().to_os_string(),
        OsString::from(&config.url_flag),
        OsString::from(url),
    ]);
    args
}

/// Downloads `url`, at most `size` bytes, into `out` with curl.
pub(crate) fn curl(
    config: &FetchConfig,
    url: &str,
    size: u64,
    out: &Path,
) -> Result<(), WordsError> {
    let program = curl_program(config, |key| std::env::var(key).ok());
    let status = Command::new(&program)
        .args(curl_args(config, url, size, out))
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

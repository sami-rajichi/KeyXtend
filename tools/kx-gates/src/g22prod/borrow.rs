//! The cloud line's borrowed key: a service's sealed blob and the wrapped data key, copied byte for byte from the
//! owner's daily data folder into the copy's, at the same places. It never opens, prints or logs them or their paths,
//! copies nothing else, and the copy lends them, so they go at the end even while the copy may still run.

use std::fs;
use std::path::{Path, PathBuf};

use kx_crypto::{KEY_FILE, SEALED_EXT};
use kx_mod_voice::keys::KEYS;
use kx_module_api::ports::BlobName;
use kx_platform::VOICE_DIR;
use kx_settings::Files;

use crate::g22cfg::DAILY;

/// The wrapped data key under data folder `data`.
fn key_file(data: &Path) -> PathBuf {
    data.join(VOICE_DIR).join(KEY_FILE)
}

/// Service `id`'s sealed key under data folder `data`; `None` for an id that names no blob.
fn blob(data: &Path, id: &str) -> Option<PathBuf> {
    let name = BlobName::new(id).ok()?;
    let file = format!("{}.{SEALED_EXT}", name.as_str());
    Some(data.join(VOICE_DIR).join(KEYS).join(file))
}

/// True when the daily folder `daily` holds service `id`'s sealed key and the data key that opens it.
pub(super) fn has(daily: &Path, id: &str) -> bool {
    blob(daily, id).is_some_and(|b| b.is_file()) && key_file(daily).is_file()
}

/// True when `daily` reads as a data folder: it holds the settings file or the voice data key, so a mistyped
/// `daily_dir` is told apart from a folder whose services have no key.
pub(super) fn is_data(daily: &Path) -> bool {
    Files::new(daily).settings.is_file() || key_file(daily).is_file()
}

/// Copies service `id`'s sealed key and the data key from `daily` into the copy's data folder `data`, only when both
/// are there: the files it made there; an error names the fault's kind, never a path.
pub(super) fn copy(daily: &Path, data: &Path, id: &str) -> Result<[PathBuf; 2], String> {
    let (Some(from), Some(to)) = (blob(daily, id), blob(data, id)) else {
        return Err("a borrowed key's service names no blob".into());
    };
    if same(daily, data) {
        return Err(format!("{DAILY} is the copy's own data folder"));
    }
    if !has(daily, id) {
        return Err("a borrowed key or its data key is missing, so nothing was copied".into());
    }
    let key = key_file(data);
    one(&key_file(daily), &key)?;
    one(&from, &to)?;
    Ok([key, to])
}

/// True when `a` and `b` are one folder.
fn same(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

/// Copies file `from` to `to`, making `to`'s folder first.
fn one(from: &Path, to: &Path) -> Result<(), String> {
    let fail = |e: std::io::Error| format!("a borrowed key did not copy: {:?}", e.kind());
    if let Some(dir) = to.parent() {
        fs::create_dir_all(dir).map_err(fail)?;
    }
    fs::copy(from, to).map(drop).map_err(fail)
}

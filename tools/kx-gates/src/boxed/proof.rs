//! Task 14's two proofs from the real install (`boxed https`, `boxed folder`): the installed worker's fetch job, under
//! the worker's own token and job, reaches HTTPS through `WinHTTP` and works in a folder made with the product's guard at
//! a second drive's root. HTTPS asks one address for a size no model has, so the job refuses the answer by its head
//! and saves no byte; the folder goes at the end, and no path prints.
#![cfg(windows)]

use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc};
use std::time::Duration;
use std::{fs, io};

use kx_mod_voice::VoiceSettings;
use kx_mod_voice::models::{Row, table};
use kx_module_api::ports::{FetchAsk, FetchNote, ModelError, ModelFiles};
use kx_platform::models::WorkerModels;
use kx_platform_windows::{QpcClock, WinDisk, WinSpawner};
use serde_json::{Value, json};

use super::COMMAND;
use super::cfg::second;
use crate::config;

/// The word that runs the HTTPS proof.
pub const HTTPS: &str = "https";
/// The word that runs the guarded folder's proof.
pub const FOLDER_WORD: &str = "folder";
/// The env var naming the system drive, such as `C:`.
const SYSTEM_DRIVE: &str = "SystemDrive";
/// The size the HTTPS proof asks for: one byte, which no model file has, so the job refuses the answer by its head.
const WRONG_SIZE: u64 = 1;
/// The scheme every address of the models table takes.
const SCHEME: &str = "https://";
/// What a proof line says when the job told no end within the wait.
const NO_END: &str = "no end within the wait";

/// One proof, by its word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Proof {
    /// The fetch job reaches HTTPS.
    Https,
    /// The fetch job works in a guarded folder at a second drive's root.
    Folder,
}

impl Proof {
    /// The proof `rest` names, when it is one proof's word alone.
    pub fn parse(rest: &[String]) -> Option<Self> {
        match rest {
            [w] if w == HTTPS => Some(Self::Https),
            [w] if w == FOLDER_WORD => Some(Self::Folder),
            _ => None,
        }
    }

    /// Its word.
    const fn word(self) -> &'static str {
        match self {
            Self::Https => HTTPS,
            Self::Folder => FOLDER_WORD,
        }
    }
}

/// The models host over the installed worker, as the product makes it.
type Models = WorkerModels<WinSpawner>;

/// Runs `proof` with the installed worker in a guarded folder it makes, then removes the folder.
pub fn run(proof: Proof) -> Result<Value, String> {
    let cfg = config::load()?;
    let b = &cfg.boxed;
    second(&b.drive, std::env::var(SYSTEM_DRIVE).ok().as_deref())?;
    let worker = PathBuf::from(&b.worker);
    if !worker.is_file() {
        return Err(format!(
            "{} is missing: install the product with cargo xtask dev-install",
            b.worker
        ));
    }
    let settings = VoiceSettings::from_defaults().map_err(|e| e.to_string())?;
    let table = table().map_err(ToString::to_string)?;
    let row = table
        .default_for(&b.lang)
        .ok_or("boxed.lang has no default model")?;
    let clock = Arc::new(QpcClock::new().map_err(|e| e.to_string())?);
    let models = WorkerModels::new(WinSpawner::at(worker), clock, Arc::new(WinDisk));
    let folder = Guarded::make(&models, b.guarded())?;
    let got = match proof {
        Proof::Https => https(&models, row, &settings, &folder.path),
        Proof::Folder => in_folder(&models, row, &settings, &folder.path, b.folder.as_bytes()),
    };
    let removed = folder.remove();
    let mut v = got.unwrap_or_else(|e| json!({ "error": e, "pass": false }));
    v["pass"] = json!(v["pass"] == true && removed.is_ok());
    v["gate"] = json!(COMMAND.to_uppercase());
    v["proof"] = json!(proof.word());
    v["folder_removed"] = json!(removed.is_ok());
    Ok(v)
}

/// The fetch job asks the row's first address for one byte: the answer's head must come back and be refused by its
/// size, leaving nothing in the folder.
fn https(models: &Models, row: &Row, s: &VoiceSettings, folder: &Path) -> Result<Value, String> {
    let ask = https_ask(row, folder, s).ok_or("the row's address is not one the frames take")?;
    let host = ask
        .links
        .first()
        .and_then(|l| host(l.expose()))
        .unwrap_or_default()
        .to_owned();
    let end = fetched(models, ask, wait(s))?;
    let left = fs::read_dir(folder)
        .map_err(|e| e.kind().to_string())?
        .count();
    let pass = https_pass(end.as_ref(), left);
    Ok(json!({ "host": host, "end": said(end.as_ref()), "left": left, "pass": pass }))
}

/// The gate writes a small part and gives its size and fingerprint; the job reads it back, checks it and names it, with
/// no network, as a part already whole asks no address.
fn in_folder(
    models: &Models,
    row: &Row,
    s: &VoiceSettings,
    folder: &Path,
    bytes: &[u8],
) -> Result<Value, String> {
    let mut ask =
        https_ask(row, folder, s).ok_or("the row's address is not one the frames take")?;
    let part = ask.part().ok_or("the file's name is not UTF-8")?;
    fs::write(&part, bytes).map_err(|e| format!("the part could not be written: {}", e.kind()))?;
    ask.bytes = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    ask.print = models.fingerprint(&part).map_err(|e| e.to_string())?;
    let file = ask.file.expose().clone();
    let end = fetched(models, ask, wait(s))?;
    let named = fs::read(&file).is_ok_and(|b| b == bytes);
    let gone = !part.exists();
    let pass = folder_pass(end.as_ref(), named, gone);
    Ok(
        json!({ "end": said(end.as_ref()), "named": named, "part_gone": gone, "bytes": bytes.len(), "pass": pass }),
    )
}

/// The row's download into `folder` from its first address alone, for a size no model has.
pub(super) fn https_ask(row: &Row, folder: &Path, s: &VoiceSettings) -> Option<FetchAsk> {
    let mut ask = row.fetch(folder, s.job(), s.min_free_mb, s.stall_s)?;
    ask.links.truncate(1);
    ask.bytes = WRONG_SIZE;
    Some(ask)
}

/// The host of HTTPS address `link`; none for another scheme or an empty host.
pub(super) fn host(link: &str) -> Option<&str> {
    let host = link.strip_prefix(SCHEME)?.split('/').next()?;
    (!host.is_empty()).then_some(host)
}

/// True when the job refused the answer by its size and left nothing, so the head came back and no body was saved.
pub(super) fn https_pass(end: Option<&FetchNote>, left: usize) -> bool {
    matches!(end, Some(FetchNote::Fault(ModelError::Size))) && left == 0
}

/// True when the job named the file, which holds the bytes it read back, and its part is gone.
pub(super) fn folder_pass(end: Option<&FetchNote>, named: bool, part_gone: bool) -> bool {
    matches!(end, Some(FetchNote::Done)) && named && part_gone
}

/// Runs `ask` to its end: the job's last note, none when it told none within `wait`.
fn fetched(models: &Models, ask: FetchAsk, wait: Duration) -> Result<Option<FetchNote>, String> {
    let (tx, rx) = mpsc::channel();
    let tell = Box::new(move |note: FetchNote| {
        if !matches!(note, FetchNote::Progress { .. }) {
            let _ = tx.send(note);
        }
    });
    let run = models.fetch(ask, tell).map_err(|e| e.to_string())?;
    let end = rx.recv_timeout(wait).ok();
    drop(run);
    Ok(end)
}

/// The longest wait for a fetch's end: the product's stall wait and one answer.
fn wait(s: &VoiceSettings) -> Duration {
    Duration::from_secs(u64::from(s.stall_s)) + Duration::from_millis(u64::from(s.ask_ms))
}

/// A fetch's end as words, which name no path.
fn said(end: Option<&FetchNote>) -> String {
    match end {
        Some(FetchNote::Fault(e)) => e.to_string(),
        Some(FetchNote::Done) => "done".into(),
        _ => NO_END.into(),
    }
}

/// The guarded folder a proof made; it goes at the end, on a panic too.
struct Guarded {
    path: PathBuf,
}

impl Guarded {
    /// Makes `path` with the product's guard; one already there is refused, as the proof deletes it.
    fn make(models: &Models, path: PathBuf) -> Result<Self, String> {
        if path.exists() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            return Err(format!(
                "{name} is there from an earlier run: delete it first"
            ));
        }
        models
            .make(&path)
            .map_err(|e| format!("the guarded folder: {e}"))?;
        Ok(Self { path })
    }

    /// Removes the folder and all in it; one already gone is fine.
    fn remove(&self) -> Result<(), String> {
        match fs::remove_dir_all(&self.path) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e.kind().to_string()),
            _ => Ok(()),
        }
    }
}

impl Drop for Guarded {
    fn drop(&mut self) {
        let _ = self.remove();
    }
}

#[cfg(test)]
mod tests;

//! The box proofs' settings (`[boxed]` in `kx-gates-p11.toml`): the real install's worker, the second drive and the
//! guarded folder made at its root, and the language whose default model's row names the HTTPS address.
#![cfg(windows)]

use std::path::{Component, Path, PathBuf, Prefix};

use kx_mod_voice::models::table;
use kx_platform_windows::worker::WORKER_EXE;
use serde::Deserialize;

use crate::g22cfg::plain_name;

/// The box proofs (`boxed https`, `boxed folder`).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoxProofs {
    /// The real install's worker, an absolute path; the proofs start it under the worker's own token and job.
    pub worker: String,
    /// The second drive's root, such as `D:\`; never the system drive.
    pub drive: String,
    /// The guarded folder made at that root, a plain name; it goes at the end.
    pub folder: String,
    /// The language whose default model's row gives the HTTPS address and its hosts.
    pub lang: String,
}

impl BoxProofs {
    /// Refuses settings the proofs could not run by, naming the first bad key.
    pub fn check(&self) -> Result<(), String> {
        let worker = Path::new(&self.worker);
        let named = worker
            .file_name()
            .is_some_and(|n| n.eq_ignore_ascii_case(WORKER_EXE));
        if !worker.is_absolute() || !named {
            return Err(format!(
                "boxed.worker must be the installed {WORKER_EXE}, an absolute path"
            ));
        }
        if !is_root(&self.drive) {
            return Err(r"boxed.drive must be a drive's root alone, such as D:\".into());
        }
        if !plain_name(&self.folder) {
            return Err("boxed.folder must be a plain folder name".into());
        }
        let row = table().ok().and_then(|t| t.default_for(&self.lang));
        if row.is_none_or(|r| r.links.is_empty()) {
            return Err(
                "boxed.lang must have a default model with an address in the models table".into(),
            );
        }
        Ok(())
    }

    /// The guarded folder: `folder` at the drive's root.
    pub fn guarded(&self) -> PathBuf {
        Path::new(&self.drive).join(&self.folder)
    }
}

/// True for a drive's root alone, such as `D:\`.
pub fn is_root(text: &str) -> bool {
    let mut parts = Path::new(text).components();
    let disk =
        |p: &Component| matches!(p, Component::Prefix(p) if matches!(p.kind(), Prefix::Disk(_)));
    let (first, root, rest) = (parts.next(), parts.next(), parts.next());
    first.as_ref().is_some_and(disk) && root == Some(Component::RootDir) && rest.is_none()
}

/// Refuses `drive` when it is the system drive `system` (`SystemDrive`, such as `C:`), as the proof is of a second
/// one.
pub fn second(drive: &str, system: Option<&str>) -> Result<(), String> {
    let letter = |t: &str| t.chars().next().map(|c| c.to_ascii_uppercase());
    match system {
        None => Err("SystemDrive is not set, so the second drive cannot be told".into()),
        Some(s) if letter(s) == letter(drive) => Err(format!(
            "boxed.drive {drive} is the system drive; name a second one"
        )),
        Some(_) => Ok(()),
    }
}

#[cfg(test)]
mod tests;

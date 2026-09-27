//! `cargo xtask dev-install`: signs a test build and copies it into Program Files.

use std::path::{Path, PathBuf};

use super::cert::{CertStore, ScriptStore};
use super::config::{self, DevSetup, DevToolsConfig};
use super::{DevError, admin, paths, ps, sdk};

/// `signtool` subcommand that signs files.
const SIGN: &str = "sign";
/// `signtool` flag naming the current-user store that holds the key.
const STORE_FLAG: &str = "/s";
/// `signtool` flag choosing the certificate by thumbprint.
const SHA1_FLAG: &str = "/sha1";
/// `signtool` flag that sets the file digest algorithm.
const DIGEST_FLAG: &str = "/fd";
/// File extension of the programs that get signed.
const EXE_EXT: &str = "exe";
/// Bytes in one MB, for the size limit.
const BYTES_PER_MB: u64 = 1024 * 1024;

/// Accepts `name` only if it is one plain folder name.
pub(crate) fn plain(name: &str) -> Result<String, DevError> {
    if paths::is_plain_name(name) {
        Ok(name.to_string())
    } else {
        Err(DevError::BadName(name.to_string()))
    }
}

/// The install name for a build folder: its last path component.
pub(crate) fn install_name(folder: &Path) -> Result<String, DevError> {
    let last = folder.file_name().and_then(|n| n.to_str());
    last.ok_or_else(|| DevError::BadName(paths::text(folder)))
        .and_then(plain)
}

/// The install target: `<secure root>\<install dir>\<name>`.
pub(crate) fn target_dir(root: &Path, config: &DevToolsConfig, name: &str) -> PathBuf {
    root.join(&config.install_dir).join(name)
}

/// `signtool` arguments that sign `exe` with the test certificate.
pub(crate) fn sign_args(config: &DevToolsConfig, thumbprint: &str, exe: &Path) -> Vec<String> {
    [SIGN, STORE_FLAG, &config.user_store, SHA1_FLAG, thumbprint]
        .into_iter()
        .chain([DIGEST_FLAG, &config.digest])
        .map(ToString::to_string)
        .chain([paths::text(exe)])
        .collect()
}

/// The `.exe` files directly inside `folder`, sorted; none means the wrong folder was given.
pub(crate) fn exes(folder: &Path) -> Result<Vec<PathBuf>, DevError> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(folder)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && is_exe(path))
        .collect();
    if found.is_empty() {
        return Err(DevError::NoExe(paths::text(folder)));
    }
    found.sort();
    Ok(found)
}

/// True if `path` has the program extension, in any letter case.
fn is_exe(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case(EXE_EXT))
}

/// Refuses a build folder that overlaps the target, holds a link, or is larger than `max_mb`.
pub(crate) fn check_source(source: &Path, target: &Path, max_mb: u64) -> Result<(), DevError> {
    if paths::overlaps(source, target) {
        return Err(DevError::Overlap(paths::text(source)));
    }
    let mb = paths::folder_size(source)?.div_ceil(BYTES_PER_MB);
    if mb > max_mb {
        return Err(DevError::TooBig { mb, max: max_mb });
    }
    Ok(())
}

/// The thumbprint of the test certificate, or a hint to run `dev-cert` first.
fn require_cert(setup: &DevSetup) -> Result<String, DevError> {
    ScriptStore { setup }.find()?.ok_or(DevError::NoCert)
}

/// Loads the settings and the install target for `name`.
fn prepare(name: &str) -> Result<(DevSetup, PathBuf), DevError> {
    let setup = config::from_cargo()?;
    let root = setup.config.install_root(|key| std::env::var(key).ok())?;
    let target = target_dir(&root, &setup.config, name);
    std::fs::create_dir_all(setup.out(""))?;
    Ok((setup, target))
}

/// Runs `cargo xtask dev-install <folder>`: signs every exe, then copies the folder.
pub(crate) fn run(folder: &str) -> Result<(), DevError> {
    let source = paths::normalize(&std::path::absolute(folder)?);
    if !source.is_dir() {
        return Err(DevError::NotAFolder(folder.to_string()));
    }
    let (setup, target) = prepare(&install_name(&source)?)?;
    check_source(&source, &target, setup.config.install_max_mb)?;
    let programs = exes(&source)?;
    let thumbprint = require_cert(&setup)?;
    let signtool = sdk::find_tool(&setup.config, &setup.config.signtool)?;
    for exe in programs {
        let args = sign_args(&setup.config, &thumbprint, &exe);
        ps::run(&setup.config.signtool, &paths::text(&signtool), &args)?;
    }
    admin::run(
        &setup,
        admin::build_params(&setup.config, Some(&source), &target),
    )?;
    println!("Installed to {}.", paths::text(&target));
    Ok(())
}

/// Runs `cargo xtask dev-install --remove <name>`: deletes that install, with no prompt if it is absent.
pub(crate) fn remove(name: &str) -> Result<(), DevError> {
    let (setup, target) = prepare(&plain(name)?)?;
    if !target.exists() {
        println!("{} is not installed.", paths::text(&target));
        return Ok(());
    }
    admin::run(&setup, admin::build_params(&setup.config, None, &target))?;
    println!("Removed {}.", paths::text(&target));
    Ok(())
}

#[cfg(test)]
mod tests;

//! The Start-menu shortcut `dev-install` adds for the installed app: in the user's own menu, never the all-users one.

use std::path::{Path, PathBuf};

use super::config::{DevSetup, DevToolsConfig};
use super::{DevError, paths, ps, script};

/// The shortcut file in the user's Start menu, under the absolute folder `start_menu_env` names, and nowhere else.
pub(crate) fn link(
    config: &DevToolsConfig,
    env: impl Fn(&str) -> Option<String>,
) -> Result<PathBuf, DevError> {
    let key = &config.start_menu_env;
    let root = env(key)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .ok_or_else(|| DevError::MissingEnv(key.clone()))?;
    Ok(paths::join_rel(root, &config.start_menu_dir).join(&config.shortcut))
}

/// Parameters for one `dev-shortcut.ps1` action on `link` for the program `exe`.
pub(crate) fn params(action: &str, link: &Path, exe: &Path) -> Vec<(&'static str, String)> {
    vec![
        (script::ACTION, action.to_string()),
        (script::LINK, paths::text(link)),
        (script::EXE, paths::text(exe)),
    ]
}

/// Runs `dev-shortcut.ps1` with `params`; it never shows a prompt.
fn call(setup: &DevSetup, params: &[(&'static str, String)]) -> Result<String, DevError> {
    ps::run_script(setup, &setup.config.shortcut_script, params, None)
}

/// Adds the user's Start-menu shortcut to `exe`, replacing an old one, and returns its path.
pub(crate) fn add(setup: &DevSetup, exe: &Path) -> Result<PathBuf, DevError> {
    let link = link(&setup.config, |key| std::env::var(key).ok())?;
    call(setup, &params(script::ADD, &link, exe))?;
    Ok(link)
}

/// Removes the user's Start-menu shortcut if it starts `exe`; true when one went.
pub(crate) fn remove(setup: &DevSetup, exe: &Path) -> Result<bool, DevError> {
    let link = link(&setup.config, |key| std::env::var(key).ok())?;
    let had = link.exists();
    call(setup, &params(script::REMOVE, &link, exe))?;
    Ok(had && !link.exists())
}

#[cfg(test)]
mod tests;

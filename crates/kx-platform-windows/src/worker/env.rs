//! The worker's environment and working folder (ADR-0044): only the variables `env.toml` keeps, a search path of the
//! system folders alone, so no engine, Vulkan loader or search-path variable of ours reaches it, and its own folder.

use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use kx_platform::worker::SpawnError;
use serde::Deserialize;
use windows::Win32::Foundation::ERROR_INVALID_DATA;
use windows::Win32::System::SystemInformation::{GetSystemDirectoryW, GetSystemWindowsDirectoryW};

use super::refused;

/// The environment table, compiled in so it is never read from disk.
const TABLE: &str = include_str!("env.toml");

/// The shipped table; tests prove it parses.
static SHIPPED: LazyLock<Option<Table>> = LazyLock::new(|| {
    let table = Table::parse(TABLE);
    if table.is_none() {
        tracing::warn!("the worker environment table does not parse or names a banned variable");
    }
    table
});

/// The variables the worker keeps, its search path's name and gap, and the starts no kept name may have.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Table {
    pub(super) keep: Vec<String>,
    pub(super) path: String,
    path_gap: String,
    banned: Vec<String>,
}

impl Table {
    /// Reads `text`; `None` when it does not parse or keeps the search path or a name with a banned start.
    pub(super) fn parse(text: &str) -> Option<Self> {
        let t: Self = toml::from_str(text).ok()?;
        let banned = |name: &String| {
            let upper = name.to_ascii_uppercase();
            t.banned
                .iter()
                .any(|b| upper.starts_with(&b.to_ascii_uppercase()))
        };
        let odd = t
            .keep
            .iter()
            .any(|n| banned(n) || n.eq_ignore_ascii_case(&t.path));
        (!odd && !t.path_gap.is_empty()).then_some(t)
    }

    /// The worker's variables: each of `ours` the table keeps, and the search path as `folders`, sorted by name in any
    /// case, as Windows wants its block.
    pub(super) fn vars(
        &self,
        ours: impl IntoIterator<Item = (OsString, OsString)>,
        folders: &[PathBuf],
    ) -> Vec<(OsString, OsString)> {
        let kept = |name: &OsStr| {
            self.keep
                .iter()
                .any(|k| OsStr::new(k).eq_ignore_ascii_case(name))
        };
        let mut vars: Vec<(OsString, OsString)> =
            ours.into_iter().filter(|(n, _)| kept(n)).collect();
        let mut path = OsString::new();
        for (at, folder) in folders.iter().enumerate() {
            if at > 0 {
                path.push(&self.path_gap);
            }
            path.push(folder);
        }
        vars.push((OsString::from(&self.path), path));
        vars.sort_by_cached_key(|(n, _)| n.to_ascii_uppercase());
        vars
    }
}

/// The shipped table; `None` when it does not parse, so no worker starts.
pub(super) fn table() -> Option<&'static Table> {
    SHIPPED.as_ref()
}

/// `vars` as a Unicode environment block: each `name=value` ends with a NUL, and one more NUL ends the block.
pub(super) fn block(vars: &[(OsString, OsString)]) -> Vec<u16> {
    let mut block = Vec::new();
    for (name, value) in vars {
        block.extend(name.encode_wide());
        block.extend(OsStr::new("=").encode_wide());
        block.extend(value.encode_wide());
        block.push(0);
    }
    block.push(0);
    block
}

/// The worker's environment block and its working folder, each UTF-16 with its NULs.
pub(super) struct Home {
    pub(super) env: Vec<u16>,
    pub(super) dir: Vec<u16>,
}

/// The home of the worker at `exe`: the kept variables of ours, the system folders as its path, and its own folder.
pub(super) fn home(exe: &Path) -> Result<Home, SpawnError> {
    let broken =
        || refused("read the worker's environment table")(ERROR_INVALID_DATA.to_hresult().into());
    let table = table().ok_or_else(broken)?;
    let folders = [
        system(GetSystemDirectoryW, "find the system folder")?,
        system(GetSystemWindowsDirectoryW, "find the Windows folder")?,
    ];
    let vars = table.vars(std::env::vars_os(), &folders);
    let dir = exe.parent().ok_or(SpawnError::Missing)?;
    let dir = dir.as_os_str().encode_wide().chain([0]).collect();
    Ok(Home {
        env: block(&vars),
        dir,
    })
}

/// The system folder `ask` names, such as System32.
#[allow(unsafe_code, reason = "Lets Windows fill a buffer of ours.")]
fn system(
    ask: unsafe fn(Option<&mut [u16]>) -> u32,
    step: &'static str,
) -> Result<PathBuf, SpawnError> {
    // SAFETY: a size query with no buffer, which returns the length with its NUL.
    let len = unsafe { ask(None) };
    let mut buf = vec![0_u16; usize::try_from(len).unwrap_or_default()];
    // SAFETY: Windows writes at most `buf.len()` units and returns the length without its NUL.
    let got = usize::try_from(unsafe { ask(Some(&mut buf)) }).unwrap_or(usize::MAX);
    if got == 0 || got >= buf.len() {
        return Err(refused(step)(windows::core::Error::from_thread()));
    }
    Ok(PathBuf::from(OsString::from_wide(&buf[..got])))
}

#[cfg(test)]
mod tests;

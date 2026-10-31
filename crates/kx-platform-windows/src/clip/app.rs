//! The app behind a copy: the clipboard owner's process, named by its file's version information when it runs from a
//! drive, else by its file stem.

use std::path::Path;
use std::ptr::null_mut;

use kx_module_api::ports::{AppRef, probe_ok};
use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
};
use windows::Win32::System::DataExchange::GetClipboardOwner;
use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
use windows::core::PCWSTR;

use super::parts::text_of;
use super::table;
use crate::front::{app_name, path_of};
use crate::wide;

/// Where version information lists its language and code page pairs.
const TRANSLATIONS: &str = r"\VarFileInfo\Translation";
/// Where version information keeps its strings, before a language and code page.
const STRINGS: &str = r"\StringFileInfo\";
/// The string that describes the file, such as "Notepad".
const DESCRIPTION: &str = r"\FileDescription";
/// Bytes in a UTF-16 unit, as version strings count in units.
const UNIT_BYTES: usize = 2;

/// The process whose window owns the clipboard now; `None` when no window does.
#[allow(unsafe_code, reason = "Plain queries.")]
pub(crate) fn owner() -> Option<u32> {
    // SAFETY: a plain query; no owner gives an error.
    let window = unsafe { GetClipboardOwner() }.ok()?;
    let mut pid = 0u32;
    // SAFETY: a plain query into a local that outlives it; a gone window leaves it 0.
    unsafe { GetWindowThreadProcessId(window, Some(&raw mut pid)) };
    (pid != 0).then_some(pid)
}

/// The app of process `pid`: its exe name in lower case as the id, its description or file stem as the name.
pub(crate) fn of(pid: Option<u32>) -> Option<AppRef> {
    let path = path_of(pid?)?;
    let t = &table().ok()?.app;
    named(&path, t.name_chars, |p| describe(p, t.version_bytes))
}

/// The app whose file is at `path`; `read` gives its description only for a drive path, so a share is never read.
pub(crate) fn named(
    path: &str,
    max_chars: usize,
    read: impl FnOnce(&str) -> Option<String>,
) -> Option<AppRef> {
    let id = app_name(path)?.to_lowercase();
    let stem = Path::new(path).file_stem()?.to_string_lossy().into_owned();
    let described = probe_ok(Path::new(path)).then(|| read(path)).flatten();
    let name = name(described, &stem, max_chars);
    Some(AppRef { id, name })
}

/// The name to show: the description unless it is blank, else the stem, cut to `max_chars` characters.
pub(crate) fn name(described: Option<String>, stem: &str, max_chars: usize) -> String {
    let shown = described.filter(|d| !d.trim().is_empty());
    shown
        .as_deref()
        .unwrap_or(stem)
        .chars()
        .take(max_chars)
        .collect()
}

/// The description in the version information of the file at `path`; `None` without one, or past `max_bytes`.
#[allow(unsafe_code, reason = "Reads version information into our own buffer.")]
pub(crate) fn describe(path: &str, max_bytes: u32) -> Option<String> {
    let file = wide(path);
    // SAFETY: the name ends with a NUL and outlives the call.
    let size = unsafe { GetFileVersionInfoSizeW(PCWSTR(file.as_ptr()), None) };
    if size == 0 || size > max_bytes {
        return None;
    }
    let mut block = vec![0u8; usize::try_from(size).ok()?];
    // SAFETY: `block` holds the `size` bytes the call may write, and both buffers outlive it.
    unsafe { GetFileVersionInfoW(PCWSTR(file.as_ptr()), None, size, block.as_mut_ptr().cast()) }
        .ok()?;
    let (at, len) = value(&mut block, TRANSLATIONS)?;
    let pair = block.get(at..)?.get(..len)?;
    let lang = u16::from_ne_bytes(*pair.first_chunk::<2>()?);
    let page = u16::from_ne_bytes(*pair.get(UNIT_BYTES..)?.first_chunk::<2>()?);
    let (at, len) = value(
        &mut block,
        &format!("{STRINGS}{lang:04x}{page:04x}{DESCRIPTION}"),
    )?;
    let bytes = block.get(at..)?.get(..len.checked_mul(UNIT_BYTES)?)?;
    let units: Vec<u16> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|b| u16::from_ne_bytes(*b))
        .collect();
    text_of(&units, usize::MAX)
}

/// Where the value at `key` sits in version information `block`, and its length (units for text, else bytes);
/// `None` when it is missing or Windows points outside the block.
#[allow(unsafe_code, reason = "Queries our own buffer.")]
fn value(block: &mut [u8], key: &str) -> Option<(usize, usize)> {
    let key = wide(key);
    let (mut ptr, mut len) = (null_mut(), 0u32);
    // SAFETY: `block` holds what `GetFileVersionInfoW` wrote, with its room for the call's own conversions, and
    // outlives the call, which points `ptr` into it; the key ends with a NUL.
    let found = unsafe {
        VerQueryValueW(
            block.as_mut_ptr().cast_const().cast(),
            PCWSTR(key.as_ptr()),
            &raw mut ptr,
            &raw mut len,
        )
    };
    let at = ptr.addr().checked_sub(block.as_ptr().addr())?;
    (found.as_bool() && at < block.len()).then_some((at, usize::try_from(len).ok()?))
}

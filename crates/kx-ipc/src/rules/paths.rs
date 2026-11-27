//! What a path or a file name may hold: never a NUL, and for Snip's paths and names no control character and no part
//! that names a device, which Windows would open as the device; a dialog's answer is a drive path. The device names
//! and separators are public, so the ports refuse the same names.

use super::ok_if;
use crate::error::IpcError;
use crate::limits::MAX_PATH_BYTES;
use crate::limits::snip::NAME_UNITS;

/// The separators a path's parts lie between; Windows reads both.
pub const SEPARATORS: &[u8] = br"\/";
/// The marks Windows refuses in a file's name besides the separators.
const NAME_MARKS: &[u8] = br#"<>:"|?*"#;
/// Follows a drive's letter, as in `C:\`.
const DRIVE_MARK: char = ':';
/// What Win32 trims from a part's end, so a part ending in one names another; `.` and `..` end in one too.
const TRIMMED: [char; 2] = ['.', ' '];
/// Names some supported OS refuses as file names (Windows keeps them for devices, even with an extension, and reads
/// `¹`, `²` and `³` as digits); no file or data is named after one.
pub const RESERVED_NAMES: &[&str] = &[
    "con", "prn", "aux", "nul", "conin$", "conout$", "com0", "com1", "com2", "com3", "com4",
    "com5", "com6", "com7", "com8", "com9", "com¹", "com²", "com³", "lpt0", "lpt1", "lpt2", "lpt3",
    "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9", "lpt¹", "lpt²", "lpt³",
];

/// True when `name`'s stem, before its first dot and without trailing spaces, is a device name (`RESERVED_NAMES`)
/// in any case, as `NUL.txt` or `COM¹`, which Windows opens as the device.
#[must_use]
pub fn names_device(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or_default().trim_end();
    RESERVED_NAMES.iter().any(|d| stem.eq_ignore_ascii_case(d))
}

/// Refuses a path holding a NUL, which ends a path early for the OS.
pub(crate) fn path(text: &str) -> Result<(), IpcError> {
    ok_if(!text.contains('\0'))
}

/// Refuses a Snip path holding a control character, or a part that names a device.
pub(crate) fn plain_path(text: &str) -> Result<(), IpcError> {
    let device = parts(text).any(names_device);
    ok_if(!text.chars().any(char::is_control) && !device)
}

/// Refuses a path whose last part is not a name, a dot and `ext` in any case.
pub(crate) fn typed(text: &str, ext: &str) -> Result<(), IpcError> {
    let last = parts(text).last().unwrap_or_default();
    let named = last.rsplit_once('.');
    ok_if(named.is_some_and(|(stem, end)| !stem.is_empty() && end.eq_ignore_ascii_case(ext)))
}

/// Refuses a file name holding a control character, a separator or a mark Windows refuses, or naming a device.
pub(crate) fn file_name(text: &str) -> Result<(), IpcError> {
    let marked = |c: char| u8::try_from(c).is_ok_and(|b| separator(b) || NAME_MARKS.contains(&b));
    let odd = text.chars().any(|c| c.is_control() || marked(c));
    ok_if(!odd && !names_device(text))
}

/// Refuses a path unless a frame's path field can carry it (`MAX_PATH_BYTES`) and it is a drive's root (an ASCII
/// letter, `DRIVE_MARK` and a separator), then `plain_part`s; so UNC, device, relative and aliased paths fail.
pub(crate) fn drive_path(text: &str) -> Result<(), IpcError> {
    ok_if(text.len() <= MAX_PATH_BYTES)?;
    let mut chars = text.chars();
    let letter = chars.next().is_some_and(|c| c.is_ascii_alphabetic());
    let marked = chars.next() == Some(DRIVE_MARK);
    let rest = chars.as_str();
    let rooted = rest.bytes().next().is_some_and(separator);
    ok_if(letter && marked && rooted)?;
    let below = rest.get(1..).unwrap_or_default();
    ok_if(below.is_empty() || below.split(split).all(plain_part))
}

/// True for a part of a drive path: a file name within `NAME_UNITS` UTF-16 units, never empty nor ending in `TRIMMED`.
fn plain_part(part: &str) -> bool {
    let short = part.encode_utf16().count() <= NAME_UNITS;
    !part.is_empty() && !part.ends_with(TRIMMED) && short && file_name(part).is_ok()
}

/// True for a byte of `SEPARATORS`.
fn separator(b: u8) -> bool {
    SEPARATORS.contains(&b)
}

/// True for a character of `SEPARATORS`.
fn split(c: char) -> bool {
    u8::try_from(c).is_ok_and(separator)
}

/// `text`'s non-empty parts between `SEPARATORS`.
fn parts(text: &str) -> impl Iterator<Item = &str> {
    text.split(split).filter(|part| !part.is_empty())
}

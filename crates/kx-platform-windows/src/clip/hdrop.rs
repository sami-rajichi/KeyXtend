//! The file list `CF_HDROP` holds: a `DROPFILES` header, then each path in UTF-16 closed by a NUL, then one more NUL.
//! Reading it checks every offset against the bytes, as another app wrote them.

use std::path::{Path, PathBuf};

/// Bytes in the `DROPFILES` header, where our lists start their paths.
pub(crate) const HEADER: usize = 20;
/// Where the header keeps the offset of the paths.
const FILES_AT: usize = 0;
/// Where the header keeps whether the paths are UTF-16.
const WIDE_AT: usize = 16;
/// Bytes in one header field.
const FIELD: usize = 4;

/// The `CF_HDROP` bytes of `paths`: UTF-16 paths right after the header.
pub(crate) fn encode(paths: &[PathBuf]) -> Vec<u8> {
    let mut out = vec![0; HEADER];
    let at = u32::try_from(HEADER).unwrap_or_default();
    out[FILES_AT..FILES_AT + FIELD].copy_from_slice(&at.to_ne_bytes());
    out[WIDE_AT..WIDE_AT + FIELD].copy_from_slice(&1i32.to_ne_bytes());
    let units = paths
        .iter()
        .flat_map(|p| units_of(p).into_iter().chain([0]));
    out.extend(units.chain([0]).flat_map(u16::to_ne_bytes));
    out
}

/// The paths in `CF_HDROP` bytes, up to the closing NUL or the last path that ended; `None` when the header is bad or
/// the paths are not UTF-16, as in a 16-bit era list.
pub(crate) fn decode(bytes: &[u8]) -> Option<Vec<PathBuf>> {
    let field = |at: usize| bytes.get(at..at + FIELD)?.first_chunk::<FIELD>().copied();
    let at = usize::try_from(u32::from_ne_bytes(field(FILES_AT)?)).ok()?;
    let wide = i32::from_ne_bytes(field(WIDE_AT)?) != 0;
    if !wide || at < HEADER {
        return None;
    }
    let units: Vec<u16> = bytes
        .get(at..)?
        .as_chunks::<2>()
        .0
        .iter()
        .map(|b| u16::from_ne_bytes(*b))
        .collect();
    let mut paths = Vec::new();
    let mut rest = units.as_slice();
    while let Some(end) = rest.iter().position(|&u| u == 0) {
        let (name, tail) = rest.split_at(end);
        if name.is_empty() {
            break;
        }
        paths.push(path_of(name));
        rest = tail.get(1..).unwrap_or_default();
    }
    Some(paths)
}

/// The UTF-16 units of `path`, exactly as Windows names it.
#[cfg(windows)]
fn units_of(path: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    path.as_os_str().encode_wide().collect()
}

/// The UTF-16 units of `path`, for the tests on other systems.
#[cfg(not(windows))]
fn units_of(path: &Path) -> Vec<u16> {
    path.to_string_lossy().encode_utf16().collect()
}

/// The path that UTF-16 `units` name, exactly, unpaired halves included.
#[cfg(windows)]
fn path_of(units: &[u16]) -> PathBuf {
    use std::os::windows::ffi::OsStringExt;
    PathBuf::from(std::ffi::OsString::from_wide(units))
}

/// The path that UTF-16 `units` name, for the tests on other systems.
#[cfg(not(windows))]
fn path_of(units: &[u16]) -> PathBuf {
    PathBuf::from(String::from_utf16_lossy(units))
}

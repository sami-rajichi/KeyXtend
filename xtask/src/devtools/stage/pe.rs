//! The library names a Windows program or library imports, read from its PE headers without running it.

use std::path::Path;

use super::StageError;
use crate::devtools::{DevError, paths};

/// The DOS header's magic, and where it keeps the PE header's place.
const DOS_MAGIC: &[u8] = b"MZ";
const PE_AT: usize = 0x3C;
/// The PE header's magic, then the COFF header with its section count and optional-header size.
const PE_MAGIC: &[u8] = b"PE\0\0";
const COFF_LEN: usize = 20;
const SECTIONS_AT: usize = 2;
const OPT_SIZE_AT: usize = 16;
/// The optional header's magic for 32- and 64-bit images, and where each keeps its data directories.
const PE32: u16 = 0x10b;
const PE64: u16 = 0x20b;
const DIRS_AT_32: usize = 96;
const DIRS_AT_64: usize = 112;
const DIR_LEN: usize = 8;
/// The data-directory count, four bytes just before the directories.
const DIR_COUNT_LEN: usize = 4;
/// A section header's length and its sizes and places in memory and in the file.
const SECTION_LEN: usize = 40;
const V_SIZE_AT: usize = 8;
const VA_AT: usize = 12;
const RAW_SIZE_AT: usize = 16;
const RAW_AT: usize = 20;
/// A delay-load descriptor flag: its places are relative to the image, as every current linker writes them.
const RVA_FLAG: u32 = 1;

/// One table of imported library names: its data directory, descriptor length, name place and flags place.
struct Table {
    index: usize,
    len: usize,
    name_at: usize,
    flags_at: Option<usize>,
}

/// The import table and the delay-load import table.
const IMPORTS: Table = Table {
    index: 1,
    len: 20,
    name_at: 12,
    flags_at: None,
};
const DELAYED: Table = Table {
    index: 13,
    len: 32,
    name_at: 4,
    flags_at: Some(0),
};

/// The names `bytes` imports, then those it delay-loads; `None` when it is not a well-formed image.
pub(crate) fn imports(bytes: &[u8]) -> Option<Vec<String>> {
    if bytes.get(..DOS_MAGIC.len())? != DOS_MAGIC {
        return None;
    }
    let pe = usize::try_from(u32_at(bytes, PE_AT)?).ok()?;
    if bytes.get(pe..pe.checked_add(PE_MAGIC.len())?)? != PE_MAGIC {
        return None;
    }
    let coff = pe + PE_MAGIC.len();
    let opt = coff.checked_add(COFF_LEN)?;
    let dirs = match u16_at(bytes, opt)? {
        PE32 => opt + DIRS_AT_32,
        PE64 => opt + DIRS_AT_64,
        _ => return None,
    };
    let image = Image {
        bytes,
        sections: opt.checked_add(usize::from(u16_at(bytes, coff + OPT_SIZE_AT)?))?,
        count: usize::from(u16_at(bytes, coff + SECTIONS_AT)?),
    };
    let dir_count = usize::try_from(u32_at(bytes, dirs - DIR_COUNT_LEN)?).ok()?;
    let mut names = Vec::new();
    for table in [&IMPORTS, &DELAYED] {
        if table.index < dir_count {
            let rva = u32_at(bytes, dirs + table.index * DIR_LEN)?;
            image.names(rva, table, &mut names)?;
        }
    }
    Some(names)
}

/// An image's bytes and its section table, which maps places in memory to places in the file.
struct Image<'a> {
    bytes: &'a [u8],
    sections: usize,
    count: usize,
}

impl Image<'_> {
    /// The file place of memory place `rva`, from the section that holds it.
    fn offset(&self, rva: u32) -> Option<usize> {
        (0..self.count).find_map(|i| {
            let at = self.sections.checked_add(i.checked_mul(SECTION_LEN)?)?;
            let va = u32_at(self.bytes, at + VA_AT)?;
            let size =
                u32_at(self.bytes, at + V_SIZE_AT)?.max(u32_at(self.bytes, at + RAW_SIZE_AT)?);
            let inside = rva.checked_sub(va).filter(|d| *d < size)?;
            let raw = u32_at(self.bytes, at + RAW_AT)?;
            usize::try_from(raw.checked_add(inside)?).ok()
        })
    }

    /// Adds the names of `table`'s descriptors, which start at `rva` and end with an all-zero one.
    fn names(&self, rva: u32, table: &Table, names: &mut Vec<String>) -> Option<()> {
        if rva == 0 {
            return Some(());
        }
        let mut at = self.offset(rva)?;
        loop {
            let desc = self.bytes.get(at..at.checked_add(table.len)?)?;
            if desc.iter().all(|b| *b == 0) {
                return Some(());
            }
            if let Some(flags_at) = table.flags_at
                && u32_at(desc, flags_at)? & RVA_FLAG == 0
            {
                return None;
            }
            names.push(text_at(
                self.bytes,
                self.offset(u32_at(desc, table.name_at)?)?,
            )?);
            at += table.len;
        }
    }
}

/// The two bytes at `at`, little-endian.
fn u16_at(bytes: &[u8], at: usize) -> Option<u16> {
    let b = bytes.get(at..at.checked_add(size_of::<u16>())?)?;
    Some(u16::from_le_bytes([b[0], b[1]]))
}

/// The four bytes at `at`, little-endian.
fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    let b = bytes.get(at..at.checked_add(size_of::<u32>())?)?;
    Some(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

/// The NUL-ended text at `at`; a name with no end is malformed.
fn text_at(bytes: &[u8], at: usize) -> Option<String> {
    let rest = bytes.get(at..)?;
    let end = rest.iter().position(|b| *b == 0)?;
    Some(String::from_utf8_lossy(&rest[..end]).into_owned())
}

/// The library names the file at `path` imports.
pub(crate) fn read(path: &Path) -> Result<Vec<String>, DevError> {
    let bytes = std::fs::read(path)?;
    imports(&bytes).ok_or_else(|| StageError::BadPe(paths::text(path)).into())
}

#[cfg(test)]
mod tests;

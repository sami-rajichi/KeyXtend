//! The built worker loads Media Foundation only when a video job asks for it: none of its libraries is in the program's
//! import table, which Windows loads at start, and each is in its delay-load table instead (edge 14, ADR-0056).
#![cfg(windows)]
#![allow(clippy::unwrap_used, reason = "a failed read should fail the test")]

/// The DOS header's magic, and where it keeps the PE header's place.
const DOS_MAGIC: &[u8] = b"MZ";
const PE_AT: usize = 0x3C;
/// The PE header's magic, the COFF header's length, its section count and its optional header's size.
const PE_MAGIC: &[u8] = b"PE\0\0";
const COFF_LEN: usize = 20;
const SECTIONS_AT: usize = 2;
const OPT_SIZE_AT: usize = 16;
/// The 64-bit optional header's magic and where it keeps its data directories.
const PE64: u16 = 0x20b;
const DIRS_AT: usize = 112;
const DIR_LEN: usize = 8;
/// A section header's length, and its sizes and places in memory and in the file.
const SECTION_LEN: usize = 40;
const V_SIZE_AT: usize = 8;
const VA_AT: usize = 12;
const RAW_AT: usize = 20;
/// The import table: its directory, descriptor length and where a descriptor names its library.
const IMPORTS: (usize, usize, usize) = (1, 20, 12);
/// The delay-load import table, the same three.
const DELAYED: (usize, usize, usize) = (13, 32, 4);

/// The four bytes at `at`, little-endian.
fn u32_at(b: &[u8], at: usize) -> usize {
    usize::try_from(u32::from_le_bytes(b[at..at + 4].try_into().unwrap())).unwrap()
}

/// The two bytes at `at`, little-endian.
fn u16_at(b: &[u8], at: usize) -> usize {
    usize::from(u16::from_le_bytes(b[at..at + 2].try_into().unwrap()))
}

/// A 64-bit image's bytes, with where its section table and data directories start.
struct Image<'a> {
    b: &'a [u8],
    sections: usize,
    count: usize,
    dirs: usize,
}

impl<'a> Image<'a> {
    fn read(b: &'a [u8]) -> Self {
        assert_eq!(&b[..DOS_MAGIC.len()], DOS_MAGIC);
        let pe = u32_at(b, PE_AT);
        assert_eq!(&b[pe..pe + PE_MAGIC.len()], PE_MAGIC);
        let coff = pe + PE_MAGIC.len();
        let opt = coff + COFF_LEN;
        assert_eq!(u16_at(b, opt), usize::from(PE64));
        Self {
            b,
            sections: opt + u16_at(b, coff + OPT_SIZE_AT),
            count: u16_at(b, coff + SECTIONS_AT),
            dirs: opt + DIRS_AT,
        }
    }

    /// The file place of memory place `rva`.
    fn at(&self, rva: usize) -> usize {
        (0..self.count)
            .map(|i| self.sections + i * SECTION_LEN)
            .find_map(|s| {
                let va = u32_at(self.b, s + VA_AT);
                let inside = rva.checked_sub(va)?;
                (inside < u32_at(self.b, s + V_SIZE_AT))
                    .then(|| u32_at(self.b, s + RAW_AT) + inside)
            })
            .unwrap()
    }

    /// The library names one table lists, lower case.
    fn names(&self, (dir, len, name_at): (usize, usize, usize)) -> Vec<String> {
        let rva = u32_at(self.b, self.dirs + dir * DIR_LEN);
        if rva == 0 {
            return Vec::new();
        }
        let mut at = self.at(rva);
        let mut names = Vec::new();
        while self.b[at..at + len].iter().any(|b| *b != 0) {
            let text = &self.b[self.at(u32_at(self.b, at + name_at))..];
            let end = text.iter().position(|b| *b == 0).unwrap();
            names.push(String::from_utf8_lossy(&text[..end]).to_lowercase());
            at += len;
        }
        names
    }
}

#[test]
fn media_foundation_is_delay_loaded_never_loaded_at_start() {
    let exe = std::fs::read(env!("CARGO_BIN_EXE_keyxtend-worker")).unwrap();
    let image = Image::read(&exe);
    let (now, later) = (image.names(IMPORTS), image.names(DELAYED));
    assert!(!now.is_empty(), "the reader found the import table");
    let media = &kx_platform_record::table::table().unwrap().media_dlls;
    for dll in media.iter().map(|d| d.to_lowercase()) {
        assert!(!now.contains(&dll), "{dll} loads at start: {now:?}");
        assert!(later.contains(&dll), "{dll} is not delay-loaded: {later:?}");
    }
}

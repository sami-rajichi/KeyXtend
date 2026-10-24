//! The Win32 side of word packs: each pack file opened for reading with read sharing only, and mapped read-only.

use std::fs::{File, OpenOptions};
use std::io::ErrorKind;
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::path::{Path, PathBuf};

use kx_module_api::ports::{PackError, PackView, WordPacks};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Memory::{
    CreateFileMappingW, FILE_MAP_READ, MEMORY_MAPPED_VIEW_ADDRESS, MapViewOfFile, PAGE_READONLY,
    UnmapViewOfFile,
};
use windows::core::PCWSTR;

use super::{Os, find, view_len};

/// The share mode that lets others read a mapped pack but never write, rename or delete it (`FILE_SHARE_READ`).
const SHARE_READ: u32 = 1;

/// The word packs on Windows: `<iso>.kxw` in each folder, the first folder first, mapped read-only.
/// It has no `Debug`, as its folders name the user.
#[derive(Clone, Default)]
pub struct WinWordPacks {
    dirs: Vec<PathBuf>,
}

impl WinWordPacks {
    /// The packs in `dirs`, looked up in that order.
    #[must_use]
    pub fn new(dirs: Vec<PathBuf>) -> Self {
        Self { dirs }
    }
}

impl WordPacks for WinWordPacks {
    fn open(&self, iso: &str) -> Result<Option<Box<dyn PackView>>, PackError> {
        let view = find(&Win32, &self.dirs, iso)?;
        Ok(view.map(|v| Box::new(v) as Box<dyn PackView>))
    }
}

/// One mapped pack: the view, the mapping and the file, which close in that order when it drops.
/// It has no `Debug`, as a file's debug output shows its path and so the user's name.
struct WinPackView {
    view: View,
    _mapping: OwnedHandle,
    _file: File,
}

impl PackView for WinPackView {
    fn bytes(&self) -> &[u8] {
        self.view.bytes()
    }
}

/// A read-only view of a whole pack file, unmapped once when dropped.
struct View {
    at: MEMORY_MAPPED_VIEW_ADDRESS,
    len: usize,
}

// SAFETY: the view is process-wide read-only memory; any thread may read or unmap it.
#[allow(unsafe_code, reason = "A mapped view may move between threads.")]
unsafe impl Send for View {}
// SAFETY: shared references only read the bytes; only the owner's drop unmaps them.
#[allow(unsafe_code, reason = "A read-only view is shared.")]
unsafe impl Sync for View {}

impl View {
    /// The whole file.
    #[allow(unsafe_code, reason = "Reads the mapped file.")]
    fn bytes(&self) -> &[u8] {
        // SAFETY: `at` maps `len` readable bytes until `self` drops, and the file is shared for reading only, so no
        // handle can write it.
        unsafe { std::slice::from_raw_parts(self.at.Value.cast::<u8>(), self.len) }
    }
}

impl Drop for View {
    #[allow(unsafe_code, reason = "Unmaps the view we own.")]
    fn drop(&mut self) {
        // SAFETY: unmaps, once, a view Windows gave us; no slice of it outlives `self`.
        let _ = unsafe { UnmapViewOfFile(self.at) };
    }
}

/// The Win32 calls behind `WinWordPacks`.
struct Win32;

impl Os for Win32 {
    type View = WinPackView;

    fn map(&self, path: &Path) -> Result<Option<WinPackView>, PackError> {
        let file = match OpenOptions::new()
            .read(true)
            .share_mode(SHARE_READ)
            .open(path)
        {
            Ok(file) => file,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(PackError::Unreadable),
        };
        let meta = file.metadata().map_err(|_| PackError::Unreadable)?;
        if !meta.is_file() {
            return Err(PackError::Unreadable);
        }
        let len = view_len(meta.len())?;
        let mapping = mapping(&file)?;
        let view = view(&mapping, len)?;
        Ok(Some(WinPackView {
            view,
            _mapping: mapping,
            _file: file,
        }))
    }
}

/// A read-only mapping of the whole of `file`, never copy-on-write.
#[allow(unsafe_code, reason = "Maps a file of ours.")]
fn mapping(file: &File) -> Result<OwnedHandle, PackError> {
    let handle = HANDLE(file.as_raw_handle());
    // SAFETY: `file` is open for reading and outlives the call; with no name, no other program can open the mapping.
    let made = unsafe { CreateFileMappingW(handle, None, PAGE_READONLY, 0, 0, PCWSTR::null()) }
        .map_err(|_| PackError::Unreadable)?;
    // SAFETY: Windows just made this handle, and nothing else owns or closes it.
    Ok(unsafe { OwnedHandle::from_raw_handle(made.0) })
}

/// A read-only view of the first `len` bytes of `mapping`.
#[allow(unsafe_code, reason = "Maps a view of a mapping of ours.")]
fn view(mapping: &OwnedHandle, len: usize) -> Result<View, PackError> {
    let handle = HANDLE(mapping.as_raw_handle());
    // SAFETY: a live read-only mapping, viewed for reading only; Windows checks `len` against the file.
    let at = unsafe { MapViewOfFile(handle, FILE_MAP_READ, 0, 0, len) };
    if at.Value.is_null() {
        return Err(PackError::Unreadable);
    }
    Ok(View { at, len })
}

#[cfg(test)]
mod tests {
    use std::fs::{self, OpenOptions};

    use kx_test_support::tempdir::TempDir;

    use super::*;
    use kx_module_api::ports::{MAX_PACK_BYTES, PACK_EXT};
    use windows::Win32::System::Memory::{MEM_MAPPED, MEMORY_BASIC_INFORMATION, VirtualQuery};

    /// A pack's length in the tests: more than one page, and not a whole number of them.
    const LEN: usize = 10_000;

    /// A folder with a `packs` folder inside, so a code that climbs out would land in the outer one.
    fn packs_dir() -> (TempDir, PathBuf) {
        let dir = TempDir::new("packs").unwrap();
        let inner = dir.path().join("packs");
        fs::create_dir(&inner).unwrap();
        (dir, inner)
    }

    /// Writes `bytes` as `iso`'s pack in `dir`, and gives its path.
    fn write(dir: &Path, iso: &str, bytes: &[u8]) -> PathBuf {
        let path = dir.join(format!("{iso}.{PACK_EXT}"));
        fs::write(&path, bytes).unwrap();
        path
    }

    /// Bytes that differ from one place to the next.
    fn sample() -> Vec<u8> {
        (0..LEN).map(|i| u8::try_from(i % 251).unwrap()).collect()
    }

    #[test]
    fn a_packs_bytes_come_back_whole_through_the_view() {
        let (_dir, inner) = packs_dir();
        write(&inner, "en", &sample());
        let view = WinWordPacks::new(vec![inner]).open("en").unwrap().unwrap();
        assert_eq!(view.bytes(), sample().as_slice());
    }

    #[test]
    fn a_missing_language_is_none() {
        let (_dir, inner) = packs_dir();
        write(&inner, "en", &sample());
        let packs = WinWordPacks::new(vec![inner.join("missing"), inner]);
        assert!(packs.open("fr").unwrap().is_none());
    }

    #[test]
    fn the_second_folder_is_used_when_the_first_lacks_the_file() {
        let (dir, inner) = packs_dir();
        write(dir.path(), "en", &sample());
        let view = WinWordPacks::new(vec![inner, dir.path().to_path_buf()])
            .open("en")
            .unwrap()
            .unwrap();
        assert_eq!(view.bytes().len(), LEN);
    }

    #[test]
    fn a_bad_code_is_none_even_with_a_file_it_would_reach() {
        let (dir, inner) = packs_dir();
        write(dir.path(), "x", &sample());
        for iso in ["en", "e", "engl"] {
            write(&inner, iso, &sample());
        }
        let packs = WinWordPacks::new(vec![inner]);
        for iso in ["../x", "EN", "e", "engl", "con", "nul"] {
            assert!(packs.open(iso).unwrap().is_none(), "{iso}");
        }
    }

    /// What Windows says of the memory region that holds `at`.
    #[allow(unsafe_code, reason = "Asks Windows about our own memory.")]
    fn region_of(at: *const u8) -> MEMORY_BASIC_INFORMATION {
        let mut info = MEMORY_BASIC_INFORMATION::default();
        let size = size_of_val(&info);
        // SAFETY: `info` is ours and its exact size goes with it; the query only reads the page tables.
        let wrote = unsafe { VirtualQuery(Some(at.cast()), std::ptr::from_mut(&mut info), size) };
        assert_eq!(wrote, size, "Windows describes the region");
        info
    }

    #[test]
    fn the_view_is_a_read_only_file_map_never_copy_on_write() {
        let (_dir, inner) = packs_dir();
        write(&inner, "en", &sample());
        let view = WinWordPacks::new(vec![inner]).open("en").unwrap().unwrap();
        let region = region_of(view.bytes().as_ptr());
        assert_eq!(region.Protect, PAGE_READONLY, "not PAGE_WRITECOPY");
        assert_eq!(region.Type, MEM_MAPPED);
        assert!(region.RegionSize >= LEN, "the whole pack is one region");
    }

    #[test]
    fn an_empty_pack_a_folder_and_an_oversized_pack_are_unreadable() {
        let (_dir, inner) = packs_dir();
        write(&inner, "en", &[]);
        fs::create_dir(inner.join(format!("fr.{PACK_EXT}"))).unwrap();
        let big = write(&inner, "ar", &[]);
        let file = OpenOptions::new().write(true).open(big).unwrap();
        file.set_len(MAX_PACK_BYTES + 1).unwrap();
        drop(file);
        let packs = WinWordPacks::new(vec![inner]);
        for iso in ["en", "fr", "ar"] {
            assert_eq!(packs.open(iso).err(), Some(PackError::Unreadable), "{iso}");
        }
    }

    #[test]
    fn a_mapped_pack_cannot_be_written_or_deleted_until_its_view_drops() {
        let (_dir, inner) = packs_dir();
        let path = write(&inner, "en", &sample());
        let view = WinWordPacks::new(vec![inner]).open("en").unwrap().unwrap();
        assert!(
            OpenOptions::new().write(true).open(&path).is_err(),
            "no writer while mapped"
        );
        assert!(fs::remove_file(&path).is_err(), "no delete while mapped");
        drop(view);
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn the_view_moves_to_another_thread_and_reads_there() {
        let (_dir, inner) = packs_dir();
        write(&inner, "en", &sample());
        let view = WinWordPacks::new(vec![inner]).open("en").unwrap().unwrap();
        let len = std::thread::spawn(move || view.bytes().len()).join();
        assert_eq!(len.ok(), Some(LEN));
    }
}
